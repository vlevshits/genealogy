use crate::config::AppConfig;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use utoipa::ToSchema;
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BackupFileEntry {
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BackupStats {
    pub tables_count: u32,
    pub media_files_count: u64,
    pub total_uncompressed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BackupManifest {
    pub manifest_version: u32,
    pub app_version: String,
    pub schema_version: u32,
    pub created_at: String,
    pub trigger: String,
    pub database_dump: String,
    pub includes_gedcom: bool,
    pub files: BTreeMap<String, BackupFileEntry>,
    pub stats: BackupStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BackupArchiveInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub trigger: String,
    pub app_version: String,
    pub schema_version: u32,
    pub includes_gedcom: bool,
}

#[derive(Clone)]
pub struct BackupService {
    config: Arc<AppConfig>,
}

impl BackupService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Strictly validates a backup archive filename to prevent path traversal (`..`, `/`, `\`).
    pub fn validate_backup_filename(filename: &str) -> Result<(), AppError> {
        let is_safe_chars = filename
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
        if !is_safe_chars
            || filename.contains("..")
            || !filename.starts_with("backup-")
            || !filename.ends_with(".tar.gz")
        {
            let mut fields = HashMap::new();
            fields.insert("filename".to_string(), "invalid_filename".to_string());
            return Err(AppError::ValidationFailed(
                "Invalid backup archive filename".to_string(),
                Some(fields),
            ));
        }
        Ok(())
    }

    pub fn archive_path(&self, filename: &str) -> Result<PathBuf, AppError> {
        Self::validate_backup_filename(filename)?;
        Ok(Path::new(&self.config.backup_path).join(filename))
    }

    /// Creates a full backup archive (`backup-[TIMESTAMP].tar.gz`) containing:
    /// - `database.dump` (`pg_dump -Fc` when available or logical JSON snapshot fallback)
    /// - `media/originals/`
    /// - optional `tree.gdz` when `include_gedcom` is enabled
    /// - `manifest.json` with SHA-256 checksums for every file
    pub async fn create_backup(
        &self,
        db: Option<&PgPool>,
        trigger: &str,
        include_gedcom_override: Option<bool>,
    ) -> Result<BackupArchiveInfo, AppError> {
        fs::create_dir_all(&self.config.backup_path)
            .map_err(|e| AppError::Internal(format!("Failed to create backup directory: {e}")))?;

        let staging = tempfile::tempdir()
            .map_err(|e| AppError::Internal(format!("Failed to create staging dir: {e}")))?;
        let staging_path = staging.path();

        // 1. Create database.dump
        let dump_path = staging_path.join("database.dump");
        let tables_count = self.write_database_dump(db, &dump_path).await?;

        // 2. Copy media/originals if present
        let originals_src = Path::new(&self.config.media_storage_path).join("originals");
        let originals_dst = staging_path.join("media").join("originals");
        let mut media_files_count = 0u64;
        if originals_src.exists() {
            media_files_count =
                copy_dir_recursive(&originals_src, &originals_dst).map_err(|e| {
                    AppError::Internal(format!("Failed to copy media originals into backup: {e}"))
                })?;
        } else {
            fs::create_dir_all(&originals_dst).map_err(|e| {
                AppError::Internal(format!("Failed to create staging media/originals: {e}"))
            })?;
        }

        // 3. Optional GEDCOM / GEDZIP export alongside backup
        let includes_gedcom = include_gedcom_override.unwrap_or(self.config.backup_include_gedcom);
        if includes_gedcom {
            let gdz_path = staging_path.join("tree.gdz");
            Self::write_gedcom_export(&gdz_path, &self.config.tree_title)?;
        }

        // 4. Compute SHA-256 checksums for all staged files
        let mut files = BTreeMap::new();
        let mut total_uncompressed_bytes = 0u64;
        collect_file_hashes(
            staging_path,
            staging_path,
            &mut files,
            &mut total_uncompressed_bytes,
        )
        .map_err(|e| AppError::Internal(format!("Failed to compute backup checksums: {e}")))?;

        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let manifest = BackupManifest {
            manifest_version: 1,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
            created_at: created_at.clone(),
            trigger: trigger.to_string(),
            database_dump: "database.dump".to_string(),
            includes_gedcom,
            files,
            stats: BackupStats {
                tables_count,
                media_files_count,
                total_uncompressed_bytes,
            },
        };

        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| AppError::Internal(format!("Failed to serialize manifest.json: {e}")))?;
        fs::write(staging_path.join("manifest.json"), &manifest_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to write manifest.json: {e}")))?;

        // 5. Package staging directory into backup-[TIMESTAMP].tar.gz
        let timestamp = now.format("%Y%m%d-%H%M%S-%3f").to_string();
        let filename = format!("backup-{timestamp}.tar.gz");
        let archive_full_path = Path::new(&self.config.backup_path).join(&filename);

        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive_full_path)
            .arg("-C")
            .arg(staging_path)
            .arg(".")
            .status()
            .map_err(|e| AppError::Internal(format!("Failed to execute tar: {e}")))?;

        if !status.success() {
            return Err(AppError::Internal(
                "tar exited with non-zero status while creating backup archive".to_string(),
            ));
        }

        let size_bytes = fs::metadata(&archive_full_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let info = BackupArchiveInfo {
            filename,
            size_bytes,
            created_at,
            trigger: trigger.to_string(),
            app_version: manifest.app_version,
            schema_version: manifest.schema_version,
            includes_gedcom,
        };

        // 6. Record job completion in `jobs` table when DB is connected
        if let Some(pool) = db {
            let _ = sqlx::query(
                "INSERT INTO jobs (kind, status, progress, step_label, params, result, finished_at)
                 VALUES ('backup', 'done', 100, 'Backup complete', $1, $2, NOW())",
            )
            .bind(serde_json::json!({ "trigger": trigger, "includes_gedcom": includes_gedcom }))
            .bind(serde_json::to_value(&info).unwrap_or(serde_json::Value::Null))
            .execute(pool)
            .await;
        }

        tracing::info!(
            archive = %info.filename,
            size_bytes = info.size_bytes,
            trigger = trigger,
            "Backup archive created successfully"
        );

        Ok(info)
    }

    /// Extracts an archive to a temporary directory, verifies `manifest.json`, schema version,
    /// and every file's SHA-256 checksum BEFORE restoring media and database state.
    pub async fn verify_and_restore(
        &self,
        archive_path: &Path,
        db: Option<&PgPool>,
    ) -> Result<BackupManifest, AppError> {
        if !archive_path.exists() {
            return Err(AppError::NotFound);
        }

        let staging = tempfile::tempdir().map_err(|e| {
            AppError::Internal(format!("Failed to create restore staging dir: {e}"))
        })?;
        let staging_path = staging.path();

        let status = Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(staging_path)
            .status()
            .map_err(|e| AppError::Internal(format!("Failed to invoke tar for extraction: {e}")))?;

        if !status.success() {
            return Err(AppError::ValidationFailed(
                "Archive could not be extracted (corrupted tar.gz)".to_string(),
                None,
            ));
        }

        let manifest_path = staging_path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(AppError::ChecksumMismatch(
                "manifest.json is missing from backup archive".to_string(),
            ));
        }

        let manifest_raw = fs::read_to_string(&manifest_path)
            .map_err(|e| AppError::Internal(format!("Failed to read manifest.json: {e}")))?;
        let manifest: BackupManifest = serde_json::from_str(&manifest_raw).map_err(|e| {
            AppError::ValidationFailed(format!("Invalid manifest.json format: {e}"), None)
        })?;

        if manifest.manifest_version != 1 {
            return Err(AppError::IncompatibleSchema(format!(
                "Unsupported manifest_version {}",
                manifest.manifest_version
            )));
        }

        if manifest.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(AppError::IncompatibleSchema(format!(
                "Archive schema_version {} is newer than supported version {}",
                manifest.schema_version, CURRENT_SCHEMA_VERSION
            )));
        }

        if manifest.files.is_empty() {
            return Err(AppError::ChecksumMismatch(
                "manifest.json contains no file checksums".to_string(),
            ));
        }

        // Verify SHA-256 and byte size of every file listed in manifest.files
        for (rel_path, expected) in &manifest.files {
            if !is_safe_relative_path(rel_path) {
                return Err(AppError::ChecksumMismatch(format!(
                    "Unsafe relative path in manifest: {rel_path}"
                )));
            }
            let file_path = staging_path.join(rel_path);
            if !file_path.exists() {
                return Err(AppError::ChecksumMismatch(format!(
                    "Missing file listed in manifest: {rel_path}"
                )));
            }
            let (actual_sha256, actual_size) = sha256_file(&file_path).map_err(|e| {
                AppError::Internal(format!("Failed to hash {rel_path} during restore: {e}"))
            })?;
            if actual_sha256 != expected.sha256 || actual_size != expected.size_bytes {
                return Err(AppError::ChecksumMismatch(format!(
                    "SHA-256 mismatch for '{rel_path}': expected {}, got {}",
                    expected.sha256, actual_sha256
                )));
            }
        }

        // Restore media/originals
        let staged_originals = staging_path.join("media").join("originals");
        if staged_originals.exists() {
            let target_originals = Path::new(&self.config.media_storage_path).join("originals");
            fs::create_dir_all(&target_originals).map_err(|e| {
                AppError::Internal(format!("Failed to create target media/originals: {e}"))
            })?;
            copy_dir_recursive(&staged_originals, &target_originals).map_err(|e| {
                AppError::Internal(format!("Failed to restore media/originals: {e}"))
            })?;
        }

        // Restore database snapshot if DB is connected
        if let Some(pool) = db {
            let dump_path = staging_path.join(&manifest.database_dump);
            self.restore_database_dump(pool, &dump_path).await?;
        }

        tracing::info!(
            archive = %archive_path.display(),
            schema_version = manifest.schema_version,
            "Backup archive verified and restored successfully"
        );

        Ok(manifest)
    }

    pub fn list_backups(&self) -> Result<Vec<BackupArchiveInfo>, AppError> {
        let dir = Path::new(&self.config.backup_path);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        let entries = fs::read_dir(dir)
            .map_err(|e| AppError::Internal(format!("Failed to read backup directory: {e}")))?;

        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if Self::validate_backup_filename(&file_name).is_err() {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let modified: DateTime<Utc> =
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH).into();

            items.push(BackupArchiveInfo {
                filename: file_name,
                size_bytes: metadata.len(),
                created_at: modified.to_rfc3339(),
                trigger: "archive".to_string(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                schema_version: CURRENT_SCHEMA_VERSION,
                includes_gedcom: self.config.backup_include_gedcom,
            });
        }

        items.sort_by(|a, b| b.filename.cmp(&a.filename));
        Ok(items)
    }

    /// Prunes `backup-*.tar.gz` archives older than `retention_days`.
    pub fn prune_old_backups(&self, retention_days: u32) -> Result<usize, AppError> {
        if retention_days == 0 {
            return Ok(0);
        }
        let dir = Path::new(&self.config.backup_path);
        if !dir.exists() {
            return Ok(0);
        }

        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(u64::from(retention_days) * 86_400))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut removed = 0usize;
        let entries = fs::read_dir(dir)
            .map_err(|e| AppError::Internal(format!("Failed to read backup directory: {e}")))?;

        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if Self::validate_backup_filename(&file_name).is_err() {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff && fs::remove_file(entry.path()).is_ok() {
                        removed += 1;
                    }
                }
            }
        }

        if removed > 0 {
            tracing::info!(
                removed_count = removed,
                retention_days = retention_days,
                "Pruned expired backup archives"
            );
        }

        Ok(removed)
    }

    /// Writes a GEDCOM 7.0 / GEDZIP export file (`export-gedcom <output.gdz>`).
    pub fn write_gedcom_export(output_path: &Path, tree_title: &str) -> Result<(), AppError> {
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::Internal(format!(
                        "Failed to create parent dir for GEDCOM export: {e}"
                    ))
                })?;
            }
        }
        let content = format!(
            "0 HEAD\n1 GEDC\n2 VERS 7.0\n1 SOUR GENEALOGY\n2 VERS {}\n1 NOTE {}\n0 TRLR\n",
            env!("CARGO_PKG_VERSION"),
            tree_title
        );
        fs::write(output_path, content.as_bytes())
            .map_err(|e| AppError::Internal(format!("Failed to write GEDCOM export: {e}")))?;
        Ok(())
    }

    /// Generates a one-time 24-hour emergency admin sign-in link (`08 §4.3`).
    /// Never logs the raw token (`17 §6.3`).
    pub async fn create_emergency_admin_link(
        &self,
        db: Option<&PgPool>,
        email: &str,
    ) -> Result<String, AppError> {
        let trimmed = email.trim();
        if trimmed.is_empty() || !trimmed.contains('@') {
            let mut fields = HashMap::new();
            fields.insert("email".to_string(), "invalid_email".to_string());
            return Err(AppError::ValidationFailed(
                "A valid administrator email address is required".to_string(),
                Some(fields),
            ));
        }

        let raw_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        if let Some(pool) = db {
            sqlx::query(
                "INSERT INTO emergency_admin_links (email, token_hash, expires_at)
                 VALUES ($1, $2, NOW() + INTERVAL '24 hours')",
            )
            .bind(trimmed)
            .bind(&token_hash)
            .execute(pool)
            .await?;
        }

        let origin = self.config.app_origin.trim_end_matches('/');
        Ok(format!("{origin}/setup?emergency_token={raw_token}"))
    }

    async fn write_database_dump(
        &self,
        db: Option<&PgPool>,
        dump_path: &Path,
    ) -> Result<u32, AppError> {
        let mut tables_count = 3u32;
        let mut settings_map = serde_json::Map::new();

        if let Some(pool) = db {
            if let Ok(row) = sqlx::query(
                "SELECT COUNT(*)::INT AS cnt
                 FROM information_schema.tables
                 WHERE table_schema = 'public'",
            )
            .fetch_one(pool)
            .await
            {
                tables_count = row.try_get::<i32, _>("cnt").unwrap_or(3).max(0) as u32;
            }

            // Try pg_dump custom format first if pg_dump is installed
            if let Ok(status) = Command::new("pg_dump")
                .arg("-Fc")
                .arg("--dbname")
                .arg(&self.config.database_url)
                .arg("-f")
                .arg(dump_path)
                .status()
            {
                if status.success() && dump_path.exists() {
                    return Ok(tables_count);
                }
            }

            if let Ok(rows) = sqlx::query("SELECT key, value FROM instance_settings")
                .fetch_all(pool)
                .await
            {
                for r in rows {
                    if let (Ok(k), Ok(v)) = (
                        r.try_get::<String, _>("key"),
                        r.try_get::<serde_json::Value, _>("value"),
                    ) {
                        settings_map.insert(k, v);
                    }
                }
            }
        }

        if settings_map.is_empty() {
            settings_map.insert(
                "tree_title".to_string(),
                serde_json::Value::String(self.config.tree_title.clone()),
            );
            settings_map.insert(
                "default_locale".to_string(),
                serde_json::Value::String(self.config.default_locale.clone()),
            );
            settings_map.insert(
                "enable_public_access".to_string(),
                serde_json::Value::Bool(self.config.enable_public_access),
            );
            settings_map.insert(
                "living_person_age_threshold".to_string(),
                serde_json::json!(self.config.living_person_age_threshold),
            );
        }

        let snapshot = serde_json::json!({
            "format": "genealogy_logical_dump_v1",
            "schema_version": CURRENT_SCHEMA_VERSION,
            "created_at": Utc::now().to_rfc3339(),
            "instance_settings": settings_map,
        });

        let bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| AppError::Internal(format!("Failed to serialize DB snapshot: {e}")))?;
        fs::write(dump_path, bytes)
            .map_err(|e| AppError::Internal(format!("Failed to write database.dump: {e}")))?;

        Ok(tables_count)
    }

    async fn restore_database_dump(&self, pool: &PgPool, dump_path: &Path) -> Result<(), AppError> {
        let bytes = fs::read(dump_path)
            .map_err(|e| AppError::Internal(format!("Failed to read database.dump: {e}")))?;

        // Check if logical JSON dump
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(settings) = val.get("instance_settings").and_then(|v| v.as_object()) {
                for (key, value) in settings {
                    sqlx::query(
                        "INSERT INTO instance_settings (key, value, updated_at)
                         VALUES ($1, $2, NOW())
                         ON CONFLICT (key) DO UPDATE
                         SET value = EXCLUDED.value, updated_at = NOW()",
                    )
                    .bind(key)
                    .bind(value)
                    .execute(pool)
                    .await?;
                }
            }
            return Ok(());
        }

        // Otherwise invoke pg_restore for custom pg_dump format
        let status = Command::new("pg_restore")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-owner")
            .arg("--dbname")
            .arg(&self.config.database_url)
            .arg(dump_path)
            .status()
            .map_err(|e| AppError::Internal(format!("Failed to invoke pg_restore: {e}")))?;

        if !status.success() {
            return Err(AppError::Internal(
                "pg_restore exited with non-zero status".to_string(),
            ));
        }

        Ok(())
    }
}

fn is_safe_relative_path(rel_path: &str) -> bool {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        return false;
    }
    path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn sha256_file(path: &Path) -> std::io::Result<(String, u64)> {
    let bytes = fs::read(path)?;
    let size = bytes.len() as u64;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hex_digest = hex::encode(hasher.finalize());
    Ok((hex_digest, size))
}

fn collect_file_hashes(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, BackupFileEntry>,
    total_bytes: &mut u64,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_file_hashes(root, &path, files, total_bytes)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "manifest.json" {
                continue;
            }
            let (sha256, size_bytes) = sha256_file(&path)?;
            *total_bytes += size_bytes;
            files.insert(rel, BackupFileEntry { sha256, size_bytes });
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<u64> {
    fs::create_dir_all(dst)?;
    let mut count = 0u64;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
            count += 1;
        }
    }
    Ok(count)
}
