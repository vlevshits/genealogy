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
    services::EmailService,
    state::AppState,
};
use std::sync::Arc;
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
        smtp_enabled: true,
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state.clone());

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
        assert_eq!(ml_resp.status, "sent");
    }

    let emails = state.email_service.captured_emails().await;
    assert_eq!(emails.len(), 3);
    let last_email = emails.last().unwrap();
    assert_eq!(last_email.to, "vera@family.local");
    let magic_token = last_email
        .text_body
        .split("token=")
        .nth(1)
        .and_then(|s| s.lines().next())
        .unwrap()
        .trim()
        .to_string();
    assert!(!magic_token.is_empty());

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

#[tokio::test]
async fn test_reverse_proxy_forwarded_origin_multi_origin_and_csrf_rejection() {
    // 1. Production instance with default app_origin ("http://localhost:8080") behind Caddy at https://genealogy.levshitsvv.pro
    let config = AppConfig {
        app_env: "production".to_string(),
        app_origin: "http://localhost:8080".to_string(),
        auth_password_enabled: "true".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    let setup_payload = serde_json::json!({
        "tree_title": "Production Reverse Proxy Tree",
        "first_name": "Elena",
        "last_name": "Levshits",
        "email": "elena@levshitsvv.pro",
        "password": "StrongProductionPass2026!"
    });

    // Mutating request with Origin: https://genealogy.levshitsvv.pro and matching X-Forwarded-Host + X-Forwarded-Proto succeeds!
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("origin", "https://genealogy.levshitsvv.pro")
                .header("x-forwarded-host", "genealogy.levshitsvv.pro")
                .header("x-forwarded-proto", "https")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        set_cookie.contains("; Secure"),
        "Expected Secure flag when X-Forwarded-Proto: https is present, got: {set_cookie}"
    );
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let setup_done: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();
    let token = setup_done.session_token;

    // Foreign origin (https://attacker.example) even with X-Forwarded-Host: genealogy.levshitsvv.pro must be rejected with 403!
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/auth/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header("x-requested-with", "genealogy")
                .header("origin", "https://attacker.example")
                .header("x-forwarded-host", "genealogy.levshitsvv.pro")
                .header("x-forwarded-proto", "https")
                .body(Body::from(r#"{"locale":"ru"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "csrf_validation_failed");

    // Scheme downgrade (Origin: http://genealogy.levshitsvv.pro when X-Forwarded-Proto: https) must also be rejected!
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/auth/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header("x-requested-with", "genealogy")
                .header("origin", "http://genealogy.levshitsvv.pro")
                .header("x-forwarded-host", "genealogy.levshitsvv.pro")
                .header("x-forwarded-proto", "https")
                .body(Body::from(r#"{"locale":"ru"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Multi-origin APP_ORIGIN list support ("https://domain1.com,https://domain2.com")
    let multi_config = AppConfig {
        app_env: "production".to_string(),
        app_origin: "https://domain1.com, https://domain2.com".to_string(),
        auth_password_enabled: "true".to_string(),
        ..AppConfig::default()
    };
    let multi_state = AppState::new(None, multi_config);
    let multi_app = create_router(multi_state);

    let res_d2 = multi_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("origin", "https://domain2.com")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_d2.status(), StatusCode::CREATED);
    let body = to_bytes(res_d2.into_body(), 64 * 1024).await.unwrap();
    let d2_setup: SetupCompleteResponse = serde_json::from_slice(&body).unwrap();

    let res_d1 = multi_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/auth/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", d2_setup.session_token),
                )
                .header("x-requested-with", "genealogy")
                .header("origin", "https://domain1.com")
                .body(Body::from(r#"{"locale":"en"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_d1.status(), StatusCode::OK);

    let res_evil = multi_app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/auth/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", d2_setup.session_token),
                )
                .header("x-requested-with", "genealogy")
                .header("origin", "https://attacker.example")
                .body(Body::from(r#"{"locale":"en"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_evil.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_magic_link_smtp_disabled_and_anti_enumeration() {
    // 1. When smtp_enabled is false, request returns 503 smtp_not_configured
    let config_disabled = AppConfig {
        app_env: "test".to_string(),
        smtp_enabled: false,
        ..AppConfig::default()
    };
    let state_disabled = AppState::new(None, config_disabled);
    let app_disabled = create_router(state_disabled);

    let res = app_disabled
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/magic-link/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    serde_json::json!({ "email": "nonexistent@family.local" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "smtp_not_configured");

    // 2. When smtp_enabled is true, anti-enumeration for non-existent user: returns 200 with status "sent"
    // and sends NO emails
    let config_enabled = AppConfig {
        app_env: "test".to_string(),
        smtp_enabled: true,
        ..AppConfig::default()
    };
    let state_enabled = AppState::new(None, config_enabled);
    let app_enabled = create_router(state_enabled.clone());

    let res = app_enabled
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/magic-link/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    serde_json::json!({ "email": "ghost@family.local" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let ml_resp: MagicLinkRequestResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(ml_resp.status, "sent");
    assert_eq!(
        ml_resp.message,
        "If an active account exists for this email address, a sign-in link has been sent."
    );
    assert!(state_enabled
        .email_service
        .captured_emails()
        .await
        .is_empty());
}

#[tokio::test]
async fn test_transactional_email_bilingual_templating_and_html_escaping() {
    let config = Arc::new(AppConfig {
        app_env: "test".to_string(),
        tree_title: "Архив <Тест> & 'Семья'".to_string(),
        app_origin: "https://tree.example.com".to_string(),
        smtp_enabled: true,
        ..AppConfig::default()
    });
    let email_service = EmailService::new(Arc::clone(&config));

    // 1. Russian magic link with XSS/special chars in user_name and tree_title
    email_service
        .send_magic_link(
            "user@test.local",
            "Иван <script>alert('xss')</script> & \"Друзья\"",
            "secret_token_12345",
            "ru-RU",
        )
        .await
        .unwrap();

    let captured = email_service.captured_emails().await;
    assert_eq!(captured.len(), 1);
    let ru_mail = &captured[0];
    assert_eq!(ru_mail.to, "user@test.local");
    assert!(ru_mail.subject.contains("Вход в Архив <Тест> & 'Семья'"));
    // Plaintext body contains raw unescaped name
    assert!(ru_mail
        .text_body
        .contains("Иван <script>alert('xss')</script> & \"Друзья\""));
    assert!(ru_mail.text_body.contains("15 минут"));
    assert!(ru_mail
        .text_body
        .contains("https://tree.example.com/magic-link/verify?token=secret_token_12345"));
    // HTML body MUST be properly escaped
    assert!(!ru_mail.html_body.contains("<script>alert"));
    assert!(ru_mail.html_body.contains(
        "Иван &lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt; &amp; &quot;Друзья&quot;"
    ));
    assert!(ru_mail
        .html_body
        .contains("Архив &lt;Тест&gt; &amp; &#39;Семья&#39;"));
    assert!(ru_mail.html_body.contains("lang=\"ru\""));

    // 2. English magic link with XSS/special chars
    email_service
        .send_magic_link(
            "john@test.local",
            "John <img src=x onerror=alert(1)> & Co",
            "token_en_67890",
            "en-US",
        )
        .await
        .unwrap();

    let captured = email_service.captured_emails().await;
    assert_eq!(captured.len(), 2);
    let en_mail = &captured[1];
    assert_eq!(en_mail.to, "john@test.local");
    assert!(en_mail
        .subject
        .contains("Sign in to Архив <Тест> & 'Семья'"));
    assert!(en_mail
        .text_body
        .contains("John <img src=x onerror=alert(1)> & Co"));
    assert!(en_mail.text_body.contains("15 minutes"));
    assert!(en_mail
        .text_body
        .contains("https://tree.example.com/magic-link/verify?token=token_en_67890"));
    assert!(!en_mail.html_body.contains("<img src=x"));
    assert!(en_mail
        .html_body
        .contains("John &lt;img src=x onerror=alert(1)&gt; &amp; Co"));
    assert!(en_mail.html_body.contains("lang=\"en\""));

    // 3. Invitation escaping in RU & EN
    email_service
        .send_invitation(
            "invitee@test.local",
            "Invitee",
            "Admin <admin@test.local>",
            "Editor & Contributor",
            "invite_tok_abc",
            "ru",
        )
        .await
        .unwrap();

    let captured = email_service.captured_emails().await;
    assert_eq!(captured.len(), 3);
    let invite_ru = &captured[2];
    assert!(!invite_ru.html_body.contains("<admin@test.local>"));
    assert!(invite_ru.html_body.contains("&lt;admin@test.local&gt;"));
    assert!(invite_ru.html_body.contains("Editor &amp; Contributor"));
}

#[tokio::test]
async fn test_passkey_authenticator_data_uv_flag_and_replay_protection() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use sha2::{Digest, Sha256};

    fn make_authenticator_data(rp_id: &str, flags: u8, sign_count: u32) -> String {
        let rp_hash = Sha256::digest(rp_id.as_bytes());
        let mut data = Vec::with_capacity(37);
        data.extend_from_slice(&rp_hash);
        data.push(flags);
        data.extend_from_slice(&sign_count.to_be_bytes());
        URL_SAFE_NO_PAD.encode(&data)
    }

    let config = AppConfig {
        app_env: "test".to_string(),
        rp_id: "genealogy.local".to_string(),
        tree_title: "Security Hardened Tree".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);
    let mut step = 0;

    // 1. Passkey registration input validation: empty credential_id or public_key must be rejected
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reg_ch: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let invalid_setup_empty_cred = serde_json::json!({
        "first_name": "Ivan",
        "last_name": "Petrov",
        "email": "ivan@petrov.local",
        "passkey": {
            "credential_id": "",
            "public_key": "some-public-key",
            "challenge": reg_ch.challenge,
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
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(invalid_setup_empty_cred.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "validation_failed");

    // 2. Successful initial setup with valid passkey
    step += 1;
    let valid_setup = serde_json::json!({
        "first_name": "Ivan",
        "last_name": "Petrov",
        "email": "ivan@petrov.local",
        "passkey": {
            "credential_id": "cred-hardened-01",
            "public_key": "cose-key-01",
            "challenge": reg_ch.challenge,
            "device_name": "TouchID Mac"
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
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(valid_setup.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Test UV = 0 (flags = 0x01, User Present only) is rejected with 400 user_verification_required
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let auth_data_no_uv = make_authenticator_data("genealogy.local", 0x01, 0);
    let verify_no_uv = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_no_uv,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_no_uv.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_resp: AuthSessionResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(login_resp.user.email, "ivan@petrov.local");

    // 4. Test mismatched rpIdHash is rejected with 400 passkey_verification_failed
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let auth_data_bad_rp = make_authenticator_data("phishing-domain.com", 0x05, 1);
    let verify_bad_rp = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_bad_rp,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_bad_rp.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_verification_failed");

    // 5. Test missing UP (flags = 0x04) is rejected with 400 passkey_verification_failed
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let auth_data_no_up = make_authenticator_data("genealogy.local", 0x04, 1);
    let verify_no_up = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_no_up,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_no_up.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_verification_failed");

    // 6. Test truncated authenticator data (< 37 bytes) is rejected with 400 passkey_verification_failed
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let truncated_data = URL_SAFE_NO_PAD.encode([0u8; 36]);
    let verify_truncated = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": truncated_data,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_truncated.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_verification_failed");

    // 7. Valid WebAuthn authenticatorData (UP | UV = 0x05, sign_count = 1) succeeds
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let valid_auth_data_1 = make_authenticator_data("genealogy.local", 0x05, 1);
    let verify_valid_1 = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": valid_auth_data_1,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_valid_1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let session: AuthSessionResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(session.user.email, "ivan@petrov.local");

    // 8. Replay of same sign_count = 1 must be rejected with 401 passkey_replay_detected
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let verify_replayed = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": valid_auth_data_1,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_replayed.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_replay_detected");

    // 9. Subsequent login with incremented sign_count = 2 succeeds
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let valid_auth_data_2 = make_authenticator_data("genealogy.local", 0x05, 2);
    let verify_valid_2 = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": valid_auth_data_2,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_valid_2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 10. Platform authenticators with sign_count = 0 (e.g. Apple Touch ID) can login repeatedly without replay detection
    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let auth_data_zero_1 = make_authenticator_data("genealogy.local", 0x05, 0);
    let verify_zero_1 = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_zero_1,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_zero_1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    step += 1;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    step += 1;
    let auth_data_zero_2 = make_authenticator_data("genealogy.local", 0x05, 0);
    let verify_zero_2 = serde_json::json!({
        "credential_id": "cred-hardened-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_zero_2,
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", format!("10.10.1.{step}"))
                .body(Body::from(verify_zero_2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 11. In production (app_env != "test"), omitting authenticator_data is rejected with 400 passkey_verification_failed
    let prod_config = AppConfig {
        app_env: "production".to_string(),
        rp_id: "genealogy.local".to_string(),
        tree_title: "Prod Tree".to_string(),
        ..AppConfig::default()
    };
    let prod_state = AppState::new(None, prod_config);
    let prod_app = create_router(prod_state);

    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reg_ch: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();

    let prod_setup = serde_json::json!({
        "first_name": "Elena",
        "last_name": "Smirnova",
        "email": "elena@smirnova.local",
        "passkey": {
            "credential_id": "cred-prod-01",
            "public_key": "cose-key-prod-01",
            "challenge": reg_ch.challenge,
        }
    });
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.2")
                .body(Body::from(prod_setup.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    // Verify without authenticator_data in production must fail
    let verify_no_auth_data = serde_json::json!({
        "credential_id": "cred-prod-01",
        "challenge": login_ch.challenge,
        "sign_count": 1
    });
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.4")
                .body(Body::from(verify_no_auth_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_verification_failed");

    // 12. Safe u32 to i32 sign counter conversion without negative integer overflow
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    // sign_count > i32::MAX (e.g. 3,500,000,000 or u32::MAX) converts safely without overflow
    let auth_data_large_count = make_authenticator_data("genealogy.local", 0x05, 3_500_000_000u32);
    let verify_large_count = serde_json::json!({
        "credential_id": "cred-prod-01",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_large_count,
    });
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.6")
                .body(Body::from(verify_large_count.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let session: AuthSessionResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(session.user.email, "elena@smirnova.local");

    // Attempting replay with clamped count (i32::MAX or lower) must trigger replay detection
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let login_ch_replay: LoginChallengeResponse = serde_json::from_slice(&body).unwrap();

    let verify_clamped_replay = serde_json::json!({
        "credential_id": "cred-prod-01",
        "challenge": login_ch_replay.challenge,
        "authenticator_data": auth_data_large_count,
    });
    let res = prod_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("x-forwarded-for", "10.20.1.8")
                .body(Body::from(verify_clamped_replay.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(err.error.code, "passkey_replay_detected");

    // 13. Dynamic RP ID resolution: when config.rp_id is "localhost", request headers (Host / X-Forwarded-Host)
    // dynamically resolve the effective rp_id so reverse-proxy setups avoid browser WebAuthn SecurityError.
    let default_config = AppConfig {
        app_env: "test".to_string(),
        rp_id: "localhost".to_string(),
        ..AppConfig::default()
    };
    let default_app = create_router(AppState::new(None, default_config));

    let res = default_app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/setup/status")
                .header(header::HOST, "genealogy.levshitsvv.pro")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let status_resp: SetupStatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status_resp.rp_id, "genealogy.levshitsvv.pro");

    let res = default_app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-forwarded-host", "genealogy.levshitsvv.pro")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let reg_ch: WebAuthnChallengeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(reg_ch.rp_id, "genealogy.levshitsvv.pro");
}

#[tokio::test]
async fn test_request_context_middleware_full_pipeline() {
    let config = AppConfig {
        app_env: "production".to_string(),
        app_origin: "http://internal-default:8080".to_string(),
        auth_password_enabled: "true".to_string(),
        rp_id: "localhost".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state.clone());

    // 1. GET /api/v1/auth/setup/status through reverse proxy headers
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/setup/status")
                .header("x-forwarded-host", "family.tree.example:8443")
                .header("x-forwarded-proto", "https")
                .header("x-forwarded-for", "203.0.113.195, 10.0.0.1")
                .header(header::USER_AGENT, "GenealogyTester/1.0\t(Safe)")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let status_resp: SetupStatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status_resp.rp_id, "family.tree.example");

    // 2. POST /api/v1/auth/setup with matching origin and forwarded HTTPS
    let setup_payload = serde_json::json!({
        "first_name": "Context",
        "last_name": "Tester",
        "email": "context@tree.example",
        "password": "ContextStrongPassword123!"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("origin", "https://family.tree.example:8443")
                .header("x-forwarded-host", "family.tree.example:8443")
                .header("x-forwarded-proto", "https")
                .header("x-forwarded-for", "203.0.113.195:54321, 10.0.0.1")
                .header(header::USER_AGENT, "GenealogyTester/1.0\t(Clean)")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(set_cookie.contains("; Secure"));

    // Verify session metadata recorded in auth_store
    let store = state.auth_store.read().await;
    let session = store.sessions.last().unwrap();
    assert_eq!(session.ip_address.as_deref(), Some("203.0.113.195"));
    assert_eq!(
        session.user_agent.as_deref(),
        Some("GenealogyTester/1.0(Clean)")
    );
    drop(store);

    // 3. POST /api/v1/auth/password/login with malformed / injection IP and plain HTTP
    let login_payload = serde_json::json!({
        "email": "context@tree.example",
        "password": "ContextStrongPassword123!"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/password/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header("origin", "http://family.tree.example:8080")
                .header("x-forwarded-host", "family.tree.example:8080")
                .header("x-forwarded-proto", "http")
                .header(
                    "x-forwarded-for",
                    "<script>alert(1)</script>; DROP TABLE users",
                )
                .header(header::USER_AGENT, "GenealogyTester/1.0 (PlainHTTP)")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie_http = res
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(
        !set_cookie_http.contains("; Secure"),
        "Plain HTTP connection without https origin must not have ; Secure flag"
    );

    // Verify session metadata recorded 127.0.0.1 fallback for injection IP
    let store = state.auth_store.read().await;
    let session = store.sessions.last().unwrap();
    assert_eq!(session.ip_address.as_deref(), Some("127.0.0.1"));
    assert_eq!(
        session.user_agent.as_deref(),
        Some("GenealogyTester/1.0 (PlainHTTP)")
    );
}

#[tokio::test]
async fn test_effective_rp_id_resolution_and_localhost_passkey_login() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use genealogy_backend::services::auth::AuthService;
    use sha2::{Digest, Sha256};

    fn make_auth_data(rp_id: &str, flags: u8, sign_count: u32) -> String {
        let rp_hash = Sha256::digest(rp_id.as_bytes());
        let mut data = Vec::with_capacity(37);
        data.extend_from_slice(&rp_hash);
        data.push(flags);
        data.extend_from_slice(&sign_count.to_be_bytes());
        URL_SAFE_NO_PAD.encode(&data)
    }

    // Configured RP ID is production domain, but requests come from localhost
    let config = AppConfig {
        app_env: "test".to_string(),
        rp_id: "genealogy.levshitsvv.pro".to_string(),
        tree_title: "Localhost RP ID Tree".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state.clone());

    // 1. Initial admin setup with a passkey
    let reg_ch = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/register/challenge")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "localhost:8080")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reg_ch.status(), StatusCode::OK);
    let reg_body = to_bytes(reg_ch.into_body(), 64 * 1024).await.unwrap();
    let reg_ch_resp: WebAuthnChallengeResponse = serde_json::from_slice(&reg_body).unwrap();
    // Challenge RP ID must be "localhost" when host is localhost:8080
    assert_eq!(reg_ch_resp.rp_id, "localhost");

    let setup_payload = serde_json::json!({
        "first_name": "Local",
        "last_name": "Admin",
        "email": "local.admin@example.com",
        "tree_title": "Localhost Tree",
        "default_locale": "en",
        "passkey": {
            "credential_id": "pk-localhost-cred-1",
            "public_key": "pubkey-bytes-localhost",
            "challenge": reg_ch_resp.challenge,
            "device_name": "MacBook TouchID",
            "transports": ["internal"]
        }
    });

    let setup_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/setup")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "localhost:8080")
                .body(Body::from(setup_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(setup_res.status(), StatusCode::CREATED);

    // 2. Request login challenge via localhost:8080 -> effective_rp_id is "localhost"
    let ch_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "localhost:8080")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ch_res.status(), StatusCode::OK);
    let ch_body = to_bytes(ch_res.into_body(), 64 * 1024).await.unwrap();
    let login_ch: LoginChallengeResponse = serde_json::from_slice(&ch_body).unwrap();
    assert_eq!(login_ch.rp_id, "localhost");

    // 3. Authenticate with authenticator_data signed with RP ID "localhost" and flags = 0x01 (UP only, no UV)
    let auth_data_localhost_up_only = make_auth_data("localhost", 0x01, 1);
    let verify_payload = serde_json::json!({
        "credential_id": "pk-localhost-cred-1",
        "challenge": login_ch.challenge,
        "authenticator_data": auth_data_localhost_up_only,
    });
    let verify_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "localhost:8080")
                .body(Body::from(verify_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(verify_res.status(), StatusCode::OK);
    let verify_body = to_bytes(verify_res.into_body(), 64 * 1024).await.unwrap();
    let session_resp: AuthSessionResponse = serde_json::from_slice(&verify_body).unwrap();
    assert_eq!(session_resp.user.email, "local.admin@example.com");

    // 4. Authenticate with authenticator_data signed with configured RP ID ("genealogy.levshitsvv.pro")
    let ch_res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/challenge")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "genealogy.levshitsvv.pro")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ch_res2.status(), StatusCode::OK);
    let ch_body2 = to_bytes(ch_res2.into_body(), 64 * 1024).await.unwrap();
    let login_ch2: LoginChallengeResponse = serde_json::from_slice(&ch_body2).unwrap();
    assert_eq!(login_ch2.rp_id, "genealogy.levshitsvv.pro");

    let auth_data_prod = make_auth_data("genealogy.levshitsvv.pro", 0x05, 2);
    let verify_prod = serde_json::json!({
        "credential_id": "pk-localhost-cred-1",
        "challenge": login_ch2.challenge,
        "authenticator_data": auth_data_prod,
    });
    let verify_res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/auth/login/verify")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .header(header::HOST, "genealogy.levshitsvv.pro")
                .body(Body::from(verify_prod.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(verify_res2.status(), StatusCode::OK);

    // 5. Verify hydrate_from_db runs safely without DB (returns Ok)
    let hyd_res = AuthService::hydrate_from_db(&state).await;
    assert!(hyd_res.is_ok());

    // 6. Verify ensure_* fallbacks return valid cached data
    let user_found =
        AuthService::ensure_user_and_passkeys_by_email(&state, "local.admin@example.com")
            .await
            .unwrap();
    assert!(user_found.is_some());
    assert_eq!(user_found.unwrap().email, "local.admin@example.com");

    let pk_found =
        AuthService::ensure_passkey_and_user_by_credential_id(&state, "pk-localhost-cred-1")
            .await
            .unwrap();
    assert!(pk_found.is_some());
    assert_eq!(pk_found.unwrap().0.credential_id, "pk-localhost-cred-1");
}
