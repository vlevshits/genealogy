use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    error::ErrorResponse,
    routes::{
        create_router,
        rbac::{
            AdminToolOperationResponse, MediaArchiveResponse, PeopleDirectoryResponse,
            PlacesDirectoryResponse, TreeResponse,
        },
    },
    services::{
        auth::{
            AdminUserSummaryDto, AuthService, CreateInviteRequest, InviteRegisterRequest,
            SetupRequest,
        },
        privacy::{evaluate_is_living, PersonLivingEvaluationInput, TreePersonNodeDto},
    },
    state::AppState,
};
use tower::ServiceExt;

#[test]
fn test_canonical_living_status_algorithm_all_five_branches() {
    let current_year = 2026;
    let threshold = 110;

    // Branch 1: Explicit living_override wins unconditionally
    let override_deceased = PersonLivingEvaluationInput {
        living_override: Some(false),
        has_death_or_burial_event: false,
        birth_year: Some(2015),
        ..Default::default()
    };
    assert!(!evaluate_is_living(
        &override_deceased,
        current_year,
        threshold
    ));

    let override_living = PersonLivingEvaluationInput {
        living_override: Some(true),
        has_death_or_burial_event: true,
        birth_year: Some(1850),
        ..Default::default()
    };
    assert!(evaluate_is_living(
        &override_living,
        current_year,
        threshold
    ));

    // Branch 2: Any death, burial, or cremation event -> Deceased (false)
    let has_death = PersonLivingEvaluationInput {
        living_override: None,
        has_death_or_burial_event: true,
        birth_year: Some(1998),
        ..Default::default()
    };
    assert!(!evaluate_is_living(&has_death, current_year, threshold));

    // Branch 3: Known birth year compared against LIVING_PERSON_AGE_THRESHOLD
    let born_116_years_ago = PersonLivingEvaluationInput {
        birth_year: Some(1910),
        ..Default::default()
    };
    assert!(!evaluate_is_living(
        &born_116_years_ago,
        current_year,
        threshold
    ));

    let born_109_years_ago = PersonLivingEvaluationInput {
        birth_year: Some(1917),
        ..Default::default()
    };
    assert!(evaluate_is_living(
        &born_109_years_ago,
        current_year,
        threshold
    ));

    // Branch 4a: Estimate from earliest child birth year (child_birth_year - 25)
    let old_child = PersonLivingEvaluationInput {
        earliest_child_birth_year: Some(1935), // est birth = 1910 -> age 116 >= 110 -> deceased
        ..Default::default()
    };
    assert!(!evaluate_is_living(&old_child, current_year, threshold));

    let young_child = PersonLivingEvaluationInput {
        earliest_child_birth_year: Some(1965), // est birth = 1940 -> age 86 < 110 -> living
        ..Default::default()
    };
    assert!(evaluate_is_living(&young_child, current_year, threshold));

    // Branch 4b: Estimate from partner birth year (±0)
    let old_partner = PersonLivingEvaluationInput {
        partner_birth_year: Some(1905), // age 121 >= 110 -> deceased
        ..Default::default()
    };
    assert!(!evaluate_is_living(&old_partner, current_year, threshold));

    let young_partner = PersonLivingEvaluationInput {
        partner_birth_year: Some(1955), // age 71 < 110 -> living
        ..Default::default()
    };
    assert!(evaluate_is_living(&young_partner, current_year, threshold));

    // Branch 4c: Estimate from parent birth year (parent_birth_year + 25)
    let old_parent = PersonLivingEvaluationInput {
        parent_birth_year: Some(1880), // est birth = 1905 -> age 121 >= 110 -> deceased
        ..Default::default()
    };
    assert!(!evaluate_is_living(&old_parent, current_year, threshold));

    let young_parent = PersonLivingEvaluationInput {
        parent_birth_year: Some(1930), // est birth = 1955 -> age 71 < 110 -> living
        ..Default::default()
    };
    assert!(evaluate_is_living(&young_parent, current_year, threshold));

    // Branch 4d: Estimate from marriage year (marriage_year - 25)
    let old_marriage = PersonLivingEvaluationInput {
        marriage_year: Some(1935), // est birth = 1910 -> age 116 >= 110 -> deceased
        ..Default::default()
    };
    assert!(!evaluate_is_living(&old_marriage, current_year, threshold));

    let recent_marriage = PersonLivingEvaluationInput {
        marriage_year: Some(1990), // est birth = 1965 -> age 61 < 110 -> living
        ..Default::default()
    };
    assert!(evaluate_is_living(
        &recent_marriage,
        current_year,
        threshold
    ));

    // Branch 5: Unknown dates and no relatives -> Living (true, protected by default)
    let completely_unknown = PersonLivingEvaluationInput::default();
    assert!(evaluate_is_living(
        &completely_unknown,
        current_year,
        threshold
    ));
}

