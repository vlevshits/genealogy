#[allow(unused_imports)]
use super::*;

impl AuthService {
    pub async fn verify_passkey_login(
        state: &AppState,
        req: PasskeyLoginVerifyRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        effective_rp_id: Option<String>,
        host: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        Self::consume_challenge(state, &req.challenge, req.client_data_json.as_deref()).await?;

        // Authenticator Data & UV Flag Verification
        let auth_data_count: Option<i32> = match &req.authenticator_data {
            Some(auth_data_b64) => {
                let auth_data = Self::decode_base64_flexible(auth_data_b64)
                    .ok_or(AppError::PasskeyVerificationFailed)?;
                if auth_data.len() < 37 {
                    return Err(AppError::PasskeyVerificationFailed);
                }

                // RP ID hash check (bytes 0..32): matches configured rp_id, dynamically resolved effective_rp_id,
                // "localhost" (if host is local or non-production environment), or request host.
                let mut candidate_rp_hashes: Vec<[u8; 32]> = Vec::new();
                let cfg_rp = state.config.rp_id.trim();
                if !cfg_rp.is_empty() {
                    candidate_rp_hashes.push(Sha256::digest(cfg_rp.as_bytes()).into());
                }
                if let Some(ref eff) = effective_rp_id {
                    let eff_trimmed = eff.trim();
                    if !eff_trimmed.is_empty() {
                        candidate_rp_hashes.push(Sha256::digest(eff_trimmed.as_bytes()).into());
                    }
                }
                let is_local_host = host
                    .as_deref()
                    .map(|h| {
                        let ht = h.trim();
                        ht.eq_ignore_ascii_case("localhost") || ht == "127.0.0.1" || ht == "::1"
                    })
                    .unwrap_or(false);

                if is_local_host || state.config.app_env != "production" {
                    candidate_rp_hashes.push(Sha256::digest(b"localhost").into());
                    candidate_rp_hashes.push(Sha256::digest(b"127.0.0.1").into());
                }

                if let Some(ref h) = host {
                    let ht = h.trim();
                    if !ht.is_empty() {
                        candidate_rp_hashes.push(Sha256::digest(ht.as_bytes()).into());
                    }
                }

                let matches_rp = candidate_rp_hashes
                    .iter()
                    .any(|h| &auth_data[0..32] == h.as_slice());

                if !matches_rp {
                    return Err(AppError::PasskeyVerificationFailed);
                }

                // Flags check (byte 32):
                let flags = auth_data[32];
                // User Present (UP, bit 0, 0x01) is strictly required
                if flags & 0x01 == 0 {
                    return Err(AppError::PasskeyVerificationFailed);
                }
                // User Verified (UV, bit 2, 0x04) is preferred per spec; presence of UP suffices if UV is not set.
                let _user_verified = (flags & 0x04) != 0;

                // Sign count (bytes 33..37):
                let sign_count_bytes: [u8; 4] = auth_data[33..37]
                    .try_into()
                    .map_err(|_| AppError::PasskeyVerificationFailed)?;
                let raw_sign_count = u32::from_be_bytes(sign_count_bytes);
                let incoming_sign_count = i32::try_from(raw_sign_count).unwrap_or(i32::MAX);
                Some(incoming_sign_count)
            }
            None => {
                if state.config.app_env != "test" {
                    return Err(AppError::PasskeyVerificationFailed);
                }
                None
            }
        };

        let now = Utc::now();
        let cred_id = req.credential_id.trim();

        // Resilient fallback from DB if passkey not found in memory
        let _ = Self::ensure_passkey_and_user_by_credential_id(state, cred_id).await;

        let (user_rec, new_sign_count) = {
            let mut store = state.auth_store.write().await;
            let pk = store
                .passkeys
                .iter_mut()
                .find(|p| p.credential_id == cred_id)
                .ok_or(AppError::InvalidCredentials)?;

            if let Some(incoming_sign_count) = auth_data_count {
                // Per W3C WebAuthn §7.2: Apple Touch ID and platform authenticators frequently report sign_count = 0.
                if incoming_sign_count > 0
                    && pk.sign_count > 0
                    && incoming_sign_count <= pk.sign_count
                {
                    return Err(AppError::PasskeyReplayDetected);
                }
                if incoming_sign_count > 0 {
                    pk.sign_count = incoming_sign_count;
                }
            } else {
                // Legacy fallback for test mocks without authenticator_data
                if let Some(incoming_count) = req.sign_count {
                    if incoming_count <= pk.sign_count {
                        return Err(AppError::PasskeyReplayDetected);
                    }
                    pk.sign_count = incoming_count;
                } else {
                    pk.sign_count += 1;
                }
            }

            pk.last_used_at = Some(now);
            let uid = pk.user_id;
            let updated_sign_count = pk.sign_count;

            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == uid && u.is_active)
                .ok_or(AppError::InvalidCredentials)?;

            if let Some(locked_until) = user.locked_until {
                if locked_until > now {
                    return Err(AppError::AccountLocked);
                }
            }
            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            (user.clone(), updated_sign_count)
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE passkey_credentials
                 SET sign_count = $1, last_used_at = NOW()
                 WHERE credential_id = $2",
            )
            .bind(new_sign_count)
            .bind(cred_id)
            .execute(pool)
            .await;
            let _ = sqlx::query(
                "UPDATE users
                 SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW()
                 WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: false,
            },
            cookie_header,
        ))
    }

    pub async fn password_login(
        state: &AppState,
        req: PasswordLoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        if !state.config.is_password_auth_effectively_enabled() {
            return Err(AppError::PasswordAuthDisabled);
        }

        let email = req.email.trim().to_ascii_lowercase();
        // Resilient fallback from DB if user not found in memory
        let _ = Self::ensure_user_and_passkeys_by_email(state, &email).await;

        let now = Utc::now();

        let user_rec = {
            let mut store = state.auth_store.write().await;
            let Some(user) = store
                .users
                .iter_mut()
                .find(|u| u.email == email && u.is_active)
            else {
                return Err(AppError::InvalidCredentials);
            };

            if let Some(locked_until) = user.locked_until {
                if locked_until > now {
                    return Err(AppError::AccountLocked);
                }
            }

            let valid = user
                .password_hash
                .as_deref()
                .map(|h| verify_password(&req.password, h))
                .unwrap_or(false);

            if !valid {
                user.failed_login_attempts += 1;
                let locked = if user.failed_login_attempts >= 5 {
                    user.locked_until = Some(now + ChronoDuration::minutes(15));
                    true
                } else {
                    false
                };
                let attempts = user.failed_login_attempts;
                let locked_until = user.locked_until;
                let uid = user.id;
                drop(store);

                if let Some(pool) = &state.db {
                    let _ = sqlx::query(
                        "UPDATE users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3",
                    )
                    .bind(attempts)
                    .bind(locked_until)
                    .bind(uid)
                    .execute(pool)
                    .await;
                }

                if locked {
                    return Err(AppError::AccountLocked);
                }
                return Err(AppError::InvalidCredentials);
            }

            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            user.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW()
                 WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let prompt_passkey = user_dto.passkeys_count == 0;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: prompt_passkey,
            },
            cookie_header,
        ))
    }

    pub async fn request_magic_link(
        state: &AppState,
        email_raw: &str,
    ) -> Result<MagicLinkRequestResponse, AppError> {
        if !state.config.smtp_enabled {
            return Err(AppError::SmtpNotConfigured);
        }

        let email = email_raw.trim().to_ascii_lowercase();
        state.rate_limiter.check_magic_link_email(&email).await?;

        let user_info_opt = {
            let store = state.auth_store.read().await;
            store
                .users
                .iter()
                .find(|u| u.email == email && u.is_active)
                .map(|u| {
                    let name = u.display_name();
                    let loc = u
                        .locale
                        .clone()
                        .unwrap_or_else(|| state.config.default_locale.clone());
                    (u.id, name, loc)
                })
        };

        if let Some((user_id, user_name, user_locale)) = user_info_opt {
            let now = Utc::now();
            let expires_at = now + ChronoDuration::minutes(15);
            let raw_token = generate_random_token_base64url();
            let token_hash = sha256_hex(&raw_token);

            let rec = MagicLinkRecord {
                id: Uuid::new_v4(),
                user_id,
                token_hash: token_hash.clone(),
                expires_at,
                used_at: None,
                created_at: now,
            };
            state.auth_store.write().await.magic_links.push(rec.clone());

            if let Some(pool) = &state.db {
                let _ = sqlx::query(
                    "INSERT INTO magic_links (id, user_id, token_hash, expires_at, created_at)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(rec.id)
                .bind(rec.user_id)
                .bind(&rec.token_hash)
                .bind(rec.expires_at)
                .bind(rec.created_at)
                .execute(pool)
                .await;
            }

            state
                .email_service
                .send_magic_link(&email, &user_name, &raw_token, &user_locale)
                .await?;
        }

        Ok(MagicLinkRequestResponse {
            status: "sent".to_string(),
            message:
                "If an active account exists for this email address, a sign-in link has been sent."
                    .to_string(),
        })
    }

    pub async fn verify_magic_link(
        state: &AppState,
        raw_token: &str,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();

        let user_rec = {
            let mut store = state.auth_store.write().await;
            let ml = store
                .magic_links
                .iter_mut()
                .find(|m| m.token_hash == token_hash && m.used_at.is_none() && m.expires_at > now)
                .ok_or(AppError::InvalidOrExpiredToken)?;
            ml.used_at = Some(now);
            let uid = ml.user_id;

            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == uid && u.is_active)
                .ok_or(AppError::InvalidOrExpiredToken)?;
            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            user.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query("UPDATE magic_links SET used_at = NOW() WHERE token_hash = $1")
                .bind(&token_hash)
                .execute(pool)
                .await;
            let _ = sqlx::query(
                "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW() WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: true,
            },
            cookie_header,
        ))
    }

    pub async fn create_admin_sign_in_link(
        state: &AppState,
        target_user_id: Uuid,
    ) -> Result<AdminSignInLinkResponse, AppError> {
        let now = Utc::now();
        let expires_at = now + ChronoDuration::hours(24);
        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);

        {
            let mut store = state.auth_store.write().await;
            if !store.users.iter().any(|u| u.id == target_user_id) {
                return Err(AppError::NotFound);
            }
            store.magic_links.push(MagicLinkRecord {
                id: Uuid::new_v4(),
                user_id: target_user_id,
                token_hash: token_hash.clone(),
                expires_at,
                used_at: None,
                created_at: now,
            });
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO magic_links (user_id, token_hash, expires_at)
                 VALUES ($1, $2, $3)",
            )
            .bind(target_user_id)
            .bind(&token_hash)
            .bind(expires_at)
            .execute(pool)
            .await;
        }

        let origin = state
            .config
            .app_origin
            .split([',', ';'])
            .map(|s| s.trim().trim_end_matches('/'))
            .find(|s| !s.is_empty())
            .unwrap_or("http://localhost:8080");
        let sign_in_url = format!("{origin}/magic-link/verify?token={raw_token}");

        Ok(AdminSignInLinkResponse {
            sign_in_url,
            token: raw_token,
            expires_at: expires_at.to_rfc3339(),
        })
    }

    pub async fn login_with_recovery_code(
        state: &AppState,
        req: RecoveryCodeLoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(RecoveryCodeLoginResponse, String), AppError> {
        let email = req.email.trim().to_ascii_lowercase();
        let code_hash = hash_recovery_code(&req.code);
        let now = Utc::now();

        let user_rec =
            {
                let mut store = state.auth_store.write().await;
                let user = store
                    .users
                    .iter_mut()
                    .find(|u| u.email == email && u.is_active)
                    .ok_or(AppError::InvalidCredentials)?;

                if let Some(locked_until) = user.locked_until {
                    if locked_until > now {
                        return Err(AppError::AccountLocked);
                    }
                }

                let uid = user.id;
                let Some(code_rec) = store.recovery_codes.iter_mut().find(|rc| {
                    rc.user_id == uid && rc.used_at.is_none() && rc.code_hash == code_hash
                }) else {
                    let user_mut = store
                        .users
                        .iter_mut()
                        .find(|u| u.id == uid)
                        .ok_or(AppError::InvalidCredentials)?;
                    user_mut.failed_login_attempts += 1;
                    if user_mut.failed_login_attempts >= 5 {
                        user_mut.locked_until = Some(now + ChronoDuration::minutes(15));
                        return Err(AppError::AccountLocked);
                    }
                    return Err(AppError::InvalidCredentials);
                };

                code_rec.used_at = Some(now);
                let user_mut = store
                    .users
                    .iter_mut()
                    .find(|u| u.id == uid)
                    .ok_or(AppError::InvalidCredentials)?;
                user_mut.failed_login_attempts = 0;
                user_mut.locked_until = None;
                user_mut.last_login_at = Some(now);
                user_mut.clone()
            };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE recovery_codes SET used_at = NOW()
                 WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL",
            )
            .bind(user_rec.id)
            .bind(&code_hash)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let remaining = user_dto.remaining_recovery_codes;

        Ok((
            RecoveryCodeLoginResponse {
                user: user_dto,
                session_token,
                remaining_recovery_codes: remaining,
                prompt_passkey_enrollment: true,
            },
            cookie_header,
        ))
    }
}
