#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/dashboard",
    params(
        ("days" = Option<u32>, Query, description = "Anniversary lookahead window in days (default 30)"),
        ("scope" = Option<String>, Query, description = "Anniversary scope ('all' or 'close')"),
        ("reference_date" = Option<String>, Query, description = "Optional reference date (YYYY-MM-DD) for deterministic anniversary calculation")
    ),
    responses(
        (status = 200, description = "Home dashboard widgets: upcoming anniversaries, recent activity, stats, latest photos, and quality summary", body = DashboardResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_dashboard(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<AnniversariesQuery>,
) -> Result<Json<DashboardResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let (preferred_home_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let resp = build_dashboard_response(
        &store,
        &access,
        &settings.default_locale,
        preferred_home_id.as_deref(),
        &query,
    );
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/anniversaries",
    params(
        ("days" = Option<u32>, Query, description = "Lookahead window in days (default 30, max 365)"),
        ("scope" = Option<String>, Query, description = "Filter scope ('all' or 'close')"),
        ("home_person_id" = Option<String>, Query, description = "Anchor person ID for 'close' scope"),
        ("reference_date" = Option<String>, Query, description = "Optional YYYY-MM-DD reference date")
    ),
    responses(
        (status = 200, description = "Upcoming birthdays, wedding anniversaries, and milestone memorials with Julian-to-Gregorian shifting", body = AnniversariesListResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_anniversaries(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(mut query): Query<AnniversariesQuery>,
) -> Result<Json<AnniversariesListResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    if query.home_person_id.is_none() {
        let (home_id, _, _, _) = resolve_home_person_for_caller(
            &state,
            &caller,
            &store.persons,
            &settings.default_locale,
            settings.default_home_person_id.as_deref(),
        )
        .await;
        query.home_person_id = home_id;
    }

    let resp = compute_upcoming_anniversaries(&store, &query, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/stats",
    responses(
        (status = 200, description = "Tree-wide demographic and completeness statistics", body = TreeStatsResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_stats(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<TreeStatsResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let resp = compute_tree_stats(&store, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/kinship",
    params(
        ("from" = Option<String>, Query, description = "Starting person UUID or GID"),
        ("to" = Option<String>, Query, description = "Target person UUID or GID"),
        ("person_a" = Option<String>, Query, description = "Alias for from"),
        ("person_b" = Option<String>, Query, description = "Alias for to"),
        ("max_traces" = Option<usize>, Query, description = "Maximum number of distinct kinship traces (1..=10, default 5)")
    ),
    responses(
        (status = 200, description = "Shortest kinship relationship path and degree of kinship between two individuals", body = KinshipResponse),
        (status = 404, description = "Person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_kinship(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<KinshipQuery>,
) -> Result<Json<KinshipResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let raw_from = query
        .from
        .as_deref()
        .or(query.person_a.as_deref())
        .or(query.id1.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_to = query
        .to
        .as_deref()
        .or(query.person_b.as_deref())
        .or(query.id2.as_deref())
        .unwrap_or_default()
        .trim();

    let from_id = store
        .resolve_person_id(raw_from)
        .ok_or(AppError::PersonNotFound)?;
    let to_id = store
        .resolve_person_id(raw_to)
        .ok_or(AppError::PersonNotFound)?;

    let endpoint_visible = |pid: &str| -> bool {
        store
            .persons
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| redact_person_for_caller(p, &access, "en"))
            .is_some()
    };

    if !endpoint_visible(&from_id) || !endpoint_visible(&to_id) {
        return Err(AppError::PersonNotFound);
    }

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .cloned()
        .collect();

    let max_traces = query.max_traces.unwrap_or(5).clamp(1, 10);
    let resp = compute_kinship_path(
        &from_id,
        &to_id,
        &visible_persons,
        &visible_families,
        max_traces,
    );
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/tree/fan",
    params(
        ("root" = Option<String>, Query, description = "Root person UUID or GID (defaults to caller's home person)"),
        ("focal_id" = Option<String>, Query, description = "Alias for root"),
        ("generations" = Option<u32>, Query, description = "Number of ancestor generations (4..=8, default 5)")
    ),
    responses(
        (status = 200, description = "Ahnentafel-indexed ancestor fan chart hierarchy with quadrant metadata", body = FanChartResponse),
        (status = 404, description = "Root person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_fan_chart(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<FanChartQuery>,
) -> Result<Json<FanChartResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();

    let raw_root = query
        .root
        .as_deref()
        .or(query.focal_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let root_id = if let Some(requested) = raw_root {
        let resolved = store
            .resolve_person_id(requested)
            .ok_or(AppError::PersonNotFound)?;
        if !visible_persons.iter().any(|p| p.id == resolved) {
            return Err(AppError::PersonNotFound);
        }
        resolved
    } else {
        let (home_opt, _, _, _) = resolve_home_person_for_caller(
            &state,
            &caller,
            &store.persons,
            &settings.default_locale,
            settings.default_home_person_id.as_deref(),
        )
        .await;
        home_opt
            .filter(|hid| visible_persons.iter().any(|p| &p.id == hid))
            .or_else(|| visible_persons.first().map(|p| p.id.clone()))
            .ok_or(AppError::PersonNotFound)?
    };

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .cloned()
        .collect();

    let generations = query.generations.unwrap_or(5).clamp(1, 8);
    let resp = compute_fan_chart(&root_id, generations, &visible_persons, &visible_families);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/places/map",
    responses(
        (status = 200, description = "Clustered map markers for all geocoded places with event and person counts (authenticated users only)", body = PlacesMapResponse),
        (status = 401, description = "Authentication required even when public tree sharing is enabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_places_map(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<PlacesMapResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();

    let resp = compute_places_map(&store, &visible_persons);
    Ok(Json(resp))
}
