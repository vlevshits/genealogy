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
        tree_title: "Search & Directory Test Archive".to_string(),
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

#[allow(clippy::too_many_lines)]
async fn setup_search_dataset() -> (axum::Router, String) {
    let state = AppState::new_empty(None, test_config());
    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Search & Directory Archive".to_string()),
            first_name: "Admin".to_string(),
            last_name: "Archivist".to_string(),
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

    // 1. Create Ivan Petrov born 1891 (deceased, matches "иван 1890" within ±2)
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
                        "gid": "I0101",
                        "given_name": "Иван",
                        "patronymic": "Сергеевич",
                        "surname": "Петров",
                        "sex": "M",
                        "birth_date": "1891",
                        "birth_place": "Тверь, Российская Империя",
                        "death_date": "1955"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let ivan_1891_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // 2. Create Ivan Petrov born 1850 (deceased, far from 1890)
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
                        "gid": "I0102",
                        "given_name": "Иван",
                        "surname": "Петров",
                        "sex": "M",
                        "birth_date": "1850",
                        "death_date": "1915"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let ivan_1850_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // Link Ivan 1850 as father of Ivan 1891 so Ivan 1891 has both parents + birth_date (`incomplete = false`)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/families")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "partner1_id": ivan_1850_id,
                        "relationship_type": "married",
                        "children": [{ "child_id": ivan_1891_id, "sort_order": 0 }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 3. Create Anna Petrova (feminine surname `Петрова`, born 1995, living)
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
                        "gid": "I0103",
                        "given_name": "Анна",
                        "surname": "Петрова",
                        "birth_surname": "Морозова",
                        "sex": "F",
                        "birth_date": "1995"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 4. Create Mikhail Shcheglov (`Щеглов`, born 1905, deceased)
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
                        "gid": "I0104",
                        "given_name": "Михаил",
                        "surname": "Щеглов",
                        "sex": "M",
                        "birth_date": "1905",
                        "death_date": "1972"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 5. Create Pre-1918 Russian orthography record: `Ѳедоръ Левшицъ` with given name `Алёна` / `Бѣловъ`
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
                        "gid": "I0105",
                        "given_name": "Ѳедоръ",
                        "patronymic": "Іоанновичъ",
                        "surname": "Левшицъ",
                        "birth_surname": "Бѣловъ",
                        "nickname": "Алёна",
                        "sex": "M",
                        " death_date": "1912",
                        "has_death_or_burial_event": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    (app, admin_cookie)
}

#[tokio::test]
async fn test_omnibox_cross_script_pre1918_orthography_gender_folding_and_year_proximity() {
    let (app, admin_cookie) = setup_search_dataset().await;

    // 1. Cross-script search: "petrov" finds both "Петров" and "Петрова" (gender folding + transliteration)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=petrov")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    let people = body["people"].as_array().unwrap();
    assert_eq!(
        people.len(),
        3,
        "Searching 'petrov' must match Ivan 1891 (Петров), Ivan 1850 (Петров), and Anna 1995 (Петрова)"
    );

    // 2. Cyrillic gender folding: searching "Петров" returns both "Петров" and "Петрова"
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=%D0%9F%D0%B5%D1%82%D1%80%D0%BE%D0%B2")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    let people = body["people"].as_array().unwrap();
    assert_eq!(people.len(), 3);
    assert!(people.iter().any(|p| p["surname"] == "Петрова"));

    // 3. Cross-script `shcheglov` finds `Щеглов`
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=shcheglov")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    let people = body["people"].as_array().unwrap();
    assert_eq!(people.len(), 1);
    assert_eq!(people[0]["gid"], "I0104");
    assert_eq!(people[0]["surname"], "Щеглов");

    // 4. Pre-1918 Russian orthography & ё/е equivalence:
    // - "Левшиц" matches "Левшицъ"
    // - "Белов" matches "Бѣловъ" (`ѣ -> е`, trailing `ъ`)
    // - "Федор" matches "Ѳедоръ" (`ѳ -> ф`, trailing `ъ`)
    // - "Иоаннович" matches "Іоанновичъ" (`і -> и`, trailing `ъ`)
    // - "Алена" matches "Алёна" (`ё -> е`)
    for modern_query in [
        "%D0%9B%D0%B5%D0%B2%D1%88%D0%B8%D1%86", // Левшиц
        "%D0%91%D0%B5%D0%BB%D0%BE%D0%B2",       // Белов
        "%D0%A4%D0%B5%D0%B4%D0%BE%D1%80",       // Федор
        "%D0%98%D0%BE%D0%B0%D0%BD%D0%BD%D0%BE%D0%B2%D0%B8%D1%87", // Иоаннович
        "%D0%90%D0%BB%D0%B5%D0%BD%D0%B0",       // Алена
    ] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/search?q={modern_query}"))
                    .header(header::COOKIE, &admin_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_json(resp).await;
        let people = body["people"].as_array().unwrap();
        assert!(
            people.iter().any(|p| p["gid"] == "I0105"),
            "Expected query {modern_query} to match pre-1918 record I0105"
        );
    }

    // 5. Birth year proximity ranking: "иван 1890" ranks Ivan 1891 (1890 ± 2) ahead of Ivan 1850
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=%D0%B8%D0%B2%D0%B0%D0%BD%201890")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    let people = body["people"].as_array().unwrap();
    assert_eq!(people.len(), 2);
    assert_eq!(
        people[0]["gid"], "I0101",
        "Ivan born 1891 must rank first for query 'иван 1890'"
    );
    assert_eq!(people[1]["gid"], "I0102");
    assert!(
        people[0]["score"].as_i64().unwrap() > people[1]["score"].as_i64().unwrap(),
        "1891 record must have higher score than 1850 record"
    );

    // 6. Exact GID match (`I0104`) jumps straight to top with score 10000+
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=I0104")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["people"][0]["gid"], "I0104");
    assert!(body["people"][0]["score"].as_i64().unwrap() >= 10_000);
}

#[tokio::test]
async fn test_people_directory_filters_sorting_incomplete_and_surnames_index() {
    let (app, admin_cookie) = setup_search_dataset().await;

    // 1. Surname filter (`surname=Петров`) matches both `Петров` and `Петрова` (3 people)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?surname=%D0%9F%D0%B5%D1%82%D1%80%D0%BE%D0%B2&sort=birth_asc")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["total"], 3);
    assert_eq!(body["total_unfiltered"], 5);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items[0]["birth_year"], 1850);
    assert_eq!(items[1]["birth_year"], 1891);
    assert_eq!(items[2]["birth_year"], 1995);

    // 2. Birth year range filter (`born_from=1880&born_to=1910`) -> Ivan 1891 and Mikhail 1905
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?born_from=1880&born_to=1910&sort=birth_asc")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["total"], 2);
    assert_eq!(body["items"][0]["gid"], "I0101");
    assert_eq!(body["items"][1]["gid"], "I0104");

    // 3. Living filter (`living=living`) -> only Anna Petrova (I0103)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?living=living")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["gid"], "I0103");

    // 4. Incomplete filter (`incomplete=false` -> only Ivan 1891 has BOTH birth_date AND parent; `incomplete=true` -> other 4)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?incomplete=false")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["gid"], "I0101");
    assert_eq!(body["items"][0]["has_parents"], true);
    assert_eq!(body["items"][0]["has_birth_date"], true);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?incomplete=true")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["total"], 4);

    // 5. Cursor pagination (`limit=2`)
    let resp_page1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people?limit=2&sort=name_asc")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_page1.status(), StatusCode::OK);
    let page1 = read_json(resp_page1).await;
    assert_eq!(page1["items"].as_array().unwrap().len(), 2);
    let next_cursor = page1["next_cursor"].as_str().unwrap();
    assert_eq!(next_cursor, "2");

    // 6. Surnames Index (`GET /api/v1/people/surnames`): groups Петров + Петрова into canonical "Петров" with count=3 and year span 1850..1995
    let resp_surnames = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/people/surnames")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_surnames.status(), StatusCode::OK);
    let surnames_body = read_json(resp_surnames).await;
    assert_eq!(surnames_body["total_people"], 5);
    assert_eq!(surnames_body["total_surnames"], 3); // Левшиц, Петров, Щеглов

    let groups = surnames_body["groups"].as_array().unwrap();
    let p_group = groups.iter().find(|g| g["letter"] == "П").unwrap();
    let petrov_entry = &p_group["items"][0];
    assert_eq!(petrov_entry["canonical_surname"], "Петров");
    assert_eq!(petrov_entry["count"], 3);
    assert_eq!(petrov_entry["earliest_year"], 1850);
    assert_eq!(petrov_entry["latest_year"], 1995);
    let variants = petrov_entry["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 2);
}

#[tokio::test]
async fn test_search_and_directory_rbac_and_living_privacy_guard() {
    let (app, admin_cookie) = setup_search_dataset().await;

    // 1. Default private mode (`enable_public_access = false`):
    // Unauthenticated requests to `/search`, `/people`, and `/people/surnames` all return 401
    for uri in [
        "/api/v1/search?q=petrov",
        "/api/v1/people",
        "/api/v1/people/surnames",
    ] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 for unauthenticated {uri} in default private mode"
        );
    }

    // 2. Enable public tree mode (`enable_public_access = true`)
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
                    json!({ "enable_public_access": true }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Even in public mode, `/api/v1/people` and `/api/v1/people/surnames` require authentication (401)
    for uri in ["/api/v1/people", "/api/v1/people/surnames"] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // 4. In public mode, `/api/v1/search?q=petrov` succeeds for anonymous callers,
    // returning ONLY the 2 deceased Petrovs (I0101, I0102) and NEVER leaking living Anna Petrova (I0103)!
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/search?q=petrov")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    let people = body["people"].as_array().unwrap();
    assert_eq!(
        people.len(),
        2,
        "Public search must only return the 2 deceased Petrovs and hide living Anna Petrova"
    );
    assert!(!people.iter().any(|p| p["gid"] == "I0103"));
    assert!(!people.iter().any(|p| p["is_living"] == true));

    // Searching directly by living person's given name ("Анна") or maiden surname ("Морозова") as anonymous returns 0 people
    for q in [
        "%D0%90%D0%BD%D0%BD%D0%B0",
        "%D0%9C%D0%BE%D1%80%D0%BE%D0%B7%D0%BE%D0%B2%D0%B0",
    ] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/search?q={q}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_json(resp).await;
        assert!(
            body["people"].as_array().unwrap().is_empty(),
            "Anonymous search must not discover living person by given name or maiden surname"
        );
    }
}
