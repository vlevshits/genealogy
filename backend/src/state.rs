use crate::config::AppConfig;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: Option<PgPool>, config: AppConfig) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}