#[tokio::test]
async fn test_tier1_default_private_gatekeeper_and_tier2_public_mode_masking_and_media_shield() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Levshits Family Archive".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);

    // Bootstrap Admin
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: None,
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminStrongPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let app = create_router(state);

    // =========================================================================
    // Tier 1: Default Private Gatekeeper (`ENABLE_PUBLIC_ACCESS=false`)
    // =========================================================================
    for blocked_uri in [
        "/api/v1/tree",
        "/api/v1/persons/I0001",
        "/api/v1/persons/I0042",
        "/api/v1/people",
        "/api/v1/places",
        "/api/v1/media",
        "/api/v1/media/00000000-0000-0000-0000-000000000101/file",
        "/api/v1/admin/settings",
    ] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(blocked_uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for unauthenticated request to {blocked_uri} when public access is disabled"
        );
    }

    // =========================================================================
    // Tier 2: Admin enables Public Tree Mode (`enable_public_access = true`)
    // =========================================================================
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"enable_public_access":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2a. Unauthenticated visitor fetches GET /api/v1/tree
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tree.access_tier, "public_anonymous");
    // Only 2 nodes visible (deceased I0001 + masked living I0042; private I0099 is stripped)
    assert_eq!(tree.nodes.len(), 2);
    assert!(!tree.nodes.iter().any(|n| n.gid == "I0099"));

    let deceased = tree.nodes.iter().find(|n| n.gid == "I0001").unwrap();
    assert!(!deceased.is_living);
    assert_eq!(deceased.primary_name.given_name, "Mikhail");
    assert_eq!(
        deceased.primary_name.patronymic.as_deref(),
        Some("Isaakovich")
    );
    assert!(deceased.birth.is_some());
    assert!(deceased.death.is_some());
    assert!(deceased.avatar_url.is_some());

    let living_masked = tree.nodes.iter().find(|n| n.gid == "I0042").unwrap();
    assert!(living_masked.is_living);
    assert_eq!(living_masked.primary_name.given_name, "Living");
    assert_eq!(living_masked.primary_name.patronymic, None);
    assert_eq!(living_masked.primary_name.surname, "Levshits");
    assert_eq!(living_masked.birth, None);
    assert_eq!(living_masked.death, None);
    assert_eq!(living_masked.avatar_url, None);
    assert_eq!(living_masked.notes, None);

    // 2b. Unauthenticated visitor fetching private record I0099 directly gets 404 Not Found
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/persons/I0099")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 2c. Direct Media Shield (`02 §7 DoD #3`):
    // Fetching a living person's photo as an unauthenticated visitor returns 404 Not Found
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/media/00000000-0000-0000-0000-000000000102/file")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Fetching a deceased person's photo in Public Mode returns 200 OK
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/media/00000000-0000-0000-0000-000000000101/file")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("image/webp")
    );

    // 2d. Auxiliary directories (/people, /places, /media) remain 401 Unauthorized in Public Mode
    for dir_uri in ["/api/v1/people", "/api/v1/places", "/api/v1/media"] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(dir_uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "Directory {dir_uri} must remain 401 for unauthenticated visitors even in public mode"
        );
    }

    // 2e. Russian locale masks living person given_name as "Живущий"
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"default_locale":"ru"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/persons/I0042")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let ru_masked: TreePersonNodeDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(ru_masked.primary_name.given_name, "Живущий");
    assert_eq!(ru_masked.primary_name.patronymic, None);
    assert_eq!(ru_masked.birth, None);
}

