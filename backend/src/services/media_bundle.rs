use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

use axum::extract::Multipart;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use utoipa::{IntoParams, ToSchema};

use crate::error::AppError;
use crate::services::genealogy::allocate_gid;
use crate::services::privacy::{GenealogyPrivacyStore, MediaAssetSummaryDto};

pub const MAX_ARCHIVE_BYTES: u64 = 500 * 1024 * 1024; // 500MB
pub const MAX_ENTRY_COUNT: usize = 2000;
pub const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2GB
pub const MAX_COMPRESSION_RATIO: u64 = 100;
pub const COMPRESSION_RATIO_MIN_UNCOMPRESSED_THRESHOLD: u64 = 1024; // 1KB

#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct MediaBundleQuery {
    /// Ingest unmatched media files as new library assets (`true`) or report as unmatched (`false`)
    pub ingest_unmatched: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaBundleReconciliationReport {
    pub total_archive_files: usize,
    #[serde(default)]
    pub total_extracted: usize,
    pub reconciled_stubs: usize,
    #[serde(default)]
    pub newly_created: usize,
    pub unmatched_files: usize,
    pub unmatched_filenames: Vec<String>,
    pub remaining_missing_stubs: usize,
    pub skipped_invalid_files: usize,
    #[serde(default)]
    pub skipped_files: usize,
    pub total_bytes_extracted: u64,
    pub duration_ms: u64,
    #[serde(default)]
    pub errors: Vec<String>,
}

pub fn save_media_bytes_to_disk(media_storage_path: &str, file_name: &str, bytes: &[u8]) {
    let primary_dir = PathBuf::from(media_storage_path).join("originals");
    if std::fs::create_dir_all(&primary_dir).is_ok() {
        let _ = std::fs::write(primary_dir.join(file_name), bytes);
    }
    let fallback_dir = PathBuf::from("/tmp/genealogy-media/originals");
    if std::fs::create_dir_all(&fallback_dir).is_ok() {
        let _ = std::fs::write(fallback_dir.join(file_name), bytes);
    }
}

pub fn extract_basename(path_or_name: &str) -> &str {
    let trimmed = path_or_name.trim();
    if let Some((_, file_part)) = trimmed.rsplit_once(['/', '\\']) {
        if !file_part.is_empty() {
            return file_part;
        }
    }
    trimmed
}

pub fn check_zip_slip(name: &str) -> Result<(), AppError> {
    if name.contains('\0')
        || name.starts_with('/')
        || name.starts_with('\\')
        || name.contains("../")
        || name.contains("..\\")
        || name.ends_with("/..")
        || name.ends_with("\\..")
        || name == ".."
        || (name.len() >= 2
            && name.as_bytes()[1] == b':'
            && name.as_bytes()[0].is_ascii_alphabetic())
    {
        return Err(AppError::ZipSlipDetected(format!(
            "Path traversal element detected in archive entry: '{name}'"
        )));
    }
    let path = Path::new(name);
    for comp in path.components() {
        if matches!(
            comp,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(AppError::ZipSlipDetected(format!(
                "Path traversal element detected in archive entry: '{name}'"
            )));
        }
    }
    Ok(())
}

pub fn detect_allowed_media_mime(bytes: &[u8]) -> Option<&'static str> {
    // Strictly reject SVG, HTML, or script content
    let check_len = bytes.len().min(512);
    let head = &bytes[..check_len];
    let head_lower = head.to_ascii_lowercase();
    if head_lower.windows(4).any(|w| w == b"<svg")
        || head_lower.windows(5).any(|w| w == b"<html")
        || head_lower.windows(9).any(|w| w == b"<!doctype")
        || head_lower.windows(7).any(|w| w == b"<script")
    {
        return None;
    }

    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some("image/jpeg");
    }
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        return Some("image/png");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    {
        return Some("image/avif");
    }
    if bytes.len() >= 4 && (&bytes[0..4] == b"II*\x00" || &bytes[0..4] == b"MM\x00*") {
        return Some("image/tiff");
    }
    if bytes.len() >= 5 && &bytes[0..5] == b"%PDF-" {
        return Some("application/pdf");
    }

    None
}

