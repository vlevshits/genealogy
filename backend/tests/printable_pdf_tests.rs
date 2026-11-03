use std::collections::HashSet;

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use tower::ServiceExt;

use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        pdf::{
            layout::compute_tree_layout,
            tiling::{plan_tiling, resolve_sheet_dimension},
            PdfExportPreviewInfo, PdfExportRequest,
        },
        privacy::{
            FamilyChildDto, FamilyDto, GenealogyPrivacyStore, PersonNameDto, TreePersonNodeDto,
            VitalEventSummaryDto,
        },
    },
    state::AppState,
};

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        tree_title: "Printable PDF Test Archive".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    }
}

async fn setup_app_and_tokens() -> (axum::Router, AppState, String, String, String) {
    let state = AppState::new_demo(None, test_config());

    // 1. Initial Admin setup
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Printable PDF Studio Tree".to_string()),
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let admin_id = uuid::Uuid::parse_str(&admin_setup.user.id).unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);

    // 2. Editor user
    let editor_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            email: Some("editor@levshits.family".to_string()),
            invitee_name: Some("Elena Editor".to_string()),
            role: "editor".to_string(),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (editor_reg, _) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: editor_inv.token,
            first_name: "Elena".to_string(),
            last_name: "Editor".to_string(),
            email: Some("editor@levshits.family".to_string()),
            passkey: None,
            password: Some("EditorPass123!".to_string()),
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let editor_auth = format!("Bearer {}", editor_reg.session_token);

    // 3. Viewer user
    let viewer_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            email: Some("viewer@levshits.family".to_string()),
            invitee_name: Some("Vera Viewer".to_string()),
            role: "viewer".to_string(),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (viewer_reg, _) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: viewer_inv.token,
            first_name: "Vera".to_string(),
            last_name: "Viewer".to_string(),
            email: Some("viewer@levshits.family".to_string()),
            passkey: None,
            password: Some("ViewerPass123!".to_string()),
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let viewer_auth = format!("Bearer {}", viewer_reg.session_token);

    let app = create_router(state.clone());
    (app, state, admin_auth, editor_auth, viewer_auth)
}

#[tokio::test]
async fn test_admin_can_generate_single_poster_pdf() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        format: Some("poster".to_string()),
        target_sheet_size: Some("A2".to_string()),
        theme: Some("modern".to_string()),
        view_mode: Some("pedigree".to_string()),
        title: Some("Poster Test Tree".to_string()),
        ..Default::default()
    };

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/pdf")
    );
    let disposition = res
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        disposition.contains("family-tree-poster.pdf"),
        "Expected poster filename in disposition, got: {disposition}"
    );

    let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF file must start with '%PDF-' magic bytes"
    );
}

#[tokio::test]
async fn test_admin_can_generate_tiled_multi_page_pdf_with_cut_marks_and_overlap() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        format: Some("tiled".to_string()),
        target_sheet_size: Some("A4".to_string()),
        overlap_mm: Some(10.0),
        show_crop_marks: Some(true),
        include_assembly_guide: Some(true),
        theme: Some("modern".to_string()),
        view_mode: Some("hourglass".to_string()),
        ..Default::default()
    };

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/pdf")
    );
    let disposition = res
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        disposition.contains("family-tree-tiled.pdf"),
        "Expected tiled filename in disposition, got: {disposition}"
    );

    let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF file must start with '%PDF-' magic bytes"
    );
    assert!(
        pdf_bytes.len() > 1000,
        "PDF content should contain binary data"
    );
}

#[tokio::test]
async fn test_admin_can_get_preview_calculation_info() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        format: Some("tiled".to_string()),
        target_sheet_size: Some("A4".to_string()),
        overlap_mm: Some(10.0),
        include_assembly_guide: Some(true),
        view_mode: Some("pedigree".to_string()),
        generations: Some(4),
        ..Default::default()
    };

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let preview: PdfExportPreviewInfo = serde_json::from_slice(&body).unwrap();

    assert!(preview.total_sheets >= 1);
    assert!(preview.grid_rows >= 1);
    assert!(preview.grid_cols >= 1);
    assert!(preview.canvas_width_mm > 0.0);
    assert!(preview.canvas_height_mm > 0.0);
    assert_eq!(preview.target_sheet_size, "A4");
    assert!(!preview.summary_text.is_empty());
}

