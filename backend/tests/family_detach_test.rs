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

async fn del(c: &Ctx, uri: &str) -> (StatusCode, Value) {
    call(&c.app, &c.cookie, Method::DELETE, uri, None).await
}

async fn undo(c: &Ctx, tx: &str) {
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/history/tx/{tx}/undo"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
}

async fn add_kira(c: &Ctx) -> String {
    let (_, body) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        &format!("/api/v1/persons/{}/quick-add", c.anna),
        Some(json!({"relation": "child", "given_name": "Kira", "surname": "Sokolov", "sex": "F", "birth_date": "2012"})),
    )
    .await;
    body["person"]["id"].as_str().unwrap().to_string()
}

fn rel(fam: &Value, child: &str) -> (String, String) {
    let ch = fam["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["child_id"] == child)
        .unwrap();
    (
        ch["rel_to_partner1"].as_str().unwrap().to_string(),
        ch["rel_to_partner2"].as_str().unwrap().to_string(),
    )
}

/// Sets distinct pedigree relations so slot remapping is observable. Returns kira id.
async fn seed_relations(c: &Ctx) -> String {
    let kira = add_kira(c).await;
    let (_, f1) = get_family(c, &c.f1).await;
    let (st, _) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{}", c.f1),
        Some(json!({"version": f1["version"], "children": [
            {"child_id": c.oleg, "rel_to_partner1": "adopted", "rel_to_partner2": "foster"},
            {"child_id": kira, "rel_to_partner1": "step", "rel_to_partner2": "birth"}
        ]})),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    kira
}

#[tokio::test]
async fn detach_partner_promotes_remaining_and_remaps_child_relations_with_undo() {
    for detach_first in [true, false] {
        let c = setup().await;
        let kira = seed_relations(&c).await;
        let (_, f1) = get_family(&c, &c.f1).await;
        let p1 = f1["partner1_id"].as_str().unwrap().to_string();
        let p2 = f1["partner2_id"].as_str().unwrap().to_string();
        let (gone, kept) = if detach_first {
            (p1.clone(), p2.clone())
        } else {
            (p2.clone(), p1.clone())
        };
        let (st, body) = del(&c, &format!("/api/v1/families/{}/partners/{gone}", c.f1)).await;
        assert_eq!(st, StatusCode::OK, "{body}");
        assert_eq!(body["dissolved"], false);
        let tx = body["tx_id"].as_str().unwrap().to_string();
        assert_eq!(body["partner1_id"], kept);
        assert!(body["partner2_id"].is_null());
        let (o, k) = if detach_first {
            // old partner2 relations become partner1 relations
            (("foster", "birth"), ("birth", "birth"))
        } else {
            (("adopted", "birth"), ("step", "birth"))
        };
        assert_eq!(rel(&body, &c.oleg), (o.0.to_string(), o.1.to_string()));
        assert_eq!(rel(&body, &kira), (k.0.to_string(), k.1.to_string()));

        undo(&c, &tx).await;
        let (_, restored) = get_family(&c, &c.f1).await;
        assert_eq!(restored["partner1_id"], p1.as_str());
        assert_eq!(restored["partner2_id"], p2.as_str());
        assert_eq!(
            rel(&restored, &c.oleg),
            ("adopted".to_string(), "foster".to_string())
        );
        assert_eq!(
            rel(&restored, &kira),
            ("step".to_string(), "birth".to_string())
        );
    }
}

#[tokio::test]
async fn patch_clearing_partner1_normalizes_slots() {
    let c = setup().await;
    let _kira = seed_relations(&c).await;
    let (_, f1) = get_family(&c, &c.f1).await;
    let p2 = f1["partner2_id"].as_str().unwrap().to_string();
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{}", c.f1),
        Some(json!({"version": f1["version"], "partner1_id": ""})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], false);
    assert_eq!(body["partner1_id"], p2.as_str());
    assert!(body["partner2_id"].is_null());
    assert_eq!(
        rel(&body, &c.oleg),
        ("foster".to_string(), "birth".to_string())
    );
}

#[tokio::test]
async fn patch_clearing_partner_of_childless_union_dissolves() {
    let c = setup().await;
    // Remove Oleg first (dissolve must not trigger: two partners remain).
    let (st, body) = del(
        &c,
        &format!("/api/v1/families/{}/children/{}", c.f1, c.oleg),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], false);
    let (_, f1) = get_family(&c, &c.f1).await;
    let (st, body) = call(
        &c.app,
        &c.cookie,
        Method::PATCH,
        &format!("/api/v1/families/{}", c.f1),
        Some(json!({"version": f1["version"], "partner2_id": ""})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], true);
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::NOT_FOUND);
    undo(&c, body["tx_id"].as_str().unwrap()).await;
    let (st, restored) = get_family(&c, &c.f1).await;
    assert_eq!(st, StatusCode::OK);
    assert!(!restored["partner2_id"].is_null());
}

