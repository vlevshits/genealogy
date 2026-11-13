use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
    state::AppState,
};

async fn call(
    app: &Router,
    cookie: &str,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, cookie)
        .header("x-requested-with", "genealogy");
    let body = match body {
        Some(v) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

struct Ctx {
    app: Router,
    cookie: String,
    anna: String,
    ivan: String,
    oleg: String,
    f1: String,
}

/// Anna + Ivan (family F1) with child Oleg.
async fn setup() -> Ctx {
    let state = AppState::new_empty(
        None,
        AppConfig {
            app_env: "test".to_string(),
            ..AppConfig::default()
        },
    );
    let (_, cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Move Test".to_string()),
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
    let (st, body) = call(
        &app,
        &cookie,
        Method::POST,
        "/api/v1/onboarding/start-with-self",
        Some(
            json!({"given_name": "Anna", "surname": "Romanova", "sex": "F", "birth_date": "1985"}),
        ),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let anna = body["individual"]["id"].as_str().unwrap().to_string();
    let (_, body) = call(
        &app,
        &cookie,
        Method::POST,
        &format!("/api/v1/persons/{anna}/quick-add"),
        Some(json!({"relation": "spouse", "given_name": "Ivan", "surname": "Sokolov", "sex": "M", "birth_date": "1984"})),
    )
    .await;
    let ivan = body["person"]["id"].as_str().unwrap().to_string();
    let f1 = body["family"]["id"].as_str().unwrap().to_string();
    let (_, body) = call(
        &app,
        &cookie,
        Method::POST,
        &format!("/api/v1/persons/{anna}/quick-add"),
        Some(json!({"relation": "child", "given_name": "Oleg", "surname": "Sokolov", "sex": "M", "birth_date": "2010"})),
    )
    .await;
    let oleg = body["person"]["id"].as_str().unwrap().to_string();
    Ctx {
        app,
        cookie,
        anna,
        ivan,
        oleg,
        f1,
    }
}

async fn new_person(c: &Ctx, name: &str) -> String {
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/persons",
        Some(json!({"given_name": name, "surname": "Test", "sex": "F", "birth_date": "1980"})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_string()
}

async fn get_family(c: &Ctx, id: &str) -> (StatusCode, Value) {
    call(
        &c.app,
        &c.cookie,
        Method::GET,
        &format!("/api/v1/families/{id}"),
        None,
    )
    .await
}

fn child_ids(fam: &Value) -> Vec<String> {
    fam["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["child_id"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn move_to_new_single_parent_family_keeps_two_partner_source_and_undoes() {
    let c = setup().await;
    let maria = new_person(&c, "Maria").await;
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/persons/{}/move-to-family", c.oleg),
        Some(json!({"partner1_id": maria, "rel_to_partner1": "adopted"})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["target_family_created"], true);
    assert_eq!(body["source_family_dissolved"], false);
    assert_eq!(
        body["target_family"]["children"][0]["rel_to_partner1"],
        "adopted"
    );
    assert_eq!(
        body["target_family"]["children"][0]["rel_to_partner2"],
        "birth"
    );
    let tx_id = body["tx_id"].as_str().unwrap().to_string();
    let new_fam = body["target_family"]["id"].as_str().unwrap().to_string();

    let (_, f1) = get_family(&c, &c.f1).await;
    assert!(child_ids(&f1).is_empty());

    // Second move to the same couple reuses the family instead of creating one.
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/persons/{}/move-to-family", c.oleg),
        Some(json!({"partner1_id": c.anna, "partner2_id": c.ivan})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["target_family_created"], false);
    assert_eq!(body["target_family"]["id"], c.f1);
    // Single-parent source (no children, 1 partner) dissolved.
    assert_eq!(body["source_family_dissolved"], true);
    assert!(body["source_family"].is_null());
    assert_eq!(get_family(&c, &new_fam).await.0, StatusCode::NOT_FOUND);

    // Undo the second move: Oleg returns to the single-parent family.
    let tx2 = body["tx_id"].as_str().unwrap().to_string();
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/history/tx/{tx2}/undo"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let (st, nf) = get_family(&c, &new_fam).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(child_ids(&nf), vec![c.oleg.clone()]);
    let (_, f1) = get_family(&c, &c.f1).await;
    assert!(child_ids(&f1).is_empty());

    // Undo the first move: back to F1, new family gone.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/history/tx/{tx_id}/undo"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let (_, f1) = get_family(&c, &c.f1).await;
    assert_eq!(child_ids(&f1), vec![c.oleg.clone()]);
    assert_eq!(get_family(&c, &new_fam).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn move_rejects_cycles_partner_equal_and_bad_input() {
    let c = setup().await;
    let uri = |id: &str| format!("/api/v1/persons/{id}/move-to-family");
    // Child equals partner.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri(&c.oleg),
        Some(json!({"partner1_id": c.oleg})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Anna is an ancestor of Oleg: Anna cannot become a child of a union with Oleg.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri(&c.anna),
        Some(json!({"partner1_id": c.oleg})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Missing target.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri(&c.oleg),
        Some(json!({})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Already a child of target family.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri(&c.oleg),
        Some(json!({"family_id": c.f1})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Unknown person / family -> 404.
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri("nope"),
        Some(json!({"family_id": c.f1})),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &uri(&c.oleg),
        Some(json!({"family_id": "nope"})),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    // Nothing changed.
    let (_, f1) = get_family(&c, &c.f1).await;
    assert_eq!(child_ids(&f1), vec![c.oleg.clone()]);
}

#[tokio::test]
async fn move_requires_authentication() {
    let c = setup().await;
    let (st, _) = call(
        &c.app,
        "",
        Method::POST,
        &format!("/api/v1/persons/{}/move-to-family", c.oleg),
        Some(json!({"family_id": c.f1})),
    )
    .await;
    assert!(
        st == StatusCode::UNAUTHORIZED || st == StatusCode::FORBIDDEN,
        "{st}"
    );
}

#[tokio::test]
async fn update_family_rejects_partner_or_ancestor_as_child() {
    let c = setup().await;
    let (_, f1) = get_family(&c, &c.f1).await;
    let version = f1["version"].as_i64().unwrap();
    let patch = |children: Value| {
        let app = c.app.clone();
        let cookie = c.cookie.clone();
        let uri = format!("/api/v1/families/{}", c.f1);
        async move {
            call(
                &app,
                &cookie,
                Method::PATCH,
                &uri,
                Some(json!({"version": version, "children": children})),
            )
            .await
        }
    };
    // Child equal to a partner.
    let (st, _) = patch(json!([{"child_id": c.anna}])).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Child that is an ancestor of the partners: make Anna a child of Oleg's family first.
    let maria = new_person(&c, "Maria").await;
    let (st, fam) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/families",
        Some(json!({"partner1_id": c.oleg, "partner2_id": maria})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{fam}");
    let fid = fam["id"].as_str().unwrap();
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{fid}"),
        Some(json!({"version": 1, "children": [{"child_id": c.anna}]})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_family_unlinking_last_child_dissolves_single_parent_union_with_undo() {
    let c = setup().await;
    let pyotr = new_person(&c, "Pyotr").await;
    let kid = new_person(&c, "Kid").await;
    let (st, fam) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/families",
        Some(json!({"partner1_id": pyotr, "children_ids": [kid]})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{fam}");
    let fid = fam["id"].as_str().unwrap().to_string();

    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{fid}"),
        Some(json!({"version": 1, "children": []})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], true);
    let tx = body["tx_id"].as_str().unwrap().to_string();
    assert_eq!(get_family(&c, &fid).await.0, StatusCode::NOT_FOUND);

    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/history/tx/{tx}/undo"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let (st, restored) = get_family(&c, &fid).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(child_ids(&restored), vec![kid.clone()]);

    // Two-partner union survives losing its last child and returns a tx id.
    let (_, f1) = get_family(&c, &c.f1).await;
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{}", c.f1),
        Some(json!({"version": f1["version"], "children": []})),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["dissolved"], false);
    assert!(body["tx_id"].as_str().is_some());
    let _ = &c.ivan;
}

#[tokio::test]
async fn delete_family_returns_undoable_tx_header() {
    let c = setup().await;
    let resp = c
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/families/{}", c.f1))
                .header(header::COOKIE, &c.cookie)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let tx = resp.headers()["x-transaction-id"]
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::NOT_FOUND);
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/history/tx/{tx}/undo"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::OK);
}

#[tokio::test]
async fn move_to_family_forbidden_for_viewer() {
    let state = AppState::new_empty(
        None,
        AppConfig {
            app_env: "test".to_string(),
            ..AppConfig::default()
        },
    );
    let (admin, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Move Viewer".to_string()),
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
    let admin_id = uuid::Uuid::parse_str(&admin.user.id).unwrap();
    let inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            role: "viewer".to_string(),
            email: Some("viewer@example.com".to_string()),
            invitee_name: None,
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (_, viewer_cookie) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: inv.token,
            first_name: "Viktor".to_string(),
            last_name: "Viewer".to_string(),
            email: Some("viewer@example.com".to_string()),
            passkey: None,
            password: Some("ViewerPass123!".to_string()),
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let app = create_router(state);
    // RBAC must reject before any record lookup.
    let (st, body) = call(
        &app,
        &viewer_cookie,
        Method::POST,
        "/api/v1/persons/any-id/move-to-family",
        Some(json!({"family_id": "any-family"})),
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "forbidden", "{body}");
}
