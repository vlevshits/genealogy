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

fn count_connected_components(
    persons: &[TreePersonNodeDto],
    families: &[genealogy_backend::services::privacy::FamilyDto],
) -> usize {
    use std::collections::{HashMap, HashSet, VecDeque};

    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for p in persons {
        adj.entry(p.id.as_str()).or_default();
    }
    for f in families {
        let mut members: Vec<&str> = Vec::new();
        if let Some(p1) = f.partner1_id.as_deref() {
            members.push(p1);
        }
        if let Some(p2) = f.partner2_id.as_deref() {
            members.push(p2);
        }
        for c in &f.children {
            members.push(c.child_id.as_str());
        }
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                adj.entry(members[i]).or_default().push(members[j]);
                adj.entry(members[j]).or_default().push(members[i]);
            }
        }
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut components = 0usize;
    for p in persons {
        let start = p.id.as_str();
        if !visited.insert(start) {
            continue;
        }
        components += 1;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(curr) = queue.pop_front() {
            if let Some(neighbors) = adj.get(curr) {
                for &nxt in neighbors {
                    if visited.insert(nxt) {
                        queue.push_back(nxt);
                    }
                }
            }
        }
    }
    components
}

#[tokio::test]
async fn test_merge_gramps_and_myheritage_real_files_both_directions() {
    use genealogy_backend::services::{
        gedcom::import_gedcom_into_store, privacy::GenealogyPrivacyStore,
    };

    let gramps_bytes =
        std::fs::read("../test-data/gramps-export/gramps-web-export-20261026130902.ged")
            .expect("gramps GEDCOM fixture should exist");
    let mh_bytes = std::fs::read("../test-data/6na71k_97365268f67d1031ya027a_A.ged")
        .expect("MyHeritage GEDCOM fixture should exist");

    // Direction 1: Gramps -> MyHeritage
    let mut store_gm = GenealogyPrivacyStore::empty();
    let r1 = import_gedcom_into_store(&mut store_gm, &gramps_bytes, "merge", None, 2026, 110)
        .expect("Gramps import should succeed");
    assert_eq!(r1.imported_individuals, 284);
    assert_eq!(r1.imported_families, 84);

    let r2 = import_gedcom_into_store(&mut store_gm, &mh_bytes, "merge", None, 2026, 110)
        .expect("MyHeritage merge should succeed");
    assert_eq!(r2.merged_individuals, 261);
    assert_eq!(r2.imported_individuals, 110);
    assert_eq!(r2.merged_families, 80);
    assert_eq!(r2.imported_families, 39);
    assert_eq!(store_gm.persons.len(), 394);
    assert_eq!(store_gm.families.len(), 123);
    assert_eq!(
        count_connected_components(&store_gm.persons, &store_gm.families),
        1
    );

    // Direction 2: MyHeritage -> Gramps
    let mut store_mg = GenealogyPrivacyStore::empty();
    let r3 = import_gedcom_into_store(&mut store_mg, &mh_bytes, "merge", None, 2026, 110)
        .expect("MyHeritage import should succeed");
    assert_eq!(r3.imported_individuals, 371);
    assert_eq!(r3.imported_families, 119);

    let r4 = import_gedcom_into_store(&mut store_mg, &gramps_bytes, "merge", None, 2026, 110)
        .expect("Gramps merge should succeed");
    assert_eq!(r4.merged_individuals, 261);
    assert_eq!(r4.imported_individuals, 23);
    assert_eq!(r4.merged_families, 80);
    assert_eq!(r4.imported_families, 4);
    assert_eq!(store_mg.persons.len(), 394);
    assert_eq!(store_mg.families.len(), 123);
    assert_eq!(
        count_connected_components(&store_mg.persons, &store_mg.families),
        1
    );

    for store in [&store_gm, &store_mg] {
        for p in &store.persons {
            assert!(
                !p.primary_name.given_name.contains("Р’") && !p.primary_name.surname.contains("Р’"),
                "Person name must not contain Windows-1251 mojibake: {:?}",
                p.primary_name
            );
        }
        for f in &store.families {
            for c in &f.children {
                assert_ne!(f.partner1_id.as_deref(), Some(c.child_id.as_str()));
                assert_ne!(f.partner2_id.as_deref(), Some(c.child_id.as_str()));
            }
        }
    }
}

