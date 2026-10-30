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
        tree_title: "Person Profile Test Archive".to_string(),
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

#[derive(Debug, Clone)]
struct FamilyGraphIds {
    focal_id: String,
    father_id: String,
    mother_id: String,
    grandfather_id: String,
    great_grandfather_id: String,
    sibling_id: String,
    uncle_id: String,
    cousin_id: String,
    niece_id: String,
    spouse_id: String,
    father_in_law_id: String,
    child_id: String,
    grandchild_id: String,
    unlinked_id: String,
    private_id: String,
}

#[allow(clippy::too_many_lines)]
async fn setup_profile_test_tree() -> (axum::Router, String, FamilyGraphIds) {
    let state = AppState::new_empty(None, test_config());
    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Romanov Dossier Archive".to_string()),
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

    // 1. Focal / Home person: Anna (I0100, F, born 1985)
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

    // 2. Father: Nikolai (M, 1955–2010, deceased)
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
                        "patronymic": "Petrovich",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "10 MAR 1955",
                        "birth_place": "Moscow, Russia",
                        "death_date": "20 NOV 2010",
                        "death_place": "Saint Petersburg, Russia"
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

    // 3. Mother: Olga (F, born 1958, living)
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
                        "given_name": "Ольга",
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

    // 4. Sibling: Brother Sergey (M, born 1988)
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
                        "relation": "sibling",
                        "given_name": "Sergey",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1988"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let sibling_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 5. Nibling: Niece Elena (child of sibling Sergey, F, born 2015)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{sibling_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "given_name": "Elena",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "2015"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let niece_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 6. Grandfather: Pyotr (father of Nikolai, M, 1925–1980)
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

    // 7. Great-Grandfather: Mikhail (father of Pyotr, M, 1898–1965)
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
                        "relation": "father",
                        "given_name": "Mikhail",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1898",
                        "death_date": "1965"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let great_grandfather_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 8. Pibling: Uncle Pavel (brother of Father Nikolai, M, born 1958)
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
                        "relation": "sibling",
                        "given_name": "Pavel",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1958"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let uncle_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 9. First Cousin: Alexey (child of Uncle Pavel, M, born 1986)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{uncle_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "given_name": "Alexey",
                        "surname": "Romanov",
                        "sex": "M",
                        "birth_date": "1986"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let cousin_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 10. Spouse: Husband Ivan (M, born 1984, living)
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

    // 11. Parent-in-law (husband's father): Viktor Sokolov (M, born 1954)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{spouse_id}/quick-add"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "father",
                        "given_name": "Viktor",
                        "surname": "Sokolov",
                        "sex": "M",
                        "birth_date": "1954"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let father_in_law_id = read_json(resp).await["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 12. Child: Son Dmitry (M, born 2010)
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

    // 13. Grandchild: Granddaughter Sofia (F, born 2032)
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
                        "birth_date": "2032"
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

    // 14. Unlinked person: Boris (M, 1940–2005, deceased)
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
                        "given_name": "Boris",
                        "surname": "Unlinked",
                        "sex": "M",
                        "birth_date": "1940",
                        "death_date": "2005"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let unlinked_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // 15. Private person: Secret Person (`is_private: true`)
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
                        "given_name": "Secret",
                        "surname": "Agent",
                        "sex": "M",
                        "is_private": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let private_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // 16. Add a geocoded place and life event + burial event + note for Father Nikolai
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/places")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "path": "Russia, Tver Oblast, Tver",
                        "place_type": "city",
                        "latitude": 56.8587,
                        "longitude": 35.9176
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let tver_place_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // Education event for Father Nikolai in 1977 (age 22) at Tver with source citation
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
                        "individual_id": father_id,
                        "event_type": "education",
                        "date_text": "1977",
                        "place_id": tver_place_id,
                        "description": "Graduated from Tver State University",
                        "source_text": "Tver University Archive, Fond 12, Op. 4"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Burial event for Father Nikolai in 2010 (post-death -> age_at_event should be null)
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
                        "individual_id": father_id,
                        "event_type": "burial",
                        "date_text": "25 NOV 2010",
                        "place_id": tver_place_id,
                        "source_text": "Tver University Archive, Fond 12, Op. 4"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Research note on Father Nikolai
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/notes")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "entity_type": "individual",
                        "entity_id": father_id,
                        "body": "Served as chief engineer in Saint Petersburg.",
                        "is_private": false,
                        "sort_order": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    (
        app,
        admin_cookie,
        FamilyGraphIds {
            focal_id,
            father_id,
            mother_id,
            grandfather_id,
            great_grandfather_id,
            sibling_id,
            uncle_id,
            cousin_id,
            niece_id,
            spouse_id,
            father_in_law_id,
            child_id,
            grandchild_id,
            unlinked_id,
            private_id,
        },
    )
}

#[tokio::test]
async fn test_person_profile_aggregated_sections_and_capabilities() {
    let (app, admin_cookie, ids) = setup_profile_test_tree().await;

    // Fetch Father Nikolai's profile by UUID
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/profile", ids.father_id))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;

    // 1. Person record
    assert_eq!(body["person"]["id"], ids.father_id);
    assert_eq!(body["person"]["primary_name"]["given_name"], "Nikolai");
    assert_eq!(body["person"]["is_living"], false);

    // 2. Kinship relative to Anna (home person) -> parent ("Father" / "Отец")
    assert_eq!(body["kinship"]["relation_key"], "parent");
    assert_eq!(body["kinship"]["label_en"], "Father");
    assert_eq!(body["kinship"]["label_ru"], "Отец");
    assert_eq!(body["kinship"]["steps"], 1);

    // 3. Events sorted chronologically with age_at_event and coordinates
    let events = body["events"].as_array().unwrap();
    assert!(events.len() >= 4);
    let birth_ev = events.iter().find(|e| e["event_type"] == "birth").unwrap();
    assert_eq!(birth_ev["age_at_event"], 0);

    let edu_ev = events
        .iter()
        .find(|e| e["event_type"] == "education")
        .unwrap();
    assert_eq!(edu_ev["age_at_event"], 22);
    assert_eq!(edu_ev["latitude"], 56.8587);
    assert_eq!(edu_ev["longitude"], 35.9176);

    let death_ev = events.iter().find(|e| e["event_type"] == "death").unwrap();
    assert_eq!(death_ev["age_at_event"], 55);

    let burial_ev = events.iter().find(|e| e["event_type"] == "burial").unwrap();
    assert!(burial_ev["age_at_event"].is_null());

    // 4. Families: parent_families (Grandfather Pyotr + sibling Uncle Pavel) & spouse_families (Mother Olga + children Anna & Sergey)
    let parent_fams = body["families"]["parent_families"].as_array().unwrap();
    assert_eq!(parent_fams.len(), 1);
    assert_eq!(parent_fams[0]["father"]["id"], ids.grandfather_id);
    assert_eq!(parent_fams[0]["siblings"][0]["id"], ids.uncle_id);

    let spouse_fams = body["families"]["spouse_families"].as_array().unwrap();
    assert_eq!(spouse_fams.len(), 1);
    assert_eq!(spouse_fams[0]["spouse"]["id"], ids.mother_id);
    assert_eq!(
        spouse_fams[0]["children"].as_array().unwrap().len(),
        2,
        "Father should have 2 children (Anna and Sergey)"
    );

    // 5. Notes
    let notes = body["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        notes[0]["body"],
        "Served as chief engineer in Saint Petersburg."
    );

    // 6. Sources summary deduplicated
    let sources = body["sources_summary"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0]["source_text"],
        "Tver University Archive, Fond 12, Op. 4"
    );
    assert_eq!(sources[0]["events_count"], 2);

    // 7. Capabilities
    assert_eq!(body["capabilities"]["can_edit"], true);
    assert_eq!(body["capabilities"]["can_delete"], true);
    assert_eq!(body["capabilities"]["can_merge"], true);
    assert_eq!(body["capabilities"]["is_home_person"], false);

    // Also test lookup by GID (`I0100` for focal person Anna) and `?include=kinship,events`
    let resp_gid = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0100/profile?include=kinship,events")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_gid.status(), StatusCode::OK);
    let gid_body = read_json(resp_gid).await;
    assert_eq!(gid_body["person"]["id"], ids.focal_id);
    assert_eq!(gid_body["capabilities"]["is_home_person"], true);
    assert_eq!(gid_body["kinship"]["relation_key"], "self");
    assert!(
        gid_body["families"]["parent_families"]
            .as_array()
            .unwrap()
            .is_empty(),
        "families should be empty when omitted from ?include="
    );
}

