use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::auth::{AuthService, SetupRequest},
    state::AppState,
};

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        tree_title: "Tree Projections Test Archive".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    }
}

async fn read_json(resp: axum::response::Response) -> Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

/// Builds a 4-generation family tree around focal person "Anna" (I0100):
/// - Grandfather (Pyotr, deceased 1980) + Grandmother (Maria, deceased 1985) -> Father (Nikolai, deceased 2010)
/// - Father (Nikolai) + Mother (Olga, living) -> Focal (Anna, living)
/// - Focal (Anna) + Spouse (Ivan, living, Cyrillic "Иван") -> Child (Dmitry, living) -> Grandchild (Sofia, living)
/// - Also creates an unrelated Private Person (Secret Person, `is_private: true`)
#[allow(clippy::too_many_lines)]
async fn setup_populated_tree() -> (
    axum::Router,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let state = AppState::new_empty(None, test_config());
    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Romanov Tree Projections".to_string()),
            first_name: "Anna".to_string(),
            last_name: "Romanova".to_string(),
            email: "admin@example.com".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        Some("127.0.0.1".to_string()),
        false,
    )
    .await
    .unwrap();

    let app = create_router(state);

    // 1. Create Focal person (Anna) via onboarding/start-with-self -> I0100
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Anna",
                        "patronymic": "Nikolaevna",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "15 MAY 1985",
                        "birth_place": "Saint Petersburg, Russia"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let focal_id = read_json(resp).await["individual"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Quick-add Father (Nikolai, deceased)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{focal_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "father",
                        "given_name": "Nikolai",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1955",
                        "death_date": "2010"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let father_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Quick-add Mother (Olga, living)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{focal_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "mother",
                        "given_name": "Olga",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "1958"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let mother_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. Quick-add Paternal Grandfather (Pyotr, deceased) to Father
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{father_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "father",
                        "given_name": "Pyotr",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1925",
                        "death_date": "1980"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let grandfather_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 5. Quick-add Spouse (Иван, living) to Focal
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{focal_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "spouse",
                        "given_name": "Иван",
                        "surname": "Sokolov",
                        "sex": "M",
                        "birth_date": "1984"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let spouse_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 6. Quick-add Child (Dmitry, living) to Focal
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{focal_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "given_name": "Dmitry",
                        "surname": "Sokolov",
                        "sex": "M",
                        "birth_date": "2010"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let child_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 7. Quick-add Grandchild (Sofia, living) to Child
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{child_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "given_name": "Sofia",
                        "surname": "Sokolova",
                        "sex": "F",
                        "birth_date": "2025"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let grandchild_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 8. Create a private person (is_private: true)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Confidential",
                        "surname": "Relative",
                        "sex": "U",
                        "is_private": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let _private_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    (
        app,
        admin_cookie,
        focal_id,
        father_id,
        mother_id,
        grandfather_id,
        spouse_id,
        child_id,
        grandchild_id,
    )
}

#[tokio::test]
async fn test_tree_response_includes_families_and_gatekeeper_enforced() {
    let (app, admin_cookie, focal_id, _, _, _, _, _, _) = setup_populated_tree().await;

    // 1. Private Gatekeeper: Unauthenticated GET /api/v1/tree/view when enable_public_access == false returns 401
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 2. GET /api/v1/tree returns both nodes and families
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["resolved_home_person_id"], focal_id);
    assert_eq!(body["nodes"].as_array().unwrap().len(), 8);
    assert_eq!(body["families"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn test_tree_view_hourglass_pedigree_and_descendant_projections() {
    let (
        app,
        admin_cookie,
        focal_id,
        father_id,
        mother_id,
        grandfather_id,
        spouse_id,
        child_id,
        grandchild_id,
    ) = setup_populated_tree().await;

    // 1. Default Hourglass mode (`GET /api/v1/tree/view`) returns focal, spouse, parents, grandfather, child, grandchild, and edges
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let hourglass = read_json(resp).await;
    assert_eq!(hourglass["mode"], "hourglass");
    assert_eq!(hourglass["generations"], 4);
    assert_eq!(hourglass["focal_id"], focal_id);

    let hg_node_ids: Vec<String> = hourglass["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    for expected in [
        &focal_id,
        &father_id,
        &mother_id,
        &grandfather_id,
        &spouse_id,
        &child_id,
        &grandchild_id,
    ] {
        assert!(
            hg_node_ids.contains(expected),
            "Hourglass view missing expected person {expected}"
        );
    }

    let edges = hourglass["edges"].as_array().unwrap();
    assert!(edges.iter().any(|e| e["kind"] == "spouse"));
    assert!(edges.iter().any(|e| e["kind"] == "parent_child"));

    // 2. Pedigree mode (`GET /api/v1/tree/view?mode=pedigree&focal_id=I0100`) returns ONLY focal + direct biological ancestors
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?mode=pedigree&focal_id=I0100")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let pedigree = read_json(resp).await;
    assert_eq!(pedigree["mode"], "pedigree");
    let ped_node_ids: Vec<String> = pedigree["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ped_node_ids.len(), 4);
    assert!(ped_node_ids.contains(&focal_id));
    assert!(ped_node_ids.contains(&father_id));
    assert!(ped_node_ids.contains(&mother_id));
    assert!(ped_node_ids.contains(&grandfather_id));
    assert!(!ped_node_ids.contains(&spouse_id));
    assert!(!ped_node_ids.contains(&child_id));
    assert!(!ped_node_ids.contains(&grandchild_id));

    // 3. Descendant mode (`GET /api/v1/tree/view?mode=descendant&focal_id=I0100`) returns focal, spouse, child, grandchild (no ancestors)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/tree/view?mode=descendant&focal_id={focal_id}"
                ))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let descendant = read_json(resp).await;
    assert_eq!(descendant["mode"], "descendant");
    let desc_node_ids: Vec<String> = descendant["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(desc_node_ids.len(), 4);
    assert!(desc_node_ids.contains(&focal_id));
    assert!(desc_node_ids.contains(&spouse_id));
    assert!(desc_node_ids.contains(&child_id));
    assert!(desc_node_ids.contains(&grandchild_id));
    assert!(!desc_node_ids.contains(&father_id));
    assert!(!desc_node_ids.contains(&mother_id));
    assert!(!desc_node_ids.contains(&grandfather_id));

    // 4. Generation depth limit (`generations=1` in pedigree mode returns focal + parents, excluding grandfather)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?mode=pedigree&focal_id=I0100&generations=1")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ped_gen1 = read_json(resp).await;
    let gen1_ids: Vec<String> = ped_gen1["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(gen1_ids.len(), 3);
    assert!(gen1_ids.contains(&focal_id));
    assert!(gen1_ids.contains(&father_id));
    assert!(gen1_ids.contains(&mother_id));
    assert!(!gen1_ids.contains(&grandfather_id));

    // 5. Non-existent focal_id returns 404 Not Found with code person_not_found
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?focal_id=I9999")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let err = read_json(resp).await;
    assert_eq!(err["error"]["code"], "person_not_found");
}

#[tokio::test]
async fn test_tree_view_public_living_person_masking() {
    let (app, admin_cookie, focal_id, father_id, _, _, spouse_id, _, _) =
        setup_populated_tree().await;

    // Enable public tree mode via PATCH /api/v1/admin/settings
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "enable_public_access": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Unauthenticated public visitor requests GET /api/v1/tree/view?focal_id=I0100
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?focal_id=I0100")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["access_tier"], "public_anonymous");

    let nodes = body["nodes"].as_array().unwrap();

    // Living focal person ("Anna Romanova") is masked to single initial with dot ("A.") and stripped of surname/dates/places
    let anna_masked = nodes.iter().find(|n| n["id"] == focal_id).unwrap();
    assert_eq!(anna_masked["is_living"], true);
    assert_eq!(anna_masked["primary_name"]["given_name"], "A.");
    assert_eq!(anna_masked["primary_name"]["surname"], "");
    assert!(anna_masked["primary_name"]["patronymic"].is_null());
    assert!(anna_masked["birth"].is_null());
    assert!(anna_masked["avatar_url"].is_null());
    assert!(anna_masked["notes"].is_null());

    // Living Cyrillic spouse ("Иван Sokolov") is masked to "И."
    let ivan_masked = nodes.iter().find(|n| n["id"] == spouse_id).unwrap();
    assert_eq!(ivan_masked["is_living"], true);
    assert_eq!(ivan_masked["primary_name"]["given_name"], "И.");
    assert_eq!(ivan_masked["primary_name"]["surname"], "");
    assert!(ivan_masked["birth"].is_null());

    // Deceased father ("Nikolai Romanov") remains unredacted
    let father_node = nodes.iter().find(|n| n["id"] == father_id).unwrap();
    assert_eq!(father_node["is_living"], false);
    assert_eq!(father_node["primary_name"]["given_name"], "Nikolai");
    assert_eq!(father_node["primary_name"]["surname"], "Romanov");
    assert_eq!(father_node["birth"]["date"], "1955");
    assert_eq!(father_node["death"]["date"], "2010");
}

#[tokio::test]
async fn test_wide_hourglass_collateral_expansion_and_lineage_metadata() {
    // Set up tree with enable_public_access = true to test both authenticated and public guests
    let mut config = test_config();
    config.enable_public_access = true;

    let state = AppState::new_empty(None, config);
    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Hourglass Traversal Test Tree".to_string()),
            first_name: "Anna".to_string(),
            last_name: "Romanova".to_string(),
            email: "admin@example.com".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        Some("127.0.0.1".to_string()),
        false,
    )
    .await
    .unwrap();

    let app = create_router(state);

    // Helper closure to quick-add a relative
    let quick_add = |app: axum::Router,
                     cookie: String,
                     anchor_id: String,
                     payload: serde_json::Value| async move {
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/v1/persons/{anchor_id}/quick-add"))
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        read_json(resp).await["person"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // 1. Focal person: Anna Romanova (living, b. 1985)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Anna",
                        "patronymic": "Nikolaevna",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "15 MAY 1985",
                        "birth_place": "Saint Petersburg, Russia"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let focal_id = read_json(resp).await["individual"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Father: Nikolai Romanov (deceased, 1955-2010)
    let father_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "father",
            "given_name": "Nikolai",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1955",
            "death_date": "2010"
        }),
    )
    .await;

    // 3. Mother: Olga Romanova (living, b. 1958)
    let mother_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "mother",
            "given_name": "Olga",
            "surname": "Romanova",
            "sex": "F",
            "birth_date": "1958"
        }),
    )
    .await;

    // 4. Paternal Grandfather: Pyotr Romanov (deceased, 1925-1980)
    let grandfather_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        father_id.clone(),
        json!({
            "relation": "father",
            "given_name": "Pyotr",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1925",
            "death_date": "1980"
        }),
    )
    .await;

    // 5. Spouse of Focal: Ivan Sokolov (living, b. 1984)
    let spouse_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "spouse",
            "given_name": "Иван",
            "surname": "Sokolov",
            "sex": "M",
            "birth_date": "1984"
        }),
    )
    .await;

    // 6. Child of Focal: Dmitry Sokolov (living, b. 2010)
    let child_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Dmitry",
            "surname": "Sokolov",
            "sex": "M",
            "birth_date": "2010"
        }),
    )
    .await;

    // 7. Grandchild: Sofia Sokolova (living, b. 2030)
    let grandchild_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        child_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Sofia",
            "surname": "Sokolova",
            "sex": "F",
            "birth_date": "2030"
        }),
    )
    .await;

    // 8. Sibling of Anna: Mikhail Romanov (living, b. 1988)
    let sibling_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "sibling",
            "given_name": "Mikhail",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1988"
        }),
    )
    .await;

    // 9. Sibling's Spouse: Elena Romanova (living, b. 1990)
    let sibling_spouse_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        sibling_id.clone(),
        json!({
            "relation": "spouse",
            "given_name": "Elena",
            "surname": "Romanova",
            "sex": "F",
            "birth_date": "1990"
        }),
    )
    .await;

    // 10. Sibling's Child (Niece of Anna): Maria Romanova (living, b. 2015)
    let niece_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        sibling_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Maria",
            "surname": "Romanova",
            "sex": "F",
            "birth_date": "2015"
        }),
    )
    .await;

    // 11. Paternal Uncle (Brother of Father Nikolai): Boris Romanov (deceased, 1952-2020)
    let uncle_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        father_id.clone(),
        json!({
            "relation": "sibling",
            "given_name": "Boris",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1952",
            "death_date": "2020"
        }),
    )
    .await;

    // 12. Paternal Uncle's Spouse: Tatiana Romanova (living, b. 1956)
    let uncle_spouse_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        uncle_id.clone(),
        json!({
            "relation": "spouse",
            "given_name": "Tatiana",
            "surname": "Romanova",
            "sex": "F",
            "birth_date": "1956"
        }),
    )
    .await;

    // 13. First Cousin (Child of Uncle Boris): Alexey Romanov (living, b. 1980)
    let cousin_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        uncle_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Alexey",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1980"
        }),
    )
    .await;

    // --- Scenario A: Wide Hourglass Traversal (collateral=true) ---
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?mode=hourglass&collateral=true")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let wide_tree = read_json(resp).await;

    assert_eq!(wide_tree["mode"], "hourglass");
    assert_eq!(wide_tree["collateral"], true);

    let nodes = wide_tree["nodes"].as_array().unwrap();
    let node_ids: Vec<String> = nodes
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    // Verify all 13 individuals (direct + collateral) are present
    for id in [
        &focal_id,
        &spouse_id,
        &father_id,
        &mother_id,
        &grandfather_id,
        &child_id,
        &grandchild_id,
        &sibling_id,
        &sibling_spouse_id,
        &niece_id,
        &uncle_id,
        &uncle_spouse_id,
        &cousin_id,
    ] {
        assert!(
            node_ids.contains(id),
            "Wide hourglass missing relative with id {id}"
        );
    }

    // Helper to find a node by id
    let get_node = |id: &str| -> &Value { nodes.iter().find(|n| n["id"] == id).unwrap() };

    // Verify Generational Tier Indices & Direct Lineage Flags
    // Gen -2: Grandfather Pyotr
    assert_eq!(get_node(&grandfather_id)["generation"], -2);
    assert_eq!(get_node(&grandfather_id)["is_direct_lineage"], true);

    // Gen -1: Parents (Nikolai, Olga) -> Direct Lineage
    assert_eq!(get_node(&father_id)["generation"], -1);
    assert_eq!(get_node(&father_id)["is_direct_lineage"], true);
    assert_eq!(get_node(&mother_id)["generation"], -1);
    assert_eq!(get_node(&mother_id)["is_direct_lineage"], true);

    // Gen -1: Collateral Aunts/Uncles (Boris, Tatiana) -> Collateral (is_direct = false)
    assert_eq!(get_node(&uncle_id)["generation"], -1);
    assert_eq!(get_node(&uncle_id)["is_direct_lineage"], false);
    assert_eq!(get_node(&uncle_spouse_id)["generation"], -1);
    assert_eq!(get_node(&uncle_spouse_id)["is_direct_lineage"], false);

    // Gen 0: Focal Person Anna -> Direct
    assert_eq!(get_node(&focal_id)["generation"], 0);
    assert_eq!(get_node(&focal_id)["is_direct_lineage"], true);

    // Gen 0: Focal Spouse Ivan -> Spouse (is_direct = false)
    assert_eq!(get_node(&spouse_id)["generation"], 0);
    assert_eq!(get_node(&spouse_id)["is_direct_lineage"], false);

    // Gen 0: Collateral Sibling & Spouse (Mikhail, Elena) -> Collateral
    assert_eq!(get_node(&sibling_id)["generation"], 0);
    assert_eq!(get_node(&sibling_id)["is_direct_lineage"], false);
    assert_eq!(get_node(&sibling_spouse_id)["generation"], 0);
    assert_eq!(get_node(&sibling_spouse_id)["is_direct_lineage"], false);

    // Gen 0: First Cousin (Alexey) -> Collateral
    assert_eq!(get_node(&cousin_id)["generation"], 0);
    assert_eq!(get_node(&cousin_id)["is_direct_lineage"], false);

    // Gen +1: Direct Child (Dmitry) -> Direct
    assert_eq!(get_node(&child_id)["generation"], 1);
    assert_eq!(get_node(&child_id)["is_direct_lineage"], true);

    // Gen +1: Niece (Maria, sibling's daughter) -> Collateral
    assert_eq!(get_node(&niece_id)["generation"], 1);
    assert_eq!(get_node(&niece_id)["is_direct_lineage"], false);

    // Gen +2: Direct Grandchild (Sofia) -> Direct
    assert_eq!(get_node(&grandchild_id)["generation"], 2);
    assert_eq!(get_node(&grandchild_id)["is_direct_lineage"], true);

    // Verify Edges
    let edges = wide_tree["edges"].as_array().unwrap();

    // Direct parent-child edges should have is_direct_lineage == true
    let edge_to_focal = edges.iter().find(|e| e["target"] == focal_id).unwrap();
    assert_eq!(edge_to_focal["is_direct_lineage"], true);

    let edge_to_child = edges.iter().find(|e| e["target"] == child_id).unwrap();
    assert_eq!(edge_to_child["is_direct_lineage"], true);

    let edge_to_grandchild = edges.iter().find(|e| e["target"] == grandchild_id).unwrap();
    assert_eq!(edge_to_grandchild["is_direct_lineage"], true);

    // Collateral parent-child edges should have is_direct_lineage == false
    let edge_to_sibling = edges.iter().find(|e| e["target"] == sibling_id).unwrap();
    assert_eq!(edge_to_sibling["is_direct_lineage"], false);

    let edge_to_uncle = edges.iter().find(|e| e["target"] == uncle_id).unwrap();
    assert_eq!(edge_to_uncle["is_direct_lineage"], false);

    let edge_to_cousin = edges.iter().find(|e| e["target"] == cousin_id).unwrap();
    assert_eq!(edge_to_cousin["is_direct_lineage"], false);

    let edge_to_niece = edges.iter().find(|e| e["target"] == niece_id).unwrap();
    assert_eq!(edge_to_niece["is_direct_lineage"], false);

    // --- Scenario B: Compact Hourglass Fallback (collateral=false) ---
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?mode=hourglass&collateral=false")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let compact_tree = read_json(resp).await;
    assert_eq!(compact_tree["collateral"], false);

    let compact_nodes = compact_tree["nodes"].as_array().unwrap();
    let compact_node_ids: Vec<String> = compact_nodes
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    // Must contain direct lineage ONLY:
    for direct_id in [
        &focal_id,
        &spouse_id,
        &father_id,
        &mother_id,
        &grandfather_id,
        &child_id,
        &grandchild_id,
    ] {
        assert!(
            compact_node_ids.contains(direct_id),
            "Compact hourglass missing direct relative {direct_id}"
        );
    }

    // Must NOT contain collateral relatives:
    for collateral_id in [
        &sibling_id,
        &sibling_spouse_id,
        &niece_id,
        &uncle_id,
        &uncle_spouse_id,
        &cousin_id,
    ] {
        assert!(
            !compact_node_ids.contains(collateral_id),
            "Compact hourglass unexpectedly included collateral relative {collateral_id}"
        );
    }

    // --- Scenario C: Default without collateral query parameter defaults to wide (collateral=true) ---
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let default_tree = read_json(resp).await;
    assert_eq!(default_tree["collateral"], true);
    let default_node_ids: Vec<String> = default_tree["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert!(default_node_ids.contains(&sibling_id));
    assert!(default_node_ids.contains(&uncle_id));

    // --- Scenario D: Unauthenticated Public Guest Privacy Redaction on Collateral Relatives ---
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/tree/view?focal_id={focal_id}&collateral=true"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let public_view = read_json(resp).await;
    assert_eq!(public_view["access_tier"], "public_anonymous");

    let pub_nodes = public_view["nodes"].as_array().unwrap();

    // Living collateral relative (Brother Mikhail) is masked to "M."
    let mikhail_masked = pub_nodes.iter().find(|n| n["id"] == sibling_id).unwrap();
    assert_eq!(mikhail_masked["is_living"], true);
    assert_eq!(mikhail_masked["primary_name"]["given_name"], "M.");
    assert_eq!(mikhail_masked["primary_name"]["surname"], "");
    assert!(mikhail_masked["birth"].is_null());
    assert_eq!(mikhail_masked["generation"], 0);
    assert_eq!(mikhail_masked["is_direct_lineage"], false);

    // Living collateral cousin (Alexey) is masked to "A."
    let alexey_masked = pub_nodes.iter().find(|n| n["id"] == cousin_id).unwrap();
    assert_eq!(alexey_masked["is_living"], true);
    assert_eq!(alexey_masked["primary_name"]["given_name"], "A.");
    assert_eq!(alexey_masked["primary_name"]["surname"], "");
    assert!(alexey_masked["birth"].is_null());
    assert_eq!(alexey_masked["generation"], 0);
    assert_eq!(alexey_masked["is_direct_lineage"], false);

    // Living niece (Maria) is masked to "M."
    let maria_masked = pub_nodes.iter().find(|n| n["id"] == niece_id).unwrap();
    assert_eq!(maria_masked["is_living"], true);
    assert_eq!(maria_masked["primary_name"]["given_name"], "M.");
    assert_eq!(maria_masked["primary_name"]["surname"], "");
    assert_eq!(maria_masked["generation"], 1);
    assert_eq!(maria_masked["is_direct_lineage"], false);

    // Deceased collateral uncle (Boris Romanov) remains unredacted
    let boris_unmasked = pub_nodes.iter().find(|n| n["id"] == uncle_id).unwrap();
    assert_eq!(boris_unmasked["is_living"], false);
    assert_eq!(boris_unmasked["primary_name"]["given_name"], "Boris");
    assert_eq!(boris_unmasked["primary_name"]["surname"], "Romanov");
    assert_eq!(boris_unmasked["birth"]["date"], "1952");
    assert_eq!(boris_unmasked["death"]["date"], "2020");
    assert_eq!(boris_unmasked["generation"], -1);
    assert_eq!(boris_unmasked["is_direct_lineage"], false);
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_all_nodes_mode_unpruned_forest_and_living_masking() {
    let mut config = test_config();
    config.enable_public_access = true;

    let state = AppState::new_empty(None, config);
    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("All Nodes Forest Test Tree".to_string()),
            first_name: "Anna".to_string(),
            last_name: "Romanova".to_string(),
            email: "admin@example.com".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        Some("127.0.0.1".to_string()),
        false,
    )
    .await
    .unwrap();

    let app = create_router(state);

    let quick_add = |app: axum::Router,
                     cookie: String,
                     anchor_id: String,
                     payload: serde_json::Value| async move {
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/v1/persons/{anchor_id}/quick-add"))
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        read_json(resp).await["person"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    };

    let create_person = |app: axum::Router, cookie: String, payload: serde_json::Value| async move {
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/persons")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        read_json(resp).await["id"].as_str().unwrap().to_string()
    };

    // --- Component 1: Primary Component (Romanov / Sokolov family around Anna) ---
    // 1. Focal person: Anna Romanova (living, b. 1985)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Anna",
                        "patronymic": "Nikolaevna",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "15 MAY 1985",
                        "birth_place": "Saint Petersburg, Russia"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let focal_id = read_json(resp).await["individual"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Father: Nikolai Romanov (deceased, 1955-2010)
    let father_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "father",
            "given_name": "Nikolai",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1955",
            "death_date": "2010"
        }),
    )
    .await;

    // 3. Mother: Olga Romanova (living, b. 1958)
    let mother_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "mother",
            "given_name": "Olga",
            "surname": "Romanova",
            "sex": "F",
            "birth_date": "1958"
        }),
    )
    .await;

    // 4. Paternal Grandfather: Pyotr Romanov (deceased, 1925-1980)
    let grandfather_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        father_id.clone(),
        json!({
            "relation": "father",
            "given_name": "Pyotr",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1925",
            "death_date": "1980"
        }),
    )
    .await;

    // 5. Spouse: Иван Sokolov (living, b. 1984)
    let spouse_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "spouse",
            "given_name": "Иван",
            "surname": "Sokolov",
            "sex": "M",
            "birth_date": "1984"
        }),
    )
    .await;

    // 6. Child: Dmitry Sokolov (living, b. 2010)
    let child_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Dmitry",
            "surname": "Sokolov",
            "sex": "M",
            "birth_date": "2010"
        }),
    )
    .await;

    // 7. Grandchild: Sofia Sokolova (living, b. 2030)
    let grandchild_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        child_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Sofia",
            "surname": "Sokolova",
            "sex": "F",
            "birth_date": "2030"
        }),
    )
    .await;

    // 8. Sibling: Mikhail Romanov (living, b. 1988)
    let sibling_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        focal_id.clone(),
        json!({
            "relation": "sibling",
            "given_name": "Mikhail",
            "surname": "Romanov",
            "sex": "M",
            "birth_date": "1988"
        }),
    )
    .await;

    // --- Component 2: Secondary Independent Branch (Tolstoy family line) ---
    // 9. Lev Tolstoy (deceased, 1828-1910)
    let lev_id = create_person(
        app.clone(),
        admin_cookie.clone(),
        json!({
            "given_name": "Lev",
            "surname": "Tolstoy",
            "sex": "M",
            "birth_date": "1828",
            "death_date": "1910"
        }),
    )
    .await;

    // 10. Sophia Tolstaya (deceased, 1844-1919) - Spouse of Lev
    let sophia_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        lev_id.clone(),
        json!({
            "relation": "spouse",
            "given_name": "Sophia",
            "surname": "Tolstaya",
            "sex": "F",
            "birth_date": "1844",
            "death_date": "1919"
        }),
    )
    .await;

    // 11. Ilya Tolstoy (deceased, 1866-1933) - Child of Lev & Sophia
    let ilya_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        lev_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Ilya",
            "surname": "Tolstoy",
            "sex": "M",
            "birth_date": "1866",
            "death_date": "1933"
        }),
    )
    .await;

    // 12. Tatiana Tolstaya (living, b. 1990) - Child of Ilya
    let tatiana_id = quick_add(
        app.clone(),
        admin_cookie.clone(),
        ilya_id.clone(),
        json!({
            "relation": "child",
            "given_name": "Tatiana",
            "surname": "Tolstaya",
            "sex": "F",
            "birth_date": "1990"
        }),
    )
    .await;

    // --- Component 3: Singleton Isolated Individual ---
    // 13. Hermit Petrov (living, b. 1995)
    let hermit_id = create_person(
        app.clone(),
        admin_cookie.clone(),
        json!({
            "given_name": "Hermit",
            "surname": "Petrov",
            "sex": "M",
            "birth_date": "1995"
        }),
    )
    .await;

    // --- Component 4: Confidential Private Person ---
    // 14. Confidential Relative (is_private: true)
    let private_id = create_person(
        app.clone(),
        admin_cookie.clone(),
        json!({
            "given_name": "Confidential",
            "surname": "Secret",
            "sex": "U",
            "birth_date": "1975",
            "is_private": true
        }),
    )
    .await;

    // =========================================================================
    // Scenario 1: Authenticated Admin Mode="all" - 100% of visible nodes returned
    // =========================================================================
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/tree/view?mode=all&focal_id={focal_id}"))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let all_view = read_json(resp).await;

    assert_eq!(all_view["mode"], "all");
    assert_eq!(all_view["access_tier"], "admin");
    let nodes = all_view["nodes"].as_array().unwrap();
    // Admin sees all 14 nodes across all components (8 primary + 4 secondary + 1 singleton + 1 private)
    assert_eq!(nodes.len(), 14);
    assert_eq!(all_view["total_visible"], 14);

    let get_node = |id: &str| -> &Value {
        nodes
            .iter()
            .find(|n| n["id"] == id)
            .unwrap_or_else(|| panic!("Node {id} missing in all nodes view"))
    };

    // --- Generational Leveling & Direct Lineage Verification ---
    // Primary Component C_primary (anchored at Anna = 0)
    assert_eq!(get_node(&focal_id)["generation"], 0);
    assert_eq!(get_node(&focal_id)["is_direct_lineage"], true);
    assert_eq!(get_node(&focal_id)["is_living"], true);
    assert_eq!(get_node(&focal_id)["primary_name"]["given_name"], "Anna");

    assert_eq!(get_node(&father_id)["generation"], -1);
    assert_eq!(get_node(&father_id)["is_direct_lineage"], true);
    assert_eq!(get_node(&father_id)["is_living"], false);

    assert_eq!(get_node(&mother_id)["generation"], -1);
    assert_eq!(get_node(&mother_id)["is_direct_lineage"], true);
    assert_eq!(get_node(&mother_id)["is_living"], true);

    assert_eq!(get_node(&grandfather_id)["generation"], -2);
    assert_eq!(get_node(&grandfather_id)["is_direct_lineage"], true);
    assert_eq!(get_node(&grandfather_id)["is_living"], false);

    assert_eq!(get_node(&spouse_id)["generation"], 0);
    assert_eq!(get_node(&spouse_id)["is_direct_lineage"], false);

    assert_eq!(get_node(&sibling_id)["generation"], 0);
    assert_eq!(get_node(&sibling_id)["is_direct_lineage"], false);

    assert_eq!(get_node(&child_id)["generation"], 1);
    assert_eq!(get_node(&child_id)["is_direct_lineage"], true);

    assert_eq!(get_node(&grandchild_id)["generation"], 2);
    assert_eq!(get_node(&grandchild_id)["is_direct_lineage"], true);

    // Secondary Component C_2 (Tolstoy line):
    // Median birth year: 1855. Focal Anna: 1985. Shift: round((1855 - 1985) / 30) = -4.
    assert_eq!(get_node(&lev_id)["generation"], -4);
    assert_eq!(get_node(&lev_id)["is_direct_lineage"], false);

    assert_eq!(get_node(&sophia_id)["generation"], -4);
    assert_eq!(get_node(&sophia_id)["is_direct_lineage"], false);

    assert_eq!(get_node(&ilya_id)["generation"], -3);
    assert_eq!(get_node(&ilya_id)["is_direct_lineage"], false);

    assert_eq!(get_node(&tatiana_id)["generation"], -2);
    assert_eq!(get_node(&tatiana_id)["is_direct_lineage"], false);

    // Singleton Component C_3: Hermit Petrov (generation = 0, is_direct_lineage = false)
    assert_eq!(get_node(&hermit_id)["generation"], 0);
    assert_eq!(get_node(&hermit_id)["is_direct_lineage"], false);

    // Private person is visible to admin
    assert_eq!(get_node(&private_id)["is_private"], true);

    // Compare with mode="hourglass": Hourglass prunes everything outside Anna's focal tree
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/tree/view?mode=hourglass&focal_id={focal_id}&generations=4"
                ))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let hg_view = read_json(resp).await;
    let hg_nodes = hg_view["nodes"].as_array().unwrap();
    // Only Romanov/Sokolov nodes included (8 nodes); Tolstoy and Hermit are pruned
    assert_eq!(hg_nodes.len(), 8);
    let hg_ids: Vec<String> = hg_nodes
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert!(!hg_ids.contains(&lev_id));
    assert!(!hg_ids.contains(&hermit_id));

    // =========================================================================
    // Scenario 2: Public Anonymous Mode="all" - Living Masking Across All Components
    // =========================================================================
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/tree/view?mode=all&focal_id={focal_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let pub_view = read_json(resp).await;

    assert_eq!(pub_view["access_tier"], "public_anonymous");
    let pub_nodes = pub_view["nodes"].as_array().unwrap();

    // Private person (Confidential Secret) is excluded from public view -> 13 nodes remain
    assert_eq!(pub_nodes.len(), 13);
    assert!(pub_nodes.iter().all(|n| n["id"] != private_id));

    let get_pub_node = |id: &str| -> &Value {
        pub_nodes
            .iter()
            .find(|n| n["id"] == id)
            .unwrap_or_else(|| panic!("Node {id} missing in public view"))
    };

    // 1. Primary Component Living Masking:
    // Focal Anna (living) masked to "A."
    let pub_anna = get_pub_node(&focal_id);
    assert_eq!(pub_anna["is_living"], true);
    assert_eq!(pub_anna["primary_name"]["given_name"], "A.");
    assert_eq!(pub_anna["primary_name"]["surname"], "");
    assert!(pub_anna["birth"].is_null());
    assert_eq!(pub_anna["generation"], 0);
    assert_eq!(pub_anna["is_direct_lineage"], true);

    // Mother Olga (living) masked to "O."
    let pub_olga = get_pub_node(&mother_id);
    assert_eq!(pub_olga["is_living"], true);
    assert_eq!(pub_olga["primary_name"]["given_name"], "O.");
    assert_eq!(pub_olga["primary_name"]["surname"], "");
    assert!(pub_olga["birth"].is_null());

    // Spouse Иван (living, Cyrillic) masked to "И."
    let pub_ivan = get_pub_node(&spouse_id);
    assert_eq!(pub_ivan["is_living"], true);
    assert_eq!(pub_ivan["primary_name"]["given_name"], "И.");
    assert_eq!(pub_ivan["primary_name"]["surname"], "");
    assert!(pub_ivan["birth"].is_null());

    // Child Dmitry (living) masked to "D."
    let pub_dmitry = get_pub_node(&child_id);
    assert_eq!(pub_dmitry["is_living"], true);
    assert_eq!(pub_dmitry["primary_name"]["given_name"], "D.");

    // Sibling Mikhail (living) masked to "M."
    let pub_mikhail = get_pub_node(&sibling_id);
    assert_eq!(pub_mikhail["is_living"], true);
    assert_eq!(pub_mikhail["primary_name"]["given_name"], "M.");

    // Grandchild Sofia (living) masked to "S."
    let pub_sofia = get_pub_node(&grandchild_id);
    assert_eq!(pub_sofia["is_living"], true);
    assert_eq!(pub_sofia["primary_name"]["given_name"], "S.");

    // 2. Primary Component Deceased Unmasked:
    let pub_nikolai = get_pub_node(&father_id);
    assert_eq!(pub_nikolai["is_living"], false);
    assert_eq!(pub_nikolai["primary_name"]["given_name"], "Nikolai");
    assert_eq!(pub_nikolai["primary_name"]["surname"], "Romanov");
    assert_eq!(pub_nikolai["birth"]["date"], "1955");
    assert_eq!(pub_nikolai["death"]["date"], "2010");

    let pub_pyotr = get_pub_node(&grandfather_id);
    assert_eq!(pub_pyotr["is_living"], false);
    assert_eq!(pub_pyotr["primary_name"]["given_name"], "Pyotr");
    assert_eq!(pub_pyotr["primary_name"]["surname"], "Romanov");

    // 3. Secondary Component (Tolstoy) Living & Deceased:
    // Deceased Lev Tolstoy remains unmasked
    let pub_lev = get_pub_node(&lev_id);
    assert_eq!(pub_lev["is_living"], false);
    assert_eq!(pub_lev["primary_name"]["given_name"], "Lev");
    assert_eq!(pub_lev["primary_name"]["surname"], "Tolstoy");
    assert_eq!(pub_lev["generation"], -4);
    assert_eq!(pub_lev["is_direct_lineage"], false);

    // Deceased Sophia Tolstaya remains unmasked
    let pub_sophia = get_pub_node(&sophia_id);
    assert_eq!(pub_sophia["is_living"], false);
    assert_eq!(pub_sophia["primary_name"]["given_name"], "Sophia");
    assert_eq!(pub_sophia["primary_name"]["surname"], "Tolstaya");

    // Living Tatiana Tolstaya is masked to "T."
    let pub_tatiana = get_pub_node(&tatiana_id);
    assert_eq!(pub_tatiana["is_living"], true);
    assert_eq!(pub_tatiana["primary_name"]["given_name"], "T.");
    assert_eq!(pub_tatiana["primary_name"]["surname"], "");
    assert!(pub_tatiana["birth"].is_null());
    assert_eq!(pub_tatiana["generation"], -2);
    assert_eq!(pub_tatiana["is_direct_lineage"], false);

    // 4. Singleton Component Living Masking:
    // Living Hermit Petrov is masked to "H."
    let pub_hermit = get_pub_node(&hermit_id);
    assert_eq!(pub_hermit["is_living"], true);
    assert_eq!(pub_hermit["primary_name"]["given_name"], "H.");
    assert_eq!(pub_hermit["primary_name"]["surname"], "");
    assert!(pub_hermit["birth"].is_null());
    assert_eq!(pub_hermit["generation"], 0);
    assert_eq!(pub_hermit["is_direct_lineage"], false);

    // 5. Default focal_id resolution in mode="all":
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree/view?mode=all")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let default_focal_view = read_json(resp).await;
    assert_eq!(default_focal_view["nodes"].as_array().unwrap().len(), 13);
}

