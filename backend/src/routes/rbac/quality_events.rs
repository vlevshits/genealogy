#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/quality/events/duplicates",
    params(DuplicateEventsFilterQuery),
    responses(
        (status = 200, description = "List duplicate life event groups for individuals and couples (Admin only)", body = DuplicateEventsListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_duplicate_events(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<DuplicateEventsFilterQuery>,
) -> Result<Json<DuplicateEventsListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let groups = compute_duplicate_event_groups(&store, query.include_dismissed.unwrap_or(false));
    Ok(Json(filter_duplicate_event_groups(groups, &query)))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/events/merge",
    request_body = MergeDuplicateEventsRequest,
    responses(
        (status = 200, description = "Merge duplicate events into a surviving event with atomic Undo tx_id (Admin only)", body = MergeDuplicateEventsResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Duplicate group, event, or place not found", body = ErrorResponse),
        (status = 400, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn merge_duplicate_events_handler(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(payload): Json<MergeDuplicateEventsRequest>,
) -> Result<Json<MergeDuplicateEventsResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let mut store = state.privacy_store.write().await;
    let resp = merge_duplicate_events_in_store(
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
    path = "/api/v1/quality/events/batch-merge",
    request_body(content = Option<BatchMergeDuplicateEventsRequest>, description = "Optional batch duplicate event merge filter (group_ids, only_high_confidence)"),
    responses(
        (status = 200, description = "Atomically merge eligible duplicate event groups under one Undo tx_id (Admin only)", body = BatchMergeDuplicateEventsResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn batch_merge_duplicate_events_handler(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<BatchMergeDuplicateEventsRequest>>,
) -> Result<Json<BatchMergeDuplicateEventsResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let mut store = state.privacy_store.write().await;
    let resp = batch_merge_duplicate_events_in_store(
        &mut store,
        req,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        current_year,
        age_threshold,
    )?;
    Ok(Json(resp))
}
