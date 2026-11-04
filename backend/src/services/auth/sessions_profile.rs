#[allow(unused_imports)]
use super::*;

impl AuthService {
    pub async fn regenerate_recovery_codes(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let now = Utc::now();
        let generated = generate_recovery_codes();
        let records: Vec<RecoveryCodeRecord> = generated
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        {
            let mut store = state.auth_store.write().await;
            store.recovery_codes.retain(|rc| rc.user_id != user_id);
            store.recovery_codes.extend(records.clone());
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query("DELETE FROM recovery_codes WHERE user_id = $1")
                .bind(user_id)
                .execute(pool)
                .await;
            Self::persist_recovery_codes_to_db(pool, user_id, &records).await?;
        }

        Ok(generated.into_iter().map(|(c, _)| c).collect())
    }

    pub async fn add_passkey(
        state: &AppState,
        user_id: Uuid,
        req: PasskeyRegistrationInput,
    ) -> Result<PasskeyCredentialDto, AppError> {
        let cred_id = req.credential_id.trim().to_string();
        if cred_id.is_empty() || req.public_key.trim().is_empty() {
            let mut fields = HashMap::new();
            fields.insert(
                "credential_id".to_string(),
                "invalid_credential".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "credential_id and public_key are required".to_string(),
                Some(fields),
            ));
        }
        Self::consume_challenge(state, &req.challenge, req.client_data_json.as_deref()).await?;
        let now = Utc::now();

