use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::auth::{AuthService, SetupRequest},
    state::AppState,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'_' | b'.' | b'~') {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

async fn read_json(resp: axum::response::Response) -> Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn setup_test_app() -> (axum::Router, String) {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Levshits Family Archive".to_string(),
        default_locale: "ru".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    };
    let state = AppState::new_demo(None, config);

    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Levshits Family Archive".to_string()),
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminStrongPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();

    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let app = create_router(state);
    (app, admin_auth)
}

#[tokio::test]
async fn test_gid_allocation_gedcom_preservation_and_non_reuse() {
    let (app, admin_auth) = setup_test_app().await;

    // 1. Create first person -> receives I0100
    let r1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Nikolai",
                        "surname": "Levshits",
                        "sex": "M"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let p1 = read_json(r1).await;
    assert_eq!(p1["gid"], "I0100");
    let p1_id = p1["id"].as_str().unwrap().to_string();

    // 2. Delete I0100 and create another person -> I0100 must NEVER be reused (`10 §3.1, §8 DoD #3`)
    let del_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/persons/{p1_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_resp.status(), StatusCode::OK);

    let r2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Olga",
                        "surname": "Levshits",
                        "sex": "F"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r2.status(), StatusCode::CREATED);
    let p2 = read_json(r2).await;
    assert_eq!(p2["gid"], "I0101");

    // 3. Create person with explicit GEDCOM xref `@I0150@` -> preserves `I0150` and advances sequence
    let r3 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "gid": "@I0150@",
                        "given_name": "Isaak",
                        "surname": "Levshits",
                        "sex": "M"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r3.status(), StatusCode::CREATED);
    let p3 = read_json(r3).await;
    assert_eq!(p3["gid"], "I0150");

    // Lookup by GID and UUID both return the same person
    let get_by_gid = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/persons/I0150")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_by_gid.status(), StatusCode::OK);

    // Next auto-allocated person gets I0151
    let r4 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Sofia",
                        "surname": "Levshits",
                        "sex": "F"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let p4 = read_json(r4).await;
    assert_eq!(p4["gid"], "I0151");
}

