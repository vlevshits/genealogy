use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::{
    auth::AuthMemoryStore, privacy::GenealogyPrivacyStore, security::RateLimiter,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct InstanceSettingsDto {
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub living_person_age_threshold: u32,
    #[serde(default)]
    pub default_home_person_id: Option<String>,
    pub settings_locked_from_env: bool,
}

fn deserialize_double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateInstanceSettingsRequest {
    pub tree_title: Option<String>,
    pub default_locale: Option<String>,
    pub enable_public_access: Option<bool>,
    pub living_person_age_threshold: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    #[schema(value_type = Option<String>)]
    pub default_home_person_id: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct InstanceSettingsValues {
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub living_person_age_threshold: u32,
    pub default_home_person_id: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub config: Arc<AppConfig>,
    pub settings_override: Arc<RwLock<InstanceSettingsValues>>,
    pub auth_store: Arc<RwLock<AuthMemoryStore>>,
    pub privacy_store: Arc<RwLock<GenealogyPrivacyStore>>,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(db: Option<PgPool>, config: AppConfig) -> Self {
        let initial = InstanceSettingsValues {
            tree_title: config.tree_title.clone(),
            default_locale: config.default_locale.clone(),
            enable_public_access: config.enable_public_access,
            living_person_age_threshold: config.living_person_age_threshold,
            default_home_person_id: None,
        };
        Self {
            db,
            config: Arc::new(config),
            settings_override: Arc::new(RwLock::new(initial)),
            auth_store: Arc::new(RwLock::new(AuthMemoryStore::default())),
            privacy_store: Arc::new(RwLock::new(GenealogyPrivacyStore::empty())),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Creates an application state with an empty genealogy store (0 individuals) for onboarding scenarios.
    pub fn new_empty(db: Option<PgPool>, config: AppConfig) -> Self {
        Self::new(db, config)
    }

    /// Creates an application state pre-seeded with the demo genealogy fixture store (for RBAC/privacy tests).
    pub fn new_demo(db: Option<PgPool>, config: AppConfig) -> Self {
        let initial = InstanceSettingsValues {
            tree_title: config.tree_title.clone(),
            default_locale: config.default_locale.clone(),
            enable_public_access: config.enable_public_access,
            living_person_age_threshold: config.living_person_age_threshold,
            default_home_person_id: None,
        };
        Self {
            db,
            config: Arc::new(config),
            settings_override: Arc::new(RwLock::new(initial)),
            auth_store: Arc::new(RwLock::new(AuthMemoryStore::default())),
            privacy_store: Arc::new(RwLock::new(GenealogyPrivacyStore::demo())),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Resolves effective instance settings according to `docs/features/10-core-data-model.md §7`:
    /// - When `SETTINGS_LOCKED_FROM_ENV=true`, environment values always win.
    /// - Otherwise, values saved in `instance_settings` (or in-memory state when running standalone) win over initial `.env` defaults.
    pub async fn resolve_instance_settings(&self) -> InstanceSettingsDto {
        let mut current = self.settings_override.read().await.clone();

        if let Some(pool) = &self.db {
            if let Ok(rows) = sqlx::query("SELECT key, value FROM instance_settings")
                .fetch_all(pool)
                .await
            {
                for row in rows {
                    let key: String = row.try_get("key").unwrap_or_default();
                    let val: serde_json::Value =
                        row.try_get("value").unwrap_or(serde_json::Value::Null);
                    match key.as_str() {
                        "tree_title" => {
                            if let Some(s) = val.as_str() {
                                current.tree_title = s.to_string();
                            }
                        }
                        "default_locale" => {
                            if let Some(s) = val.as_str() {
                                current.default_locale = s.to_string();
                            }
                        }
                        "enable_public_access" => {
                            if let Some(b) = val.as_bool() {
                                current.enable_public_access = b;
                            }
                        }
                        "living_person_age_threshold" => {
                            if let Some(n) = val.as_u64() {
                                current.living_person_age_threshold = n as u32;
                            }
                        }
                        "default_home_person_id" => {
                            if val.is_null() {
                                current.default_home_person_id = None;
                            } else if let Some(s) = val.as_str() {
                                let trimmed = s.trim();
                                current.default_home_person_id = if trimmed.is_empty() {
                                    None
                                } else {
                                    Some(trimmed.to_string())
                                };
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.config.settings_locked_from_env {
            return InstanceSettingsDto {
                tree_title: self.config.tree_title.clone(),
                default_locale: self.config.default_locale.clone(),
                enable_public_access: self.config.enable_public_access,
                living_person_age_threshold: self.config.living_person_age_threshold,
                default_home_person_id: current.default_home_person_id,
                settings_locked_from_env: true,
            };
        }

        InstanceSettingsDto {
            tree_title: current.tree_title,
            default_locale: current.default_locale,
            enable_public_access: current.enable_public_access,
            living_person_age_threshold: current.living_person_age_threshold,
            default_home_person_id: current.default_home_person_id,
            settings_locked_from_env: false,
        }
    }

    pub async fn set_default_home_person_if_unset(&self, person_id: &str) {
        let mut guard = self.settings_override.write().await;
        if guard.default_home_person_id.is_none() {
            guard.default_home_person_id = Some(person_id.to_string());
        }
        drop(guard);

        if let Some(pool) = &self.db {
            let _ = sqlx::query(
                "INSERT INTO instance_settings (key, value, updated_at)
                 VALUES ('default_home_person_id', $1, NOW())
                 ON CONFLICT (key) DO UPDATE
                 SET value = CASE
                     WHEN instance_settings.value IS NULL OR instance_settings.value = 'null'::jsonb
                     THEN EXCLUDED.value
                     ELSE instance_settings.value
                 END,
                 updated_at = NOW()",
            )
            .bind(serde_json::Value::String(person_id.to_string()))
            .execute(pool)
            .await;
        }
    }

    pub async fn update_instance_settings(
        &self,
        req: UpdateInstanceSettingsRequest,
    ) -> Result<InstanceSettingsDto, AppError> {
        if self.config.settings_locked_from_env {
            return Err(AppError::SettingsLockedFromEnv);
        }

        let mut field_errors = HashMap::new();

        if let Some(ref title) = req.tree_title {
            let trimmed = title.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 200 {
                field_errors.insert("tree_title".to_string(), "invalid_length".to_string());
            }
        }

        if let Some(ref locale) = req.default_locale {
            if !matches!(locale.as_str(), "en" | "ru") {
                field_errors.insert("default_locale".to_string(), "invalid_locale".to_string());
            }
        }

        if let Some(threshold) = req.living_person_age_threshold {
            if !(1..=150).contains(&threshold) {
                field_errors.insert(
                    "living_person_age_threshold".to_string(),
                    "out_of_range".to_string(),
                );
            }
        }

        let resolved_default_home_person = match req.default_home_person_id.as_ref() {
            Some(None) => Some(None),
            Some(Some(raw)) if raw.trim().is_empty() => Some(None),
            Some(Some(raw)) => {
                let target = raw.trim();
                let store = self.privacy_store.read().await;
                if let Some(p) = store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    Some(Some(p.id.clone()))
                } else if uuid::Uuid::parse_str(target).is_ok() {
                    Some(Some(target.to_string()))
                } else {
                    field_errors.insert(
                        "default_home_person_id".to_string(),
                        "not_found".to_string(),
                    );
                    None
                }
            }
            None => None,
        };

        if !field_errors.is_empty() {
            return Err(AppError::ValidationFailed(
                "Invalid instance settings".to_string(),
                Some(field_errors),
            ));
        }

        let mut guard = self.settings_override.write().await;
        if let Some(title) = req.tree_title {
            guard.tree_title = title.trim().to_string();
        }
        if let Some(locale) = req.default_locale {
            guard.default_locale = locale;
        }
        if let Some(public_access) = req.enable_public_access {
            guard.enable_public_access = public_access;
        }
        if let Some(threshold) = req.living_person_age_threshold {
            guard.living_person_age_threshold = threshold;
        }
        if let Some(dhp_opt) = resolved_default_home_person {
            guard.default_home_person_id = dhp_opt;
        }
        let updated = guard.clone();
        drop(guard);

        if let Some(pool) = &self.db {
            let dhp_json = match &updated.default_home_person_id {
                Some(id) => serde_json::Value::String(id.clone()),
                None => serde_json::Value::Null,
            };
            let entries: [(&str, serde_json::Value); 5] = [
                (
                    "tree_title",
                    serde_json::Value::String(updated.tree_title.clone()),
                ),
                (
                    "default_locale",
                    serde_json::Value::String(updated.default_locale.clone()),
                ),
                (
                    "enable_public_access",
                    serde_json::Value::Bool(updated.enable_public_access),
                ),
                (
                    "living_person_age_threshold",
                    serde_json::json!(updated.living_person_age_threshold),
                ),
                ("default_home_person_id", dhp_json),
            ];

            for (key, value) in entries {
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

        Ok(InstanceSettingsDto {
            tree_title: updated.tree_title,
            default_locale: updated.default_locale,
            enable_public_access: updated.enable_public_access,
            living_person_age_threshold: updated.living_person_age_threshold,
            default_home_person_id: updated.default_home_person_id,
            settings_locked_from_env: false,
        })
    }
}