pub async fn spool_multipart_to_temp_file(
    mut multipart: Multipart,
) -> Result<NamedTempFile, AppError> {
    let mut temp_file = tempfile::Builder::new()
        .prefix("media_bundle_")
        .suffix(".tmp")
        .tempfile_in(std::env::temp_dir())
        .or_else(|_| {
            tempfile::Builder::new()
                .prefix("media_bundle_")
                .suffix(".tmp")
                .tempfile()
        })
        .map_err(|e| AppError::Internal(format!("Failed to create temporary spool file: {e}")))?;

    let mut total_spooled: u64 = 0;
    let mut found_file = false;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ValidationFailed(format!("Multipart stream error: {e}"), None))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "file"
            || field_name == "archive"
            || field_name == "bundle"
            || field.file_name().is_some()
        {
            found_file = true;
            while let Some(chunk) = field.chunk().await.map_err(|e| {
                AppError::ValidationFailed(format!("Error reading upload chunk: {e}"), None)
            })? {
                total_spooled += chunk.len() as u64;
                if total_spooled > MAX_ARCHIVE_BYTES {
                    return Err(AppError::PayloadTooLarge(
                        "Upload exceeds 500MB limit".to_string(),
                    ));
                }
                temp_file.write_all(&chunk).map_err(|e| {
                    AppError::Internal(format!("Failed writing chunk to temporary file: {e}"))
                })?;
            }
            temp_file
                .flush()
                .map_err(|e| AppError::Internal(format!("Failed flushing temporary file: {e}")))?;
            break;
        }
    }

    if !found_file {
        return Err(AppError::ValidationFailed(
            "Multipart field 'file' is required".to_string(),
            None,
        ));
    }

    // Verify magic bytes: must start with PK\x03\x04 or PK\x05\x06 or PK\x07\x08
    temp_file
        .seek(SeekFrom::Start(0))
        .map_err(|e| AppError::Internal(format!("Failed seeking in temporary spool file: {e}")))?;

    let mut header = [0u8; 4];
    let bytes_read = temp_file.read(&mut header).unwrap_or(0);
    if bytes_read < 4
        || header[0] != b'P'
        || header[1] != b'K'
        || !(header[2] == 3 && header[3] == 4
            || header[2] == 5 && header[3] == 6
            || header[2] == 7 && header[3] == 8)
    {
        return Err(AppError::UnsupportedArchiveFormat(
            "Uploaded file is not a valid ZIP archive".to_string(),
        ));
    }

    temp_file
        .seek(SeekFrom::Start(0))
        .map_err(|e| AppError::Internal(format!("Failed seeking in temporary spool file: {e}")))?;

    Ok(temp_file)
}