#[tokio::test]
async fn test_kinship_solver_all_relationship_paths() {
    let (app, admin_cookie, ids) = setup_profile_test_tree().await;

    let cases = [
        (&ids.focal_id, "self", "You (Home Person)", 0usize),
        (&ids.father_id, "parent", "Father", 1),
        (&ids.mother_id, "parent", "Mother", 1),
        (&ids.grandfather_id, "grandparent", "Grandfather", 2),
        (
            &ids.great_grandfather_id,
            "great_grandparent",
            "Great-grandfather",
            3,
        ),
        (&ids.sibling_id, "sibling", "Brother", 2),
        (&ids.uncle_id, "pibling", "Uncle", 3),
        (&ids.cousin_id, "first_cousin", "First cousin", 4),
        (&ids.niece_id, "nibling", "Niece", 3),
        (&ids.spouse_id, "spouse", "Husband", 1),
        (
            &ids.father_in_law_id,
            "parent_in_law_husband",
            "Father-in-law (husband's father)",
            2,
        ),
        (&ids.child_id, "child", "Son", 1),
        (&ids.grandchild_id, "grandchild", "Granddaughter", 2),
    ];

    for (target_id, expected_key, expected_en, expected_steps) in cases {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/api/v1/persons/{target_id}/profile?include=kinship"
                    ))
                    .header(header::COOKIE, &admin_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_json(resp).await;
        assert_eq!(
            body["kinship"]["relation_key"], expected_key,
            "Unexpected relation_key for target {target_id}"
        );
        assert_eq!(body["kinship"]["label_en"], expected_en);
        assert_eq!(body["kinship"]["steps"], expected_steps);
    }

    // Unlinked person returns kinship: null
    let resp_unlinked = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/persons/{}/profile?include=kinship",
                    ids.unlinked_id
                ))
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_unlinked.status(), StatusCode::OK);
    let unlinked_body = read_json(resp_unlinked).await;
    assert!(unlinked_body["kinship"].is_null());
}

