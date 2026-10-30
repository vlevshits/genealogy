use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use genealogy_backend::{
    config::AppConfig,
    error::ErrorResponse,
    routes::{create_router, rbac::MediaArchiveResponse},
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        gedcom::GedcomImportResult,
        media_bundle::MediaBundleReconciliationReport,
    },
    state::AppState,
};
use serde_json::json;
use tower::ServiceExt;
use zip::write::SimpleFileOptions;

async fn setup_test_env() -> (axum::Router, String, String, String) {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Gramps Media Bundle Test Tree".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new_demo(None, config);
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: None,
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let admin_id = uuid::Uuid::parse_str(&admin_setup.user.id).unwrap();

    let create_role = |role: &str, email: &str| {
        let state_clone = state.clone();
        let role_str = role.to_string();
        let email_str = email.to_string();
        async move {
            let inv = AuthService::create_invitation(
                &state_clone,
                Some(admin_id),
                CreateInviteRequest {
                    role: role_str.clone(),
                    email: Some(email_str.clone()),
                    invitee_name: None,
                    person_id: None,
                },
            )
            .await
            .unwrap();
            let (reg, _) = AuthService::register_with_invitation(
                &state_clone,
                InviteRegisterRequest {
                    invite_token: inv.token,
                    first_name: "Relative".to_string(),
                    last_name: role_str,
                    email: Some(email_str),
                    passkey: None,
                    password: Some("UserPass123!".to_string()),
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
            format!("Bearer {}", reg.session_token)
        }
    };

    let editor_auth = create_role("editor", "editor@levshits.family").await;
    let viewer_auth = create_role("viewer", "viewer@levshits.family").await;
    let app = create_router(state);

    (app, admin_auth, editor_auth, viewer_auth)
}

fn build_multipart_request(
    uri: &str,
    auth_header: &str,
    filename: &str,
    file_bytes: &[u8],
) -> Request<Body> {
    let boundary = "---------------------------98765432101234567890";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/zip\r\n\r\n");
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("x-requested-with", "genealogy")
        .header(header::ORIGIN, "http://localhost:3000")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        );
    if !auth_header.is_empty() {
        builder = builder.header(header::AUTHORIZATION, auth_header);
    }
    builder.body(Body::from(body)).unwrap()
}

fn create_custom_zip(entries: Vec<(&str, &[u8])>) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, content) in entries {
            zip.start_file(name, options).unwrap();
            use std::io::Write;
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }
    buf.into_inner()
}

