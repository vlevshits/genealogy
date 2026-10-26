use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    error::{AppError, ErrorResponse},
    routes::{
        create_router,
        system::{
            AdminSystemResponse, BackupListResponse, SystemInfoResponse, TriggerBackupResponse,
        },
    },
    services::{
        backup::BackupService,
        db_init::{retry_backoff_schedule, should_trigger_pre_upgrade_backup},
        scheduler::parse_cron_hour_minute,
    },
    state::{AppState, InstanceSettingsDto},
};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tower::ServiceExt;

#[test]
fn test_full_32_variable_config_parsing_and_defaults() {
    let defaults = AppConfig::default();
    assert_eq!(defaults.port, 3000);
    assert_eq!(defaults.app_env, "development");
    assert!(!defaults.database_required);
    assert_eq!(defaults.thumbnail_quality, 85);
    assert!(!defaults.enable_public_access);
    assert!(!defaults.enable_registration);
    assert_eq!(defaults.living_person_age_threshold, 110);
    assert!(!defaults.smtp_enabled);
    assert_eq!(defaults.smtp_port, 587);
    assert!(defaults.backup_enabled);
    assert_eq!(defaults.backup_cron, "0 3 * * *");
    assert_eq!(defaults.backup_retention_days, 30);
    assert!(!defaults.backup_include_gedcom);
    assert_eq!(defaults.auth_password_enabled, "auto");
    assert!(defaults.is_password_auth_effectively_enabled());
    assert_eq!(defaults.session_ttl_days, 30);
    assert_eq!(defaults.max_import_size_mb, 2048);
    assert_eq!(defaults.job_workers, 2);
    assert!(defaults.map_tile_url.is_some());
    assert!(!defaults.geocoding_enabled);
    assert_eq!(defaults.history_retention_days, 365);
    assert!(!defaults.settings_locked_from_env);
    assert!(!defaults.public_search_show_living);

    // Custom 32-variable environment map
    let mut vars = HashMap::new();
    vars.insert("PORT".to_string(), "4000".to_string());
    vars.insert("APP_ENV".to_string(), "production".to_string());
    vars.insert("RUST_LOG".to_string(), "warn".to_string());
    vars.insert(
        "APP_ORIGIN".to_string(),
        "https://tree.example.com".to_string(),
    );
    vars.insert("RP_ID".to_string(), "tree.example.com".to_string());
    vars.insert("RP_NAME".to_string(), "Levshits Family Tree".to_string());
    vars.insert("POSTGRES_USER".to_string(), "archiver".to_string());
    vars.insert("POSTGRES_PASSWORD".to_string(), "secret123".to_string());
    vars.insert("POSTGRES_DB".to_string(), "family_db".to_string());
    vars.insert("MAX_UPLOAD_SIZE_MB".to_string(), "250".to_string());
    vars.insert("THUMBNAIL_QUALITY".to_string(), "92".to_string());
    vars.insert(
        "MEDIA_STORAGE_PATH".to_string(),
        "/custom/media".to_string(),
    );
    vars.insert("TREE_TITLE".to_string(), "Архив Семьи Левшиц".to_string());
    vars.insert("DEFAULT_LOCALE".to_string(), "ru".to_string());
    vars.insert("ENABLE_PUBLIC_ACCESS".to_string(), "true".to_string());
    vars.insert("ENABLE_REGISTRATION".to_string(), "true".to_string());
    vars.insert("LIVING_PERSON_AGE_THRESHOLD".to_string(), "105".to_string());
    vars.insert("SMTP_ENABLED".to_string(), "true".to_string());
    vars.insert("SMTP_HOST".to_string(), "smtp.example.com".to_string());
    vars.insert("SMTP_PORT".to_string(), "465".to_string());
    vars.insert("SMTP_USER".to_string(), "mailer@example.com".to_string());
    vars.insert("SMTP_PASSWORD".to_string(), "smtp-pass".to_string());
    vars.insert(
        "SMTP_FROM_EMAIL".to_string(),
        "noreply@example.com".to_string(),
    );
    vars.insert("SMTP_FROM_NAME".to_string(), "Levshits Archive".to_string());
    vars.insert("BACKUP_ENABLED".to_string(), "true".to_string());
    vars.insert("BACKUP_CRON".to_string(), "15 4 * * *".to_string());
    vars.insert("BACKUP_RETENTION_DAYS".to_string(), "14".to_string());
    vars.insert("BACKUP_INCLUDE_GEDCOM".to_string(), "true".to_string());
    vars.insert("BACKUP_PATH".to_string(), "/custom/backups".to_string());
    vars.insert("AUTH_PASSWORD_ENABLED".to_string(), "auto".to_string());
    vars.insert("SESSION_TTL_DAYS".to_string(), "60".to_string());
    vars.insert("MAX_IMPORT_SIZE_MB".to_string(), "4096".to_string());
    vars.insert("JOB_WORKERS".to_string(), "4".to_string());
    vars.insert("MAP_TILE_URL".to_string(), "".to_string()); // Empty disables maps
    vars.insert("MAP_ATTRIBUTION".to_string(), "Custom Tiles".to_string());
    vars.insert("GEOCODING_ENABLED".to_string(), "true".to_string());
    vars.insert(
        "GEOCODING_URL".to_string(),
        "https://geo.internal".to_string(),
    );
    vars.insert("HISTORY_RETENTION_DAYS".to_string(), "0".to_string());
    vars.insert("SETTINGS_LOCKED_FROM_ENV".to_string(), "true".to_string());
    vars.insert("PUBLIC_SEARCH_SHOW_LIVING".to_string(), "true".to_string());

    let cfg = AppConfig::from_map(&vars).expect("Must parse 32-variable config");
    assert_eq!(cfg.port, 4000);
    assert_eq!(cfg.app_env, "production");
    assert!(cfg.database_required);
    assert_eq!(
        cfg.database_url,
        "postgresql://archiver:secret123@postgres:5432/family_db?sslmode=disable"
    );
    assert_eq!(cfg.max_upload_size_mb, 250);
    assert_eq!(cfg.thumbnail_quality, 92);
    assert_eq!(cfg.tree_title, "Архив Семьи Левшиц");
    assert_eq!(cfg.default_locale, "ru");
    assert!(cfg.enable_public_access);
    assert!(cfg.enable_registration);
    assert_eq!(cfg.living_person_age_threshold, 105);
    assert!(cfg.smtp_enabled);
    assert_eq!(cfg.smtp_port, 465);
    // HTTPS + SMTP enabled -> auto password fallback is false
    assert!(!cfg.is_password_auth_effectively_enabled());
    assert_eq!(cfg.backup_cron, "15 4 * * *");
    assert_eq!(parse_cron_hour_minute(&cfg.backup_cron), (4, 15));
    assert_eq!(cfg.backup_retention_days, 14);
    assert!(cfg.backup_include_gedcom);
    assert_eq!(cfg.session_ttl_days, 60);
    assert_eq!(cfg.max_import_size_mb, 4096);
    assert_eq!(cfg.job_workers, 4);
    assert_eq!(cfg.map_tile_url, None);
    assert!(cfg.geocoding_enabled);
    assert_eq!(cfg.history_retention_days, 0);
    assert!(cfg.settings_locked_from_env);
    assert!(cfg.public_search_show_living);
}

