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
