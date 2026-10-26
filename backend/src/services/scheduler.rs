use crate::services::backup::BackupService;
use crate::state::AppState;
use chrono::{Datelike, Timelike, Utc};
use std::time::Duration;

/// Parses the hour and minute from a standard 5-field cron expression (`MIN HOUR * * *`),
/// falling back to `03:00` (`0 3 * * *`) if unparseable.
pub fn parse_cron_hour_minute(cron_expr: &str) -> (u32, u32) {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() >= 2 {
        let minute = parts[0].parse::<u32>().unwrap_or(0).min(59);
        let hour = parts[1].parse::<u32>().unwrap_or(3).min(23);
        (hour, minute)
    } else {
        (3, 0)
    }
}

/// Spawns the in-process background maintenance & backup scheduler (`17-api-security-operations.md §5`).
pub fn spawn_background_scheduler(state: AppState) {
    if !state.config.backup_enabled {
        tracing::info!("Scheduled backups disabled via BACKUP_ENABLED=false");
        return;
    }

    let (target_hour, target_minute) = parse_cron_hour_minute(&state.config.backup_cron);
    let retention_days = state.config.backup_retention_days;
    let backup_service = BackupService::new(state.config.clone());

    tracing::info!(
        cron = %state.config.backup_cron,
        target_hour = target_hour,
        target_minute = target_minute,
        retention_days = retention_days,
        "Spawned in-process backup and retention scheduler"
    );

    tokio::spawn(async move {
        let mut last_run_day: Option<(i32, u32)> = None;
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            let now = Utc::now();
            let today_key = (now.year(), now.ordinal());

            if now.hour() == target_hour
                && now.minute() == target_minute
                && last_run_day != Some(today_key)
            {
                last_run_day = Some(today_key);
                if let Err(err) = backup_service
                    .create_backup(state.db.as_ref(), "scheduled", None)
                    .await
                {
                    tracing::error!(error = %err, "Scheduled nightly backup failed");
                }
                if let Err(err) = backup_service.prune_old_backups(retention_days) {
                    tracing::error!(error = %err, "Scheduled backup retention pruning failed");
                }
            }
        }
    });
}