#[test]
fn test_db_retry_schedule_and_pre_upgrade_backup_trigger_logic() {
    let prod_schedule = retry_backoff_schedule(true);
    assert_eq!(prod_schedule.len(), 7);
    assert_eq!(prod_schedule[0], Duration::from_millis(500));
    assert_eq!(prod_schedule[1], Duration::from_secs(1));
    assert_eq!(prod_schedule[2], Duration::from_secs(2));
    assert_eq!(prod_schedule[3], Duration::from_secs(4));
    assert_eq!(prod_schedule[4], Duration::from_secs(5));

    let dev_schedule = retry_backoff_schedule(false);
    assert!(dev_schedule.is_empty());

    // Empty DB (fresh install) -> no pre-upgrade backup needed
    assert!(!should_trigger_pre_upgrade_backup(
        &[],
        &[20261026000001, 20261026000002]
    ));

    // Existing DB with migration 1 applied and migration 2 pending -> MUST trigger pre-upgrade backup
    assert!(should_trigger_pre_upgrade_backup(
        &[20261026000001],
        &[20261026000001, 20261026000002]
    ));

    // Existing DB with all migrations already applied -> no pre-upgrade backup needed
    assert!(!should_trigger_pre_upgrade_backup(
        &[20261026000001, 20261026000002],
        &[20261026000001, 20261026000002]
    ));
}