pub fn process_media_bundle_archive(
    temp_file: NamedTempFile,
    store: &mut GenealogyPrivacyStore,
    media_storage_path: &str,
    ingest_unmatched: bool,
) -> Result<MediaBundleReconciliationReport, AppError> {
    let start_time = Instant::now();
    let file = temp_file
        .reopen()
        .map_err(|e| AppError::Internal(format!("Failed to reopen temporary archive file: {e}")))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::InvalidArchive(format!("Failed to read ZIP archive: {e}")))?;

    if archive.len() > MAX_ENTRY_COUNT {
        return Err(AppError::DecompressionLimitExceeded(format!(
            "Archive entry count {} exceeds maximum allowed {}",
            archive.len(),
            MAX_ENTRY_COUNT
        )));
    }

    let mut total_archive_files = 0;
    let mut reconciled_stubs = 0;
    let mut newly_created = 0;
    let mut unmatched_files = 0;
    let mut unmatched_filenames = Vec::new();
    let mut skipped_invalid_files = 0;
    let mut total_bytes_extracted: u64 = 0;
    let mut total_uncompressed_bytes: u64 = 0;
    let mut errors = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            AppError::InvalidArchive(format!("Error reading archive entry #{i}: {e}"))
        })?;

        let entry_name = entry.name().to_string();

        // 1. Security Check: Zip Slip
        check_zip_slip(&entry_name)?;

        // 2. Ignore directories
        if entry.is_dir() {
            continue;
        }

        // 3. Ignore macOS / OS metadata
        let name_lower = entry_name.to_lowercase();
        if name_lower.starts_with("__macosx/")
            || name_lower.starts_with("__macosx\\")
            || name_lower.contains("/__macosx/")
            || name_lower.contains("\\__macosx\\")
        {
            continue;
        }

        let basename = extract_basename(&entry_name).to_string();
        if basename.is_empty()
            || basename.eq_ignore_ascii_case(".ds_store")
            || basename.eq_ignore_ascii_case("thumbs.db")
        {
            continue;
        }

        // 4. Security Check: Decompression Bomb
        let uncompressed_size = entry.size();
        let compressed_size = entry.compressed_size();

        if uncompressed_size > COMPRESSION_RATIO_MIN_UNCOMPRESSED_THRESHOLD
            && compressed_size > 0
            && uncompressed_size > compressed_size * MAX_COMPRESSION_RATIO
        {
            return Err(AppError::DecompressionLimitExceeded(format!(
                "Decompression bomb ratio exceeded for entry '{entry_name}'"
            )));
        }

        total_uncompressed_bytes += uncompressed_size;
        if total_uncompressed_bytes > MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(AppError::DecompressionLimitExceeded(
                "Total uncompressed archive size exceeds 2GB limit".to_string(),
            ));
        }

        // 5. Read entry data
        let mut entry_bytes = Vec::with_capacity(uncompressed_size.min(10 * 1024 * 1024) as usize);
        entry.read_to_end(&mut entry_bytes).map_err(|e| {
            AppError::InvalidArchive(format!("Failed decompressing entry '{entry_name}': {e}"))
        })?;

        // 6. Magic Bytes Validation
        let Some(mime_type) = detect_allowed_media_mime(&entry_bytes) else {
            skipped_invalid_files += 1;
            errors.push(format!(
                "File '{basename}' has unsupported format or magic bytes"
            ));
            continue;
        };

        total_archive_files += 1;

        // 7. Compute SHA256 and size
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, &entry_bytes);
        let hash_hex = hex::encode(Digest::finalize(hasher));
        let file_size = entry_bytes.len() as u64;

        // 8. Reconciliation matching against is_missing = true stubs
        let matched_idx = store.media.iter().position(|m| {
            m.is_missing
                && (m
                    .original_file_name
                    .as_deref()
                    .map(extract_basename)
                    .unwrap_or("")
                    .eq_ignore_ascii_case(&basename)
                    || m.file_name
                        .as_deref()
                        .map(extract_basename)
                        .unwrap_or("")
                        .eq_ignore_ascii_case(&basename)
                    || m.external_path
                        .as_deref()
                        .map(extract_basename)
                        .unwrap_or("")
                        .eq_ignore_ascii_case(&basename))
        });

        if let Some(idx) = matched_idx {
            // Save binary to disk
            save_media_bytes_to_disk(media_storage_path, &basename, &entry_bytes);
            if let Some(ref fn_str) = store.media[idx].file_name {
                if fn_str != &basename {
                    save_media_bytes_to_disk(media_storage_path, fn_str, &entry_bytes);
                }
            }

            store.media[idx].is_missing = false;
            store.media[idx].file_size_bytes = Some(file_size);
            store.media[idx].sha256 = Some(hash_hex);
            store.media[idx].mime_type = mime_type.to_string();
            if store.media[idx].file_name.is_none() {
                store.media[idx].file_name = Some(basename.clone());
            }
            if store.media[idx].original_file_name.is_none() {
                store.media[idx].original_file_name = Some(basename.clone());
            }
            store.media[idx].version += 1;

            reconciled_stubs += 1;
            total_bytes_extracted += file_size;
        } else if ingest_unmatched {
            let media_id = uuid::Uuid::new_v4().to_string();
            let existing_gids: HashSet<String> =
                store.media.iter().map(|m| m.gid.clone()).collect();
            let gid = allocate_gid('M', &mut store.next_media_gid_seq, None, &existing_gids);

            save_media_bytes_to_disk(media_storage_path, &basename, &entry_bytes);

            store.media.push(MediaAssetSummaryDto {
                id: media_id.clone(),
                gid,
                title: basename.clone(),
                mime_type: mime_type.to_string(),
                file_url: format!("/api/v1/media/{media_id}/file"),
                file_name: Some(basename.clone()),
                original_file_name: Some(basename.clone()),
                file_size_bytes: Some(file_size),
                sha256: Some(hash_hex),
                width: None,
                height: None,
                description: None,
                date_taken: None,
                date_sort: None,
                place_id: None,
                place_full_name: None,
                is_missing: false,
                external_path: None,
                is_private: false,
                only_living_persons: false,
                linked_person_ids: vec![],
                face_tags: vec![],
                links: vec![],
                version: 1,
            });

            newly_created += 1;
            total_bytes_extracted += file_size;
        } else {
            unmatched_files += 1;
            unmatched_filenames.push(basename);
        }
    }

    let remaining_missing_stubs = store.media.iter().filter(|m| m.is_missing).count();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(MediaBundleReconciliationReport {
        total_archive_files,
        total_extracted: total_archive_files,
        reconciled_stubs,
        newly_created,
        unmatched_files,
        unmatched_filenames,
        remaining_missing_stubs,
        skipped_invalid_files,
        skipped_files: skipped_invalid_files,
        total_bytes_extracted,
        duration_ms,
        errors,
    })
}