#[tokio::test]
async fn test_tier3_viewer_and_tier4_editor_and_tier5_admin_rbac_matrix() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "RBAC Matrix Archive".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);

    // 1. Setup Admin, Viewer, and Editor accounts
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: None,
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminStrongPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
    )
    .await
    .unwrap();
    let admin_id = uuid::Uuid::parse_str(&admin_setup.user.id).unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);

    let viewer_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            email: Some("viewer@levshits.family".to_string()),
            invitee_name: Some("Vera Viewer".to_string()),
            role: "viewer".to_string(),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (viewer_reg, _) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: viewer_inv.token,
            first_name: "Vera".to_string(),
            last_name: "Viewer".to_string(),
            email: Some("viewer@levshits.family".to_string()),
            passkey: None,
            password: Some("ViewerStrongPass123!".to_string()),
        },
        None,
        None,
    )
    .await
    .unwrap();
    let viewer_auth = format!("Bearer {}", viewer_reg.session_token);

    let editor_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            email: Some("editor@levshits.family".to_string()),
            invitee_name: Some("Dmitry Editor".to_string()),
            role: "editor".to_string(),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (editor_reg, _) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: editor_inv.token,
            first_name: "Dmitry".to_string(),
            last_name: "Editor".to_string(),
            email: Some("editor@levshits.family".to_string()),
            passkey: None,
            password: Some("EditorStrongPass123!".to_string()),
        },
        None,
        None,
    )
    .await
    .unwrap();
    let editor_auth = format!("Bearer {}", editor_reg.session_token);

    let app = create_router(state);

    // =========================================================================
    // Tier 3: Authenticated Viewer (`02 §3.3 & §7 DoD #4, #5, #6, #7, #8, #9`)
    // =========================================================================
    // Viewer sees full unredacted living relatives on /api/v1/tree, but NOT is_private=true
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let viewer_tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(viewer_tree.access_tier, "viewer");
    assert_eq!(viewer_tree.nodes.len(), 2);
    assert!(!viewer_tree.nodes.iter().any(|n| n.gid == "I0099"));

    let elena = viewer_tree.nodes.iter().find(|n| n.gid == "I0042").unwrap();
    assert_eq!(elena.primary_name.given_name, "Elena");
    assert_eq!(
        elena.primary_name.patronymic.as_deref(),
        Some("Alexandrovna")
    );
    assert_eq!(
        elena.birth.as_ref().and_then(|b| b.date.as_deref()),
        Some("1994-06-14")
    );
    assert!(elena.avatar_url.is_some());
    assert!(elena.notes.is_some());

    // Viewer can fetch living person's photo via /api/v1/media/{id}/file (200 OK)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/media/00000000-0000-0000-0000-000000000102/file")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Viewer can browse /people, /places, and /media directories
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/people")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let people_dir: PeopleDirectoryResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(people_dir.total, 2);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/places")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let places_dir: PlacesDirectoryResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(places_dir.total, 2);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/media")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let media_dir: MediaArchiveResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(media_dir.total, 2); // M0001 + M0002; private M0003 is hidden from Viewer

    // Viewer Read-Only Boundary (`02 §7 DoD #5`): POST/PATCH/DELETE on /persons return 403 Forbidden
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &viewer_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"given_name":"Pavel","surname":"Levshits"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/persons/I0042")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &viewer_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"given_name":"Modified"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/v1/persons/I0042")
                .header(header::AUTHORIZATION, &viewer_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // =========================================================================
    // Tier 4: Authenticated Editor (`02 §3.4 & §7 DoD #6, #7, #8, #9`)
    // =========================================================================
    // Editor sees all 3 records on /api/v1/tree including private I0099
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::AUTHORIZATION, &editor_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let editor_tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(editor_tree.access_tier, "editor");
    assert_eq!(editor_tree.nodes.len(), 3);
    assert!(editor_tree.nodes.iter().any(|n| n.gid == "I0099"));

    // Editor can POST, PATCH, and DELETE individuals
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &editor_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    r#"{"given_name":"Sofia","surname":"Levshits","sex":"F","birth_year":2021}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let created_person: TreePersonNodeDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(created_person.primary_name.given_name, "Sofia");
    assert!(created_person.is_living);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", created_person.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &editor_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"is_private":true,"living_override":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let patched_person: TreePersonNodeDto = serde_json::from_slice(&body).unwrap();
    assert!(patched_person.is_private);
    assert!(!patched_person.is_living);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/persons/{}", created_person.id))
                .header(header::AUTHORIZATION, &editor_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // =========================================================================
    // Admin-Only Isolation for both Viewer and Editor (`02 §7 DoD #6, #7, #8, #9`)
    // =========================================================================
    let admin_only_requests = [
        (Method::GET, "/api/v1/gedcom/export"),
        (Method::POST, "/api/v1/gedcom/import"),
        (Method::POST, "/api/v1/export/pdf"),
        (Method::GET, "/api/v1/history"),
        (Method::GET, "/api/v1/quality"),
        (Method::POST, "/api/v1/people/merge"),
        (Method::GET, "/api/v1/admin/settings"),
        (Method::PATCH, "/api/v1/admin/settings"),
        (Method::GET, "/api/v1/admin/backups"),
        (Method::POST, "/api/v1/admin/backups"),
        (Method::GET, "/api/v1/admin/system"),
        (Method::GET, "/api/v1/admin/users"),
        (Method::GET, "/api/v1/admin/invites"),
        (Method::POST, "/api/v1/admin/invites"),
    ];

    for (role_label, auth_header) in [("viewer", &viewer_auth), ("editor", &editor_auth)] {
        for (method, uri) in &admin_only_requests {
            let body_payload = if *method == Method::POST || *method == Method::PATCH {
                Body::from("{}")
            } else {
                Body::empty()
            };
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(*uri)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header(header::AUTHORIZATION, auth_header)
                        .header("x-requested-with", "genealogy")
                        .body(body_payload)
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                res.status(),
                StatusCode::FORBIDDEN,
                "Expected 403 Forbidden when {role_label} calls {method} {uri}"
            );
        }
    }

    // =========================================================================
    // Tier 5: Administrator succeeds on all isolated bulk, audit, quality & merge endpoints
    // =========================================================================
    for (method, uri, expected_op) in [
        (Method::GET, "/api/v1/gedcom/export", "gedcom_export"),
        (Method::POST, "/api/v1/gedcom/import", "gedcom_import"),
        (Method::POST, "/api/v1/export/pdf", "pdf_export"),
        (Method::GET, "/api/v1/history", "audit_history"),
        (Method::GET, "/api/v1/quality", "data_quality"),
        (Method::POST, "/api/v1/people/merge", "person_merge"),
    ] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, &admin_auth)
                    .header("x-requested-with", "genealogy")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
        let op_res: AdminToolOperationResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(op_res.operation, expected_op);
    }
}

