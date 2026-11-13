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
    services::auth::{AuthService, SetupRequest},
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

/// Creates self + spouse (+ optional child) and a marriage event on the union.
/// Returns (app, cookie, self_id, spouse_id, family_id, marriage_event_id).
async fn setup(with_child: bool) -> (Router, String, String, String, String, String) {
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
            tree_title: Some("Family Link Test".to_string()),
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
    let self_id = body["individual"]["id"].as_str().unwrap().to_string();

    let (st, body) = call(
        &app,
        &cookie,
        Method::POST,
        &format!("/api/v1/persons/{self_id}/quick-add"),
        Some(json!({"relation": "spouse", "given_name": "Ivan", "surname": "Sokolov", "sex": "M", "birth_date": "1984"})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let spouse_id = body["person"]["id"].as_str().unwrap().to_string();

    if with_child {
        let (st, _) = call(
            &app,
            &cookie,
            Method::POST,
            &format!("/api/v1/persons/{self_id}/quick-add"),
            Some(json!({"relation": "child", "given_name": "Oleg", "surname": "Sokolov", "sex": "M", "birth_date": "2010"})),
        )
        .await;
        assert_eq!(st, StatusCode::CREATED);
    }

    let (_, profile) = call(
        &app,
        &cookie,
        Method::GET,
        &format!("/api/v1/persons/{self_id}/profile"),
        None,
    )
    .await;
    let family_id = profile["families"]["spouse_families"][0]["family_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (st, ev) = call(
        &app,
        &cookie,
        Method::POST,
        "/api/v1/events",
        Some(json!({"family_id": family_id, "event_type": "marriage", "date_text": "2005"})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let event_id = ev["id"].as_str().unwrap().to_string();

    (app, cookie, self_id, spouse_id, family_id, event_id)
}

async fn spouse_families(app: &Router, cookie: &str, id: &str) -> Vec<Value> {
    let (_, profile) = call(
        app,
        cookie,
        Method::GET,
        &format!("/api/v1/persons/{id}/profile"),
        None,
    )
    .await;
    profile["families"]["spouse_families"]
        .as_array()
        .unwrap()
        .clone()
}

#[tokio::test]
async fn deleting_marriage_event_removes_childless_spouse_union() {
    let (app, cookie, self_id, spouse_id, _fam, event_id) = setup(false).await;
    assert_eq!(spouse_families(&app, &cookie, &self_id).await.len(), 1);

    let (st, _) = call(
        &app,
        &cookie,
        Method::DELETE,
        &format!("/api/v1/events/{event_id}"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::NO_CONTENT);

    assert!(spouse_families(&app, &cookie, &self_id).await.is_empty());
    assert!(spouse_families(&app, &cookie, &spouse_id).await.is_empty());
    // Profiles themselves survive.
    let (st, _) = call(
        &app,
        &cookie,
        Method::GET,
        &format!("/api/v1/persons/{spouse_id}/profile"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
}

#[tokio::test]
async fn deleting_marriage_event_keeps_union_with_children_and_spouse_can_be_detached() {
    let (app, cookie, self_id, spouse_id, family_id, event_id) = setup(true).await;

    let (st, _) = call(
        &app,
        &cookie,
        Method::DELETE,
        &format!("/api/v1/events/{event_id}"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    let fams = spouse_families(&app, &cookie, &self_id).await;
    assert_eq!(fams.len(), 1, "union with children is retained");
    let version = fams[0]["version"].as_i64().unwrap();
    assert_eq!(fams[0]["children"].as_array().unwrap().len(), 1);

    // Manually detach the spouse; children stay.
    let (st, _) = call(
        &app,
        &cookie,
        Method::PATCH,
        &format!("/api/v1/families/{family_id}"),
        Some(json!({"version": version, "partner1_id": self_id, "partner2_id": ""})),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let fams = spouse_families(&app, &cookie, &self_id).await;
    assert!(fams[0]["spouse"].is_null());
    assert_eq!(fams[0]["children"].as_array().unwrap().len(), 1);
    assert!(spouse_families(&app, &cookie, &spouse_id).await.is_empty());

    // Detach the child; profile survives.
    let version = fams[0]["version"].as_i64().unwrap();
    let (st, _) = call(
        &app,
        &cookie,
        Method::PATCH,
        &format!("/api/v1/families/{family_id}"),
        Some(json!({"version": version, "children": []})),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    // The emptied single-parent union is dissolved (same rule as marriage removal).
    assert!(spouse_families(&app, &cookie, &self_id).await.is_empty());
    let (st, _) = call(
        &app,
        &cookie,
        Method::GET,
        &format!("/api/v1/families/{family_id}"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}