#[tokio::test]
async fn test_cyrillic_names_render_cleanly_without_error() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        format: Some("tiled".to_string()),
        target_sheet_size: Some("A4".to_string()),
        theme: Some("heritage".to_string()),
        view_mode: Some("pedigree".to_string()),
        title: Some("Родословное древо рода Левшиц (1912—2026)".to_string()),
        ..Default::default()
    };

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn test_non_admin_callers_receive_403_or_401() {
    let (app, _state, _admin_auth, editor_auth, viewer_auth) = setup_app_and_tokens().await;

    let payload = serde_json::to_vec(&PdfExportRequest {
        format: Some("tiled".to_string()),
        ..Default::default()
    })
    .unwrap();

    // 1. Unauthenticated / Anonymous caller -> 401 Unauthorized
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Viewer caller -> 403 Forbidden
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &viewer_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &viewer_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 3. Editor caller -> 403 Forbidden
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &editor_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &editor_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_living_person_redaction_masks_dates() {
    let (_app, state, _admin_auth, _, _) = setup_app_and_tokens().await;
    let store = state.privacy_store.read().await;

    // In demo store:
    // I0042 is Elena Levshits (living, born 1994)
    // I0001 is Mikhail Levshits (deceased, born 1912, died 1988)

    // With redact_living = true
    let layout_redacted = compute_tree_layout(
        &store,
        "I0042",
        "hourglass",
        4,
        true, // redact_living = true
        true,
        None,
        None,
        true,
        false,
        true,
        "Redaction Test Tree",
        None,
    )
    .unwrap();

    let elena_card = layout_redacted
        .cards
        .iter()
        .find(|c| c.gid == "I0042")
        .expect("Elena (I0042) must be present in layout");

    assert!(
        elena_card.is_living,
        "Elena must be marked as living in card"
    );
    assert!(
        !elena_card.lifespan.contains("1994"),
        "Redacted living person must not reveal birth year 1994, got: '{}'",
        elena_card.lifespan
    );
    assert_eq!(
        elena_card.lifespan, "Living",
        "Living person lifespan must be masked to 'Living'"
    );

    // With redact_living = false
    let layout_unredacted = compute_tree_layout(
        &store,
        "I0042",
        "hourglass",
        4,
        false, // redact_living = false
        true,
        None,
        None,
        true,
        false,
        true,
        "Unredacted Test Tree",
        None,
    )
    .unwrap();

    let elena_unredacted = layout_unredacted
        .cards
        .iter()
        .find(|c| c.gid == "I0042")
        .expect("Elena (I0042) must be present in unredacted layout");

    assert!(
        elena_unredacted.lifespan.contains("1994"),
        "Unredacted living person must show birth year, got: '{}'",
        elena_unredacted.lifespan
    );
}

#[tokio::test]
async fn test_single_page_pdf_generation_all_sizes_a0_a1_a2_a3() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let sizes = ["A0", "A1", "A2", "A3"];
    for size in sizes {
        let payload = PdfExportRequest {
            format: Some("poster".to_string()),
            target_sheet_size: Some(size.to_string()),
            theme: Some("modern".to_string()),
            view_mode: Some("pedigree".to_string()),
            title: Some(format!("Poster Test {}", size)),
            ..Default::default()
        };

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/export/pdf")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, &admin_auth)
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Failed generating poster for size {}",
            size
        );
        assert_eq!(
            res.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/pdf"),
            "Size {} must return application/pdf",
            size
        );

        let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "Size {} PDF must start with %PDF-",
            size
        );
        assert!(
            pdf_bytes.len() > 1000,
            "Size {} PDF content should be substantial vector data, got {} bytes",
            size,
            pdf_bytes.len()
        );
    }
}

#[test]
fn test_multi_page_tiling_math_cleanly_partitions_grid_without_dropped_content() {
    // 1. A4 paper tiling test (297x210 mm)
    let a4_dim = resolve_sheet_dimension("A4");
    assert_eq!(a4_dim.width_mm, 297.0);
    assert_eq!(a4_dim.height_mm, 210.0);

    let canvas_w = 800.0;
    let canvas_h = 500.0;
    let overlap_mm = 10.0;

    let plan = plan_tiling(
        canvas_w, canvas_h, "A4", overlap_mm, false, // not single poster
        true,  // include guide
        true,  // show crop marks
    );

    assert!(
        plan.grid_cols >= 2,
        "Expected at least 2 columns for 800mm canvas"
    );
    assert!(
        plan.grid_rows >= 2,
        "Expected at least 2 rows for 500mm canvas"
    );
    assert_eq!(
        plan.total_sheets,
        plan.grid_cols * plan.grid_rows + 1,
        "Total sheets should include guide sheet"
    );

    // Verify all tiles partition the canvas without gaps
    let usable_w = 297.0 - 20.0; // 277mm
    let usable_h = 210.0 - 20.0; // 190mm
    let step_w = usable_w - overlap_mm; // 267mm
    let step_h = usable_h - overlap_mm; // 180mm

    assert_eq!(plan.tiles.len(), (plan.grid_rows * plan.grid_cols) as usize);

    for tile in &plan.tiles {
        assert!(
            tile.window_width >= usable_w - 0.01,
            "Window width must match usable sheet width"
        );
        assert!(
            tile.window_height >= usable_h - 0.01,
            "Window height must match usable sheet height"
        );

        let expected_x = (tile.col as f64) * step_w;
        let expected_y = (tile.row as f64) * step_h;
        assert!(
            (tile.canvas_window_x - expected_x).abs() < 0.01,
            "Tile x window coordinate mismatch"
        );
        assert!(
            (tile.canvas_window_y - expected_y).abs() < 0.01,
            "Tile y window coordinate mismatch"
        );

        assert!(
            tile.footer_text.contains(&format!(
                "Sheet {} of {}",
                tile.sheet_index,
                plan.grid_rows * plan.grid_cols
            )),
            "Tile footer text must include sheet number and total: {}",
            tile.footer_text
        );
        assert!(
            tile.footer_text
                .contains(&format!("[Row {}, Column {}]", tile.row + 1, tile.col + 1)),
            "Tile footer text must include row and column: {}",
            tile.footer_text
        );
    }

    // Check last column and last row cover or exceed the canvas dimensions
    let last_col_max_x = ((plan.grid_cols - 1) as f64) * step_w + usable_w;
    let last_row_max_y = ((plan.grid_rows - 1) as f64) * step_h + usable_h;
    assert!(
        last_col_max_x >= canvas_w,
        "Tiles grid must completely cover canvas width ({} >= {})",
        last_col_max_x,
        canvas_w
    );
    assert!(
        last_row_max_y >= canvas_h,
        "Tiles grid must completely cover canvas height ({} >= {})",
        last_row_max_y,
        canvas_h
    );

    // 2. Letter paper tiling test (279.4 x 215.9 mm)
    let letter_dim = resolve_sheet_dimension("Letter");
    assert_eq!(letter_dim.width_mm, 279.4);
    assert_eq!(letter_dim.height_mm, 215.9);

    let plan_letter = plan_tiling(
        canvas_w, canvas_h, "Letter", overlap_mm, false, false, // no guide
        true,
    );
    assert_eq!(
        plan_letter.total_sheets,
        plan_letter.grid_cols * plan_letter.grid_rows,
        "Without guide, total sheets equals tiles count"
    );
}