#[tokio::test]
async fn test_sole_admin_protection_role_promotion_and_deactivation_session_purge() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Sole Admin Protection Archive".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);

    // 1. Bootstrap sole Admin
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: None,
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminStrongPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
    )
    .await
    .unwrap();
    let admin_id = admin_setup.user.id.clone();
    let admin_uuid = uuid::Uuid::parse_str(&admin_id).unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);

    // 2. Invite & register an Editor
    let inv = AuthService::create_invitation(
        &state,
        Some(admin_uuid),
        CreateInviteRequest {
            email: Some("boris@levshits.family".to_string()),
            invitee_name: Some("Boris Levshits".to_string()),
            role: "editor".to_string(),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (boris_reg, _) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: inv.token,
            first_name: "Boris".to_string(),
            last_name: "Levshits".to_string(),
            email: Some("boris@levshits.family".to_string()),
            passkey: None,
            password: Some("BorisStrongPass123!".to_string()),
        },
        None,
        None,
    )
    .await
    .unwrap();
    let boris_id = boris_reg.user.id.clone();
    let boris_auth = format!("Bearer {}", boris_reg.session_token);

    let app = create_router(state);

    // 3. Sole Admin attempting to demote self to Editor is rejected with 400 sole_admin_protected
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{admin_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"role":"editor"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "sole_admin_protected");

    // 4. Sole Admin attempting to deactivate self (`is_active: false`) is rejected with 400 sole_admin_protected
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{admin_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"is_active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "sole_admin_protected");

    // 5. Admin promotes Boris to `admin` -> now 2 active admins exist
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{boris_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"role":"admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let promoted: AdminUserSummaryDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(promoted.role, "admin");

    // 6. With 2 active admins, demoting Boris back to `viewer` succeeds
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{boris_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"role":"viewer"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let demoted: AdminUserSummaryDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(demoted.role, "viewer");

    // 7. Admin deactivates Boris (`is_active: false`) -> purges Boris's sessions immediately
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{boris_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"is_active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let deactivated: AdminUserSummaryDto = serde_json::from_slice(&body).unwrap();
    assert!(!deactivated.is_active);
    assert!(deactivated.deactivated_at.is_some());

    // Boris's active session token immediately returns 401 Unauthorized on both /auth/me and /tree
    for uri in ["/api/v1/auth/me", "/api/v1/tree"] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(header::AUTHORIZATION, &boris_auth)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