        let pk_rec = PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: cred_id,
            public_key: req.public_key.trim().to_string(),
            sign_count: 0,
            transports: req
                .transports
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: req
                .device_name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Additional Passkey".to_string()),
            aaguid: req.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        };

        {
            let mut store = state.auth_store.write().await;
            if store
                .passkeys
                .iter()
                .any(|p| p.credential_id == pk_rec.credential_id)
            {
                let mut fields = HashMap::new();
                fields.insert("credential_id".to_string(), "duplicate".to_string());
                return Err(AppError::ValidationFailed(
                    "Passkey is already registered".to_string(),
                    Some(fields),
                ));
            }
            store.passkeys.push(pk_rec.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_passkey_to_db(pool, &pk_rec).await?;
        }

        Ok(Self::to_passkey_dto(&pk_rec))
    }

    pub async fn rename_passkey(
        state: &AppState,
        user_id: Uuid,
        passkey_id: Uuid,
        device_name: &str,
    ) -> Result<PasskeyCredentialDto, AppError> {
        let trimmed = device_name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 100 {
            let mut fields = HashMap::new();
            fields.insert("device_name".to_string(), "invalid_length".to_string());
            return Err(AppError::ValidationFailed(
                "Passkey device name must be 1..=100 characters".to_string(),
                Some(fields),
            ));
        }

        let updated = {
            let mut store = state.auth_store.write().await;
            let pk = store
                .passkeys
                .iter_mut()
                .find(|p| p.id == passkey_id && p.user_id == user_id)
                .ok_or(AppError::NotFound)?;
            pk.device_name = trimmed.to_string();
            pk.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE passkey_credentials SET device_name = $1 WHERE id = $2 AND user_id = $3",
            )
            .bind(trimmed)
            .bind(passkey_id)
            .bind(user_id)
            .execute(pool)
            .await;
        }

        Ok(Self::to_passkey_dto(&updated))
    }

    pub async fn delete_passkey(
        state: &AppState,
        user_id: Uuid,
        passkey_id: Uuid,
    ) -> Result<(), AppError> {
        {
            let mut store = state.auth_store.write().await;
            let user_passkeys_count = store
                .passkeys
                .iter()
                .filter(|p| p.user_id == user_id)
                .count();
            let has_target = store
                .passkeys
                .iter()
                .any(|p| p.id == passkey_id && p.user_id == user_id);

            if !has_target {
                return Err(AppError::NotFound);
            }

            let has_password = store
                .users
                .iter()
                .find(|u| u.id == user_id)
                .and_then(|u| u.password_hash.as_ref())
                .is_some();

            // Prevent removing the last passkey when no password is set (or when only 1 passkey remains)
            if user_passkeys_count <= 1 && !has_password {
                return Err(AppError::LastAuthMethod);
            }

            store
                .passkeys
                .retain(|p| !(p.id == passkey_id && p.user_id == user_id));
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query("DELETE FROM passkey_credentials WHERE id = $1 AND user_id = $2")
                .bind(passkey_id)
                .bind(user_id)
                .execute(pool)
                .await;
        }

        Ok(())
    }

    pub async fn set_password(
        state: &AppState,
        user_id: Uuid,
        req: SetPasswordRequest,
    ) -> Result<(), AppError> {
        let new_hash = hash_password(&req.new_password)?;

        {
            let mut store = state.auth_store.write().await;
            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;

            if let Some(ref existing_hash) = user.password_hash {
                let current_ok = req
                    .current_password
                    .as_deref()
                    .map(|pw| verify_password(pw, existing_hash))
                    .unwrap_or(false);
                if !current_ok {
                    return Err(AppError::InvalidCredentials);
                }
            }

            user.password_hash = Some(new_hash.clone());
            user.updated_at = Utc::now();
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&new_hash)
            .bind(user_id)
            .execute(pool)
            .await;
        }

        Ok(())
    }

    pub async fn create_session(
        state: &AppState,
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(String, String), AppError> {
        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);
        let now = Utc::now();
        let ttl_days = i64::from(state.config.session_ttl_days.max(1));
        let expires_at = now + ChronoDuration::days(ttl_days);

        let session_rec = SessionRecord {
            id: Uuid::new_v4(),
            user_id,
            token_hash: token_hash.clone(),
            user_agent,
            ip_address,
            created_at: now,
            last_seen_at: now,
            expires_at,
        };

        state
            .auth_store
            .write()
            .await
            .sessions
            .push(session_rec.clone());

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO sessions (id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(session_rec.id)
            .bind(session_rec.user_id)
            .bind(&session_rec.token_hash)
            .bind(&session_rec.user_agent)
            .bind(&session_rec.ip_address)
            .bind(session_rec.created_at)
            .bind(session_rec.last_seen_at)
            .bind(session_rec.expires_at)
            .execute(pool)
            .await;
        }

        let max_age_secs = ttl_days * 86_400;
        let is_configured_https = state
            .config
            .app_origin
            .split([',', ';'])
            .any(|o| o.trim().starts_with("https://"));
        let secure_flag = if is_https || is_configured_https {
            "; Secure"
        } else {
            ""
        };
        let cookie = format!(
            "session={raw_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_secs}{secure_flag}"
        );

        Ok((raw_token, cookie))
    }

    pub fn apply_forwarded_proto_secure_cookie(
        cookie: String,
        headers: &axum::http::HeaderMap,
    ) -> String {
        let is_forwarded_https = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .is_some_and(|proto| proto.trim().eq_ignore_ascii_case("https"));
        if is_forwarded_https && !cookie.contains("; Secure") {
            format!("{cookie}; Secure")
        } else {
            cookie
        }
    }

    pub async fn authenticate_session(
        state: &AppState,
        raw_token: &str,
    ) -> Result<(UserRecord, SessionRecord), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();
        let ttl_days = i64::from(state.config.session_ttl_days.max(1));

        let sess_opt = {
            let store = state.auth_store.read().await;
            store
                .sessions
                .iter()
                .find(|s| s.token_hash == token_hash)
                .cloned()
        };

        let session = match sess_opt {
            Some(s) => s,
            None => {
                if let Some(pool) = &state.db {
                    let row_opt = sqlx::query(
                        "SELECT id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at \
                         FROM sessions \
                         WHERE token_hash = $1 AND expires_at > NOW()",
                    )
                    .bind(&token_hash)
                    .fetch_optional(pool)
                    .await?;

                    if let Some(r) = row_opt {
                        let s = SessionRecord {
                            id: r.try_get("id")?,
                            user_id: r.try_get("user_id")?,
                            token_hash: r.try_get("token_hash")?,
                            user_agent: r.try_get("user_agent").ok().flatten(),
                            ip_address: r.try_get("ip_address").ok().flatten(),
                            created_at: r.try_get("created_at")?,
                            last_seen_at: r.try_get("last_seen_at")?,
                            expires_at: r.try_get("expires_at")?,
                        };
                        let mut store = state.auth_store.write().await;
                        store.sessions.push(s.clone());
                        s
                    } else {
                        return Err(AppError::NotAuthenticated);
                    }
                } else {
                    return Err(AppError::NotAuthenticated);
                }
            }
        };

        if session.expires_at <= now {
            let mut store = state.auth_store.write().await;
            store.sessions.retain(|s| s.id != session.id);
            return Err(AppError::SessionExpired);
        }

        let _ = Self::ensure_user_by_id(state, session.user_id).await;

        let user = {
            let mut store = state.auth_store.write().await;
            if let Some(pos) = store.sessions.iter().position(|s| s.id == session.id) {
                store.sessions[pos].last_seen_at = now;
                store.sessions[pos].expires_at = now + ChronoDuration::days(ttl_days);
            }
            store
                .users
                .iter()
                .find(|u| u.id == session.user_id && u.is_active)
                .cloned()
                .ok_or(AppError::NotAuthenticated)?
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE sessions SET last_seen_at = NOW(), expires_at = $1 WHERE id = $2",
            )
            .bind(now + ChronoDuration::days(ttl_days))
            .bind(session.id)
            .execute(pool)
            .await;
        }

        Ok((user, session))
    }

    pub async fn build_user_dto(state: &AppState, user: &UserRecord) -> UserDto {
        let store = state.auth_store.read().await;
        let passkeys_count = store
            .passkeys
            .iter()
            .filter(|p| p.user_id == user.id)
            .count();
        let remaining_recovery_codes = store
            .recovery_codes
            .iter()
            .filter(|rc| rc.user_id == user.id && rc.used_at.is_none())
            .count();

        UserDto {
            id: user.id.to_string(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            display_name: format!("{} {}", user.first_name, user.last_name),
            role: user.role.clone(),
            person_id: user.person_id.map(|id| id.to_string()),
            is_active: user.is_active,
            locale: user.locale.clone(),
            theme: user.theme.clone(),
            home_person_id: user.home_person_id.map(|id| id.to_string()),
            start_page: user.start_page.clone(),
            digest_opt_in: user.digest_opt_in,
            has_password: user.password_hash.is_some(),
            passkeys_count,
            remaining_recovery_codes,
            last_login_at: user.last_login_at.map(|d| d.to_rfc3339()),
            created_at: user.created_at.to_rfc3339(),
        }
    }

    pub async fn update_profile(
        state: &AppState,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserDto, AppError> {
        let mut fields = HashMap::new();

        if let Some(ref first) = req.first_name {
            let trimmed = first.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 60 {
                fields.insert("first_name".to_string(), "invalid_length".to_string());
            }
        }
        if let Some(ref last) = req.last_name {
            let trimmed = last.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 60 {
                fields.insert("last_name".to_string(), "invalid_length".to_string());
            }
        }
        if let Some(ref locale) = req.locale {
            if !matches!(locale.trim(), "en" | "ru") {
                fields.insert("locale".to_string(), "invalid_locale".to_string());
            }
        }
        if let Some(ref theme) = req.theme {
            if !matches!(theme.trim(), "light" | "dark" | "system") {
                fields.insert("theme".to_string(), "invalid_theme".to_string());
            }
        }
        if let Some(ref sp) = req.start_page {
            if !matches!(sp.trim(), "tree" | "dashboard") {
                fields.insert("start_page".to_string(), "invalid_start_page".to_string());
            }
        }

        let resolved_home_person: Option<Option<Uuid>> = match req.home_person_id.as_deref() {
            Some(raw) if raw.trim().is_empty() => Some(None),
            Some(raw) => {
                let target = raw.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    Some(Uuid::parse_str(&p.id).ok())
                } else if let Ok(uid) = Uuid::parse_str(target) {
                    Some(Some(uid))
                } else {
                    fields.insert("home_person_id".to_string(), "invalid_uuid".to_string());
                    None
                }
            }
            None => None,
        };

        if !fields.is_empty() {
            return Err(AppError::ValidationFailed(
                "Invalid user profile preferences".to_string(),
                Some(fields),
            ));
        }

        let updated_user = {
            let mut store = state.auth_store.write().await;
            let u = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;

            if let Some(first) = req.first_name {
                u.first_name = first.trim().to_string();
            }
            if let Some(last) = req.last_name {
                u.last_name = last.trim().to_string();
            }
            if let Some(locale) = req.locale {
                u.locale = Some(locale.trim().to_string());
            }
            if let Some(theme) = req.theme {
                u.theme = Some(theme.trim().to_string());
            }
            if let Some(hp_opt) = resolved_home_person {
                u.home_person_id = hp_opt;
            }
            if let Some(sp) = req.start_page {
                u.start_page = sp.trim().to_string();
            }
            if let Some(digest) = req.digest_opt_in {
                u.digest_opt_in = digest;
            }
            u.updated_at = Utc::now();
            u.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET first_name = $1, last_name = $2, locale = $3, theme = $4,
                     home_person_id = $5, start_page = $6, digest_opt_in = $7, updated_at = NOW()
                 WHERE id = $8",
            )
            .bind(&updated_user.first_name)
            .bind(&updated_user.last_name)
            .bind(&updated_user.locale)
            .bind(&updated_user.theme)
            .bind(updated_user.home_person_id)
            .bind(&updated_user.start_page)
            .bind(updated_user.digest_opt_in)
            .bind(updated_user.id)
            .execute(pool)
            .await;
        }

        Ok(Self::build_user_dto(state, &updated_user).await)
    }

    pub async fn set_home_person(
        state: &AppState,
        user_id: Uuid,
        person_id_raw: Option<String>,
    ) -> Result<UserDto, AppError> {
        let resolved_uuid: Option<Uuid> = match person_id_raw.as_deref() {
            None => None,
            Some(raw) if raw.trim().is_empty() => None,
            Some(raw) => {
                let target = raw.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    let uid = Uuid::parse_str(&p.id).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "Resolved person ID is not a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(uid)
                } else {
                    return Err(AppError::NotFound);
                }
            }
        };

        let updated_user = {
            let mut store = state.auth_store.write().await;
            let u = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;
            u.home_person_id = resolved_uuid;
            u.updated_at = Utc::now();
            u.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users SET home_person_id = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(updated_user.home_person_id)
            .bind(updated_user.id)
            .execute(pool)
            .await;
        }

        Ok(Self::build_user_dto(state, &updated_user).await)
    }
}
