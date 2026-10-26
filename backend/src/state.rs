use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::{auth::AuthMemoryStore, security::RateLimiter};
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
    pub settings_locked_from_env: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateInstanceSettingsRequest {
    pub tree_title: Option<String>,
    pub default_locale: Option<String>,
    pub enable_public_access: Option<bool>,
    pub living_person_age_threshold: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct InstanceSettingsValues {
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub living_person_age_threshold: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub config: Arc<AppConfig>,
    pub settings_override: Arc<RwLock<InstanceSettingsValues>>,
    pub auth_store: Arc<RwLock<AuthMemoryStore>>,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(db: Option<PgPool>, config: AppConfig) -> Self {
        let initial = InstanceSettingsValues {
            tree_title: config.tree_title.clone(),
            default_locale: config.default_locale.clone(),
            enable_public_access: config.enable_public_access,
            living_person_age_threshold: config.living_person_age_threshold,
        };
        Self {
            db,
            config: Arc::new(config),
            settings_override: Arc::new(RwLock::new(initial)),
            auth_store: Arc::new(RwLock::new(AuthMemoryStore::default())),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Resolves effective instance settings according to `docs/features/10-core-data-model.md §7`:
    /// - When `SETTINGS_LOCKED_FROM_ENV=true`, environment values always win.
    /// - Otherwise, values saved in `instance_settings` (or in-memory state when running standalone) win over initial `.env` defaults.
    pub async fn resolve_instance_settings(&self) -> InstanceSettingsDto {
        if self.config.settings_locked_from_env {
            return InstanceSettingsDto {
                tree_title: self.config.tree_title.clone(),
                default_locale: self.config.default_locale.clone(),
                enable_public_access: self.config.enable_public_access,
                living_person_age_threshold: self.config.living_person_age_threshold,
                settings_locked_from_env: true,
            };
        }

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
                        _ => {}
                    }
                }
            }
        }

        InstanceSettingsDto {
            tree_title: current.tree_title,
            default_locale: current.default_locale,
            enable_public_access: current.enable_public_access,
            living_person_age_threshold: current.living_person_age_threshold,
            settings_locked_from_env: false,
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
        let updated = guard.clone();
        drop(guard);

        if let Some(pool) = &self.db {
            let entries: [(&str, serde_json::Value); 4] = [
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
            settings_locked_from_env: false,
        })
    }
}