#[test]
fn test_configurable_overlap_margins_5mm_to_20mm() {
    let canvas_w = 600.0;
    let canvas_h = 400.0;

    let plan_5mm = plan_tiling(canvas_w, canvas_h, "A4", 5.0, false, false, true);
    let plan_10mm = plan_tiling(canvas_w, canvas_h, "A4", 10.0, false, false, true);
    let plan_20mm = plan_tiling(canvas_w, canvas_h, "A4", 20.0, false, false, true);

    assert_eq!(plan_5mm.overlap_mm, 5.0);
    assert_eq!(plan_10mm.overlap_mm, 10.0);
    assert_eq!(plan_20mm.overlap_mm, 20.0);

    // Larger overlap means smaller step, which may require equal or more columns/rows
    assert!(plan_20mm.grid_cols >= plan_5mm.grid_cols);
}

#[tokio::test]
async fn test_both_modern_and_heritage_themes_generate_valid_pdf() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    for theme_name in &["modern", "heritage"] {
        let payload = PdfExportRequest {
            format: Some("tiled".to_string()),
            target_sheet_size: Some("A4".to_string()),
            theme: Some(theme_name.to_string()),
            view_mode: Some("pedigree".to_string()),
            generations: Some(3),
            ..Default::default()
        };

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/export/pdf")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, &admin_auth)
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "Theme {} must produce valid PDF",
            theme_name
        );
    }
}

#[tokio::test]
async fn test_photo_inclusion_and_living_person_photo_masking() {
    let (_app, state, _admin_auth, _, _) = setup_app_and_tokens().await;
    let store = state.privacy_store.read().await;

    // Test with redact_living = true, show_photos = true
    let layout_redacted = compute_tree_layout(
        &store,
        "I0042",
        "hourglass",
        4,
        true, // redact_living = true
        true, // show_photos = true
        None,
        None,
        true,
        false,
        true,
        "Photo Test Tree",
        None,
    )
    .unwrap();

    let elena_redacted = layout_redacted
        .cards
        .iter()
        .find(|c| c.gid == "I0042")
        .expect("Elena must be in tree");
    assert!(elena_redacted.is_living);
    assert!(
        elena_redacted.avatar_bytes.is_none(),
        "Living person photo MUST be masked when redact_living is true"
    );

    // Test with show_photos = false
    let layout_no_photos = compute_tree_layout(
        &store,
        "I0042",
        "hourglass",
        4,
        false, // redact_living = false
        false, // show_photos = false
        None,
        None,
        true,
        false,
        true,
        "No Photos Tree",
        None,
    )
    .unwrap();

    for card in &layout_no_photos.cards {
        assert!(
            card.avatar_bytes.is_none(),
            "When show_photos is false, no card should contain avatar bytes"
        );
    }
}

fn make_test_person(
    id: &str,
    gid: &str,
    given: &str,
    surname: &str,
    sex: &str,
    birth_year: i32,
) -> TreePersonNodeDto {
    TreePersonNodeDto {
        id: id.to_string(),
        gid: gid.to_string(),
        sex: sex.to_string(),
        is_living: false,
        living_override: None,
        is_private: false,
        primary_name: PersonNameDto {
            given_name: given.to_string(),
            patronymic: None,
            surname: surname.to_string(),
        },
        birth_surname: None,
        nickname: None,
        name_prefix: None,
        name_suffix: None,
        biography: None,
        birth: Some(VitalEventSummaryDto {
            date: Some(format!("{birth_year}")),
            place: None,
        }),
        death: None,
        avatar_url: None,
        avatar_media_id: None,
        avatar_crop: None,
        notes: None,
        birth_sort: Some(birth_year * 10000),
        death_sort: None,
        has_death_event: false,
        alt_names: vec![],
        external_ids: std::collections::HashMap::new(),
        version: 1,
        tx_id: None,
        generation: None,
        is_direct_lineage: None,
    }
}