#[tokio::test]
async fn test_backup_service_create_restore_checksum_verification_and_pruning() {
    let temp = tempfile::tempdir().unwrap();
    let media_dir = temp.path().join("media");
    let backup_dir = temp.path().join("backups");

    // Seed an original media asset
    let orig_subdir = media_dir.join("originals").join("ab").join("cd");
    fs::create_dir_all(&orig_subdir).unwrap();
    let sample_photo = orig_subdir.join("portrait.jpg");
    fs::write(&sample_photo, b"fake-jpeg-binary-payload-12345").unwrap();

    let config = Arc::new(AppConfig {
        media_storage_path: media_dir.to_string_lossy().to_string(),
        backup_path: backup_dir.to_string_lossy().to_string(),
        tree_title: "Levshits Test Archive".to_string(),
        backup_include_gedcom: true,
        ..AppConfig::default()
    });

    let service = BackupService::new(config.clone());

    // 1. Create backup
    let info = service
        .create_backup(None, "manual", Some(true))
        .await
        .expect("Backup creation must succeed");
    assert!(info.filename.starts_with("backup-"));
    assert!(info.filename.ends_with(".tar.gz"));
    assert!(info.size_bytes > 0);
    assert!(info.includes_gedcom);

    let archive_path = backup_dir.join(&info.filename);
    assert!(archive_path.exists());

    // 2. Delete original media file, then restore from backup and verify it returns
    fs::remove_file(&sample_photo).unwrap();
    assert!(!sample_photo.exists());

    let manifest = service
        .verify_and_restore(&archive_path, None)
        .await
        .expect("Restore of valid backup must succeed");
    assert_eq!(manifest.manifest_version, 1);
    assert!(manifest.files.contains_key("database.dump"));
    assert!(manifest.files.contains_key("tree.gdz"));
    assert!(manifest
        .files
        .contains_key("media/originals/ab/cd/portrait.jpg"));
    assert!(sample_photo.exists());
    assert_eq!(
        fs::read(&sample_photo).unwrap(),
        b"fake-jpeg-binary-payload-12345"
    );

    // 3. Corrupt a file inside the archive and verify restore rejects with ChecksumMismatch
    let corrupt_work = tempfile::tempdir().unwrap();
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(corrupt_work.path())
        .status()
        .unwrap();
    assert!(status.success());

    // Tamper with database.dump without updating manifest.json
    fs::write(corrupt_work.path().join("database.dump"), b"tampered-data").unwrap();
    let corrupted_archive = backup_dir.join("backup-20261026-corrupted.tar.gz");
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&corrupted_archive)
        .arg("-C")
        .arg(corrupt_work.path())
        .arg(".")
        .status()
        .unwrap();
    assert!(status.success());

    let err = service
        .verify_and_restore(&corrupted_archive, None)
        .await
        .expect_err("Corrupted archive must fail checksum verification");
    assert!(matches!(err, AppError::ChecksumMismatch(_)));
    assert_eq!(err.code(), "checksum_mismatch");

    // 3b. Tamper with schema_version in manifest.json to exceed CURRENT_SCHEMA_VERSION -> IncompatibleSchema
    let mut future_manifest = manifest.clone();
    future_manifest.schema_version = 999;
    fs::write(
        corrupt_work.path().join("manifest.json"),
        serde_json::to_vec_pretty(&future_manifest).unwrap(),
    )
    .unwrap();
    let future_archive = backup_dir.join("backup-20261026-future.tar.gz");
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&future_archive)
        .arg("-C")
        .arg(corrupt_work.path())
        .arg(".")
        .status()
        .unwrap();
    assert!(status.success());

    let schema_err = service
        .verify_and_restore(&future_archive, None)
        .await
        .expect_err("Future schema version must fail with IncompatibleSchema");
    assert!(matches!(schema_err, AppError::IncompatibleSchema(_)));
    assert_eq!(schema_err.code(), "incompatible_schema");
    let _ = fs::remove_file(&future_archive);

    // 4. Test emergency admin link generation
    let link = service
        .create_emergency_admin_link(None, "admin@example.com")
        .await
        .expect("Emergency admin link must succeed");
    assert!(link.starts_with("http://localhost:8080/setup?emergency_token="));

    // 5. Test retention pruning (0 days keeps everything; expired mtime is pruned)
    assert_eq!(service.prune_old_backups(0).unwrap(), 0);
    // Backdate corrupted_archive modification time by 40 days using `touch -t`
    let _ = Command::new("touch")
        .arg("-t")
        .arg("202001010000")
        .arg(&corrupted_archive)
        .status();
    if let Ok(meta) = fs::metadata(&corrupted_archive) {
        if let Ok(mtime) = meta.modified() {
            if mtime < SystemTime::now() - Duration::from_secs(31 * 86_400) {
                let pruned = service.prune_old_backups(30).unwrap();
                assert_eq!(pruned, 1);
                assert!(!corrupted_archive.exists());
            }
        }
    }
}

