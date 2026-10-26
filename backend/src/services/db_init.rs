use crate::config::AppConfig;
use crate::services::backup::BackupService;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::HashSet;
use std::time::Duration;

const MIGRATION_ADVISORY_LOCK_ID: i64 = 5_135_656_515_814_555_463;

/// Returns the exponential backoff retry schedule for initial PostgreSQL connection (`TD-001`).
/// When `database_required` is `true`, retries up to 8 times (`500ms`, `1s`, `2s`, `4s`, `5s`...).
/// When `database_required` is `false` (standalone dev/test mode), performs a single fast attempt.
pub fn retry_backoff_schedule(database_required: bool) -> Vec<Duration> {
    if database_required {
        vec![
            Duration::from_millis(500),
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(5),
            Duration::from_secs(5),
            Duration::from_secs(5),
        ]
    } else {
        Vec::new()
    }
}

/// Determines whether an automatic pre-upgrade backup must be taken (`17-api-security-operations.md §6.2`).
/// Returns `true` when the database already has applied migrations (`!applied_versions.is_empty()`)
/// AND at least one embedded migration version has not yet been applied.
pub fn should_trigger_pre_upgrade_backup(
    applied_versions: &[i64],
    embedded_versions: &[i64],
) -> bool {
    if applied_versions.is_empty() {
        return false;
    }
    let applied_set: HashSet<i64> = applied_versions.iter().copied().collect();
    embedded_versions.iter().any(|v| !applied_set.contains(v))
}

/// Connects to PostgreSQL using the exponential backoff loop (`TD-001`), acquires an advisory lock,
/// triggers a `pre_upgrade` backup if pending migrations exist on an existing database, runs SQLx
/// migrations, and seeds initial `instance_settings`.
pub async fn connect_and_migrate(
    config: &AppConfig,
    backup_service: &BackupService,
) -> Result<Option<PgPool>, String> {
    let schedule = retry_backoff_schedule(config.database_required);
    let total_attempts = schedule.len() + 1;
    let acquire_timeout = if config.database_required {
        Duration::from_secs(5)
    } else {
        Duration::from_secs(2)
    };

    let mut last_err = String::new();
    let mut connected_pool: Option<PgPool> = None;

    for attempt in 1..=total_attempts {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(acquire_timeout)
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                tracing::info!(
                    attempt = attempt,
                    total_attempts = total_attempts,
                    "Connected to PostgreSQL database successfully"
                );
                connected_pool = Some(pool);
                break;
            }
            Err(err) => {
                last_err = err.to_string();
                if let Some(delay) = schedule.get(attempt - 1) {
                    tracing::warn!(
                        attempt = attempt,
                        total_attempts = total_attempts,
                        retry_in_ms = delay.as_millis() as u64,
                        error = %err,
                        "PostgreSQL not ready yet; backing off before retry"
                    );
                    tokio::time::sleep(*delay).await;
                }
            }
        }
    }

    let pool = match connected_pool {
        Some(p) => p,
        None => {
            if config.database_required {
                return Err(format!(
                    "Failed to connect to PostgreSQL after {total_attempts} attempts: {last_err}"
                ));
            }
            tracing::warn!(
                error = %last_err,
                "PostgreSQL unavailable and DATABASE_REQUIRED=false; booting in standalone mode"
            );
            return Ok(None);
        }
    };

    run_migrations_with_advisory_lock(&pool, config, backup_service).await?;
    Ok(Some(pool))
}

pub async fn run_migrations_with_advisory_lock(
    pool: &PgPool,
    config: &AppConfig,
    backup_service: &BackupService,
) -> Result<(), String> {
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(MIGRATION_ADVISORY_LOCK_ID)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to acquire migration advisory lock: {e}"))?;

    let result = async {
        let migrator = sqlx::migrate!("./migrations");
        let embedded_versions: Vec<i64> = migrator.iter().map(|m| m.version).collect();
        let applied_versions = fetch_applied_migration_versions(pool).await;

        if should_trigger_pre_upgrade_backup(&applied_versions, &embedded_versions) {
            tracing::info!(
                applied_count = applied_versions.len(),
                embedded_count = embedded_versions.len(),
                "Pending database migrations detected on existing database; creating automatic pre-upgrade backup"
            );
            backup_service
                .create_backup(Some(pool), "pre_upgrade", None)
                .await
                .map_err(|e| format!("Pre-upgrade backup failed: {e}"))?;
        }

        migrator
            .run(pool)
            .await
            .map_err(|e| format!("Database migration failed: {e}"))?;

        seed_instance_settings_defaults(pool, config).await?;
        Ok(())
    }
    .await;

    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_ADVISORY_LOCK_ID)
        .execute(pool)
        .await;

    result
}

async fn fetch_applied_migration_versions(pool: &PgPool) -> Vec<i64> {
    let exists_row = sqlx::query(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = '_sqlx_migrations'
        ) AS exists",
    )
    .fetch_one(pool)
    .await;

    let table_exists = match exists_row {
        Ok(row) => row.try_get::<bool, _>("exists").unwrap_or(false),
        Err(_) => false,
    };

    if !table_exists {
        return Vec::new();
    }

    match sqlx::query("SELECT version FROM _sqlx_migrations WHERE success = TRUE ORDER BY version")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .filter_map(|r| r.try_get::<i64, _>("version").ok())
            .collect(),
        Err(_) => Vec::new(),
    }
}

async fn seed_instance_settings_defaults(pool: &PgPool, config: &AppConfig) -> Result<(), String> {
    let defaults: [(&str, serde_json::Value); 4] = [
        (
            "tree_title",
            serde_json::Value::String(config.tree_title.clone()),
        ),
        (
            "default_locale",
            serde_json::Value::String(config.default_locale.clone()),
        ),
        (
            "enable_public_access",
            serde_json::Value::Bool(config.enable_public_access),
        ),
        (
            "living_person_age_threshold",
            serde_json::json!(config.living_person_age_threshold),
        ),
    ];

    for (key, value) in defaults {
        sqlx::query(
            "INSERT INTO instance_settings (key, value, updated_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (key) DO NOTHING",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to seed instance_settings '{key}': {e}"))?;
    }

    Ok(())
}