fn make_test_family(
    id: &str,
    gid: &str,
    p1: Option<&str>,
    p2: Option<&str>,
    children: &[&str],
) -> FamilyDto {
    FamilyDto {
        id: id.to_string(),
        gid: gid.to_string(),
        partner1_id: p1.map(ToString::to_string),
        partner2_id: p2.map(ToString::to_string),
        relationship_type: "married".to_string(),
        is_private: false,
        children: children
            .iter()
            .enumerate()
            .map(|(idx, cid)| FamilyChildDto {
                child_id: cid.to_string(),
                sort_order: idx as i32,
                rel_to_partner1: "birth".to_string(),
                rel_to_partner2: "birth".to_string(),
            })
            .collect(),
        external_ids: std::collections::HashMap::new(),
        version: 1,
        marriage_date: None,
        marriage_place: None,
    }
}

fn create_extended_test_store() -> GenealogyPrivacyStore {
    let mut store = GenealogyPrivacyStore::empty();

    // 1. Grandparents
    let p_gf = make_test_person("p_gf", "I001", "Lev", "Levshits", "M", 1880);
    let p_gm = make_test_person("p_gm", "I002", "Anna", "Levshits", "F", 1885);

    // 2. Parents, Aunt, Uncle
    let p_father = make_test_person("p_father", "I003", "Isaak", "Levshits", "M", 1910);
    let p_mother = make_test_person("p_mother", "I004", "Sarra", "Levshits", "F", 1912);
    let p_aunt = make_test_person("p_aunt", "I005", "Rachel", "Levshits", "F", 1915);
    let p_uncle = make_test_person("p_uncle", "I006", "David", "Cohen", "M", 1914);

    // 3. Focal, Spouse, Sibling, First Cousin
    let p_focal = make_test_person("p_focal", "I_FOCAL", "Mikhail", "Levshits", "M", 1935);
    let p_spouse = make_test_person("p_spouse", "I_SPOUSE", "Maria", "Petrova", "F", 1938);
    let p_sibling = make_test_person("p_sibling", "I_SIBLING", "Yakov", "Levshits", "M", 1937);
    let p_cousin = make_test_person("p_cousin", "I_COUSIN", "Boris", "Cohen", "M", 1940);

    // 4. Children, Niece
    let p_child = make_test_person("p_child", "I_CHILD", "Dmitry", "Levshits", "M", 1960);
    let p_niece = make_test_person("p_niece", "I_NIECE", "Olga", "Levshits", "F", 1965);

    // 5. Grandchild
    let p_grandchild = make_test_person(
        "p_grandchild",
        "I_GRANDCHILD",
        "Elena",
        "Levshits",
        "F",
        1988,
    );

    // 6. Secondary Component (Tolstoy line)
    let p_sec1 = make_test_person("p_sec1", "I_SEC1", "Lev", "Tolstoy", "M", 1828);
    let p_sec2 = make_test_person("p_sec2", "I_SEC2", "Sophia", "Tolstaya", "F", 1844);
    let p_sec3 = make_test_person("p_sec3", "I_SEC3", "Ilya", "Tolstoy", "M", 1866);

    // 7. Singletons (0 parents, 0 spouses, 0 children)
    let p_sing1 = make_test_person("p_sing1", "I_SING1", "Hermit", "Petrov", "M", 1950);
    let p_sing2 = make_test_person("p_sing2", "I_SING2", "Lone", "Wanderer", "M", 1960);

    // Families
    let f_gp = make_test_family(
        "f_gp",
        "F001",
        Some("p_gf"),
        Some("p_gm"),
        &["p_father", "p_aunt"],
    );
    let f_parents = make_test_family(
        "f_parents",
        "F002",
        Some("p_father"),
        Some("p_mother"),
        &["p_focal", "p_sibling"],
    );
    let f_aunt = make_test_family(
        "f_aunt",
        "F003",
        Some("p_aunt"),
        Some("p_uncle"),
        &["p_cousin"],
    );
    let f_focal = make_test_family(
        "f_focal",
        "F004",
        Some("p_focal"),
        Some("p_spouse"),
        &["p_child"],
    );
    let f_sibling = make_test_family("f_sibling", "F005", Some("p_sibling"), None, &["p_niece"]);
    let f_child = make_test_family("f_child", "F006", Some("p_child"), None, &["p_grandchild"]);
    let f_sec = make_test_family("f_sec", "F007", Some("p_sec1"), Some("p_sec2"), &["p_sec3"]);

    store.persons = vec![
        p_gf,
        p_gm,
        p_father,
        p_mother,
        p_aunt,
        p_uncle,
        p_focal,
        p_spouse,
        p_sibling,
        p_cousin,
        p_child,
        p_niece,
        p_grandchild,
        p_sec1,
        p_sec2,
        p_sec3,
        p_sing1,
        p_sing2,
    ];
    store.families = vec![f_gp, f_parents, f_aunt, f_focal, f_sibling, f_child, f_sec];

    store
}

