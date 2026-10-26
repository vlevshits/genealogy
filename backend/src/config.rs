use serde::Deserialize;
use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub app_env: String,
    pub rust_log: String,
    pub app_origin: String,
    pub rp_id: String,
    pub rp_name: String,
    pub tree_title: String,
    pub default_locale: String,
    pub database_url: String,
    pub media_storage_path: String,
    pub backup_path: String,
    pub enable_public_access: bool,
    pub enable_registration: bool,
    pub living_person_age_threshold: u32,
    pub max_upload_size_mb: usize,
    pub auth_password_enabled: String,
    pub session_ttl_days: u32,
    pub job_workers: usize,
    pub map_tile_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid PORT: {e}"))?;

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info,genealogy=debug".to_string());
        let app_origin =
            env::var("APP_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let rp_id = env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rp_name = env::var("RP_NAME").unwrap_or_else(|_| "Family Tree Archive".to_string());
        let tree_title =
            env::var("TREE_TITLE").unwrap_or_else(|_| "Family Tree Archive".to_string());
        let default_locale = env::var("DEFAULT_LOCALE").unwrap_or_else(|_| "en".to_string());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "genealogy".to_string());
            let pass = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "genealogy".to_string());
            let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "genealogy".to_string());
            format!("postgresql://{user}:{pass}@postgres:5432/{db}?sslmode=disable")
        });

        let media_storage_path =
            env::var("MEDIA_STORAGE_PATH").unwrap_or_else(|_| "/app/data/media".to_string());
        let backup_path =
            env::var("BACKUP_PATH").unwrap_or_else(|_| "/app/data/backups".to_string());

        let enable_public_access = env::var("ENABLE_PUBLIC_ACCESS")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let enable_registration = env::var("ENABLE_REGISTRATION")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let living_person_age_threshold = env::var("LIVING_PERSON_AGE_THRESHOLD")
            .unwrap_or_else(|_| "110".to_string())
            .parse::<u32>()
            .unwrap_or(110);

        let max_upload_size_mb = env::var("MAX_UPLOAD_SIZE_MB")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);

        let auth_password_enabled =
            env::var("AUTH_PASSWORD_ENABLED").unwrap_or_else(|_| "auto".to_string());

        let session_ttl_days = env::var("SESSION_TTL_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u32>()
            .unwrap_or(30);

        let job_workers = env::var("JOB_WORKERS")
            .unwrap_or_else(|_| "2".to_string())
            .parse::<usize>()
            .unwrap_or(2);

        let map_tile_url = env::var("MAP_TILE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Ok(Self {
            port,
            app_env,
            rust_log,
            app_origin,
            rp_id,
            rp_name,
            tree_title,
            default_locale,
            database_url,
            media_storage_path,
            backup_path,
            enable_public_access,
            enable_registration,
            living_person_age_threshold,
            max_upload_size_mb,
            auth_password_enabled,
            session_ttl_days,
            job_workers,
            map_tile_url,
        })
    }
}
