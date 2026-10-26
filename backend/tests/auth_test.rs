use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    error::ErrorResponse,
    routes::create_router,
    services::auth::{
        AdminSignInLinkResponse, AdminUserListResponse, AuthSessionResponse, InviteCreatedResponse,
        InvitePreviewResponse, LoginChallengeResponse, MagicLinkRequestResponse,
        PasskeyCredentialDto, PasskeyListResponse, RecoveryCodeLoginResponse,
        RegenerateRecoveryCodesResponse, SessionListResponse, SetupCompleteResponse,
        SetupStatusResponse, UserDto, WebAuthnChallengeResponse,
    },
    state::AppState,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_first_run_setup_passkey_login_replay_and_device_management() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Initial Archive".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    // 1. GET /api/v1/auth/setup/status reports uninitialized
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let status: SetupStatusResponse = serde_json::from_slice(&body).unwrap();
    assert!(!status.is_initialized);

    // 2. Request WebAuthn registration challenge
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reg_challenge: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();
    assert!(!reg_challenge.challenge.is_empty());

    // 3. Complete initial setup with Passkey
    let setup_payload = serde_json::json!({
        "tree_title": "Levshits Family Archive",
        "first_name": "Aleksandr",
        "last_name": "Levshits",
        "email": "admin@levshits.family",
        "passkey": {
            "credential_id": "cred-macbook-touchid-01",
            "public_key": "cose-public-key-base64url-01",
            "challenge": reg_challenge.challenge,
            "device_name": "MacBook Pro Touch ID",
            "transports": ["internal"]
        }
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.headers().get(header::SET_COOKIE).is_some());
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let setup_done: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(setup_done.user.role, "admin");
    assert_eq!(setup_done.user.display_name, "Aleksandr Levshits");
    assert_eq!(setup_done.recovery_codes.len(), 10);
    assert_eq!(setup_done.user.passkeys_count, 1);

    let admin_token = setup_done.session_token.clone();

    // 4. Attempting setup a second time returns 409 setup_already_completed
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "setup_already_completed");

    // 5. Passkey Login Challenge & Verify (with sign_count = 2)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"email":"admin@levshits.family"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        login_ch.allow_credentials,
        vec!["cred-macbook-touchid-01".to_string()]
    );

    let verify_payload = serde_json::json!({
        "credential_id": "cred-macbook-touchid-01",
        "challenge": login_ch.challenge,
        "sign_count": 2
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(verify_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 6. Replay check: attempting login with stale sign_count <= 2 is rejected with 401 passkey_replay_detected
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch2: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    let replay_payload = serde_json::json!({
        "credential_id": "cred-macbook-touchid-01",
        "challenge": login_ch2.challenge,
        "sign_count": 2
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(replay_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let replay_err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(replay_err.error.code, "passkey_replay_detected");

    // 7. Prevent deleting the sole remaining passkey when no password is set (400 last_auth_method)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/passkeys")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let pk_list: PasskeyListResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(pk_list.items.len(), 1);
    let first_pk_id = pk_list.items[0].id.clone();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/auth/passkeys/{first_pk_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let last_err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(last_err.error.code, "last_auth_method");

    // 8. Enroll a second passkey ("iPhone 15 Pro"), rename it, and then delete the old one
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/passkeys/add/challenge")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let add_ch: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();

    let add_pk_payload = serde_json::json!({
        "credential_id": "cred-iphone-faceid-02",
        "public_key": "cose-public-key-base64url-02",
        "challenge": add_ch.challenge,
        "device_name": "iPhone 15 Pro",
        "transports": ["internal", "hybrid"]
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/passkeys/add/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::from(add_pk_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let second_pk: PasskeyCredentialDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(second_pk.device_name, "iPhone 15 Pro");

    // Rename second passkey
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/auth/passkeys/{}", second_pk.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"device_name":"YubiKey 5C NFC"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let renamed: PasskeyCredentialDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(renamed.device_name, "YubiKey 5C NFC");

    // Now deleting the first passkey succeeds because a second passkey exists
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/auth/passkeys/{first_pk_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_password_login_lockout_recovery_codes_and_magic_link() {
    let config = AppConfig {
        app_env: "test".to_string(),
        auth_password_enabled: "true".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    // 1. Setup admin with password (< 10 chars rejected first, then valid 12-char password accepted)
    let short_setup = serde_json::json!({
        "first_name": "Vera",
        "last_name": "Petrova",
        "email": "vera@family.local",
        "password": "short"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(short_setup.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let valid_setup = serde_json::json!({
        "first_name": "Vera",
        "last_name": "Petrova",
        "email": "vera@family.local",
        "password": "StrongFamilyPassword2026!"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(valid_setup.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let setup_done: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();
    let first_recovery_code = setup_done.recovery_codes[0].clone();

    // 2. Password login succeeds
    let login_ok = serde_json::json!({
        "email": "vera@family.local",
        "password": "StrongFamilyPassword2026!"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/password/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(login_ok.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let sess_resp: AuthSessionResponse = serde_json::from_slice(&body).unwrap();
    assert!(sess_resp.prompt_passkey_enrollment);

    // 3. Recovery code single-use login succeeds once, then fails if reused
    let rc_req = serde_json::json!({
        "email": "vera@family.local",
        "code": first_recovery_code
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/recovery-code")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(rc_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let rc_resp: RecoveryCodeLoginResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(rc_resp.remaining_recovery_codes, 9);
    assert!(rc_resp.prompt_passkey_enrollment);

    // Reusing the same recovery code must fail (single-use)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/recovery-code")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(rc_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Regenerate recovery codes returns 10 fresh codes
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/recovery-codes/regenerate")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", rc_resp.session_token),
                )
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let regen: RegenerateRecoveryCodesResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(regen.recovery_codes.len(), 10);

    // 4. Magic link request (3 per 15 min per email rate limit) & single-use verify
    let ml_req = serde_json::json!({ "email": "vera@family.local" });
    let mut magic_token = String::new();
    for i in 0..3 {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/magic-link/request")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .body(Body::from(ml_req.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "Attempt {i} should succeed");
        let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
        let ml_resp: MagicLinkRequestResponse = serde_json::from_slice(&body).unwrap();
        magic_token = ml_resp.preview_token.unwrap();
    }

    // 4th magic link request within 15 min is rate limited (429 rate_limited)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/magic-link/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(ml_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

    // Verify magic link token via GET
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/auth/magic-link/verify?token={magic_token}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Reusing the same magic link token fails (single-use)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/auth/magic-link/verify?token={magic_token}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 5. Account lockout after 5 consecutive failed password attempts (429 account_locked)
    let bad_login = serde_json::json!({
        "email": "vera@family.local",
        "password": "WrongPasswordAttempt!"
    });
    for attempt in 1..=5 {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/password/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .header("x-forwarded-for", format!("10.0.0.{attempt}"))
                    .body(Body::from(bad_login.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        if attempt < 5 {
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        } else {
            assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
            let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
            let lock_err: ErrorResponse = serde_json::from_slice(&body).unwrap();
            assert_eq!(lock_err.error.code, "account_locked");
        }
    }
}

#[tokio::test]
async fn test_invitations_sessions_admin_signin_link_and_csrf_enforcement() {
    let config = AppConfig {
        app_env: "production".to_string(),
        app_origin: "https://tree.example.com".to_string(),
        auth_password_enabled: "true".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    // 1. CSRF check: POST without X-Requested-With is rejected with 403 csrf_validation_failed
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let csrf_err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(csrf_err.error.code, "csrf_validation_failed");

    // 2. Foreign Origin check: POST with mismatched Origin in production is rejected with 403
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .header("origin", "https://evil.attacker.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 3. Bootstrap Admin
    let setup_payload = serde_json::json!({
        "first_name": "Dmitry",
        "last_name": "Kuznetsov",
        "email": "dmitry@tree.example.com",
        "password": "AdminSecretPassword123!"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("origin", "https://tree.example.com")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let admin_setup: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();
    let admin_token = admin_setup.session_token;

    // 4. Admin creates an Editor invitation for a relative
    let invite_req = serde_json::json!({
        "email": "elena@tree.example.com",
        "invitee_name": "Elena Morozova",
        "role": "editor"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/admin/invites")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::from(invite_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let invite_created: InviteCreatedResponse = serde_json::from_slice(&body).unwrap();
    assert!(invite_created
        .invite_url
        .starts_with("https://tree.example.com/register?invite_token="));

    // 5. Preview invitation via GET /api/v1/auth/invites/{token}
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/auth/invites/{}", invite_created.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let preview: InvitePreviewResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(preview.role, "editor");
    assert_eq!(preview.inviter_name, "Dmitry Kuznetsov");

    // 6. Relative registers using the one-time invite token
    let reg_req = serde_json::json!({
        "invite_token": invite_created.token,
        "first_name": "Elena",
        "last_name": "Morozova",
        "password": "ElenaSecurePassword2026!"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(reg_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let elena_reg: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(elena_reg.user.role, "editor");
    let elena_id = elena_reg.user.id.clone();
    let elena_token = elena_reg.session_token.clone();

    // 7. Admin generates a 24h sign-in link for Elena (`POST /api/v1/admin/users/{id}/sign-in-link`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/admin/users/{elena_id}/sign-in-link"))
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let sign_in_link: AdminSignInLinkResponse = serde_json::from_slice(&body).unwrap();

    // Verify the admin-generated sign-in link logs Elena in (creating a second session for Elena)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/magic-link/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    serde_json::json!({ "token": sign_in_link.token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 8. Elena lists her active sessions (should see 2 sessions)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/sessions")
                .header(header::AUTHORIZATION, format!("Bearer {elena_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let sess_list: SessionListResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(sess_list.items.len(), 2);

    // 9. Admin revokes all sessions for Elena (`DELETE /api/v1/admin/users/{id}/sessions`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/admin/users/{elena_id}/sessions"))
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Elena's session token is now immediately invalid (401 not_authenticated)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {elena_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 10. Admin lists all users (`GET /api/v1/admin/users`)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/users")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let users_list: AdminUserListResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(users_list.items.len(), 2);

    // 11. Admin updates own profile via PATCH /api/v1/auth/me
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/auth/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {admin_token}"))
                .header("x-requested-with", "genealogy")
                .header("origin", "https://tree.example.com")
                .body(Body::from(r#"{"locale":"ru","theme":"dark"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let me_updated: UserDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(me_updated.locale.as_deref(), Some("ru"));
    assert_eq!(me_updated.theme.as_deref(), Some("dark"));
}

#[tokio::test]
async fn test_passkey_login_client_data_json_direct_and_utf8_double_encoded_challenge() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "WebAuthn ClientDataJSON Archive".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    // 1. Request registration challenge and complete setup with browser-style client_data_json
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reg_ch: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();

    let reg_cdj_raw = serde_json::json!({
        "type": "webauthn.create",
        "challenge": URL_SAFE_NO_PAD.encode(reg_ch.challenge.as_bytes()),
        "origin": "http://localhost:8080",
        "crossOrigin": false
    });
    let reg_cdj_b64 = URL_SAFE_NO_PAD.encode(reg_cdj_raw.to_string().as_bytes());

    let setup_payload = serde_json::json!({
        "tree_title": "WebAuthn ClientDataJSON Archive",
        "first_name": "Nina",
        "last_name": "Volkova",
        "email": "nina@volkova.family",
        "passkey": {
            "credential_id": "cred-browser-passkey-01",
            "public_key": "cose-public-key-browser-01",
            "challenge": reg_ch.challenge,
            "client_data_json": reg_cdj_b64,
            "device_name": "Chrome on macOS",
            "transports": ["internal"]
        }
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Request login challenge #1
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch1: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    // 2a. Mismatched clientDataJSON.challenge must be rejected BEFORE consuming login_ch1
    let bad_cdj_raw = serde_json::json!({
        "type": "webauthn.get",
        "challenge": URL_SAFE_NO_PAD.encode(b"forged-challenge-bytes"),
        "origin": "http://localhost:8080"
    });
    let bad_verify = serde_json::json!({
        "credential_id": "cred-browser-passkey-01",
        "challenge": login_ch1.challenge,
        "client_data_json": URL_SAFE_NO_PAD.encode(bad_cdj_raw.to_string().as_bytes()),
        "sign_count": 2
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(bad_verify.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "invalid_or_expired_challenge");

    // 2b. Direct base64url challenge in clientDataJSON (when frontend decodes base64url challenge to bytes)
    // succeeds using the still-unconsumed login_ch1.challenge
    let direct_cdj_raw = serde_json::json!({
        "type": "webauthn.get",
        "challenge": login_ch1.challenge,
        "origin": "http://localhost:8080"
    });
    let direct_verify = serde_json::json!({
        "credential_id": "cred-browser-passkey-01",
        "challenge": login_ch1.challenge,
        "client_data_json": URL_SAFE_NO_PAD.encode(direct_cdj_raw.to_string().as_bytes()),
        "sign_count": 2
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(direct_verify.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Request login challenge #2 and verify with UTF-8 double-encoded challenge in clientDataJSON
    // (when frontend passes TextEncoder().encode(challenge) to navigator.credentials.get)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch2: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    let utf8_cdj_raw = serde_json::json!({
        "type": "webauthn.get",
        "challenge": URL_SAFE_NO_PAD.encode(login_ch2.challenge.as_bytes()),
        "origin": "http://localhost:8080"
    });
    let utf8_verify = serde_json::json!({
        "credential_id": "cred-browser-passkey-01",
        "challenge": login_ch2.challenge,
        "client_data_json": URL_SAFE_NO_PAD.encode(utf8_cdj_raw.to_string().as_bytes()),
        "sign_count": 3
    });
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(utf8_verify.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