#[tokio::test]
async fn test_custom_title_subtitle_and_fallbacks() {
    let store = create_extended_test_store();

    // 1. Explicit custom title and subtitle provided
    let layout_custom = compute_tree_layout(
        &store,
        "I_FOCAL",
        "pedigree",
        4,
        false,
        true,
        Some("Custom Family Heritage"),
        Some("Commemorative Reunion Edition • Summer 2026"),
        true,
        false,
        true,
        "Archive Title",
        None,
    )
    .unwrap();

    assert_eq!(layout_custom.title, "Custom Family Heritage");
    assert_eq!(
        layout_custom.subtitle,
        "Commemorative Reunion Edition • Summer 2026"
    );

    // 2. Default fallback title and subtitle in pedigree mode
    let layout_default = compute_tree_layout(
        &store,
        "I_FOCAL",
        "pedigree",
        4,
        false,
        true,
        None,
        None,
        true,
        false,
        true,
        "Levshits Archive",
        None,
    )
    .unwrap();

    assert_eq!(layout_default.title, "Levshits Archive: Mikhail Levshits");
    assert!(layout_default
        .subtitle
        .starts_with("Layout: pedigree • Generations: 4"));
    assert!(layout_default.subtitle.contains("Persons:"));

    // 3. Fallback subtitle in all_nodes mode
    let layout_all = compute_tree_layout(
        &store,
        "I_FOCAL",
        "all_nodes",
        5,
        false,
        true,
        None,
        None,
        true,
        false,
        true,
        "Levshits Archive",
        None,
    )
    .unwrap();

    assert!(
        layout_all.subtitle.starts_with("Total Individuals:"),
        "Expected all_nodes subtitle to start with 'Total Individuals:', got: '{}'",
        layout_all.subtitle
    );
    assert_eq!(layout_all.total_persons, 18);
}

#[tokio::test]
async fn test_include_partner_and_ochre_marital_connectors() {
    let store = create_extended_test_store();

    // 1. With include_partner = true: spouse card is placed with ochre connector
    let layout_with_partner = compute_tree_layout(
        &store,
        "I_FOCAL",
        "descendant",
        4,
        false,
        true,
        None,
        None,
        true, // include_partner = true
        false,
        true,
        "Partner Test Tree",
        None,
    )
    .unwrap();

    let spouse_card = layout_with_partner
        .cards
        .iter()
        .find(|c| c.gid == "I_SPOUSE");
    assert!(
        spouse_card.is_some(),
        "Spouse must be present when include_partner is true"
    );

    let has_ochre_connector = layout_with_partner
        .connectors
        .iter()
        .any(|c| c.color.as_deref() == Some("#d97706"));
    assert!(
        has_ochre_connector,
        "Must render warm ochre (#d97706) marital connector for paired couple"
    );

    // 2. With include_partner = false: spouse card and ochre connector are omitted
    let layout_no_partner = compute_tree_layout(
        &store,
        "I_FOCAL",
        "descendant",
        4,
        false,
        true,
        None,
        None,
        false, // include_partner = false
        false,
        true,
        "Partner Test Tree",
        None,
    )
    .unwrap();

    let spouse_card_omitted = layout_no_partner.cards.iter().find(|c| c.gid == "I_SPOUSE");
    assert!(
        spouse_card_omitted.is_none(),
        "Spouse must be omitted when include_partner is false"
    );

    let has_ochre_omitted = layout_no_partner
        .connectors
        .iter()
        .any(|c| c.color.as_deref() == Some("#d97706"));
    assert!(
        !has_ochre_omitted,
        "No ochre marital connectors should be present when include_partner is false"
    );
}

#[tokio::test]
async fn test_include_siblings_placement() {
    let store = create_extended_test_store();

    // 1. With include_siblings = true
    let layout_with_siblings = compute_tree_layout(
        &store,
        "I_FOCAL",
        "pedigree",
        4,
        false,
        true,
        None,
        None,
        false,
        true, // include_siblings = true
        false,
        "Siblings Test Tree",
        None,
    )
    .unwrap();

    let sibling_card = layout_with_siblings
        .cards
        .iter()
        .find(|c| c.gid == "I_SIBLING");
    assert!(
        sibling_card.is_some(),
        "Sibling must be present when include_siblings is true"
    );
    assert_eq!(sibling_card.unwrap().generation, 0);

    // 2. With include_siblings = false and collateral = false
    let layout_no_siblings = compute_tree_layout(
        &store,
        "I_FOCAL",
        "pedigree",
        4,
        false,
        true,
        None,
        None,
        false,
        false, // include_siblings = false
        false,
        "Siblings Test Tree",
        None,
    )
    .unwrap();

    let sibling_omitted = layout_no_siblings
        .cards
        .iter()
        .find(|c| c.gid == "I_SIBLING");
    assert!(
        sibling_omitted.is_none(),
        "Sibling must be omitted when include_siblings is false"
    );
}

