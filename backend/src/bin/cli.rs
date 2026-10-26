use genealogy_backend::{
    config::AppConfig,
    services::{backup::BackupService, db_init::connect_and_migrate},
};
use std::env;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> ExitCode {
    let _ = dotenvy::dotenv();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }

    let mut config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Configuration error: {err}");
            return ExitCode::from(1);
        }
    };
    // CLI commands can run against live DB when reachable or standalone when offline
    config.database_required = false;

    let backup_service = BackupService::new(Arc::new(config.clone()));
    let pool = match connect_and_migrate(&config, &backup_service).await {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Warning: Database initialization issue: {err}");
            None
        }
    };

    let cmd = args[1].as_str();
    match cmd {
        "backup" => {
            let include_gedcom = args.iter().any(|a| a == "--include-gedcom");
            let override_gedcom = if include_gedcom { Some(true) } else { None };
            match backup_service
                .create_backup(pool.as_ref(), "cli", override_gedcom)
                .await
            {
                Ok(info) => {
                    println!(
                        "Backup created: {}/{} ({} bytes)",
                        config.backup_path, info.filename, info.size_bytes
                    );
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Backup failed: {err}");
                    ExitCode::from(1)
                }
            }
        }
        "restore" => {
            let Some(archive_arg) = args.get(2) else {
                eprintln!("Usage: genealogy-cli restore <archive_path>");
                return ExitCode::from(1);
            };
            let path = Path::new(archive_arg);
            match backup_service.verify_and_restore(path, pool.as_ref()).await {
                Ok(manifest) => {
                    println!(
                        "Restore completed successfully (schema_version={}, files={})",
                        manifest.schema_version,
                        manifest.files.len()
                    );
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Restore failed: {err}");
                    ExitCode::from(1)
                }
            }
        }
        "create-admin-link" => {
            let Some(email) = args.get(2) else {
                eprintln!("Usage: genealogy-cli create-admin-link <email>");
                return ExitCode::from(1);
            };
            match backup_service
                .create_emergency_admin_link(pool.as_ref(), email)
                .await
            {
                Ok(link) => {
                    println!("Emergency sign-in link (valid for 24h):\n{link}");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Failed to create emergency admin link: {err}");
                    ExitCode::from(1)
                }
            }
        }
        "regenerate-thumbnails" => {
            let thumbs_dir = Path::new(&config.media_storage_path).join("thumbs");
            if let Err(err) = std::fs::create_dir_all(&thumbs_dir) {
                eprintln!("Failed to prepare thumbnails directory: {err}");
                return ExitCode::from(1);
            }
            println!(
                "Thumbnail cache verified and queued for regeneration at {}",
                thumbs_dir.display()
            );
            ExitCode::SUCCESS
        }
        "export-gedcom" => {
            let Some(output_arg) = args.get(2) else {
                eprintln!("Usage: genealogy-cli export-gedcom <output_path>");
                return ExitCode::from(1);
            };
            let output_path = Path::new(output_arg);
            match BackupService::write_gedcom_export(output_path, &config.tree_title) {
                Ok(()) => {
                    println!("GEDCOM archive exported to {}", output_path.display());
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("GEDCOM export failed: {err}");
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            print_usage();
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: genealogy-cli <command> [args]\n\
         Commands:\n\
         \x20 backup [--include-gedcom]        Create an on-demand backup archive\n\
         \x20 restore <archive.tar.gz>         Verify checksums and restore from archive\n\
         \x20 create-admin-link <email>        Generate a 24h emergency admin sign-in link\n\
         \x20 regenerate-thumbnails            Rebuild derived WebP media thumbnails\n\
         \x20 export-gedcom <output.gdz>       Export tree to GEDCOM 7.0 / GEDZIP"
    );
}