#[tokio::test]
async fn test_multi_spouse_hourglass_ancestor_remarriages_and_marriage_facts() {
    let (
        app,
        admin_cookie,
        focal_id,
        father_id,
        _mother_id,
        grandfather_id,
        _spouse_id,
        _child_id,
        _grandchild_id,
    ) = setup_populated_tree().await;

    // 0. Quick-add Paternal Grandmother (Maria) to Father Nikolai
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{father_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "mother",
                        "given_name": "Maria",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "1928",
                        "death_date": "1985"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let grandmother_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 1. Quick-add second spouse (Anna-Maria) to Grandfather Pyotr
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{grandfather_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "spouse",
                        "given_name": "Anna-Maria",
                        "surname": "Petrova",
                        "sex": "F",
                        "birth_date": "1935",
                        "death_date": "2015"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let add_res = read_json(resp).await;
    let second_wife_id = add_res["person"]["id"].as_str().unwrap().to_string();
    let remarriage_fam_id = add_res["family"]["id"].as_str().unwrap().to_string();

    // 2. Quick-add half-uncle Mikhail (child of Pyotr and Anna-Maria)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{grandfather_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "target_family_id": remarriage_fam_id,
                        "given_name": "Mikhail",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1960",
                        "death_date": "2020"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let half_uncle_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 3. Find primary grandparent family (Pyotr + Maria)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/families")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let fam_list = read_json(resp).await;
    let items = fam_list["items"].as_array().unwrap();
    let gp_fam = items
        .iter()
        .find(|f| {
            let p1 = f["partner1_id"].as_str();
            let p2 = f["partner2_id"].as_str();
            (p1 == Some(&grandfather_id) && p2 == Some(&grandmother_id))
                || (p1 == Some(&grandmother_id) && p2 == Some(&grandfather_id))
        })
        .expect("Primary grandparent family should exist");
    let gp_fam_id = gp_fam["id"].as_str().unwrap().to_string();

    // 4. Create marriage event for gp_fam_id
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/events")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "family_id": gp_fam_id,
                        "event_type": "marriage",
                        "date_text": "12 JUL 1950",
                        "place_path": "Moscow, Russia"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 5. Verify GET /api/v1/families/{gp_fam_id} returns enriched marriage date and place
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/families/{gp_fam_id}"))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let fam_detail = read_json(resp).await;
    assert_eq!(fam_detail["marriage_date"], "12 JUL 1950");
    assert_eq!(fam_detail["marriage_place"], "Moscow, Russia");

    // 6. Verify Wide Hourglass (mode=hourglass, collateral=true)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/tree/view?mode=hourglass&focal_id={focal_id}&collateral=true"
                ))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let wide_tree = read_json(resp).await;
    let wide_nodes = wide_tree["nodes"].as_array().unwrap();

    let node_pyotr = wide_nodes
        .iter()
        .find(|n| n["id"] == grandfather_id)
        .unwrap();
    assert_eq!(node_pyotr["generation"], -2);
    assert_eq!(node_pyotr["is_direct_lineage"], true);

    let node_father = wide_nodes.iter().find(|n| n["id"] == father_id).unwrap();
    assert_eq!(node_father["generation"], -1);
    assert_eq!(node_father["is_direct_lineage"], true);

    let node_wife2 = wide_nodes
        .iter()
        .find(|n| n["id"] == second_wife_id)
        .expect("Second wife must be included in wide hourglass");
    assert_eq!(node_wife2["generation"], -2);
    assert_eq!(node_wife2["is_direct_lineage"], false);

    let node_half_uncle = wide_nodes
        .iter()
        .find(|n| n["id"] == half_uncle_id)
        .expect("Half-uncle must be included in wide hourglass");
    assert_eq!(node_half_uncle["generation"], -1);
    assert_eq!(node_half_uncle["is_direct_lineage"], false);

    let wide_fams = wide_tree["families"].as_array().unwrap();
    let tree_gp_fam = wide_fams
        .iter()
        .find(|f| f["id"] == gp_fam_id)
        .expect("Grandparent family in tree");
    assert_eq!(tree_gp_fam["marriage_date"], "12 JUL 1950");
    assert_eq!(tree_gp_fam["marriage_place"], "Moscow, Russia");

    assert!(
        wide_fams.iter().any(|f| f["id"] == remarriage_fam_id),
        "Remarriage family must be in wide hourglass"
    );

    // 7. Verify Compact Hourglass (mode=hourglass, collateral=false) prunes ancestor remarriages
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/tree/view?mode=hourglass&focal_id={focal_id}&collateral=false"
                ))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let compact_tree = read_json(resp).await;
    let compact_nodes = compact_tree["nodes"].as_array().unwrap();
    assert!(
        !compact_nodes.iter().any(|n| n["id"] == second_wife_id),
        "Second wife must be pruned in compact hourglass"
    );
    assert!(
        !compact_nodes.iter().any(|n| n["id"] == half_uncle_id),
        "Half-uncle must be pruned in compact hourglass"
    );

    let compact_fams = compact_tree["families"].as_array().unwrap();
    assert!(
        !compact_fams.iter().any(|f| f["id"] == remarriage_fam_id),
        "Remarriage family must be pruned in compact hourglass"
    );
}