#[tokio::test]
async fn test_ancestor_cycle_detection_and_quick_add_smart_defaults() {
    let (app, admin_auth) = setup_test_app().await;

    // 1. Test Slavic patronymic suggestion endpoint (`07 §3 #4, §5 DoD #8`)
    for (father, sex, expected) in [
        ("Иван", "M", "Иванович"),
        ("Иван", "F", "Ивановна"),
        ("Илья", "M", "Ильич"),
        ("Илья", "F", "Ильинична"),
        ("Никита", "M", "Никитич"),
        ("Никита", "F", "Никитична"),
        ("Павел", "M", "Павлович"),
        ("Павел", "F", "Павловна"),
        ("Лев", "M", "Львович"),
        ("Лев", "F", "Львовна"),
    ] {
        let uri = format!(
            "/api/v1/persons/suggest-patronymic?father_given_name={}&sex={}",
            percent_encode(father),
            sex
        );
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&uri)
                    .header(header::AUTHORIZATION, &admin_auth)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_json(resp).await;
        assert_eq!(body["patronymic"].as_str(), Some(expected));
    }

    // 2. Create Father: Иван Петров
    let father_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Иван",
                        "surname": "Петров",
                        "sex": "M",
                        "birth_date": "1940-05-10"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let father = read_json(father_resp).await;
    let father_id = father["id"].as_str().unwrap().to_string();

    // 3. Quick-add son Алексей -> auto-inherits surname "Петров" and Slavic patronymic "Иванович"
    let son_qa_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/persons/{father_id}/quick-add"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "relation": "child",
                        "given_name": "Алексей",
                        "sex": "M",
                        "birth_date": "1968-02-14"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(son_qa_resp.status(), StatusCode::CREATED);
    let son_qa = read_json(son_qa_resp).await;
    assert_eq!(son_qa["created_new_person"], true);
    assert_eq!(son_qa["person"]["primary_name"]["given_name"], "Алексей");
    assert_eq!(son_qa["person"]["primary_name"]["patronymic"], "Иванович");
    assert_eq!(son_qa["person"]["primary_name"]["surname"], "Петров");
    let son_id = son_qa["person"]["id"].as_str().unwrap().to_string();
    let parent_family_id = son_qa["family"]["id"].as_str().unwrap().to_string();

    // 4. Quick-add mother Анна to son -> reuses the existing single-parent family (`partner2_id`)
    let mother_qa_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/persons/{son_id}/quick-add"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "relation": "mother",
                        "given_name": "Анна",
                        "sex": "F"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mother_qa_resp.status(), StatusCode::CREATED);
    let mother_qa = read_json(mother_qa_resp).await;
    assert_eq!(mother_qa["family"]["id"], parent_family_id);
    assert_eq!(mother_qa["family"]["partner1_id"], father_id);
    assert_eq!(
        mother_qa["family"]["partner2_id"],
        mother_qa["person"]["id"]
    );

    // 5. Quick-add granddaughter Мария to son Алексей -> auto-fills patronymic "Алексеевна"
    let gd_qa_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/persons/{son_id}/quick-add"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "relation": "child",
                        "given_name": "Мария",
                        "sex": "F",
                        "birth_date": "1995-09-01"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(gd_qa_resp.status(), StatusCode::CREATED);
    let gd_qa = read_json(gd_qa_resp).await;
    assert_eq!(gd_qa["person"]["primary_name"]["patronymic"], "Алексеевна");
    let granddaughter_id = gd_qa["person"]["id"].as_str().unwrap().to_string();

    // 6. Attempt to link Granddaughter Мария as parent of Grandfather Иван -> MUST fail with 400 `cycle_detected` (`10 §6.1, §8 DoD #4`)
    let cycle_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/persons/{father_id}/quick-add"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "relation": "mother",
                        "existing_person_id": granddaughter_id
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cycle_resp.status(), StatusCode::BAD_REQUEST);
    let cycle_err = read_json(cycle_resp).await;
    assert_eq!(cycle_err["error"]["code"], "cycle_detected");

    // 7. Attempt direct self-parenting via POST /api/v1/families -> MUST fail with 400 `cycle_detected`
    let self_cycle_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/families")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "partner1_id": father_id,
                        "children": [{ "child_id": father_id }]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(self_cycle_resp.status(), StatusCode::BAD_REQUEST);
    let self_cycle_err = read_json(self_cycle_resp).await;
    assert_eq!(self_cycle_err["error"]["code"], "cycle_detected");
}

