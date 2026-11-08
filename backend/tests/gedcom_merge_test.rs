use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    routes::{
        create_router,
        rbac::{
            EventListResponse, MediaArchiveResponse, NoteListResponse, PersonProfileResponseDto,
        },
    },
    services::{
        auth::{AuthService, SetupRequest},
        gedcom::GedcomImportResult,
        privacy::TreePersonNodeDto,
    },
    state::AppState,
};
use serde_json::json;
use tower::ServiceExt;

async fn setup_admin_env() -> (axum::Router, String) {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "GEDCOM Merge Test Archive".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new_demo(None, config);
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
        false,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let app = create_router(state);
    (app, admin_auth)
}

#[tokio::test]
async fn test_gedcom_smart_merge_enrichment_and_non_destructive_fields() {
    let (app, admin_auth) = setup_admin_env().await;

    // Step 1: Import initial GEDCOM in "merge" mode when store has `is_demo_seed == true`.
    // It should clear the demo seed records first and import the 3 initial people & 1 family cleanly.
    let base_gedcom = "\
0 HEAD
1 SOUR FamilyTreeApp
2 VERS 1.0
1 GEDC
2 VERS 5.5.1
1 CHAR UTF-8
0 @O0001@ OBJE
1 FILE /photos/ivan_portrait.jpg
2 FORM jpg
0 @I0001@ INDI
1 NAME Иван /Соколов/
2 GIVN Иван
2 SURN Соколов
2 NICK Ваня
1 SEX M
1 BIRT
2 DATE 1950
1 OBJE @O0001@
1 NOTE Первая заметка об Иване
0 @I0002@ INDI
1 NAME Мария /Соколова/
2 GIVN Мария
2 SURN Соколова
1 SEX F
1 BIRT
2 DATE 12 MAR 1952
2 PLAC Москва, Россия
0 @I0003@ INDI
1 NAME Алексей Иванович /Соколов/
2 GIVN Алексей Иванович
2 SURN Соколов
1 SEX M
1 BIRT
2 DATE 10 JUN 1975
1 FAMC @F0001@
0 @F0001@ FAM
1 HUSB @I0001@
1 WIFE @I0002@
1 CHIL @I0003@
1 MARR
2 DATE 1974
0 TRLR
";

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/import")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "data": base_gedcom,
                        "mode": "merge"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 128 * 1024).await.unwrap();
    let first_import: GedcomImportResult = serde_json::from_slice(&body).unwrap();
    assert_eq!(first_import.mode, "merge");
    assert_eq!(first_import.imported_individuals, 3);
    assert_eq!(first_import.imported_families, 1);
    assert_eq!(first_import.merged_individuals, 0);
    assert_eq!(first_import.merged_families, 0);
    assert_eq!(first_import.imported_media, 1);

    // Step 2: Import a second GEDCOM with `mode: "merge"` that has:
    // - Different XREFs (`@I0010@` for Иван Соколов, `@I0020@` for Мария Соколова, `@I0001@` for a NEW person Ольга Соколова to test no false GID collision!)
    // - Enriches Иван Соколов (`@I0010@`): adds patronymic `Петрович`, suffix `ст.`, upgrades BIRT date from `1950` to `15 MAY 1950` and adds PLAC `Тверь, Россия`, adds DEAT `2020`, tries to set NICK `Иванушка` (must NOT overwrite existing `Ваня`), adds UID, reuses `@O0099@` (`ivan_portrait.jpg`) and enriches its TITL `Портрет Ивана`.
    // - Enriches Мария Соколова (`@I0020@`): adds patronymic `Сергеевна`, adds maiden/birth surname `Орлова` (`_MARNM Соколова`), keeps existing BIRT date `12 MAR 1952`.
    // - Matches existing family (`@F0099@` for Иван + Мария) and adds a second child `@I0001@` (Ольга Ивановна Соколова, born 1978) plus marriage place `Москва, ЗАГС` without duplicating the family!
    // - Adds a new spouse for Ольга (`@I0040@` Дмитрий Морозов) in a new family `@F0100@`.
    let second_gedcom = "\
0 HEAD
1 SOUR MyHeritage
2 VERS 8.0
1 GEDC
2 VERS 5.5.1
1 CHAR UTF-8
0 @O0099@ OBJE
1 FILE C:\\Users\\Admin\\Photos\\ivan_portrait.jpg
2 FORM jpg
2 TITL Портрет Ивана
0 @I0010@ INDI
1 _UID 11111111-2222-3333-4444-555555555555
1 NAME Иван Петрович /Соколов/
2 GIVN Иван Петрович
2 SURN Соколов
2 NSFX ст.
2 NICK Иванушка
1 SEX M
1 BIRT
2 DATE 15 MAY 1950
2 PLAC Тверь, Россия
1 DEAT
2 DATE 20 NOV 2020
2 PLAC Москва, Россия
1 OBJE @O0099@
1 NOTE Первая заметка об Иване
1 NOTE Вторая дополнительная заметка об Иване
0 @I0020@ INDI
1 NAME Мария Сергеевна /Орлова/
2 GIVN Мария
2 _PATR Сергеевна
2 SURN Орлова
2 _MARNM Соколова
1 SEX F
1 BIRT
2 DATE 1952
0 @I0001@ INDI
1 NAME Ольга Ивановна /Соколова/
2 GIVN Ольга Ивановна
2 SURN Соколова
1 SEX F
1 BIRT
2 DATE 4 AUG 1978
1 FAMC @F0099@
0 @I0040@ INDI
1 NAME Дмитрий /Морозов/
2 GIVN Дмитрий
2 SURN Морозов
1 SEX M
1 BIRT
2 DATE 1976
0 @F0099@ FAM
1 HUSB @I0010@
1 WIFE @I0020@
1 CHIL @I0001@
1 MARR
2 DATE 1974
2 PLAC Москва, Россия
0 @F0100@ FAM
1 HUSB @I0040@
1 WIFE @I0001@
1 MARR
2 DATE 2001
0 TRLR
";

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/import")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "data": second_gedcom,
                        "mode": "merge"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 128 * 1024).await.unwrap();
    let merge_res: GedcomImportResult = serde_json::from_slice(&body).unwrap();

    assert_eq!(merge_res.mode, "merge");
    assert_eq!(merge_res.merged_individuals, 2); // Иван and Мария matched and enriched
    assert_eq!(merge_res.imported_individuals, 2); // Ольга and Дмитрий added
    assert_eq!(merge_res.merged_families, 1); // Иван + Мария family matched and enriched
    assert_eq!(merge_res.imported_families, 1); // Дмитрий + Ольга family added
    assert_eq!(merge_res.imported_media, 0); // O0099 deduplicated into O0001

    // Verify Иван Соколов (`I0001`) was enriched non-destructively
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0001/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let ivan_prof: PersonProfileResponseDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(ivan_prof.person.primary_name.given_name, "Иван");
    assert_eq!(
        ivan_prof.person.primary_name.patronymic.as_deref(),
        Some("Петрович")
    );
    assert_eq!(ivan_prof.person.primary_name.surname, "Соколов");
    // Existing nickname "Ваня" was NOT overwritten by "Иванушка"
    assert_eq!(ivan_prof.person.nickname.as_deref(), Some("Ваня"));
    // Missing suffix "ст." was filled in
    assert_eq!(ivan_prof.person.name_suffix.as_deref(), Some("ст."));
    // External UID was added
    assert_eq!(
        ivan_prof.person.external_ids.get("uid").map(String::as_str),
        Some("11111111-2222-3333-4444-555555555555")
    );
    // Birth event was upgraded from "1950" to "15 MAY 1950" with place "Тверь, Россия" without duplicating birth events
    let ivan_births: Vec<_> = ivan_prof
        .events
        .iter()
        .filter(|e| e.event_type == "birth")
        .collect();
    assert_eq!(ivan_births.len(), 1);
    assert_eq!(ivan_births[0].gedcom_date.as_deref(), Some("15 MAY 1950"));
    assert_eq!(
        ivan_births[0].place_full_name.as_deref(),
        Some("Тверь, Россия")
    );
    // Death event was added and marked Иван deceased
    assert!(!ivan_prof.person.is_living);
    assert!(ivan_prof.person.has_death_event);
    assert!(ivan_prof
        .events
        .iter()
        .any(|e| e.event_type == "death" && e.gedcom_date.as_deref() == Some("20 NOV 2020")));
    // Иван + Мария family now has 2 children (Алексей and Ольга)
    assert_eq!(ivan_prof.families.spouse_families.len(), 1);
    assert_eq!(ivan_prof.families.spouse_families[0].children.len(), 2);

    // Verify Мария Соколова (`I0002`) was enriched with patronymic and birth_surname, while keeping her full birth date `12 MAR 1952`
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0002/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let maria_prof: PersonProfileResponseDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(
        maria_prof.person.primary_name.patronymic.as_deref(),
        Some("Сергеевна")
    );
    assert_eq!(maria_prof.person.birth_surname.as_deref(), Some("Орлова"));
    let maria_births: Vec<_> = maria_prof
        .events
        .iter()
        .filter(|e| e.event_type == "birth")
        .collect();
    assert_eq!(maria_births.len(), 1);
    assert_eq!(maria_births[0].gedcom_date.as_deref(), Some("12 MAR 1952"));

    // Verify notes on Иван: duplicate note was skipped, new note was added (total 2 notes)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/notes?entity_type=individual&entity_id={}",
                    ivan_prof.person.id
                ))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let ivan_notes: NoteListResponse = serde_json::from_slice(&b).unwrap();
    assert_eq!(ivan_notes.items.len(), 2);

    // Verify media O0001 was enriched with title "Портрет Ивана" and not duplicated
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/media")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let media_list: MediaArchiveResponse = serde_json::from_slice(&b).unwrap();
    assert_eq!(media_list.total, 1);
    assert_eq!(media_list.items[0].title, "Портрет Ивана");

    // Verify marriage event on F0001 was enriched with place "Москва, Россия" without duplicating the marriage event
    let fam_id = &ivan_prof.families.spouse_families[0].family_id;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/events?family_id={fam_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let fam_events: EventListResponse = serde_json::from_slice(&b).unwrap();
    let marr_events: Vec<_> = fam_events
        .items
        .iter()
        .filter(|e| e.event_type == "marriage")
        .collect();
    assert_eq!(marr_events.len(), 1);
    assert_eq!(
        marr_events[0].place_full_name.as_deref(),
        Some("Москва, Россия")
    );

    // Step 3: Verify `mode: "clean_slate"` replaces the entire tree and echoes `mode: "clean_slate"`
    let replacement_gedcom = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
1 CHAR UTF-8
0 @I0001@ INDI
1 NAME Павел /Кузнецов/
1 SEX M
1 DEAT Y
0 TRLR
";
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/import")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "data": replacement_gedcom,
                        "mode": "clean_slate",
                        "confirm_text": "CLEAN_SLATE"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let cs_res: GedcomImportResult = serde_json::from_slice(&b).unwrap();
    assert_eq!(cs_res.mode, "clean_slate");
    assert_eq!(cs_res.imported_individuals, 1);
    assert_eq!(cs_res.merged_individuals, 0);
    assert_eq!(cs_res.imported_families, 0);
    assert_eq!(cs_res.merged_families, 0);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0001")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let p_i0001: TreePersonNodeDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(p_i0001.primary_name.given_name, "Павел");
    assert_eq!(p_i0001.primary_name.surname, "Кузнецов");
}
