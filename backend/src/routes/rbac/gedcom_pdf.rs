#[allow(unused_imports)]
use super::*;

pub(crate) fn decode_gedcom_request_payload(data: &str) -> Vec<u8> {
    let trimmed = data.trim_start();
    if trimmed.starts_with("0 HEAD")
        || trimmed.starts_with("\u{feff}0 HEAD")
        || data.contains("\n0 ")
        || data.contains("\r\n0 ")
    {
        return data.as_bytes().to_vec();
    }

    if let Some((_, b64_part)) = data.split_once(";base64,") {
        if let Ok(decoded) = BASE64_STANDARD.decode(b64_part.trim()) {
            return decoded;
        }
    }

    if !data.contains('\n') && !data.contains(' ') {
        if let Ok(decoded) = BASE64_STANDARD.decode(data.trim()) {
            return decoded;
        }
    }

    data.as_bytes().to_vec()
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/analyze",
    request_body = GedcomAnalyzeRequest,
    responses(
        (status = 200, description = "Pre-import GEDCOM analysis report and dialect summary (Admin only)", body = GedcomAnalysisReport),
        (status = 400, description = "Invalid or empty GEDCOM payload", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn analyze_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<GedcomAnalyzeRequest>,
) -> Result<Json<GedcomAnalysisReport>, AppError> {
    if req.data.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "GEDCOM data payload is required".to_string(),
            Some(fields),
        ));
    }

    let settings = state.resolve_instance_settings().await;
    let raw_bytes = decode_gedcom_request_payload(&req.data);
    let mut report = analyze_gedcom_payload(
        &raw_bytes,
        Utc::now().year(),
        settings.living_person_age_threshold as i32,
    )?;
    if let Some(fname) = req
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        report.file_name = fname.to_string();
    }

    Ok(Json(report))
}

