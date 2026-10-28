use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use genealogy_backend::{
    config::AppConfig,
    routes::{
        create_router,
        rbac::{
            EventListResponse, MediaArchiveResponse, NoteListResponse, PersonProfileResponseDto,
            PlacesDirectoryResponse,
        },
    },
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        gedcom::{GedcomAnalysisReport, GedcomImportResult},
        privacy::TreePersonNodeDto,
    },
    state::AppState,
};
use serde_json::json;
use tower::ServiceExt;

async fn setup_test_env() -> (axum::Router, String, String, String) {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "GEDCOM Test Archive".to_string(),
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
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let admin_id = uuid::Uuid::parse_str(&admin_setup.user.id).unwrap();

    let create_role = |role: &str, email: &str| {
        let state_clone = state.clone();
        let role_str = role.to_string();
        let email_str = email.to_string();
        async move {
            let inv = AuthService::create_invitation(
                &state_clone,
                Some(admin_id),
                CreateInviteRequest {
                    role: role_str.clone(),
                    email: Some(email_str.clone()),
                    invitee_name: None,
                    person_id: None,
                },
            )
            .await
            .unwrap();
            let (reg, _) = AuthService::register_with_invitation(
                &state_clone,
                InviteRegisterRequest {
                    invite_token: inv.token,
                    first_name: "Relative".to_string(),
                    last_name: role_str,
                    email: Some(email_str),
                    passkey: None,
                    password: Some("UserStrongPass123!".to_string()),
                },
                None,
                None,
            )
            .await
            .unwrap();
            format!("Bearer {}", reg.session_token)
        }
    };

    let editor_auth = create_role("editor", "editor@levshits.family").await;
    let viewer_auth = create_role("viewer", "viewer@levshits.family").await;
    let app = create_router(state);

    (app, admin_auth, editor_auth, viewer_auth)
}