#[tokio::test]
async fn test_merge_all_three_real_files_any_order() {
    use genealogy_backend::services::{
        gedcom::import_gedcom_into_store, privacy::GenealogyPrivacyStore,
    };

    let gramps_bytes =
        std::fs::read("../test-data/gramps-export/gramps-web-export-20261026130902.ged")
            .expect("gramps GEDCOM fixture should exist");
    let mh_bytes = std::fs::read("../test-data/6na71k_97365268f67d1031ya027a_A.ged")
        .expect("MyHeritage GEDCOM fixture should exist");
    let filimon_bytes = std::fs::read("../test-data/filimon-levshits.ged")
        .expect("filimon-levshits GEDCOM fixture should exist");

    let orders: [(&str, [&[u8]; 3]); 3] = [
        (
            "Gramps -> MyHeritage -> Filimon",
            [&gramps_bytes, &mh_bytes, &filimon_bytes],
        ),
        (
            "MyHeritage -> Gramps -> Filimon",
            [&mh_bytes, &gramps_bytes, &filimon_bytes],
        ),
        (
            "Filimon -> Gramps -> MyHeritage",
            [&filimon_bytes, &gramps_bytes, &mh_bytes],
        ),
    ];

    for (label, files) in orders {
        let mut store = GenealogyPrivacyStore::empty();
        for (step, raw) in files.iter().enumerate() {
            let res = import_gedcom_into_store(&mut store, raw, "merge", None, 2026, 110)
                .unwrap_or_else(|e| panic!("{label} step {step} failed: {e:?}"));
            if step > 0 {
                assert!(
                    !res.matched_keys_preview.is_empty(),
                    "{label} step {step} should populate matched_keys_preview"
                );
            }
        }

        assert_eq!(
            store.persons.len(),
            407,
            "{label}: expected 407 total individuals"
        );
        assert_eq!(
            store.families.len(),
            129,
            "{label}: expected 129 total families"
        );
        assert_eq!(
            count_connected_components(&store.persons, &store.families),
            1,
            "{label}: expected 1 connected component"
        );

        let filimon = store
            .persons
            .iter()
            .find(|p| p.primary_name.given_name == "Филимон" && p.primary_name.surname == "Левшиц")
            .expect("Филимон Левшиц should exist");
        assert_eq!(
            filimon.primary_name.patronymic.as_deref(),
            Some("Кузьмич"),
            "{label}: Филимон patronymic"
        );
        assert_eq!(
            filimon.birth_sort.map(|bs| bs / 10_000),
            Some(1908),
            "{label}: Филимон birth year"
        );
        assert_eq!(
            filimon.death_sort.map(|ds| ds / 10_000),
            Some(1945),
            "{label}: Филимон death year"
        );

        let filimon_parent_fams: Vec<_> = store
            .families
            .iter()
            .filter(|f| f.children.iter().any(|c| c.child_id == filimon.id))
            .collect();
        assert_eq!(
            filimon_parent_fams.len(),
            1,
            "{label}: Филимон should have 1 parent family (F2)"
        );

        let filimon_spouse_fams: Vec<_> = store
            .families
            .iter()
            .filter(|f| {
                f.partner1_id.as_deref() == Some(filimon.id.as_str())
                    || f.partner2_id.as_deref() == Some(filimon.id.as_str())
            })
            .collect();
        assert_eq!(
            filimon_spouse_fams.len(),
            1,
            "{label}: Филимон should have 1 spouse family"
        );
        assert_eq!(
            filimon_spouse_fams[0].children.len(),
            5,
            "{label}: Филимон + Анна family should have 5 children"
        );

        let antonina_matches: Vec<_> = store
            .persons
            .iter()
            .filter(|p| {
                p.primary_name.given_name == "Антонина"
                    && p.birth_sort.map(|bs| bs / 10_000) == Some(1935)
            })
            .collect();
        assert_eq!(
            antonina_matches.len(),
            1,
            "{label}: Антонина (b. 1935) must not be duplicated"
        );
        assert_eq!(
            antonina_matches[0].birth_surname.as_deref(),
            Some("Швед"),
            "{label}: Антонина birth_surname should be Швед"
        );
        assert_eq!(
            antonina_matches[0].primary_name.surname, "Левшиц",
            "{label}: Антонина married surname should be Левшиц"
        );
    }
}