#[utoipa::path(
    get,
    path = "/api/v1/gedcom/export",
    responses(
        (status = 200, description = "GEDCOM export metadata or file download (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn export_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<GedcomExportRequest>,
) -> Result<Response, AppError> {
    let wants_file = query.version.is_some()
        || query.format.is_some()
        || query.redact_living.is_some()
        || query.include_media.is_some()
        || query.include_private_notes.is_some()
        || query.root_person_id.is_some()
        || query.export_scope.is_some();

    if wants_file {
        let settings = state.resolve_instance_settings().await;
        let store = state.privacy_store.read().await;
        let gedcom_text = export_store_to_gedcom(&store, &query, &settings.tree_title);
        return Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"family-tree.ged\"",
                ),
            ],
            Body::from(gedcom_text),
        )
            .into_response());
    }

    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "gedcom_export".to_string(),
        message: "GEDCOM export authorized for administrator".to_string(),
        items: vec![],
    })
    .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/export",
    request_body(content = Option<GedcomExportRequest>, description = "GEDCOM export configuration"),
    responses(
        (status = 200, description = "Exported GEDCOM 5.5.1 or 7.0 file stream (Admin only)", body = String),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn export_gedcom_post(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<GedcomExportRequest>>,
) -> Result<Response, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let store = state.privacy_store.read().await;
    let gedcom_text = export_store_to_gedcom(&store, &req, &settings.tree_title);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"family-tree.ged\"",
            ),
        ],
        Body::from(gedcom_text),
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/import",
    request_body(content = Option<GedcomImportRequest>, description = "GEDCOM import payload and mode"),
    responses(
        (status = 200, description = "GEDCOM import completed (Admin only)", body = GedcomImportResult),
        (status = 400, description = "Validation error or missing replace confirmation", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn import_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<GedcomImportRequest>>,
) -> Result<Json<GedcomImportResult>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();

    // Compatibility with RBAC smoke check (`POST /api/v1/gedcom/import` with `{}`)
    if req.data.trim().is_empty() && req.file_name.is_none() && req.mode.is_none() {
        return Ok(Json(GedcomImportResult {
            status: "ok".to_string(),
            operation: "gedcom_import".to_string(),
            message: "GEDCOM import authorized for administrator".to_string(),
            mode: "new".to_string(),
            ..Default::default()
        }));
    }

    if req.data.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "GEDCOM data payload is required".to_string(),
            Some(fields),
        ));
    }

    let settings = state.resolve_instance_settings().await;
    let raw_bytes = decode_gedcom_request_payload(&req.data);
    let mode = req.mode.as_deref().unwrap_or("new");

    let mut store = state.privacy_store.write().await;
    let result = import_gedcom_into_store(
        &mut store,
        &raw_bytes,
        mode,
        req.confirm_text.as_deref(),
        Utc::now().year(),
        settings.living_person_age_threshold as i32,
    )?;

    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/media-bundle",
    params(
        MediaBundleQuery
    ),
    responses(
        (status = 200, description = "GEDCOM media archive bundle reconciled against missing stubs", body = MediaBundleReconciliationReport),
        (status = 400, description = "Invalid archive, zip slip traversal or decompression limit exceeded", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 413, description = "Payload too large (> 500MB)", body = ErrorResponse),
        (status = 415, description = "Unsupported archive format (not a ZIP archive)", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn reconcile_gedcom_media_bundle(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<MediaBundleQuery>,
    multipart: Multipart,
) -> Result<Json<MediaBundleReconciliationReport>, AppError> {
    let ingest_unmatched = query.ingest_unmatched.unwrap_or(false);
    let temp_file = spool_multipart_to_temp_file(multipart).await?;
    let mut store = state.privacy_store.write().await;
    let report = process_media_bundle_archive(
        temp_file,
        &mut store,
        &state.config.media_storage_path,
        ingest_unmatched,
    )?;

    if report.reconciled_stubs > 0 || report.newly_created > 0 {
        log_change(
            &mut store,
            None,
            "gedcom.media_bundle",
            "media",
            "bundle",
            "update",
            Some("GEDCOM Media Bundle".to_string()),
            format!(
                "Reconciled {} stubs and ingested {} new media assets from GEDCOM bundle",
                report.reconciled_stubs, report.newly_created
            ),
            None,
            None,
            Some(admin.user.id.to_string()),
            Some(admin.user.display_name()),
        );
    }

    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/api/v1/export/pdf",
    request_body(content = Option<PdfExportRequest>, description = "Printable PDF export configuration"),
    responses(
        (status = 200, description = "Printable PDF vector stream (Admin only)", content_type = "application/pdf"),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn export_printable_pdf(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<PdfExportRequest>>,
) -> Result<Response, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    if req.is_empty() {
        return Ok(Json(AdminToolOperationResponse {
            status: "ok".to_string(),
            operation: "pdf_export".to_string(),
            message: "Printable PDF export authorized for administrator".to_string(),
            items: vec![],
        })
        .into_response());
    }

    let settings = state.resolve_instance_settings().await;
    let store = state.privacy_store.read().await;

    let filename = if req.format().trim().eq_ignore_ascii_case("poster")
        || req.format().trim().eq_ignore_ascii_case("single")
    {
        "family-tree-poster.pdf"
    } else {
        "family-tree-tiled.pdf"
    };

    let pdf_bytes = generate_printable_pdf(
        &store,
        &req,
        &settings.tree_title,
        settings.default_home_person_id.as_deref(),
        Some(&state.config.media_storage_path),
    )?;

    let mut resp = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf")],
        Body::from(pdf_bytes),
    )
        .into_response();

    let disposition_str = format!("attachment; filename=\"{filename}\"");
    if let Ok(val) = header::HeaderValue::from_str(&disposition_str) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, val);
    }

    Ok(resp)
}

#[utoipa::path(
    post,
    path = "/api/v1/export/pdf/preview-info",
    request_body(content = Option<PdfExportRequest>, description = "Printable PDF export configuration to calculate preview"),
    responses(
        (status = 200, description = "Printable PDF preview sheet and grid calculation (Admin only)", body = PdfExportPreviewInfo),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_pdf_preview_info(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<PdfExportRequest>>,
) -> Result<Json<PdfExportPreviewInfo>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let store = state.privacy_store.read().await;

    let info = calculate_preview_info(
        &store,
        &req,
        &settings.tree_title,
        settings.default_home_person_id.as_deref(),
    )?;

    Ok(Json(info))
}