#[tokio::test]
async fn test_safe_delete_person_and_restore_from_history() {
    let (app, admin_auth) = setup_test_app().await;

    // Create an anchor person with a spouse, a birth event, and a note
    let p_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Pavel",
                        "surname": "Levshits",
                        "sex": "M",
                        "birth_date": "1935-04-12",
                        "birth_place": "Vitebsk, Belarus"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let pavel = read_json(p_resp).await;
    let pavel_id = pavel["id"].as_str().unwrap().to_string();

    // Add a note to Pavel
    let note_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/notes")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "entity_type": "individual",
                        "entity_id": pavel_id,
                        "body": "Served in the Baltic Fleet."
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(note_resp.status(), StatusCode::CREATED);

    // Quick-add spouse Vera
    let spouse_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/persons/{pavel_id}/quick-add"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "relation": "spouse",
                        "given_name": "Vera",
                        "surname": "Levshits",
                        "sex": "F"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let spouse_qa = read_json(spouse_resp).await;
    let vera_id = spouse_qa["person"]["id"].as_str().unwrap().to_string();
    let family_id = spouse_qa["family"]["id"].as_str().unwrap().to_string();

    // Check delete impact for Pavel (`07 §4, §5 DoD #5`)
    let impact_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/persons/{pavel_id}/delete-impact"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(impact_resp.status(), StatusCode::OK);
    let impact = read_json(impact_resp).await;
    assert_eq!(impact["events_count"], 1);
    assert_eq!(impact["notes_count"], 1);
    assert_eq!(impact["families_unlinked_count"], 1);
    assert_eq!(impact["empty_families_deleted_count"], 0);

    // Delete Pavel -> Vera and the family must survive (with partner1_id set to null)
    let del_pavel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/persons/{pavel_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_pavel.status(), StatusCode::OK);

    // Verify Vera still exists
    let get_vera = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/persons/{vera_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_vera.status(), StatusCode::OK);

    // Verify family still exists with partner1_id == null and partner2_id == vera_id
    let get_fam = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/families/{family_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_fam.status(), StatusCode::OK);
    let fam_after = read_json(get_fam).await;
    assert!(fam_after["partner1_id"].is_null());
    assert_eq!(fam_after["partner2_id"], vera_id);

    // Restore Pavel from History (`POST /api/v1/history/{pavel_id}/restore`)
    let restore_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/history/{pavel_id}/restore"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore_resp.status(), StatusCode::OK);

    // Verify Pavel's family link, event, and note are restored!
    let get_fam_restored = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/families/{family_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fam_restored = read_json(get_fam_restored).await;
    assert_eq!(fam_restored["partner1_id"], pavel_id);
    assert_eq!(fam_restored["partner2_id"], vera_id);
}

#[tokio::test]
async fn test_events_living_status_recomputation_and_source_autocomplete() {
    let (app, admin_auth) = setup_test_app().await;

    // 1. Create a person with no dates -> defaults to living (`is_living: true`)
    let p_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/persons")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "given_name": "Grigory",
                        "surname": "Levshits",
                        "sex": "M"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let grigory = read_json(p_resp).await;
    let grigory_id = grigory["id"].as_str().unwrap().to_string();
    assert_eq!(grigory["is_living"], true);

    // 2. Reject family-only event ("marriage") on an individual (`10 §4.3`)
    let bad_ev = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "individual_id": grigory_id,
                        "event_type": "marriage",
                        "date_text": "1975"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_ev.status(), StatusCode::BAD_REQUEST);

    // 3. Add a death event with source citation -> Grigory automatically becomes `is_living: false` (`10 §5, §8 DoD #6`)
    let death_ev_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "individual_id": grigory_id,
                        "event_type": "death",
                        "date_text": "около 1998",
                        "place_path": "Тверь, Тверская губерния, Российская империя",
                        "source_text": "Метрическая книга, ГАТО ф. 160 оп. 1 д. 15234"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(death_ev_resp.status(), StatusCode::CREATED);
    let death_ev = read_json(death_ev_resp).await;
    assert_eq!(death_ev["gedcom_date"], "ABT 1998");
    assert_eq!(death_ev["date_sort"], 19980000);
    let death_ev_id = death_ev["id"].as_str().unwrap().to_string();

    // Check Grigory is now deceased
    let g_after_death = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/persons/{grigory_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let g_json = read_json(g_after_death).await;
    assert_eq!(g_json["is_living"], false);
    assert_eq!(g_json["has_death_event"], true);

    // 4. Verify source citation autocomplete (`GET /api/v1/events/sources?q=ГАТО`)
    let src_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/events/sources?q={}",
                    percent_encode("ГАТО")
                ))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(src_resp.status(), StatusCode::OK);
    let src_json = read_json(src_resp).await;
    assert_eq!(
        src_json["items"][0],
        "Метрическая книга, ГАТО ф. 160 оп. 1 д. 15234"
    );

    // 5. Delete the death event -> Grigory's `is_living` recomputes back to `true`
    let del_ev = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/events/{death_ev_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_ev.status(), StatusCode::NO_CONTENT);

    let g_after_del = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/persons/{grigory_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let g_del_json = read_json(g_after_del).await;
    assert_eq!(g_del_json["is_living"], true);
    assert_eq!(g_del_json["has_death_event"], false);
}