#[tokio::test]
async fn test_gramps_web_gedcom_analyze_import_and_round_trip() {
    let (app, admin_auth, _editor_auth, _viewer_auth) = setup_test_env().await;

    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-data/gramps-export/gramps-web-export-20261026130902.ged");
    let gedcom_content = std::fs::read_to_string(&fixture_path)
        .expect("Failed to read Gramps Web 6.0.1 GEDCOM fixture");

    // 1. Pre-import analysis (`POST /api/v1/gedcom/analyze`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/analyze")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "data": gedcom_content,
                        "file_name": "gramps-web-export-20261026130902.ged"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 256 * 1024).await.unwrap();
    let analysis: GedcomAnalysisReport = serde_json::from_slice(&body).unwrap();

    assert_eq!(analysis.total_individuals, 284);
    assert_eq!(analysis.total_families, 84);
    assert_eq!(analysis.total_media, 129);
    assert_eq!(analysis.missing_media_count, 129);
    assert_eq!(analysis.detected_software, "Gramps 6.0.1");
    assert_eq!(analysis.gedcom_version, "5.5.1");
    assert_eq!(analysis.character_encoding, "UTF-8");
    assert_eq!(analysis.file_name, "gramps-web-export-20261026130902.ged");
    assert!(analysis.file_size_bytes > 0);
    assert!(analysis.dialect_summary.coordinates_normalized > 0);
    assert!(analysis.dialect_summary.uids_preserved > 0);
    assert!(analysis.dialect_summary.vendor_upd_filtered > 0);
    assert!(analysis.dialect_summary.married_names_resolved > 0);
    assert_eq!(analysis.dialect_summary.parental_asso_deduplicated, 4);
    assert_eq!(analysis.dialect_summary.missing_media_stubs, 129);

    assert_eq!(analysis.individuals_count, 284);
    assert_eq!(analysis.families_count, 84);
    assert_eq!(analysis.media_count, 129);
    assert_eq!(analysis.media_missing_count, 129);
    assert!(analysis.can_import);
    assert_eq!(analysis.dialect.source_system, "Gramps");
    assert_eq!(analysis.dialect.source_version.as_deref(), Some("6.0.1"));
    assert_eq!(analysis.dialect.gedcom_version, "5.5.1");
    assert_eq!(analysis.dialect.charset, "UTF-8");
    assert!(analysis.dialect.normalized_coordinates_count > 0);
    assert!(analysis.dialect.preserved_uids_count > 0);
    assert!(analysis.dialect.preserved_rins_count > 0);
    assert!(analysis.dialect.filtered_upd_timestamps_count > 0);
    assert_eq!(analysis.dialect.deduplicated_parental_associations_count, 4);
    assert_eq!(analysis.dialect.stripped_media_paths_count, 129);
    assert!(analysis.dialect.marriage_licenses_count >= 1);
    assert!(analysis.dialect.deceased_flag_only_count > 0);

    // 2. Execute import (`POST /api/v1/gedcom/import`, mode = "new")
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
                        "data": gedcom_content,
                        "file_name": "gramps-web-export-20261026130902.ged",
                        "mode": "new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 256 * 1024).await.unwrap();
    let import_res: GedcomImportResult = serde_json::from_slice(&body).unwrap();

    assert_eq!(import_res.imported_individuals, 284);
    assert_eq!(import_res.imported_families, 84);
    assert_eq!(import_res.imported_media, 129);
    assert_eq!(import_res.missing_media_count, 129);
    assert!(import_res.snapshot_id.is_some());
    assert!(import_res.prompt_home_person);
    assert_eq!(import_res.individuals_imported, 284);
    assert_eq!(import_res.families_imported, 84);
    assert_eq!(import_res.media_imported, 129);
    assert_eq!(import_res.media_missing_count, 129);

    // 3. Verify Dialect #1: Multiple `1 NAME` blocks (`birth` + `married`) on `@I0001@` (Виктория Багрей / Левшиц)
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
    assert_eq!(p_i0001.primary_name.given_name, "Виктория");
    assert_eq!(p_i0001.primary_name.surname, "Левшиц");
    assert_eq!(p_i0001.birth_surname.as_deref(), Some("Багрей"));
    assert!(p_i0001
        .alt_names
        .iter()
        .any(|a| a.name_type == "married" && a.surname.as_deref() == Some("Левшиц")));

    // 4. Verify Dialect #2 & #3: Directional coordinates (`N52.31724`, `E26.86921`) and place deduplication
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/places")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 256 * 1024).await.unwrap();
    let places_res: PlacesDirectoryResponse = serde_json::from_slice(&b).unwrap();
    let kupovtsy = places_res
        .items
        .iter()
        .find(|p| p.name.contains("Куповцы"))
        .expect("Expected д. Куповцы place record");
    assert!((kupovtsy.latitude.unwrap() - 52.31724).abs() < 1e-5);
    assert!((kupovtsy.longitude.unwrap() - 26.86921).abs() < 1e-5);

    // 5. Verify Dialect #4: Media server path stripping (`/root/gramps/...` -> basename + `is_missing: true`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/media?limit=200")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 512 * 1024).await.unwrap();
    let media_res: MediaArchiveResponse = serde_json::from_slice(&b).unwrap();
    assert_eq!(media_res.total, 129);
    let o0000 = media_res
        .items
        .iter()
        .find(|m| m.gid == "O0000")
        .expect("Expected O0000 media stub");
    assert!(o0000.is_missing);
    assert_eq!(o0000.title, "Виктор Левшиц");
    assert_eq!(
        o0000.original_file_name.as_deref(),
        Some("000085_971532825e5c5dtfb56015_Y.jpg")
    );
    assert_eq!(
        o0000.external_path.as_deref(),
        Some("/root/gramps/000085_971532825e5c5dtfb56015_Y.jpg")
    );

    // 6. Verify Dialect #5: `_UID` and `FACT MH:...` (`TYPE RIN`) preserved in `external_ids`
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0010")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let p_i0010: TreePersonNodeDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(
        p_i0010.external_ids.get("uid").map(String::as_str),
        Some("CF665D02-7132-46C3-868D-7BF37FD03AC5")
    );
    assert_eq!(
        p_i0010.external_ids.get("mh_rin").map(String::as_str),
        Some("MH:I10")
    );

    // 7. Verify Dialect #7: Redundant parental `ASSO` on `@I0000@` deduplicated (only 1 parent family `@F0001@`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0000/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let prof_i0000: PersonProfileResponseDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(prof_i0000.families.parent_families.len(), 1);

    // 8. Verify Dialect #8: `1 MARL` on `@F0001@` imported as `marriage_license`
    let f0001_id = &prof_i0000.families.parent_families[0].family_id;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/events?family_id={f0001_id}"))
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let f0001_events: EventListResponse = serde_json::from_slice(&b).unwrap();
    assert!(f0001_events
        .items
        .iter()
        .any(|e| e.event_type == "marriage_license"
            && e.gedcom_date.as_deref() == Some("1 AUG 2020")));

    // 9. Verify Dialect #9: `1 DEAT Y` on `@I0005@` marks person deceased, and `CAUS`+`AGE` on `@I500012@`
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0005")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let p_i0005: TreePersonNodeDto = serde_json::from_slice(&b).unwrap();
    assert!(!p_i0005.is_living);
    assert!(p_i0005.has_death_event);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I500012/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let prof_i500012: PersonProfileResponseDto = serde_json::from_slice(&b).unwrap();
    let death_ev = prof_i500012
        .events
        .iter()
        .find(|e| e.event_type == "death")
        .expect("Expected death event on I500012");
    assert_eq!(
        death_ev.description.as_deref(),
        Some("общая сердечная недостаточность (возраст: 72)")
    );

    // 10. Verify Dialect #10: `@@` unescaped in `RESI` -> `EMAIL` on `@I0012@`
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/persons/I0012/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let prof_i0012: PersonProfileResponseDto = serde_json::from_slice(&b).unwrap();
    let resi_ev = prof_i0012
        .events
        .iter()
        .find(|e| e.event_type == "residence")
        .expect("Expected residence event on I0012");
    assert_eq!(resi_ev.description.as_deref(), Some("levshits@yahoo.com"));

    // 11. Round-trip export (`POST /api/v1/gedcom/export`) and re-import into a clean instance
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/export")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "format": "gedcom70",
                        "include_media": true,
                        "redact_living": false,
                        "include_private_notes": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let exported_bytes = to_bytes(res.into_body(), 2 * 1024 * 1024).await.unwrap();
    let exported_gedcom = String::from_utf8(exported_bytes.to_vec()).unwrap();
    assert!(exported_gedcom.starts_with("0 HEAD\n"));
    assert!(exported_gedcom.contains("2 VERS 7.0\n"));
    assert!(exported_gedcom.ends_with("0 TRLR\n"));

    let (clean_app, clean_admin_auth, _, _) = setup_test_env().await;
    let res = clean_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/import")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &clean_admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "data": exported_gedcom,
                        "mode": "new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 256 * 1024).await.unwrap();
    let reimport_res: GedcomImportResult = serde_json::from_slice(&b).unwrap();

    assert_eq!(
        reimport_res.individuals_imported,
        import_res.individuals_imported
    );
    assert_eq!(reimport_res.families_imported, import_res.families_imported);
    assert_eq!(reimport_res.media_imported, import_res.media_imported);
    assert_eq!(reimport_res.events_imported, import_res.events_imported);
    assert_eq!(reimport_res.places_imported, import_res.places_imported);
    assert_eq!(reimport_res.notes_imported, import_res.notes_imported);
    assert_eq!(
        reimport_res.dialect.preserved_uids_count,
        import_res.dialect.preserved_uids_count
    );
    assert_eq!(
        reimport_res.dialect.preserved_rins_count,
        import_res.dialect.preserved_rins_count
    );
}

