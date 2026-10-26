use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fs;
use std::path::Path as FsPath;
use std::process::Command;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    services::backup::{BackupArchiveInfo, BackupService},
    state::{AppState, InstanceSettingsDto, UpdateInstanceSettingsRequest},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemInfoResponse {
    pub app_version: String,
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub auth_password_enabled: bool,
    pub settings_locked_from_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminDatabaseStatus {
    pub status: String,
    pub size_bytes: u64,
    pub tables_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminStorageStatus {
    pub media_size_bytes: u64,
    pub media_files_count: u64,
    pub free_disk_bytes: u64,
    pub total_disk_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminBackupStatus {
    pub last_backup_at: Option<String>,
    pub last_backup_status: String,
    pub total_backups: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminJobsStatus {
    pub queue_length: u64,
    pub active_workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminSystemResponse {
    pub app_version: String,
    pub database: AdminDatabaseStatus,
    pub storage: AdminStorageStatus,
    pub backups: AdminBackupStatus,
    pub jobs: AdminJobsStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BackupListResponse {
    pub items: Vec<BackupArchiveInfo>,
    pub retention_days: u32,
    pub cron_schedule: String,
    pub backup_enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct TriggerBackupRequest {
    pub include_gedcom: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TriggerBackupResponse {
    pub job_id: String,
    pub status: String,
    pub archive: BackupArchiveInfo,
}

#[utoipa::path(
    get,
    path = "/api/v1/system/info",
    responses(
        (status = 200, description = "Public instance metadata", body = SystemInfoResponse)
    ),
    tag = "System"
)]
pub async fn get_system_info(State(state): State<AppState>) -> Json<SystemInfoResponse> {
    let settings = state.resolve_instance_settings().await;
    Json(SystemInfoResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tree_title: settings.tree_title,
        default_locale: settings.default_locale,
        enable_public_access: settings.enable_public_access,
        auth_password_enabled: state.config.is_password_auth_effectively_enabled(),
        settings_locked_from_env: settings.settings_locked_from_env,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/system",
    responses(
        (status = 200, description = "Operational diagnostics for Settings System card", body = AdminSystemResponse)
    ),
    tag = "Admin"
)]
pub async fn get_admin_system(State(state): State<AppState>) -> Json<AdminSystemResponse> {
    let mut db_status = "standalone_ready".to_string();
    let mut db_size_bytes = 0u64;
    let mut tables_count = 3u32;
    let mut queue_length = 0u64;

    if let Some(pool) = &state.db {
        match sqlx::query("SELECT pg_database_size(current_database()) AS db_size")
            .fetch_one(pool)
            .await
        {
            Ok(row) => {
                db_status = "connected".to_string();
                db_size_bytes = row.try_get::<i64, _>("db_size").unwrap_or(0).max(0) as u64;
            }
            Err(_) => {
                db_status = "error".to_string();
            }
        }

        if let Ok(row) = sqlx::query(
            "SELECT COUNT(*)::INT AS cnt FROM information_schema.tables WHERE table_schema = 'public'",
        )
        .fetch_one(pool)
        .await
        {
            tables_count = row.try_get::<i32, _>("cnt").unwrap_or(3).max(0) as u32;
        }

        if let Ok(row) = sqlx::query(
            "SELECT COUNT(*)::BIGINT AS cnt FROM jobs WHERE status IN ('queued', 'running')",
        )
        .fetch_one(pool)
        .await
        {
            queue_length = row.try_get::<i64, _>("cnt").unwrap_or(0).max(0) as u64;
        }
    }

    let media_path = FsPath::new(&state.config.media_storage_path);
    let (media_size_bytes, media_files_count) = compute_dir_usage(media_path);
    let (free_disk_bytes, total_disk_bytes) = query_disk_space(media_path);

    let backup_service = BackupService::new(state.config.clone());
    let backups = backup_service.list_backups().unwrap_or_default();
    let last_backup_at = backups.first().map(|b| b.created_at.clone());
    let last_backup_status = if backups.is_empty() {
        "none".to_string()
    } else {
        "ok".to_string()
    };

    Json(AdminSystemResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        database: AdminDatabaseStatus {
            status: db_status,
            size_bytes: db_size_bytes,
            tables_count,
        },
        storage: AdminStorageStatus {
            media_size_bytes,
            media_files_count,
            free_disk_bytes,
            total_disk_bytes,
        },
        backups: AdminBackupStatus {
            last_backup_at,
            last_backup_status,
            total_backups: backups.len(),
        },
        jobs: AdminJobsStatus {
            queue_length,
            active_workers: state.config.job_workers,
        },
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/settings",
    responses(
        (status = 200, description = "Current instance settings", body = InstanceSettingsDto)
    ),
    tag = "Admin"
)]
pub async fn get_admin_settings(State(state): State<AppState>) -> Json<InstanceSettingsDto> {
    Json(state.resolve_instance_settings().await)
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/settings",
    request_body = UpdateInstanceSettingsRequest,
    responses(
        (status = 200, description = "Updated instance settings", body = InstanceSettingsDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Settings locked by SETTINGS_LOCKED_FROM_ENV", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn update_admin_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateInstanceSettingsRequest>,
) -> Result<Json<InstanceSettingsDto>, AppError> {
    let updated = state.update_instance_settings(payload).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/backups",
    responses(
        (status = 200, description = "List of available backup archives", body = BackupListResponse)
    ),
    tag = "Admin"
)]
pub async fn list_admin_backups(
    State(state): State<AppState>,
) -> Result<Json<BackupListResponse>, AppError> {
    let backup_service = BackupService::new(state.config.clone());
    let items = backup_service.list_backups()?;
    Ok(Json(BackupListResponse {
        items,
        retention_days: state.config.backup_retention_days,
        cron_schedule: state.config.backup_cron.clone(),
        backup_enabled: state.config.backup_enabled,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/backups",
    request_body(content = Option<TriggerBackupRequest>, description = "Optional backup options"),
    responses(
        (status = 201, description = "On-demand backup created", body = TriggerBackupResponse),
        (status = 500, description = "Backup creation failed", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn trigger_admin_backup(
    State(state): State<AppState>,
    payload: Option<Json<TriggerBackupRequest>>,
) -> Result<(StatusCode, Json<TriggerBackupResponse>), AppError> {
    let include_gedcom = payload.and_then(|Json(p)| p.include_gedcom);
    let backup_service = BackupService::new(state.config.clone());
    let archive = backup_service
        .create_backup(state.db.as_ref(), "manual", include_gedcom)
        .await?;

    let response = TriggerBackupResponse {
        job_id: Uuid::new_v4().to_string(),
        status: "done".to_string(),
        archive,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/backups/{filename}",
    params(
        ("filename" = String, Path, description = "Backup archive filename (backup-YYYYMMDD-HHMMSS.tar.gz)")
    ),
    responses(
        (status = 200, description = "Gzipped tar backup archive stream", content_type = "application/gzip"),
        (status = 400, description = "Invalid backup filename", body = ErrorResponse),
        (status = 404, description = "Backup archive not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn download_admin_backup(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response, AppError> {
    let backup_service = BackupService::new(state.config.clone());
    let archive_path = backup_service.archive_path(&filename)?;

    if !archive_path.exists() {
        return Err(AppError::NotFound);
    }

    let bytes = fs::read(&archive_path)
        .map_err(|e| AppError::Internal(format!("Failed to read backup archive: {e}")))?;

    let disposition = format!("attachment; filename=\"{filename}\"");
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build download response: {e}")))?;

    Ok(response.into_response())
}

fn compute_dir_usage(dir: &FsPath) -> (u64, u64) {
    if !dir.exists() {
        return (0, 0);
    }
    let mut size = 0u64;
    let mut count = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (sub_size, sub_count) = compute_dir_usage(&path);
                size += sub_size;
                count += sub_count;
            } else if let Ok(meta) = entry.metadata() {
                size += meta.len();
                count += 1;
            }
        }
    }
    (size, count)
}

fn query_disk_space(target: &FsPath) -> (u64, u64) {
    let check_path = if target.exists() {
        target
    } else {
        FsPath::new(".")
    };

    if let Ok(output) = Command::new("df").arg("-k").arg(check_path).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 4 {
                    let total_kb = cols[1].parse::<u64>().unwrap_or(0);
                    let avail_kb = cols[3].parse::<u64>().unwrap_or(0);
                    return (avail_kb * 1024, total_kb * 1024);
                }
            }
        }
    }
    (0, 0)
}