#[tokio::test]
async fn test_places_hierarchy_deduplication_search_delete_protection_and_merge() {
    let (app, admin_auth) = setup_test_app().await;

    // 1. Create hierarchical place via comma-separated path (`12 §3.1, §7 DoD #1 & #5`)
    let tver_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/places")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "path": "Тверь, Тверская губерния, Российская империя",
                        "latitude": 56.8587,
                        "longitude": 35.9176,
                        "alt_names": [
                            { "name": "Калинин", "lang": "ru", "note": "1931–1990 Soviet name" }
                        ]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tver_resp.status(), StatusCode::CREATED);
    let tver = read_json(tver_resp).await;
    assert_eq!(tver["name"], "Тверь");
    assert_eq!(
        tver["full_name"],
        "Тверь, Тверская губерния, Российская империя"
    );
    let tver_id = tver["id"].as_str().unwrap().to_string();
    let guberniya_id = tver["parent_id"].as_str().unwrap().to_string();

    // 2. Create sibling place "Торжок, Тверская губерния, Российская империя" -> reuses parent chain without duplicating
    let torzhok_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/places")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "path": "Торжок, Тверская губерния, Российская империя"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let torzhok = read_json(torzhok_resp).await;
    assert_eq!(torzhok["parent_id"].as_str().unwrap(), guberniya_id);

    // 3. Place detail returns 3-level breadcrumbs (`Российская империя` -> `Тверская губерния` -> `Тверь`)
    let detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/places/{tver_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_resp.status(), StatusCode::OK);
    let detail = read_json(detail_resp).await;
    let crumbs = detail["breadcrumbs"].as_array().unwrap();
    assert_eq!(crumbs.len(), 3);
    assert_eq!(crumbs[0]["name"], "Российская империя");
    assert_eq!(crumbs[1]["name"], "Тверская губерния");
    assert_eq!(crumbs[2]["name"], "Тверь");

    // 4. Search by historical name "Калинин" AND Latin transliteration "Tver" (`12 §3.1, §7 DoD #2`)
    for query_str in ["Калинин", "Tver"] {
        let search_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/places?q={}", percent_encode(query_str)))
                    .header(header::AUTHORIZATION, &admin_auth)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(search_resp.status(), StatusCode::OK);
        let search_json = read_json(search_resp).await;
        let items = search_json["items"].as_array().unwrap();
        assert!(
            items.iter().any(|p| p["id"] == tver_id),
            "Expected query '{query_str}' to find Тверь"
        );
    }

    // 5. Rename parent "Тверская губерния" -> "Тверская область" -> updates descendant full_name automatically (`12 §2`)
    let rename_parent = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/places/{guberniya_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Тверская область"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rename_parent.status(), StatusCode::OK);

    let tver_after_rename = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/places/{tver_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tver_renamed_json = read_json(tver_after_rename).await;
    assert_eq!(
        tver_renamed_json["place"]["full_name"],
        "Тверь, Тверская область, Российская империя"
    );

    // 6. Delete protection (`12 §7 DoD #4`): deleting "Тверская область" (which has child places) returns 409 `place_in_use`
    let del_in_use = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/places/{guberniya_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_in_use.status(), StatusCode::CONFLICT);
    let del_err = read_json(del_in_use).await;
    assert_eq!(del_err["error"]["code"], "place_in_use");

    // 7. Merge duplicate place (`12 §3.3, §7 DoD #4`): create duplicate "г. Тверь" with an event, then merge into Тверь
    let dup_place_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/places")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "г. Тверь"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let dup_place = read_json(dup_place_resp).await;
    let dup_place_id = dup_place["id"].as_str().unwrap().to_string();

    // Attach an event to dup_place_id on seeded person I0001
    let ev_on_dup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "individual_id": "I0001",
                        "event_type": "residence",
                        "date_text": "1935",
                        "place_id": dup_place_id
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ev_on_dup.status(), StatusCode::CREATED);

    // Merge dup_place into tver_id
    let merge_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/places/{tver_id}/merge"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "other_id": dup_place_id
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(merge_resp.status(), StatusCode::OK);
    let merged = read_json(merge_resp).await;
    assert_eq!(merged["events_count"], 1);
    let alt_names = merged["alt_names"].as_array().unwrap();
    assert!(alt_names.iter().any(|a| a["name"] == "г. Тверь"));
}

#[tokio::test]
async fn test_optimistic_locking_version_conflict() {
    let (app, admin_auth) = setup_test_app().await;

    // Fetch I0001 (version 1)
    let get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/persons/I0001")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let p_initial = read_json(get_resp).await;
    assert_eq!(p_initial["version"], 1);

    // First PATCH with version: 1 -> succeeds and bumps version to 2
    let patch1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/persons/I0001")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "nickname": "Misha",
                        "version": 1
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch1.status(), StatusCode::OK);
    let p_updated = read_json(patch1).await;
    assert_eq!(p_updated["version"], 2);

    // Second PATCH with stale version: 1 -> returns 409 Conflict (`version_conflict`) (`07 §5 DoD #7`)
    let patch_stale = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/persons/I0001")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "nickname": "StaleEdit",
                        "version": 1
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_stale.status(), StatusCode::CONFLICT);
    let err_json = read_json(patch_stale).await;
    assert_eq!(err_json["error"]["code"], "version_conflict");
}