#[tokio::test]
async fn test_person_profile_public_masking_and_privacy_404() {
    let (app, admin_cookie, ids) = setup_profile_test_tree().await;

    // 1. Default private instance: unauthenticated caller gets 401
    let resp_unauth = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/profile", ids.father_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Enable public tree sharing
    let resp_enable = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({ "enable_public_access": true }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_enable.status(), StatusCode::OK);

    // 3. Anonymous public caller fetching living person (Anna) -> initial with dot ("A."), empty events/notes/media/sources, capabilities read-only
    let resp_living = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/profile", ids.focal_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_living.status(), StatusCode::OK);
    let living_body = read_json(resp_living).await;
    assert_eq!(living_body["person"]["primary_name"]["given_name"], "A.");
    assert_eq!(living_body["person"]["primary_name"]["surname"], "");
    assert!(living_body["person"]["birth"].is_null());
    assert!(living_body["events"].as_array().unwrap().is_empty());
    assert!(living_body["notes"].as_array().unwrap().is_empty());
    assert!(living_body["media"].as_array().unwrap().is_empty());
    assert!(living_body["sources_summary"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(living_body["kinship"].is_null());
    assert_eq!(living_body["capabilities"]["can_edit"], false);

    // Living relatives inside Father Nikolai's profile are also masked to initials for AnonymousPublic
    let resp_deceased = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/profile", ids.father_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_deceased.status(), StatusCode::OK);
    let dec_body = read_json(resp_deceased).await;
    assert_eq!(dec_body["person"]["primary_name"]["given_name"], "Nikolai");
    assert_eq!(
        dec_body["families"]["spouse_families"][0]["spouse"]["given_name"], "О.",
        "Living mother Olga should be masked to 'О.' in anonymous public profile"
    );
    assert!(dec_body["families"]["spouse_families"][0]["spouse"]["birth_year"].is_null());

    // 4. Anonymous public caller fetching private person gets 404 person_not_found
    let resp_priv = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/profile", ids.private_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_priv.status(), StatusCode::NOT_FOUND);
    let priv_err = read_json(resp_priv).await;
    assert_eq!(priv_err["error"]["code"], "person_not_found");

    // 5. Non-existent ID returns 404 person_not_found
    let resp_missing = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I9999/profile")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_missing.status(), StatusCode::NOT_FOUND);
    let missing_err = read_json(resp_missing).await;
    assert_eq!(missing_err["error"]["code"], "person_not_found");
}