#[tokio::test]
async fn test_collateral_density_wide_vs_compact() {
    let store = create_extended_test_store();

    // 1. Hourglass with collateral = true (Wide mode)
    let layout_wide = compute_tree_layout(
        &store,
        "I_FOCAL",
        "hourglass",
        4,
        false,
        true,
        None,
        None,
        true,
        true,
        true, // collateral = true (Wide mode)
        "Collateral Test Tree",
        None,
    )
    .unwrap();

    let card_gids: HashSet<String> = layout_wide.cards.iter().map(|c| c.gid.clone()).collect();

    // Direct ancestors and descendants
    assert!(card_gids.contains("I_FOCAL"));
    assert!(card_gids.contains("I003")); // Father
    assert!(card_gids.contains("I004")); // Mother
    assert!(card_gids.contains("I_CHILD")); // Child
    assert!(card_gids.contains("I_GRANDCHILD")); // Grandchild

    // Collateral relatives included in Wide mode:
    assert!(
        card_gids.contains("I005"),
        "Aunt Rachel must be included in Wide mode"
    );
    assert!(
        card_gids.contains("I_COUSIN"),
        "First cousin Boris must be included in Wide mode"
    );
    assert!(
        card_gids.contains("I_NIECE"),
        "Niece Olga must be included in Wide mode"
    );

    // 2. Hourglass with collateral = false (Compact mode)
    let layout_compact = compute_tree_layout(
        &store,
        "I_FOCAL",
        "hourglass",
        4,
        false,
        true,
        None,
        None,
        true,
        false, // direct siblings off
        false, // collateral = false (Compact mode)
        "Collateral Test Tree",
        None,
    )
    .unwrap();

    let compact_gids: HashSet<String> =
        layout_compact.cards.iter().map(|c| c.gid.clone()).collect();

    // Direct lineage present
    assert!(compact_gids.contains("I_FOCAL"));
    assert!(compact_gids.contains("I003"));
    assert!(compact_gids.contains("I004"));
    assert!(compact_gids.contains("I_CHILD"));
    assert!(compact_gids.contains("I_GRANDCHILD"));

    // Collateral relatives excluded in Compact mode:
    assert!(
        !compact_gids.contains("I005"),
        "Aunt must NOT be in Compact mode"
    );
    assert!(
        !compact_gids.contains("I_COUSIN"),
        "Cousin must NOT be in Compact mode"
    );
    assert!(
        !compact_gids.contains("I_NIECE"),
        "Niece must NOT be in Compact mode"
    );
    assert!(layout_compact.cards.len() < layout_wide.cards.len());
}

#[tokio::test]
async fn test_all_nodes_forest_view_dsu_partitioning_and_singletons_shelf() {
    let store = create_extended_test_store();

    // Compute all_nodes layout
    let layout = compute_tree_layout(
        &store,
        "I_FOCAL",
        "all_nodes",
        5,
        false,
        true,
        Some("All Nodes Complete Forest"),
        None,
        true,
        true,
        true,
        "Complete Archive",
        None,
    )
    .unwrap();

    // 1. All 18 individuals across primary component, secondary component, and singletons must be present
    assert_eq!(
        layout.total_persons, 18,
        "All 18 persons from store must be present in all_nodes layout"
    );
    assert_eq!(layout.cards.len(), 18);

    // 2. Subtitle reflects total individuals count
    assert_eq!(layout.subtitle, "Total Individuals: 18");
    assert_eq!(layout.title, "All Nodes Complete Forest");

    // 3. Both secondary component members and singletons are placed
    let card_gids: HashSet<String> = layout.cards.iter().map(|c| c.gid.clone()).collect();
    assert!(
        card_gids.contains("I_SEC1"),
        "Lev Tolstoy from component 2 must be present"
    );
    assert!(
        card_gids.contains("I_SEC2"),
        "Sophia from component 2 must be present"
    );
    assert!(
        card_gids.contains("I_SEC3"),
        "Ilya from component 2 must be present"
    );
    assert!(
        card_gids.contains("I_SING1"),
        "Singleton Hermit must be present"
    );
    assert!(
        card_gids.contains("I_SING2"),
        "Singleton Wanderer must be present"
    );

    // 4. Singletons shelf is positioned below the connected components
    let primary_and_sec_max_y = layout
        .cards
        .iter()
        .filter(|c| c.gid != "I_SING1" && c.gid != "I_SING2")
        .map(|c| c.y + c.height)
        .fold(0.0, f64::max);

    let sing1_card = layout.cards.iter().find(|c| c.gid == "I_SING1").unwrap();
    let sing2_card = layout.cards.iter().find(|c| c.gid == "I_SING2").unwrap();

    assert!(
        sing1_card.y >= primary_and_sec_max_y + 25.0,
        "Singleton shelf must be placed below component tiers (sing1_y: {}, max_comp_y: {})",
        sing1_card.y,
        primary_and_sec_max_y
    );
    assert_eq!(
        sing1_card.y, sing2_card.y,
        "Singletons on row 0 of shelf must share identical horizontal baseline Y"
    );

    // 5. Verify API endpoints with all_nodes
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        view_mode: Some("all_nodes".to_string()),
        title: Some("All Nodes API Test".to_string()),
        subtitle: Some("Custom Subtitle".to_string()),
        include_partner: Some(true),
        include_siblings: Some(true),
        collateral: Some(true),
        ..Default::default()
    };

    // Preview calculation endpoint
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let preview_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    let preview_json: serde_json::Value = serde_json::from_slice(&preview_bytes).unwrap();
    assert_eq!(preview_json["title"], "All Nodes API Test");
    assert_eq!(preview_json["subtitle"], "Custom Subtitle");
    assert!(preview_json["cards"].as_array().unwrap().len() >= 3);

    // PDF generation endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn test_multiple_spouses_render_without_collision() {
    let mut store = create_extended_test_store();
    let p_spouse2 = make_test_person("p_spouse2", "I_SPOUSE2", "Anna", "Smirnova", "F", 1942);
    let f_focal2 = make_test_family("f_focal2", "F008", Some("p_focal"), Some("p_spouse2"), &[]);
    store.persons.push(p_spouse2);
    store.families.push(f_focal2);

    let layout = compute_tree_layout(
        &store,
        "I_FOCAL",
        "hourglass",
        4,
        false,
        true,
        None,
        None,
        true, // include_partner = true
        false,
        true,
        "Multi-Spouse Test Tree",
        None,
    )
    .unwrap();

    let focal_card = layout.cards.iter().find(|c| c.gid == "I_FOCAL").unwrap();
    let sp1_card = layout.cards.iter().find(|c| c.gid == "I_SPOUSE").unwrap();
    let sp2_card = layout.cards.iter().find(|c| c.gid == "I_SPOUSE2").unwrap();

    // Verify both spouses present on generation 0
    assert_eq!(focal_card.generation, 0);
    assert_eq!(sp1_card.generation, 0);
    assert_eq!(sp2_card.generation, 0);

    // Verify no card collision between focal and both spouses
    let cards = [focal_card, sp1_card, sp2_card];
    for i in 0..cards.len() {
        for j in (i + 1)..cards.len() {
            let c1 = cards[i];
            let c2 = cards[j];
            let overlap_x = (c1.x < c2.x + c2.width) && (c2.x < c1.x + c1.width);
            let overlap_y = (c1.y < c2.y + c2.height) && (c2.y < c1.y + c1.height);
            assert!(
                !(overlap_x && overlap_y),
                "Cards {} and {} collide!",
                c1.gid,
                c2.gid
            );
        }
    }

    // Verify ochre connectors exist
    let ochre_connectors: Vec<_> = layout
        .connectors
        .iter()
        .filter(|c| c.color.as_deref() == Some("#d97706"))
        .collect();
    assert!(
        !ochre_connectors.is_empty(),
        "Must render ochre marital connectors for married couples"
    );
}

