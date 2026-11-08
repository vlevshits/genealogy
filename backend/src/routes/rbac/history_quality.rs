#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/history",
    params(HistoryFeedQuery),
    responses(
        (status = 200, description = "Global system audit history feed with filters and transaction grouping (Admin only)", body = HistoryFeedResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_audit_history(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<HistoryFeedQuery>,
) -> Result<Json<HistoryFeedResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = query_history_feed(&store, &query);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/history/{entity_type}/{entity_id}",
    params(
        ("entity_type" = String, Path, description = "Entity type ('individual', 'person', 'family', 'event', 'place', 'media', 'note')"),
        ("entity_id" = String, Path, description = "Entity UUID or GID")
    ),
    responses(
        (status = 200, description = "Per-record chronological change history with field-level diffs (Editor or Admin)", body = RecordHistoryResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_record_history(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<RecordHistoryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = query_record_history(&store, &entity_type, &entity_id)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/history/tx/{tx_id}/undo",
    params(
        ("tx_id" = String, Path, description = "Transaction ID or change log entry ID to undo")
    ),
    responses(
        (status = 200, description = "Transaction reversed in reverse chronological order", body = UndoTransactionResponse),
        (status = 403, description = "Only the author (within 10 minutes) or an Administrator can undo", body = ErrorResponse),
        (status = 404, description = "Transaction not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn undo_history_transaction(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(tx_id): Path<String>,
) -> Result<Json<UndoTransactionResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let is_admin = editor.user.role == "admin";

    let mut store = state.privacy_store.write().await;
    let resp = undo_transaction(
        &mut store,
        &tx_id,
        &editor.user.id.to_string(),
        Some(editor.user.display_name()),
        is_admin,
        current_year,
        age_threshold,
    )?;

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/snapshots",
    responses(
        (status = 200, description = "List of point-in-time tree snapshots (Admin only)", body = SnapshotListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn list_snapshots(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<SnapshotListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let mut items = store.snapshots.clone();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let total = items.len();
    Ok(Json(SnapshotListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/snapshots",
    request_body(content = Option<CreateSnapshotRequest>, description = "Optional snapshot label and reason"),
    responses(
        (status = 201, description = "Point-in-time tree snapshot created (Admin only)", body = TreeSnapshotDto),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn create_manual_snapshot(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<CreateSnapshotRequest>>,
) -> Result<(StatusCode, Json<TreeSnapshotDto>), AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let reason = req.reason.as_deref().unwrap_or("manual");

    let mut store = state.privacy_store.write().await;
    let snapshot = create_snapshot(
        &mut store,
        req.label,
        reason,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        &state.config.backup_path,
    )?;

    Ok((StatusCode::CREATED, Json(snapshot)))
}

#[utoipa::path(
    post,
    path = "/api/v1/snapshots/{id}/rollback",
    params(
        ("id" = String, Path, description = "Snapshot UUID to roll back to")
    ),
    request_body = RollbackSnapshotRequest,
    responses(
        (status = 200, description = "Tree rolled back to snapshot after taking automatic pre_rollback safety snapshot", body = RollbackSnapshotResponse),
        (status = 400, description = "Missing or invalid ROLLBACK confirmation text", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Snapshot not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn rollback_to_snapshot(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Option<Json<RollbackSnapshotRequest>>,
) -> Result<Json<RollbackSnapshotResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let resp = rollback_snapshot(
        &mut store,
        &id,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        req.confirm_text.as_deref(),
        &state.config.backup_path,
        current_year,
        age_threshold,
    )?;

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/{entity_type}/{id}/version",
    params(
        ("entity_type" = String, Path, description = "Entity type ('persons', 'individual', 'families', 'events', 'places', 'media', 'notes')"),
        ("id" = String, Path, description = "Entity UUID or GID")
    ),
    responses(
        (status = 200, description = "Current entity version and last-modified metadata for optimistic concurrency polling", body = EntityVersionResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Entity not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_record_version(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((entity_type, id)): Path<(String, String)>,
) -> Result<Json<EntityVersionResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = get_entity_version(&store, &entity_type, &id)?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality",
    responses(
        (status = 200, description = "Data quality summary, open warnings, and duplicate candidates (Admin only)", body = QualitySummaryResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_data_quality_warnings(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<QualitySummaryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(build_quality_summary(&store)))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/summary",
    responses(
        (status = 200, description = "Data quality summary and category counts (Admin only)", body = QualitySummaryResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_quality_summary(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<QualitySummaryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(build_quality_summary(&store)))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/warnings",
    params(
        ("rule" = Option<String>, Query, description = "Filter by rule code (e.g. DEATH_BEFORE_BIRTH)"),
        ("code" = Option<String>, Query, description = "Alias for rule code"),
        ("category" = Option<String>, Query, description = "Filter by category ('chronology', 'family_age', 'completeness', 'duplicates')"),
        ("severity" = Option<String>, Query, description = "Filter by severity ('warning' or 'info')"),
        ("include_dismissed" = Option<bool>, Query, description = "Include dismissed warnings"),
        ("limit" = Option<usize>, Query, description = "Page size (default 50)"),
        ("cursor" = Option<String>, Query, description = "Pagination offset cursor")
    ),
    responses(
        (status = 200, description = "Paginated list of data quality warnings (Admin only)", body = QualityWarningsListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_quality_warnings(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<QualityWarningsFilterQuery>,
) -> Result<Json<QualityWarningsListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let include_dismissed = query.include_dismissed.unwrap_or(false);
    let mut warnings = compute_quality_warnings(&store, include_dismissed);

    if let Some(rule_filter) = query
        .rule
        .as_deref()
        .or(query.code.as_deref())
        .or(query.rule_code.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.rule_code.eq_ignore_ascii_case(rule_filter));
    }
    if let Some(cat_filter) = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.category.eq_ignore_ascii_case(cat_filter));
    }
    if let Some(sev_filter) = query
        .severity
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.severity.eq_ignore_ascii_case(sev_filter));
    }

    let total = warnings.len();
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(2000).clamp(1, 5000);
    let page: Vec<_> = warnings.into_iter().skip(offset).take(limit).collect();
    let next_cursor = if offset + page.len() < total {
        Some((offset + page.len()).to_string())
    } else {
        None
    };

    Ok(Json(QualityWarningsListResponse {
        items: page,
        next_cursor,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/warnings/{id}/dismiss",
    params(
        ("id" = String, Path, description = "Composite warning ID (e.g. DEATH_BEFORE_BIRTH:<entity_id>)")
    ),
    request_body(content = Option<DismissWarningRequest>, description = "Optional dismissal metadata"),
    responses(
        (status = 200, description = "Warning dismissed and persisted in dismissal memory (Admin only)", body = DismissWarningResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn dismiss_quality_warning_by_id(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Option<Json<DismissWarningRequest>>,
) -> Result<Json<DismissWarningResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let mut store = state.privacy_store.write().await;
    let resp =
        dismiss_warning_in_store(&mut store, Some(&id), req, Some(admin.user.id.to_string()))?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/warnings/dismiss",
    request_body = DismissWarningRequest,
    responses(
        (status = 200, description = "Warning dismissed by rule_code + entity_id (Admin only)", body = DismissWarningResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn dismiss_quality_warning(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<DismissWarningRequest>,
) -> Result<Json<DismissWarningResponse>, AppError> {
    let mut store = state.privacy_store.write().await;
    let resp = dismiss_warning_in_store(&mut store, None, req, Some(admin.user.id.to_string()))?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/duplicates",
    params(
        ("min_score" = Option<i32>, Query, description = "Minimum similarity score (default 75)"),
        ("limit" = Option<usize>, Query, description = "Page size (default 50)"),
        ("cursor" = Option<String>, Query, description = "Pagination offset cursor")
    ),
    responses(
        (status = 200, description = "Candidate duplicate pairs sorted by similarity score descending (Admin only)", body = DuplicateCandidatesListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_duplicate_candidates(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<DuplicateCandidatesFilterQuery>,
) -> Result<Json<DuplicateCandidatesListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let min_score = query.min_score.unwrap_or(75).clamp(1, 100);
    let all = compute_duplicate_candidates(&store, min_score);
    let total = all.len();
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(500).clamp(1, 2000);
    let items: Vec<_> = all.into_iter().skip(offset).take(limit).collect();
    let next_cursor = if offset + items.len() < total {
        Some((offset + items.len()).to_string())
    } else {
        None
    };

    Ok(Json(DuplicateCandidatesListResponse {
        items,
        next_cursor,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/duplicates/not-duplicate",
    request_body = NotDuplicatePairRequest,
    responses(
        (status = 200, description = "Pair marked as not-duplicates and excluded from future candidate lists (Admin only)", body = NotDuplicatePairResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn mark_pair_not_duplicate(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<NotDuplicatePairRequest>,
) -> Result<Json<NotDuplicatePairResponse>, AppError> {
    let mut store = state.privacy_store.write().await;
    let resp = mark_not_duplicate_in_store(&mut store, req)?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/match",
    params(
        ("name" = Option<String>, Query, description = "Full name to match"),
        ("given_name" = Option<String>, Query, description = "Given name"),
        ("surname" = Option<String>, Query, description = "Surname"),
        ("patronymic" = Option<String>, Query, description = "Patronymic"),
        ("birth_year" = Option<i32>, Query, description = "Approximate birth year"),
        ("sex" = Option<String>, Query, description = "Sex ('M', 'F', 'U')")
    ),
    responses(
        (status = 200, description = "Inline duplicate match candidates during person creation (Editor or Admin)", body = PersonMatchResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn match_people_candidates(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Query(query): Query<PersonMatchQuery>,
) -> Result<Json<PersonMatchResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = if editor.user.role == "admin" {
        CallerAccess::Admin
    } else {
        CallerAccess::Editor
    };
    let store = state.privacy_store.read().await;
    let resp = match_prospective_person(&store, &query, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/merge/preview",
    params(
        ("keep_id" = Option<String>, Query, description = "Surviving (keep/target) person UUID or GID"),
        ("merge_id" = Option<String>, Query, description = "Source (merge) person UUID or GID"),
        ("target_id" = Option<String>, Query, description = "Alias for keep_id"),
        ("source_id" = Option<String>, Query, description = "Alias for merge_id")
    ),
    responses(
        (status = 200, description = "Side-by-side person merge preview and conflict analysis (Admin only)", body = MergePreviewResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn preview_merge_people_get(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<MergePreviewQuery>,
) -> Result<Json<MergePreviewResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = preview_person_merge(&store, &query)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/people/merge/preview",
    request_body = MergePreviewQuery,
    responses(
        (status = 200, description = "Side-by-side person merge preview and conflict analysis (Admin only)", body = MergePreviewResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn preview_merge_people_post(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(query): Json<MergePreviewQuery>,
) -> Result<Json<MergePreviewResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = preview_person_merge(&store, &query)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/people/merge",
    request_body(content = Option<PersonMergeRequest>, description = "Merge parameters (keep_id/target_id, merge_id/source_id, field_choices)"),
    responses(
        (status = 200, description = "Atomic undoable duplicate person merge executed (Admin only)", body = PersonMergeResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse),
        (status = 422, description = "Self-merge or ancestor cycle detected", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn merge_duplicate_people(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<PersonMergeRequest>>,
) -> Result<Json<PersonMergeResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let mut resp = execute_person_merge(
        &mut store,
        req,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        &state.config.backup_path,
    )?;

    if let Some(chosen_surv) = resp.surviving_person.clone() {
        recompute_all_places_metadata(&mut store);
        recompute_all_individuals_vitals(&mut store, current_year, age_threshold);
        if let Some(person) = store.persons.iter_mut().find(|p| p.id == chosen_surv.id) {
            if chosen_surv.birth.is_some() {
                person.birth = chosen_surv.birth;
                person.birth_sort = chosen_surv.birth_sort;
            }
            if chosen_surv.death.is_some() {
                person.death = chosen_surv.death;
                person.death_sort = chosen_surv.death_sort;
                person.has_death_event = chosen_surv.has_death_event;
            }
            resp.surviving_person = Some(person.clone());
        }
    }

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/places/analysis",
    params(PlaceHierarchyAnalysisQuery),
    responses(
        (status = 200, description = "Place hierarchy normalization and coordinate proximity analysis (Admin only)", body = PlaceHierarchyAnalysisResponseDto),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_quality_places_analysis(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<PlaceHierarchyAnalysisQuery>,
) -> Result<Json<PlaceHierarchyAnalysisResponseDto>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = analyze_place_hierarchies(&store, &query);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/places/preview",
    params(PlaceHierarchyPreviewQuery),
    responses(
        (status = 200, description = "Detailed place hierarchy fix preview for a single place (Admin only)", body = PlaceHierarchyIssueDto),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Place not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn preview_quality_place_issue(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<PlaceHierarchyPreviewQuery>,
) -> Result<Json<PlaceHierarchyIssueDto>, AppError> {
    let raw_id = query
        .id
        .as_deref()
        .or(query.place_id.as_deref())
        .unwrap_or_default();
    let store = state.privacy_store.read().await;
    let resp = preview_place_hierarchy_issue(&store, raw_id)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/places/fix",
    request_body = FixPlaceHierarchyRequest,
    responses(
        (status = 200, description = "Apply place hierarchy restructuring for a single place with atomic Undo tx_id (Admin only)", body = FixPlaceHierarchyResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 422, description = "Validation error or place hierarchy cycle detected", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn fix_quality_place_hierarchy(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(payload): Json<FixPlaceHierarchyRequest>,
) -> Result<Json<FixPlaceHierarchyResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let mut store = state.privacy_store.write().await;
    let resp = fix_place_hierarchy(
        &mut store,
        payload,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        current_year,
        age_threshold,
    )?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/places/batch-fix",
    request_body(content = Option<BatchFixPlacesRequest>, description = "Optional batch place fix filter (place_ids, min_confidence, issue_type)"),
    responses(
        (status = 200, description = "Batch auto-fix high-confidence place hierarchy issues under a single atomic tx_id (Admin only)", body = BatchFixPlacesResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn batch_fix_quality_places(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<BatchFixPlacesRequest>>,
) -> Result<Json<BatchFixPlacesResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let mut store = state.privacy_store.write().await;
    let resp = batch_fix_places(
        &mut store,
        req,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        current_year,
        age_threshold,
    )?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/places/dismiss",
    request_body = DismissPlaceIssueRequest,
    responses(
        (status = 200, description = "Dismiss a place hierarchy quality suggestion (Admin only)", body = DismissWarningResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Place not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn dismiss_quality_place_issue(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(payload): Json<DismissPlaceIssueRequest>,
) -> Result<Json<DismissWarningResponse>, AppError> {
    let mut store = state.privacy_store.write().await;
    let resp = dismiss_place_issue_in_store(&mut store, payload, Some(admin.user.id.to_string()))?;
    Ok(Json(resp))
}