#[tokio::test]
async fn test_media_bundle_role_segregation() {
    let (app, _admin_auth, editor_auth, viewer_auth) = setup_test_env().await;

    let valid_jpeg =
        b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x01\x00`\x00`\x00\x00\xFF\xDB\x00C\x00";
    let zip_bytes = create_custom_zip(vec![("sample.jpg", valid_jpeg)]);

    // 1. Anonymous caller on /api/v1/gedcom/media-bundle -> 401
    let req = build_multipart_request("/api/v1/gedcom/media-bundle", "", "bundle.zip", &zip_bytes);
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 2. Viewer caller on /api/v1/gedcom/media-bundle -> 403 Forbidden
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &viewer_auth,
        "bundle.zip",
        &zip_bytes,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 3. Editor caller on /api/v1/gedcom/media-bundle -> 403 Forbidden (Admin only)
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &editor_auth,
        "bundle.zip",
        &zip_bytes,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 4. Viewer caller on /api/v1/media/bundle -> 403 Forbidden
    let req = build_multipart_request(
        "/api/v1/media/bundle",
        &viewer_auth,
        "bundle.zip",
        &zip_bytes,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 5. Editor caller on /api/v1/media/bundle -> 200 OK
    let req = build_multipart_request(
        "/api/v1/media/bundle",
        &editor_auth,
        "bundle.zip",
        &zip_bytes,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_media_bundle_security_zip_slip_and_bomb() {
    let (app, admin_auth, _editor_auth, _viewer_auth) = setup_test_env().await;

    let valid_jpeg = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x01\x00`\x00`\x00\x00";

    // 1. Zip slip path traversal entry
    let malicious_zip = create_custom_zip(vec![("../../traversal.jpg", valid_jpeg)]);
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &admin_auth,
        "malicious.zip",
        &malicious_zip,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let err_resp: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_resp.error.code, "zip_slip_detected");

    // 2. Unsupported archive format (non-zip file)
    let non_zip_bytes = b"This is plain text, not a valid ZIP file archive!";
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &admin_auth,
        "not_a_zip.txt",
        non_zip_bytes,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let err_resp: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_resp.error.code, "unsupported_archive_format");

    // 3. Skip invalid / unsupported files (e.g. SVG) inside archive
    let svg_content = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><text>malicious</text></svg>";
    let mixed_zip = create_custom_zip(vec![
        ("valid.jpg", valid_jpeg),
        ("vector.svg", svg_content),
        (".DS_Store", b"macosx junk"),
    ]);
    let req = build_multipart_request(
        "/api/v1/media/bundle?ingest_unmatched=true",
        &admin_auth,
        "mixed.zip",
        &mixed_zip,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let report: MediaBundleReconciliationReport = serde_json::from_slice(&body).unwrap();
    assert_eq!(report.total_archive_files, 1);
    assert_eq!(report.skipped_invalid_files, 1);
    assert_eq!(report.newly_created, 1);

    // 4. Decompression bomb ratio (> 100:1)
    let bomb_data = vec![0u8; 200_000]; // 200KB of zeros compresses to ~200B (ratio ~1000:1)
    let ratio_bomb_zip = create_custom_zip(vec![("bomb.jpg", &bomb_data)]);
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &admin_auth,
        "ratio_bomb.zip",
        &ratio_bomb_zip,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let err_resp: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_resp.error.code, "decompression_limit_exceeded");

    // 5. Decompression bomb entry count (> 2000 entries)
    let mut many_entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(2005);
    for i in 0..2005 {
        many_entries.push((format!("file_{i}.jpg"), valid_jpeg.to_vec()));
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in &many_entries {
            zip.start_file(name, options).unwrap();
            use std::io::Write;
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }
    let entry_bomb_zip = buf.into_inner();
    let req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &admin_auth,
        "entry_bomb.zip",
        &entry_bomb_zip,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let err_resp: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_resp.error.code, "decompression_limit_exceeded");

    // 6. Magic bytes enforcement: PNG, WebP, AVIF, TIFF, PDF accepted; HTML rejected
    let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4";
    let webp_bytes = b"RIFF\x24\x00\x00\x00WEBPVP8 \x18\x00\x00\x000\x01\x00\x9d\x01\x2a\x01\x00\x01\x00\x02\x004\x25\xa4\x00\x03\x70\x00\xfe\xfb\xfd\x00\x00";
    let avif_bytes = b"\x00\x00\x00\x20ftypavif\x00\x00\x00\x00avifmif1miafMA1B";
    let tiff_bytes = b"II*\x00\x08\x00\x00\x00\x00\x00";
    let pdf_bytes =
        b"%PDF-1.4\n1 0 obj<</Type/Catalog>>endobj\nxref\ntrailer<</Root 1 0 R>>\n%%EOF";
    let html_bytes = b"<!DOCTYPE html><html><body><script>alert(1)</script></body></html>";

    let formats_zip = create_custom_zip(vec![
        ("image.png", png_bytes),
        ("image.webp", webp_bytes),
        ("image.avif", avif_bytes),
        ("image.tiff", tiff_bytes),
        ("doc.pdf", pdf_bytes),
        ("malicious.html", html_bytes),
    ]);
    let req = build_multipart_request(
        "/api/v1/media/bundle?ingest_unmatched=true",
        &admin_auth,
        "formats.zip",
        &formats_zip,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let report: MediaBundleReconciliationReport = serde_json::from_slice(&body).unwrap();
    // 5 accepted formats (png, webp, avif, tiff, pdf), 1 rejected (html)
    assert_eq!(report.total_archive_files, 5);
    assert_eq!(report.skipped_invalid_files, 1);
    assert_eq!(report.newly_created, 5);
}

#[tokio::test]
async fn test_gramps_web_media_bundle_reconciliation_end_to_end() {
    let (app, admin_auth, _editor_auth, _viewer_auth) = setup_test_env().await;

    // 1. Import GEDCOM fixture that generates 129 media stubs
    let ged_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-data/gramps-export/gramps-web-export-20261026130902.ged");
    let ged_content = std::fs::read_to_string(&ged_path).expect("GEDCOM test fixture must exist");

    let import_req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/gedcom/import")
        .header(header::AUTHORIZATION, &admin_auth)
        .header("x-requested-with", "genealogy")
        .header(header::ORIGIN, "http://localhost:3000")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "data": BASE64_STANDARD.encode(ged_content.as_bytes()),
                "mode": "new",
                "confirm_text": "OVERWRITE",
                "file_name": "gramps-web-export-20261026130902.ged"
            })
            .to_string(),
        ))
        .unwrap();

    let import_resp = app.clone().oneshot(import_req).await.unwrap();
    assert_eq!(import_resp.status(), StatusCode::OK);
    let import_body = to_bytes(import_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let import_result: GedcomImportResult = serde_json::from_slice(&import_body).unwrap();
    assert_eq!(import_result.media_imported, 129);
    assert_eq!(import_result.media_missing_count, 129);

    // Verify GET /api/v1/media?is_missing=true returns 129 missing stubs
    let list_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?is_missing=true&limit=200")
        .header(header::AUTHORIZATION, &admin_auth)
        .body(Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = to_bytes(list_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let list_result: MediaArchiveResponse = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list_result.total, 129);
    assert!(list_result.items.iter().all(|m| m.is_missing));

    // 2. Upload actual Gramps Web media bundle ZIP (53.9MB, 129 JPEGs)
    let zip_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-data/gramps-web-media-export-20261026130904.zip");
    let zip_bytes = std::fs::read(&zip_path).expect("Gramps Web media ZIP fixture must exist");
    assert_eq!(zip_bytes.len(), 53990655);

    let bundle_req = build_multipart_request(
        "/api/v1/gedcom/media-bundle",
        &admin_auth,
        "gramps-web-media-export-20261026130904.zip",
        &zip_bytes,
    );
    let bundle_resp = app.clone().oneshot(bundle_req).await.unwrap();
    assert_eq!(bundle_resp.status(), StatusCode::OK);
    let bundle_body = to_bytes(bundle_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let report: MediaBundleReconciliationReport = serde_json::from_slice(&bundle_body).unwrap();

    assert_eq!(report.total_archive_files, 129);
    assert_eq!(report.reconciled_stubs, 129);
    assert_eq!(report.unmatched_files, 0);
    assert_eq!(report.remaining_missing_stubs, 0);
    assert_eq!(report.skipped_invalid_files, 0);
    assert!(report.total_bytes_extracted > 50_000_000);

    // 3. Verify GET /api/v1/media?is_missing=true now returns 0
    let list_missing_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?is_missing=true&limit=200")
        .header(header::AUTHORIZATION, &admin_auth)
        .body(Body::empty())
        .unwrap();
    let list_missing_resp = app.clone().oneshot(list_missing_req).await.unwrap();
    assert_eq!(list_missing_resp.status(), StatusCode::OK);
    let list_missing_body = to_bytes(list_missing_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let list_missing_result: MediaArchiveResponse =
        serde_json::from_slice(&list_missing_body).unwrap();
    assert_eq!(list_missing_result.total, 0);

    // 4. Verify a reconciled media asset binary can be fetched via GET /api/v1/media/{id}/file
    let list_all_req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?limit=5")
        .header(header::AUTHORIZATION, &admin_auth)
        .body(Body::empty())
        .unwrap();
    let list_all_resp = app.clone().oneshot(list_all_req).await.unwrap();
    let list_all_body = to_bytes(list_all_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let list_all_result: MediaArchiveResponse = serde_json::from_slice(&list_all_body).unwrap();
    assert!(!list_all_result.items.is_empty());
    let reconciled_item = &list_all_result.items[0];
    assert!(!reconciled_item.is_missing);
    assert!(reconciled_item.file_size_bytes.unwrap_or(0) > 0);
    assert!(reconciled_item.sha256.is_some());

    let file_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1/media/{}/file", reconciled_item.id))
        .header(header::AUTHORIZATION, &admin_auth)
        .body(Body::empty())
        .unwrap();
    let file_resp = app.clone().oneshot(file_req).await.unwrap();
    assert_eq!(file_resp.status(), StatusCode::OK);
    assert_eq!(
        file_resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/jpeg"
    );
    let file_bytes = to_bytes(file_resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert!(!file_bytes.is_empty());
    assert_eq!(&file_bytes[0..3], b"\xFF\xD8\xFF"); // JPEG magic bytes
}

#[test]
fn test_media_bundle_helpers_and_security_checks() {
    use genealogy_backend::services::media_bundle::{
        check_zip_slip, detect_allowed_media_mime, extract_basename, MAX_ARCHIVE_BYTES,
        MAX_COMPRESSION_RATIO, MAX_ENTRY_COUNT, MAX_TOTAL_UNCOMPRESSED_BYTES,
    };

    // Constants check
    assert_eq!(MAX_ARCHIVE_BYTES, 500 * 1024 * 1024);
    assert_eq!(MAX_ENTRY_COUNT, 2000);
    assert_eq!(MAX_TOTAL_UNCOMPRESSED_BYTES, 2 * 1024 * 1024 * 1024);
    assert_eq!(MAX_COMPRESSION_RATIO, 100);

    // Zip slip detections
    assert!(check_zip_slip("../../etc/passwd").is_err());
    assert!(check_zip_slip("/root/secret.jpg").is_err());
    assert!(check_zip_slip("\\Windows\\System32").is_err());
    assert!(check_zip_slip("C:\\evil.exe").is_err());
    assert!(check_zip_slip("photos/../../etc").is_err());
    assert!(check_zip_slip("photos/..").is_err());
    assert!(check_zip_slip("photos/..\\secret").is_err());
    assert!(check_zip_slip("test\0traversal.jpg").is_err());
    assert!(check_zip_slip("safe_folder/photo.jpg").is_ok());
    assert!(check_zip_slip("photo.jpg").is_ok());

    // Basename extraction
    assert_eq!(extract_basename("/path/to/file.jpg"), "file.jpg");
    assert_eq!(extract_basename("C:\\path\\to\\file.png"), "file.png");
    assert_eq!(extract_basename("file.webp"), "file.webp");

    // MIME detection
    assert_eq!(
        detect_allowed_media_mime(b"\xFF\xD8\xFF\xE0"),
        Some("image/jpeg")
    );
    assert_eq!(
        detect_allowed_media_mime(b"\x89PNG\r\n\x1a\n\x00"),
        Some("image/png")
    );
    assert_eq!(
        detect_allowed_media_mime(b"RIFF\x24\x00\x00\x00WEBP"),
        Some("image/webp")
    );
    assert_eq!(
        detect_allowed_media_mime(b"\x00\x00\x00\x20ftypavif"),
        Some("image/avif")
    );
    assert_eq!(
        detect_allowed_media_mime(b"II*\x00\x08"),
        Some("image/tiff")
    );
    assert_eq!(
        detect_allowed_media_mime(b"MM\x00*\x08"),
        Some("image/tiff")
    );
    assert_eq!(
        detect_allowed_media_mime(b"%PDF-1.7"),
        Some("application/pdf")
    );

    // Explicitly forbidden MIME / content
    assert_eq!(
        detect_allowed_media_mime(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"),
        None
    );
    assert_eq!(detect_allowed_media_mime(b"<!DOCTYPE html><html>"), None);
    assert_eq!(detect_allowed_media_mime(b"<html><head>"), None);
    assert_eq!(
        detect_allowed_media_mime(b"<script>alert(1)</script>"),
        None
    );
    assert_eq!(detect_allowed_media_mime(b"random plain text"), None);
}