#[tokio::test]
async fn test_fan_chart_export_and_preview_guard() {
    let (app, _state, admin_auth, _, _) = setup_app_and_tokens().await;

    let payload = PdfExportRequest {
        format: Some("poster".to_string()),
        target_sheet_size: Some("A2".to_string()),
        view_mode: Some("fan".to_string()),
        include_partner: Some(false),
        include_siblings: Some(false),
        collateral: Some(false),
        title: Some("Fan Chart Poster".to_string()),
        ..Default::default()
    };

    // Preview
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf/preview-info")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Export PDF
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/export/pdf")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let pdf_bytes = to_bytes(res.into_body(), 10 * 1024 * 1024).await.unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn test_date_formatting_sanitizes_sort_integers_and_eight_digit_strings() {
    let mut store = GenealogyPrivacyStore::empty();

    // 1. Person with YYYYMMDD sort integers
    let mut p1 = make_test_person("p1", "I_DATE1", "Ivan", "Ivanov", "M", 1989);
    p1.birth_sort = Some(19890625);
    p1.death_sort = Some(20130612);
    p1.is_living = false;

    // 2. Living person with 8-digit date string
    let mut p2 = make_test_person("p2", "I_DATE2", "Anna", "Ivanova", "F", 1995);
    p2.birth = Some(VitalEventSummaryDto {
        date: Some("19950715".to_string()),
        place: None,
    });
    p2.birth_sort = None;
    p2.is_living = true;

    // Family connecting p1 and p2
    let fam = make_test_family("f_date", "F_DATE", Some("p1"), Some("p2"), &[]);
    store.persons = vec![p1.clone(), p2];
    store.families = vec![fam];

    // English layout
    let layout_en = compute_tree_layout(
        &store,
        "I_DATE1",
        "hourglass",
        1,
        false, // redact_living = false
        false,
        None,
        None,
        true,
        false,
        false,
        "Date Test Tree",
        None,
    )
    .unwrap();

    let card1_en = layout_en.cards.iter().find(|c| c.gid == "I_DATE1").unwrap();
    assert_eq!(card1_en.lifespan, "1989 – 2013");
    assert!(!card1_en.lifespan.contains("19890625"));
    assert!(!card1_en.lifespan.contains("20130612"));

    let card2_en = layout_en.cards.iter().find(|c| c.gid == "I_DATE2").unwrap();
    assert_eq!(card2_en.lifespan, "b. 1995");
    assert!(!card2_en.lifespan.contains("19950715"));

    // Russian layout (focal has Cyrillic name so is_ru is true)
    let mut p1_ru = p1.clone();
    p1_ru.primary_name.given_name = "Иван".to_string();
    p1_ru.primary_name.surname = "Иванов".to_string();
    let mut store_ru = store.clone();
    store_ru.persons[0] = p1_ru;

    let layout_ru = compute_tree_layout(
        &store_ru,
        "I_DATE1",
        "hourglass",
        1,
        false,
        false,
        None,
        None,
        true,
        false,
        false,
        "Date Test Tree",
        None,
    )
    .unwrap();

    let card1_ru = layout_ru.cards.iter().find(|c| c.gid == "I_DATE1").unwrap();
    assert_eq!(card1_ru.lifespan, "1989 – 2013");

    let card2_ru = layout_ru.cards.iter().find(|c| c.gid == "I_DATE2").unwrap();
    assert_eq!(card2_ru.lifespan, "р. 1995");
}

