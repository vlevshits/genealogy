use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use base64::Engine as _;
use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
    state::AppState,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        tree_title: "Media & Face Tagging Test Archive".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        media_storage_path: "/tmp/genealogy-media-test".to_string(),
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

async fn setup_state_and_admin() -> (AppState, String) {
    let state = AppState::new_demo(None, test_config());
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
    (state, admin_auth)
}

async fn create_role_user(
    state: &AppState,
    admin_user_id: uuid::Uuid,
    email: &str,
    role: &str,
) -> String {
    let inv = AuthService::create_invitation(
        state,
        Some(admin_user_id),
        CreateInviteRequest {
            role: role.to_string(),
            email: Some(email.to_string()),
            invitee_name: None,
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (reg, _) = AuthService::register_with_invitation(
        state,
        InviteRegisterRequest {
            invite_token: inv.token,
            first_name: "Relative".to_string(),
            last_name: role.to_string(),
            email: Some(email.to_string()),
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

#[tokio::test]
async fn media_upload_gallery_filtering_and_binary_streaming_work() {
    let (state, admin_auth) = setup_state_and_admin().await;
    let app = create_router(state);

    // 1. Upload a photo with base64 payload, date_taken, place_path, and description
    let custom_payload = b"RIFF\x1a\x00\x00\x00WEBPVP8L\x0d\x00\x00\x00/\x00\x00\x00\x10\x07\x10\x11\x11\x88\x88\xfe\x07\x00CUSTOM_PORTRAIT_DATA";
    let b64_data = base64::engine::general_purpose::STANDARD.encode(custom_payload);

    let upload_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "file_name": "victory_parade_1945.webp",
                        "mime_type": "image/webp",
                        "title": "Парад Победы 1945",
                        "description": "Дедушка Николай после возвращения с фронта",
                        "date_taken": "9 MAY 1945",
                        "place_path": "Москва, СССР",
                        "width": 1600,
                        "height": 1200,
                        "data": format!("data:image/webp;base64,{b64_data}")
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_resp.status(), StatusCode::CREATED);
    let uploaded = read_json(upload_resp).await;

    let media_id = uploaded["id"].as_str().unwrap().to_string();
    assert_eq!(uploaded["gid"], "M0100");
    assert_eq!(uploaded["title"], "Парад Победы 1945");
    assert_eq!(uploaded["original_file_name"], "victory_parade_1945.webp");
    assert_eq!(uploaded["file_size_bytes"], custom_payload.len());
    assert_eq!(uploaded["date_sort"], 19450509);
    assert_eq!(uploaded["place_full_name"], "Москва, СССР");
    assert!(!uploaded["sha256"].as_str().unwrap().is_empty());

    // 2. Upload a PDF document from the 1910s linked to I0001
    let doc_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "file_name": "birth_certificate_1912.pdf",
                        "mime_type": "application/pdf",
                        "title": "Метрическая выписка 1912",
                        "date_taken": "14 MAR 1912",
                        "linked_person_ids": ["I0001"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(doc_resp.status(), StatusCode::CREATED);

    // 3. Stream binary file via GET /api/v1/media/{id}/file (by GID)
    let file_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/media/M0100/file")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(file_resp.status(), StatusCode::OK);
    assert_eq!(
        file_resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/webp"
    );
    let streamed_bytes = to_bytes(file_resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&streamed_bytes[..], custom_payload);

    // 4. Filter gallery by decade=1940s
    let dec_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/media?decade=1940s")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dec_resp.status(), StatusCode::OK);
    let dec_json = read_json(dec_resp).await;
    let dec_items = dec_json["items"].as_array().unwrap();
    assert!(dec_items.iter().any(|m| m["id"] == media_id));
    assert!(dec_json["total_unfiltered"].as_u64().unwrap() >= 5);

    // 5. Filter gallery by type=documents
    let type_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/media?type=documents")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(type_resp.status(), StatusCode::OK);
    let type_json = read_json(type_resp).await;
    let type_items = type_json["items"].as_array().unwrap();
    assert_eq!(type_items.len(), 1);
    assert_eq!(type_items[0]["mime_type"], "application/pdf");

    // 6. Filter gallery by untagged=true (M0100 has no face tags or linked persons yet)
    let untagged_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/media?untagged=true")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(untagged_resp.status(), StatusCode::OK);
    let untagged_json = read_json(untagged_resp).await;
    let untagged_items = untagged_json["items"].as_array().unwrap();
    assert!(untagged_items.iter().any(|m| m["id"] == media_id));

    // 7. Update media metadata with optimistic locking
    let patch_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/media/{media_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "title": "Парад Победы на Красной площади (1945)",
                        "version": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_resp.status(), StatusCode::OK);

    // Stale version conflict returns 409
    let conflict_resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/media/{media_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "title": "Stale Edit",
                        "version": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conflict_resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn face_tagging_primary_avatar_crop_and_entity_linking_work() {
    let (state, admin_auth) = setup_state_and_admin().await;
    let app = create_router(state);

    // 1. Out-of-bounds face tag coordinates are rejected with 422
    let bad_tag_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media/M0001/tags")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "person_id": "I0001",
                        "x": 1.25,
                        "y": 0.2,
                        "w": 0.15,
                        "h": 0.20
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_tag_resp.status(), StatusCode::BAD_REQUEST);

    // 2. Upload a portrait photo and tag I0001 (Nikolai Ivanovich Levshits) as Primary Avatar
    let upload_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "file_name": "nikolai_portrait.webp",
                        "mime_type": "image/webp",
                        "title": "Портрет Николая Ивановича"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_resp.status(), StatusCode::CREATED);
    let media_json = read_json(upload_resp).await;
    let media_id = media_json["id"].as_str().unwrap().to_string();

    let tag_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/media/{media_id}/tags"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "person_id": "I0001",
                        "x": 0.25,
                        "y": 0.15,
                        "w": 0.30,
                        "h": 0.40,
                        "is_primary_avatar": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tag_resp.status(), StatusCode::CREATED);
    let tag_json = read_json(tag_resp).await;
    let tag_id = tag_json["id"].as_str().unwrap().to_string();
    assert_eq!(tag_json["is_primary_avatar"], true);
    assert_eq!(tag_json["person_name"], "Levshits Mikhail Isaakovich");

    // 3. Verify person I0001 now has avatar_media_id, avatar_crop, and avatar_url in profile & tree
    let profile_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/persons/I0001/profile")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_resp.status(), StatusCode::OK);
    let profile_json = read_json(profile_resp).await;
    assert_eq!(profile_json["person"]["avatar_media_id"], media_id);
    assert!((profile_json["person"]["avatar_crop"]["x"].as_f64().unwrap() - 0.25).abs() < 1e-4);
    assert!(
        (profile_json["person"]["avatar_crop"]["width"]
            .as_f64()
            .unwrap()
            - 0.30)
            .abs()
            < 1e-4
    );
    let avatar_url = profile_json["person"]["avatar_url"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(avatar_url.contains("/crop?"));

    // 4. Stream the cropped avatar via GET /api/v1/media/{id}/crop
    let crop_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&avatar_url)
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(crop_resp.status(), StatusCode::OK);
    assert_eq!(
        crop_resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/webp"
    );

    // 5. Update face tag coordinates and verify person's avatar_crop updates
    let update_tag_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/media/{media_id}/tags/{tag_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "x": 0.35,
                        "y": 0.20,
                        "w": 0.25,
                        "h": 0.35
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_tag_resp.status(), StatusCode::OK);

    let person_resp = app
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
    let person_json = read_json(person_resp).await;
    assert!((person_json["avatar_crop"]["x"].as_f64().unwrap() - 0.35).abs() < 1e-4);
    assert!((person_json["avatar_crop"]["width"].as_f64().unwrap() - 0.25).abs() < 1e-4);

    // 6. Create a life event and link media to it, then unlink it
    let ev_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "individual_id": "I0001",
                        "event_type": "residence",
                        "date_text": "1925",
                        "place_path": "Санкт-Петербург, СССР"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ev_resp.status(), StatusCode::CREATED);
    let ev_json = read_json(ev_resp).await;
    let ev_id = ev_json["id"].as_str().unwrap().to_string();

    let link_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/media/{media_id}/links"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "entity_type": "event",
                        "entity_id": ev_id,
                        "role": "evidence"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_resp.status(), StatusCode::CREATED);
    let link_json = read_json(link_resp).await;
    let link_id = link_json["id"].as_str().unwrap().to_string();

    let unlink_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/media/{media_id}/links/{link_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unlink_resp.status(), StatusCode::NO_CONTENT);

    // 7. Delete the primary avatar face tag and verify avatar_media_id is cleared on I0001
    let del_tag_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/media/{media_id}/tags/{tag_id}"))
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_tag_resp.status(), StatusCode::NO_CONTENT);

    let person_after_resp = app
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
    let person_after_json = read_json(person_after_resp).await;
    assert!(person_after_json["avatar_media_id"].is_null());
    assert!(person_after_json["avatar_crop"].is_null());
}

#[tokio::test]
async fn media_rbac_and_living_person_privacy_shield_work() {
    let (state, admin_auth) = setup_state_and_admin().await;
    let admin_user_id = {
        let auth_store = state.auth_store.read().await;
        auth_store.users[0].id
    };
    let viewer_auth =
        create_role_user(&state, admin_user_id, "viewer@levshits.family", "viewer").await;
    let app = create_router(state);

    // Enable public tree mode
    let settings_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({ "enable_public_access": true }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_resp.status(), StatusCode::OK);

    // 1. Viewer cannot upload media or create face tags (403 Forbidden)
    let viewer_upload = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::from(
                    json!({
                        "file_name": "forbidden.webp",
                        "mime_type": "image/webp"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(viewer_upload.status(), StatusCode::FORBIDDEN);

    // 2. Upload a photo and tag ONLY living person I0042 (Elena Alexandrovna Levshits)
    let upload_living = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/media")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "file_name": "elena_2022.webp",
                        "mime_type": "image/webp",
                        "title": "Елена в отпуске 2022"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_living.status(), StatusCode::CREATED);
    let living_media = read_json(upload_living).await;
    let living_media_id = living_media["id"].as_str().unwrap().to_string();

    let tag_living = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/media/{living_media_id}/tags"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::from(
                    json!({
                        "person_id": "I0042",
                        "x": 0.4,
                        "y": 0.2,
                        "w": 0.2,
                        "h": 0.3,
                        "is_primary_avatar": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tag_living.status(), StatusCode::CREATED);

    // 3. Anonymous visitor in public mode gets 404 Not Found on living-only photo detail, file, and crop
    for path in [
        format!("/api/v1/media/{living_media_id}"),
        format!("/api/v1/media/{living_media_id}/file"),
        format!("/api/v1/media/{living_media_id}/crop?x=0.4&y=0.2&w=0.2&h=0.3"),
    ] {
        let anon_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            anon_resp.status(),
            StatusCode::NOT_FOUND,
            "Expected 404 for anonymous request to {path}"
        );
    }

    // 4. Authenticated Viewer CAN access the living photo detail, file, and crop
    let viewer_detail = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/media/{living_media_id}"))
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(viewer_detail.status(), StatusCode::OK);

    // 5. Group photo M0001 (deceased + living) is accessible to anonymous visitor, with living face tag stripped
    let anon_group = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/media/M0001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anon_group.status(), StatusCode::OK);
    let group_json = read_json(anon_group).await;
    let tags = group_json["face_tags"].as_array().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0]["is_living"], false);
}