#[tokio::test]
async fn test_gedcom_export_redaction_and_encoding_dialects() {
    let (app, admin_auth, editor_auth, viewer_auth) = setup_test_env().await;

    // 1. RBAC isolation: Viewer, Editor, and Anonymous cannot access /gedcom/analyze, /gedcom/import, or POST /gedcom/export
    for uri in [
        "/api/v1/gedcom/analyze",
        "/api/v1/gedcom/import",
        "/api/v1/gedcom/export",
    ] {
        for auth in [&viewer_auth, &editor_auth] {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri(uri)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header(header::AUTHORIZATION, auth)
                        .header("x-requested-with", "genealogy")
                        .body(Body::from(r#"{"data":"0 HEAD\n0 TRLR\n"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        }

        let anon_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(r#"{"data":"0 HEAD\n0 TRLR\n"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anon_res.status(), StatusCode::UNAUTHORIZED);
    }

    // 2. Import synthetic GEDCOM with single-parent ASSO, social ASSO (Крёстный), living person, deceased person, and private person
    let synthetic_ged = "\
0 HEAD
1 SOUR MyHeritage
2 VERS 8.0
1 GEDC
2 VERS 5.5.1
1 CHAR UTF-8
0 @I0001@ INDI
1 NAME Иван Петрович /Соколов/
2 GIVN Иван Петрович
2 SURN Соколов
1 SEX M
1 BIRT
2 DATE 10 MAY 1995
2 PLAC Москва, Россия
3 MAP
4 LATI N55.7558
4 LONG E37.6173
1 ASSO @I0002@
2 RELA Отец
1 ASSO @I0003@
2 RELA Крёстный
1 NOTE Личная заметка живущего человека
0 @I0002@ INDI
1 NAME Пётр Ильич /Соколов/
2 GIVN Пётр Ильич
2 SURN Соколов
1 SEX M
1 BIRT
2 DATE 1920
1 DEAT
2 DATE 1990
0 @I0003@ INDI
1 NAME Николай /Морозов/
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
                        "data": synthetic_ged,
                        "mode": "new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let imp: GedcomImportResult = serde_json::from_slice(&b).unwrap();
    assert_eq!(imp.individuals_imported, 3);
    assert_eq!(imp.dialect.synthesized_single_parent_families_count, 1);
    assert_eq!(imp.dialect.social_associations_notes_count, 1);

    // Check synthesized single-parent family and social ASSO note on I0001
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/notes?entity_type=individual")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let notes_list: NoteListResponse = serde_json::from_slice(&b).unwrap();
    assert!(notes_list
        .items
        .iter()
        .any(|n| n.body.contains("Связь (Крёстный): @I0003@ Николай Морозов")));

    // 3. Export with `redact_living: true` -> living person @I0001@ masked to "И. //", birth date and notes omitted, deceased @I0002@ unredacted
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/export")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "format": "gedcom551",
                        "redact_living": true,
                        "include_private_notes": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let redacted_bytes = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let redacted_ged = String::from_utf8(redacted_bytes.to_vec()).unwrap();

    assert!(redacted_ged.contains("1 NAME И. //\n"));
    assert!(!redacted_ged.contains("Иван Петрович"));
    assert!(!redacted_ged.contains("10 MAY 1995"));
    assert!(!redacted_ged.contains("Личная заметка живущего человека"));
    assert!(redacted_ged.contains("Пётр Ильич /Соколов/"));

    // 4. Replace mode confirmation guard (`ЗАМЕНИТЬ` / `REPLACE`)
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
                        "data": synthetic_ged,
                        "mode": "replace",
                        "confirm_text": "wrong"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 5. ANSEL encoding support (`1 CHAR ANSEL` with 0xE8 umlaut combining mark before 'o' -> 'ö')
    let ansel_bytes: &[u8] = b"0 HEAD\n1 CHAR ANSEL\n0 @I0001@ INDI\n1 NAME J\xE8org /M\xE8uller/\n1 SEX M\n1 DEAT Y\n0 TRLR\n";
    let ansel_b64 = BASE64_STANDARD.encode(ansel_bytes);

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
                        "data": ansel_b64,
                        "mode": "replace",
                        "confirm_text": "ЗАМЕНИТЬ"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

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
    let ansel_person: TreePersonNodeDto = serde_json::from_slice(&b).unwrap();
    assert_eq!(ansel_person.primary_name.given_name, "Jörg");
    assert_eq!(ansel_person.primary_name.surname, "Müller");

    // 6. Verify `mode: "clean_slate"` + `confirm_text: "CLEAN_SLATE"`, `version: "5.5.1"` export, and missing media stub reconciliation
    let stub_ged = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n0 @O0000@ OBJE\n1 FILE /root/gramps/000085_971532825e5c5dtfb56015_Y.jpg\n2 FORM jpg\n2 TITL 000085_971532825e5c5dtfb56015_Y\n0 @I0001@ INDI\n1 NAME Иван /Иванов/\n1 SEX M\n1 DEAT Y\n1 OBJE @O0000@\n0 TRLR\n";
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
                        "data": stub_ged,
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
    assert_eq!(cs_res.imported_individuals, 1);
    assert_eq!(cs_res.imported_media, 1);
    assert_eq!(cs_res.missing_media_count, 1);

    // Export with `version: "5.5.1"` via POST and GET
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/gedcom/export")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "version": "5.5.1",
                        "redact_living": true,
                        "include_media": true,
                        "include_private_notes": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let exported_551 = String::from_utf8(b.to_vec()).unwrap();
    assert!(exported_551.contains("2 VERS 5.5.1"));

    // Reconcile the missing `@O0000@` media stub by uploading matching `file_name` to `POST /api/v1/media`
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "file_name": "000085_971532825e5c5dtfb56015_Y.jpg",
                        "mime_type": "image/jpeg"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let b = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reconciled_asset: genealogy_backend::services::privacy::MediaAssetSummaryDto =
        serde_json::from_slice(&b).unwrap();
    assert_eq!(reconciled_asset.gid, "O0000");
    assert!(!reconciled_asset.is_missing);
    assert!(reconciled_asset.file_size_bytes.unwrap_or(0) > 0);
}