#[tokio::test]
async fn test_hourglass_hierarchical_centering_parents_above_children() {
    let store = create_extended_test_store();

    let layout = compute_tree_layout(
        &store,
        "I_FOCAL",
        "hourglass",
        4,
        false,
        false,
        None,
        None,
        true, // include_partner = true
        true, // include_siblings = true
        true, // collateral = true
        "Centered Hourglass Test",
        None,
    )
    .unwrap();

    let focal = layout.cards.iter().find(|c| c.gid == "I_FOCAL").unwrap();
    let spouse = layout.cards.iter().find(|c| c.gid == "I_SPOUSE").unwrap();
    let father = layout.cards.iter().find(|c| c.gid == "I003").unwrap();
    let mother = layout.cards.iter().find(|c| c.gid == "I004").unwrap();
    let child = layout.cards.iter().find(|c| c.gid == "I_CHILD").unwrap();
    let aunt = layout.cards.iter().find(|c| c.gid == "I005").unwrap();
    let uncle = layout.cards.iter().find(|c| c.gid == "I006").unwrap();
    let cousin = layout.cards.iter().find(|c| c.gid == "I_COUSIN").unwrap();

    // 1. Verify generations
    assert_eq!(focal.generation, 0);
    assert_eq!(spouse.generation, 0);
    assert_eq!(father.generation, -1);
    assert_eq!(mother.generation, -1);
    assert_eq!(child.generation, 1);
    assert_eq!(aunt.generation, -1);
    assert_eq!(uncle.generation, -1);
    assert_eq!(cousin.generation, 0);

    // 2. Parents must NOT be huddled at the left margin x = 20.0
    assert!(
        father.x > 30.0,
        "Father must not be pinned to left margin x=20.0 (got x={})",
        father.x
    );
    assert!(
        focal.x > 30.0,
        "Focal must not be pinned to left margin x=20.0 (got x={})",
        focal.x
    );

    // 3. Focal couple center and Father/Mother union midpoint must align
    let focal_couple_mid = (focal.x + spouse.x + spouse.width) / 2.0;
    let parents_union_mid = (father.x.min(mother.x) + father.width + father.x.max(mother.x)) / 2.0;
    assert!(
        (focal_couple_mid - parents_union_mid).abs() < 2.0,
        "Father & Mother union (x={}) must be centered directly above Focal couple (x={})",
        parents_union_mid,
        focal_couple_mid
    );

    // 4. Child must be centered beneath Focal couple
    let child_mid = child.x + child.width / 2.0;
    assert!(
        (child_mid - focal_couple_mid).abs() < 2.0,
        "Child Dmitry (mid_x={}) must be centered beneath Focal couple (mid_x={})",
        child_mid,
        focal_couple_mid
    );

    // 5. Aunt Rachel & Uncle David are to the left of Father, and Cousin Boris is directly beneath them
    assert!(
        aunt.x < father.x,
        "Paternal aunt (x={}) must be placed to the left of Father (x={})",
        aunt.x,
        father.x
    );
    let aunt_couple_mid = (aunt.x + uncle.x + uncle.width) / 2.0;
    let cousin_mid = cousin.x + cousin.width / 2.0;
    assert!(
        (aunt_couple_mid - cousin_mid).abs() < 2.0,
        "Cousin Boris (mid_x={}) must be centered beneath Aunt Rachel & Uncle David (mid_x={})",
        cousin_mid,
        aunt_couple_mid
    );
}

#[tokio::test]
async fn test_all_nodes_hierarchical_centering_not_all_left_aligned() {
    let store = create_extended_test_store();

    let layout = compute_tree_layout(
        &store,
        "I_FOCAL",
        "all_nodes",
        4,
        false,
        false,
        None,
        None,
        true,
        true,
        true,
        "Centered All Nodes Test",
        None,
    )
    .unwrap();

    let sec1 = layout.cards.iter().find(|c| c.gid == "I_SEC1").unwrap();
    let sec2 = layout.cards.iter().find(|c| c.gid == "I_SEC2").unwrap();
    let sec3 = layout.cards.iter().find(|c| c.gid == "I_SEC3").unwrap();

    // In component 2 (Tolstoy family): Sec1 (Lev) and Sec2 (Sophia) are parents of Sec3 (Ilya)
    let parents_mid = (sec1.x.min(sec2.x) + sec1.width + sec1.x.max(sec2.x)) / 2.0;
    let child_mid = sec3.x + sec3.width / 2.0;

    assert!(
        (parents_mid - child_mid).abs() < 2.0,
        "Parents Lev & Sophia (union_mid={}) must be centered above child Ilya (mid={})",
        parents_mid,
        child_mid
    );
    assert_ne!(
        sec1.x, sec3.x,
        "Parents and children must not be left-aligned to the exact same x in all_nodes"
    );
}