#[tokio::test]
async fn test_operations_and_settings_endpoints() {
    let temp = tempfile::tempdir().unwrap();
    let media_dir = temp.path().join("media");
    let backup_dir = temp.path().join("backups");

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Initial Env Title".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        living_person_age_threshold: 110,
        media_storage_path: media_dir.to_string_lossy().to_string(),
        backup_path: backup_dir.to_string_lossy().to_string(),
        settings_locked_from_env: false,
        ..AppConfig::default()
    };

    let state = AppState::new(None, config);
    let (setup_res, _) = genealogy_backend::services::auth::AuthService::setup_initial_admin(
        &state,
        genealogy_backend::services::auth::SetupRequest {
            tree_title: None,
            first_name: "Ops".to_string(),
            last_name: "Admin".to_string(),
            email: "ops-admin@levshits.family".to_string(),
            passkey: None,
            password: Some("StrongOpsPassword123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", setup_res.session_token);
    let app = create_router(state);

    // 1. GET /api/v1/system/info returns initial env defaults
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let info: SystemInfoResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(info.tree_title, "Initial Env Title");
    assert_eq!(info.default_locale, "en");
    assert!(!info.enable_public_access);
    assert!(info.auth_password_enabled);
    assert!(!info.settings_locked_from_env);

    // 2. PATCH /api/v1/admin/settings updates settings and wins over initial env defaults
    let patch_body = serde_json::json!({
        "tree_title": "Updated Family Archive",
        "default_locale": "ru",
        "enable_public_access": true,
        "living_person_age_threshold": 100
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(patch_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let updated: InstanceSettingsDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated.tree_title, "Updated Family Archive");
    assert_eq!(updated.default_locale, "ru");
    assert!(updated.enable_public_access);
    assert_eq!(updated.living_person_age_threshold, 100);

    // Confirm GET /api/v1/system/info reflects the UI-saved settings
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let info_after: SystemInfoResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(info_after.tree_title, "Updated Family Archive");
    assert_eq!(info_after.default_locale, "ru");
    assert!(info_after.enable_public_access);

    // 3. PATCH /api/v1/admin/settings with invalid locale / threshold returns 400 validation_failed
    let invalid_body = serde_json::json!({
        "default_locale": "de",
        "living_person_age_threshold": 250
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(invalid_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let err_env: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_env.error.code, "validation_failed");
    let field_errors = err_env.error.field_errors.unwrap();
    assert_eq!(
        field_errors.get("default_locale").map(String::as_str),
        Some("invalid_locale")
    );
    assert_eq!(
        field_errors
            .get("living_person_age_threshold")
            .map(String::as_str),
        Some("out_of_range")
    );

    // 4. POST /api/v1/admin/backups triggers an on-demand backup
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/admin/backups")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"include_gedcom":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let triggered: TriggerBackupResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(triggered.status, "done");
    assert!(triggered.archive.filename.starts_with("backup-"));

    // 5. GET /api/v1/admin/backups lists the created archive
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/backups")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let list: BackupListResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].filename, triggered.archive.filename);

    // 6. GET /api/v1/admin/backups/{filename} downloads the archive
    let download_uri = format!("/api/v1/admin/backups/{}", triggered.archive.filename);
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&download_uri)
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/gzip")
    );
    let archive_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(!archive_bytes.is_empty());

    // 7. Path traversal / invalid filename in GET /api/v1/admin/backups/{filename} is rejected with 400
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/backups/..%2Fetc%2Fpasswd")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 7b. Valid filename format that does not exist returns 404 not_found with standard error envelope
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/backups/backup-19990101-000000-000.tar.gz")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let err_404: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_404.error.code, "not_found");

    // 8. GET /api/v1/admin/system returns diagnostics including backup count
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/system")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let sys_diag: AdminSystemResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(sys_diag.backups.total_backups, 1);
    assert_eq!(sys_diag.backups.last_backup_status, "ok");
}

#[tokio::test]
async fn test_settings_locked_from_env_blocks_mutation() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Locked Env Title".to_string(),
        settings_locked_from_env: true,
        ..AppConfig::default()
    };

    let state = AppState::new(None, config);
    let (setup_res, _) = genealogy_backend::services::auth::AuthService::setup_initial_admin(
        &state,
        genealogy_backend::services::auth::SetupRequest {
            tree_title: None,
            first_name: "Ops".to_string(),
            last_name: "Admin".to_string(),
            email: "ops-admin@levshits.family".to_string(),
            passkey: None,
            password: Some("StrongOpsPassword123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", setup_res.session_token);
    let app = create_router(state);

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"tree_title":"Attempted Override"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(res.into_body(), 1024 * 64).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "settings_locked_from_env");
}

#[test]
fn test_genealogy_cli_binary_subcommands() {
    let cli_bin = env!("CARGO_BIN_EXE_genealogy-cli");
    let temp = tempfile::tempdir().unwrap();
    let media_dir = temp.path().join("media");
    let backup_dir = temp.path().join("backups");
    let gedcom_out = temp.path().join("exported.gdz");

    // 1. backup --include-gedcom
    let out = Command::new(cli_bin)
        .arg("backup")
        .arg("--include-gedcom")
        .env("MEDIA_STORAGE_PATH", &media_dir)
        .env("BACKUP_PATH", &backup_dir)
        .env("DATABASE_REQUIRED", "false")
        .output()
        .expect("Failed to run genealogy-cli backup");
    assert!(
        out.status.success(),
        "CLI backup failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Backup created:"));

    // Find created archive
    let archive_entry = fs::read_dir(&backup_dir)
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().ends_with(".tar.gz"))
        .expect("Archive file must exist");

    // 2. restore <archive>
    let out = Command::new(cli_bin)
        .arg("restore")
        .arg(archive_entry.path())
        .env("MEDIA_STORAGE_PATH", &media_dir)
        .env("BACKUP_PATH", &backup_dir)
        .env("DATABASE_REQUIRED", "false")
        .output()
        .expect("Failed to run genealogy-cli restore");
    assert!(
        out.status.success(),
        "CLI restore failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("Restore completed successfully"));

    // 3. create-admin-link <email>
    let out = Command::new(cli_bin)
        .arg("create-admin-link")
        .arg("owner@example.com")
        .env("APP_ORIGIN", "https://tree.example.com")
        .env("DATABASE_REQUIRED", "false")
        .output()
        .expect("Failed to run genealogy-cli create-admin-link");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout)
        .contains("https://tree.example.com/setup?emergency_token="));

    // 4. regenerate-thumbnails
    let out = Command::new(cli_bin)
        .arg("regenerate-thumbnails")
        .env("MEDIA_STORAGE_PATH", &media_dir)
        .env("DATABASE_REQUIRED", "false")
        .output()
        .expect("Failed to run genealogy-cli regenerate-thumbnails");
    assert!(out.status.success());
    assert!(media_dir.join("thumbs").exists());

    // 5. export-gedcom <output.gdz>
    let out = Command::new(cli_bin)
        .arg("export-gedcom")
        .arg(&gedcom_out)
        .env("TREE_TITLE", "CLI Export Tree")
        .env("DATABASE_REQUIRED", "false")
        .output()
        .expect("Failed to run genealogy-cli export-gedcom");
    assert!(out.status.success());
    assert!(gedcom_out.exists());
    let gdz_text = fs::read_to_string(&gedcom_out).unwrap();
    assert!(gdz_text.contains("0 HEAD"));
    assert!(gdz_text.contains("CLI Export Tree"));
}

#[test]
fn test_docker_compose_and_non_root_dockerfiles_verification() {
    let compose = include_str!("../../docker-compose.yml");
    assert!(compose.contains("frontend:"));
    assert!(compose.contains("backend:"));
    assert!(compose.contains("postgres:"));
    assert!(compose.contains("condition: service_healthy"));
    assert!(compose.contains("postgres-data:"));
    assert!(compose.contains("media-data:"));
    assert!(compose.contains("backup-data:"));

    let backend_dockerfile = include_str!("../Dockerfile");
    assert!(backend_dockerfile.contains("USER 10001:10001"));
    assert!(backend_dockerfile.contains("genealogy-cli"));

    let frontend_dockerfile = include_str!("../../frontend/Dockerfile");
    assert!(frontend_dockerfile.contains("USER 10001:10001"));
}