#[tokio::test]
async fn detach_child_dissolves_single_parent_family_and_undoes() {
    let c = setup().await;
    let pyotr = new_person(&c, "Pyotr").await;
    let kid = new_person(&c, "Kid").await;
    let (_, fam) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/families",
        Some(json!({"partner1_id": pyotr, "children_ids": [kid]})),
    )
    .await;
    let fid = fam["id"].as_str().unwrap().to_string();

    // Not a child of this family.
    let (st, err) = del(&c, &format!("/api/v1/families/{fid}/children/{}", c.oleg)).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(err["error"]["code"], "child_not_found");
    // Stale version.
    let (st, err) = del(
        &c,
        &format!("/api/v1/families/{fid}/children/{kid}?version=99"),
    )
    .await;
    assert_eq!(st, StatusCode::CONFLICT);
    assert_eq!(err["error"]["code"], "version_conflict");

    let (st, body) = del(
        &c,
        &format!("/api/v1/families/{fid}/children/{kid}?version=1"),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], true);
    assert_eq!(get_family(&c, &fid).await.0, StatusCode::NOT_FOUND);
    undo(&c, body["tx_id"].as_str().unwrap()).await;
    let (st, restored) = get_family(&c, &fid).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(child_ids(&restored), vec![kid]);
}

#[tokio::test]
async fn detach_partner_of_childless_union_dissolves_with_events_restored_on_undo() {
    let c = setup().await;
    del(
        &c,
        &format!("/api/v1/families/{}/children/{}", c.f1, c.oleg),
    )
    .await;
    let (st, ev) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/events",
        Some(json!({"family_id": c.f1, "event_type": "marriage", "date_text": "2005"})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let ev_id = ev["id"].as_str().unwrap().to_string();

    let (st, err) = del(
        &c,
        &format!("/api/v1/families/{}/partners/{}", c.f1, c.oleg),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(err["error"]["code"], "partner_not_found");

    let (st, body) = del(
        &c,
        &format!("/api/v1/families/{}/partners/{}", c.f1, c.ivan),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(body["dissolved"], true);
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::NOT_FOUND);
    undo(&c, body["tx_id"].as_str().unwrap()).await;
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::OK);
    let (_, evs) = call(
        &c.app,
        &c.cookie,
        Method::GET,
        &format!("/api/v1/events?family_id={}", c.f1),
        None,
    )
    .await;
    assert!(evs["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == ev_id.as_str()));
}

async fn delete_marriage(c: &Ctx, ev_id: &str) -> (StatusCode, String) {
    let resp = c
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/events/{ev_id}"))
                .header(header::COOKIE, &c.cookie)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tx = resp
        .headers()
        .get("x-transaction-id")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    (resp.status(), tx)
}

async fn add_marriage(c: &Ctx) -> String {
    let (st, ev) = call(
        &c.app,
        &c.cookie,
        Method::POST,
        "/api/v1/events",
        Some(json!({"family_id": c.f1, "event_type": "marriage", "date_text": "2005"})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    ev["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn deleting_marriage_dissolves_childless_two_partner_union_and_undo_restores_both() {
    let c = setup().await;
    del(
        &c,
        &format!("/api/v1/families/{}/children/{}", c.f1, c.oleg),
    )
    .await;
    let ev_id = add_marriage(&c).await;
    let (st, tx) = delete_marriage(&c, &ev_id).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    assert!(!tx.is_empty());
    // Two partners, no children: dissolved (marriage-event rule is "childless", not "orphaned").
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::NOT_FOUND);
    undo(&c, &tx).await;
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::OK);
    let (_, evs) = call(
        &c.app,
        &c.cookie,
        Method::GET,
        &format!("/api/v1/events?family_id={}", c.f1),
        None,
    )
    .await;
    assert!(evs["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == ev_id.as_str()));
}

#[tokio::test]
async fn deleting_marriage_keeps_union_with_children() {
    let c = setup().await;
    let ev_id = add_marriage(&c).await;
    let (st, tx) = delete_marriage(&c, &ev_id).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    assert!(!tx.is_empty());
    assert_eq!(get_family(&c, &c.f1).await.0, StatusCode::OK);
    undo(&c, &tx).await;
    let (_, evs) = call(
        &c.app,
        &c.cookie,
        Method::GET,
        &format!("/api/v1/events?family_id={}", c.f1),
        None,
    )
    .await;
    assert!(evs["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == ev_id.as_str()));
}

#[tokio::test]
async fn detach_endpoints_require_authentication() {
    let c = setup().await;
    for uri in [
        format!("/api/v1/families/{}/children/{}", c.f1, c.oleg),
        format!("/api/v1/families/{}/partners/{}", c.f1, c.ivan),
    ] {
        let (st, _) = call(&c.app, "", Method::DELETE, &uri, None).await;
        assert!(
            st == StatusCode::UNAUTHORIZED || st == StatusCode::FORBIDDEN,
            "{st}"
        );
    }
}
