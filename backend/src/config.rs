use serde::Deserialize;
use std::collections::HashMap;
use std::env;

/// Exhaustive single-instance configuration loaded from environment variables
/// (`docs/features/08-deployment-single-instance.md §3`).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // 3.1 Web & Server Configuration
    pub port: u16,
    pub app_env: String,
    pub rust_log: String,

    // 3.2 Domain & Passkeys (WebAuthn / FIDO2)
    pub app_origin: String,
    pub rp_id: String,
    pub rp_name: String,

    // 3.3 Database & Cache
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub database_url: String,
    pub database_required: bool,

    // 3.4 Media & Storage Settings
    pub max_upload_size_mb: usize,
    pub thumbnail_quality: u8,
    pub media_storage_path: String,

    // 3.5 Privacy, Access & Internationalization
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub enable_registration: bool,
    pub living_person_age_threshold: u32,

    // 3.6 Transactional Email (SMTP)
    pub smtp_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,

    // 3.7 Automated Backups & Disaster Recovery
    pub backup_enabled: bool,
    pub backup_cron: String,
    pub backup_retention_days: u32,
    pub backup_include_gedcom: bool,
    pub backup_path: String,

    // 3.8 Authentication, Sessions & Limits
    pub auth_password_enabled: String,
    pub session_ttl_days: u32,
    pub max_import_size_mb: usize,
    pub job_workers: usize,

    // 3.9 Maps, History & Settings
    pub map_tile_url: Option<String>,
    pub map_attribution: String,
    pub geocoding_enabled: bool,
    pub geocoding_url: String,
    pub history_retention_days: u32,
    pub settings_locked_from_env: bool,
    pub public_search_show_living: bool,

    // 3.10 AI & Multimodal Story Intake (Gemini)
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,

    // 3.11 Frontend Static Serving (Standalone / LXC)
    pub frontend_static_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_map(&HashMap::new()).expect("Default AppConfig must be valid")
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let vars: HashMap<String, String> = env::vars().collect();
        Self::from_map(&vars)
    }

    pub fn from_map(vars: &HashMap<String, String>) -> Result<Self, String> {
        let get = |key: &str| -> Option<String> {
            vars.get(key)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };

        let parse_bool = |key: &str, default: bool| -> bool {
            match get(key) {
                Some(v) => matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes"),
                None => default,
            }
        };

        // 3.1 Web & Server
        let port = match get("PORT") {
            Some(v) => v
                .parse::<u16>()
                .map_err(|e| format!("Invalid PORT '{v}': {e}"))?,
            None => 3000,
        };
        let app_env = get("APP_ENV").unwrap_or_else(|| "development".to_string());
        let rust_log = get("RUST_LOG").unwrap_or_else(|| "info,genealogy=debug".to_string());

        // 3.2 Domain & Passkeys
        let app_origin = get("APP_ORIGIN").unwrap_or_else(|| "http://localhost:8080".to_string());
        let rp_id = get("RP_ID").unwrap_or_else(|| "localhost".to_string());
        let rp_name = get("RP_NAME").unwrap_or_else(|| "Family Tree Archive".to_string());

        // 3.3 Database
        let postgres_user = get("POSTGRES_USER").unwrap_or_else(|| "genealogy".to_string());
        let postgres_password = get("POSTGRES_PASSWORD").unwrap_or_else(|| "genealogy".to_string());
        let postgres_db = get("POSTGRES_DB").unwrap_or_else(|| "genealogy".to_string());
        let database_url = get("DATABASE_URL").unwrap_or_else(|| {
            format!(
                "postgresql://{postgres_user}:{postgres_password}@postgres:5432/{postgres_db}?sslmode=disable"
            )
        });
        let database_required = parse_bool(
            "DATABASE_REQUIRED",
            app_env.eq_ignore_ascii_case("production"),
        );

        // 3.4 Media & Storage
        let max_upload_size_mb = get("MAX_UPLOAD_SIZE_MB")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(100);
        let thumbnail_quality = get("THUMBNAIL_QUALITY")
            .and_then(|v| v.parse::<u8>().ok())
            .map(|q| q.clamp(1, 100))
            .unwrap_or(85);
        let media_storage_path =
            get("MEDIA_STORAGE_PATH").unwrap_or_else(|| "/app/data/media".to_string());

        // 3.5 Privacy, Access & Internationalization
        let tree_title = get("TREE_TITLE").unwrap_or_else(|| "Family Tree Archive".to_string());
        let default_locale = get("DEFAULT_LOCALE").unwrap_or_else(|| "en".to_string());
        let enable_public_access = parse_bool("ENABLE_PUBLIC_ACCESS", false);
        let enable_registration = parse_bool("ENABLE_REGISTRATION", false);
        let living_person_age_threshold = get("LIVING_PERSON_AGE_THRESHOLD")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(110);

        // 3.6 Transactional Email (SMTP)
        let smtp_enabled = parse_bool("SMTP_ENABLED", false);
        let smtp_host = get("SMTP_HOST").unwrap_or_default();
        let smtp_port = get("SMTP_PORT")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(587);
        let smtp_user = get("SMTP_USER").unwrap_or_default();
        let smtp_password = get("SMTP_PASSWORD").unwrap_or_default();
        let smtp_from_email = get("SMTP_FROM_EMAIL").unwrap_or_default();
        let smtp_from_name = get("SMTP_FROM_NAME").unwrap_or_else(|| "Family Tree".to_string());

        // 3.7 Automated Backups & Disaster Recovery
        let backup_enabled = parse_bool("BACKUP_ENABLED", true);
        let backup_cron = get("BACKUP_CRON").unwrap_or_else(|| "0 3 * * *".to_string());
        let backup_retention_days = get("BACKUP_RETENTION_DAYS")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(30);
        let backup_include_gedcom = parse_bool("BACKUP_INCLUDE_GEDCOM", false);
        let backup_path = get("BACKUP_PATH").unwrap_or_else(|| "/app/data/backups".to_string());

        // 3.8 Authentication, Sessions & Limits
        let auth_password_enabled =
            get("AUTH_PASSWORD_ENABLED").unwrap_or_else(|| "auto".to_string());
        let session_ttl_days = get("SESSION_TTL_DAYS")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(30);
        let max_import_size_mb = get("MAX_IMPORT_SIZE_MB")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2048);
        let job_workers = get("JOB_WORKERS")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2)
            .max(1);

        // 3.9 Maps, History & Settings
        let map_tile_url = match vars.get("MAP_TILE_URL") {
            Some(v) if v.trim().is_empty() => None,
            Some(v) => Some(v.trim().to_string()),
            None => Some("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png".to_string()),
        };
        let map_attribution =
            get("MAP_ATTRIBUTION").unwrap_or_else(|| "© OpenStreetMap contributors".to_string());
        let geocoding_enabled = parse_bool("GEOCODING_ENABLED", false);
        let geocoding_url = get("GEOCODING_URL")
            .unwrap_or_else(|| "https://nominatim.openstreetmap.org".to_string());
        let history_retention_days = get("HISTORY_RETENTION_DAYS")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(365);
        let settings_locked_from_env = parse_bool("SETTINGS_LOCKED_FROM_ENV", false);
        let public_search_show_living = parse_bool("PUBLIC_SEARCH_SHOW_LIVING", false);

        // 3.10 AI & Multimodal Story Intake (Gemini)
        let gemini_api_key = get("GEMINI_API_KEY").or_else(|| get("VITE_GEMINI_API_KEY"));
        let gemini_model = get("GEMINI_MODEL").unwrap_or_else(|| "gemini-3.8-flash".to_string());

        // 3.11 Frontend Static Serving (Standalone / LXC)
        let frontend_static_path = get("FRONTEND_STATIC_PATH");

        Ok(Self {
            port,
            app_env,
            rust_log,
            app_origin,
            rp_id,
            rp_name,
            postgres_user,
            postgres_password,
            postgres_db,
            database_url,
            database_required,
            max_upload_size_mb,
            thumbnail_quality,
            media_storage_path,
            tree_title,
            default_locale,
            enable_public_access,
            enable_registration,
            living_person_age_threshold,
            smtp_enabled,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
            backup_enabled,
            backup_cron,
            backup_retention_days,
            backup_include_gedcom,
            backup_path,
            auth_password_enabled,
            session_ttl_days,
            max_import_size_mb,
            job_workers,
            map_tile_url,
            map_attribution,
            geocoding_enabled,
            geocoding_url,
            history_retention_days,
            settings_locked_from_env,
            public_search_show_living,
            gemini_api_key,
            gemini_model,
            frontend_static_path,
        })
    }

    /// Evaluates `AUTH_PASSWORD_ENABLED` (`auto` | `true` | `false`) per `17-api-security-operations.md §3.5`:
    /// When `auto`, password login is enabled whenever the app is not served over HTTPS OR SMTP is disabled.
    pub fn is_password_auth_effectively_enabled(&self) -> bool {
        match self.auth_password_enabled.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => !self.app_origin.starts_with("https://") || !self.smtp_enabled,
        }
    }
}
