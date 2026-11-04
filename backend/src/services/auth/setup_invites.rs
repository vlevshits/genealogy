#[allow(unused_imports)]
use super::*;

impl AuthService {
    pub async fn setup_initial_admin(
        state: &AppState,
        req: SetupRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(SetupCompleteResponse, String), AppError> {
        let already_initialized = Self::is_initialized(state).await;

        // If already initialized, only allow if a valid emergency_token is supplied
        if already_initialized {
            let Some(ref em_token) = req.emergency_token else {
                return Err(AppError::SetupAlreadyCompleted);
            };
            Self::consume_emergency_admin_token(state, em_token).await?;
        }

        Self::validate_user_names_and_email(&req.first_name, &req.last_name, &req.email)?;

        if req.passkey.is_none() && req.password.is_none() {
            let mut fields = HashMap::new();
            fields.insert(
                "auth".to_string(),
                "passkey_or_password_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Either a Passkey or a password is required to complete setup".to_string(),
                Some(fields),
            ));
        }

        let password_hash = match &req.password {
            Some(pw) if !pw.is_empty() => Some(hash_password(pw)?),
            _ => None,
        };

        if let Some(ref pk) = req.passkey {
            let cred_id = pk.credential_id.trim();
            let pub_key = pk.public_key.trim();
            if cred_id.is_empty() || pub_key.is_empty() {
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
            Self::consume_challenge(state, &pk.challenge, pk.client_data_json.as_deref()).await?;
        }

        if let Some(ref title) = req.tree_title {
            let trimmed = title.trim();
            if !trimmed.is_empty() && !state.config.settings_locked_from_env {
                let _ = state
                    .update_instance_settings(crate::state::UpdateInstanceSettingsRequest {
                        tree_title: Some(trimmed.to_string()),
                        ..Default::default()
                    })
                    .await;
            }
        }

        let now = Utc::now();
        let user_id = Uuid::new_v4();
        let email = req.email.trim().to_ascii_lowercase();
        let first_name = req.first_name.trim().to_string();
        let last_name = req.last_name.trim().to_string();

        let user_rec = UserRecord {
            id: user_id,
            email: email.clone(),
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            role: "admin".to_string(),
            person_id: None,
            is_active: true,
            password_hash: password_hash.clone(),
            locale: Some(state.config.default_locale.clone()),
            theme: None,
            home_person_id: None,
            start_page: "dashboard".to_string(),
            digest_opt_in: false,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: Some(now),
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        };

        let generated_codes = generate_recovery_codes();
        let code_records: Vec<RecoveryCodeRecord> = generated_codes
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        let passkey_rec = req.passkey.as_ref().map(|pk| PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: pk.credential_id.trim().to_string(),
            public_key: pk.public_key.trim().to_string(),
            sign_count: 0,
            transports: pk
                .transports
                .clone()
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: pk
                .device_name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Primary Passkey".to_string()),
            aaguid: pk.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        });

        {
            let mut store = state.auth_store.write().await;
            store.users.push(user_rec.clone());
            if let Some(ref pk_rec) = passkey_rec {
                store.passkeys.push(pk_rec.clone());
            }
            store.recovery_codes.extend(code_records.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_user_to_db(pool, &user_rec).await?;
            if let Some(ref pk_rec) = passkey_rec {
                Self::persist_passkey_to_db(pool, pk_rec).await?;
            }
            Self::persist_recovery_codes_to_db(pool, user_id, &code_records).await?;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        let plain_codes = generated_codes.into_iter().map(|(c, _)| c).collect();

        Ok((
            SetupCompleteResponse {
                user: user_dto,
                recovery_codes: plain_codes,
                session_token,
            },
            cookie_header,
        ))
    }

    pub async fn create_invitation(
        state: &AppState,
        created_by: Option<Uuid>,
        req: CreateInviteRequest,
    ) -> Result<InviteCreatedResponse, AppError> {
        let role = req.role.trim().to_ascii_lowercase();
        if !matches!(role.as_str(), "admin" | "editor" | "viewer") {
            let mut fields = HashMap::new();
            fields.insert("role".to_string(), "invalid_role".to_string());
            return Err(AppError::ValidationFailed(
                "Invitation role must be admin, editor, or viewer".to_string(),
                Some(fields),
            ));
        }

        let person_id = match &req.person_id {
            Some(pid) if !pid.trim().is_empty() => Some(Uuid::parse_str(pid).map_err(|_| {
                let mut fields = HashMap::new();
                fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                AppError::ValidationFailed("Invalid person_id UUID".to_string(), Some(fields))
            })?),
            _ => None,
        };

        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);
        let now = Utc::now();
        let expires_at = now + ChronoDuration::days(7);

        let inv = InvitationRecord {
            id: Uuid::new_v4(),
            token_hash: token_hash.clone(),
            role: role.clone(),
            person_id,
            email: req
                .email
                .as_ref()
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            invitee_name: req
                .invitee_name
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            created_by,
            expires_at,
            used_at: None,
            revoked_at: None,
            created_at: now,
        };

        state.auth_store.write().await.invitations.push(inv.clone());

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO invitations (id, token_hash, role, person_id, email, invitee_name, created_by, expires_at, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(inv.id)
            .bind(&inv.token_hash)
            .bind(&inv.role)
            .bind(inv.person_id)
            .bind(&inv.email)
            .bind(&inv.invitee_name)
            .bind(inv.created_by)
            .bind(inv.expires_at)
            .bind(inv.created_at)
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
        let invite_url = format!("{origin}/register?invite_token={raw_token}");

        Ok(InviteCreatedResponse {
            invitation: Self::to_invitation_dto(&inv),
            invite_url,
            token: raw_token,
        })
    }

    pub async fn preview_invitation(
        state: &AppState,
        raw_token: &str,
    ) -> Result<InvitePreviewResponse, AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();

        let store = state.auth_store.read().await;
        let inv = store
            .invitations
            .iter()
            .find(|i| {
                i.token_hash == token_hash
                    && i.used_at.is_none()
                    && i.revoked_at.is_none()
                    && i.expires_at > now
            })
            .cloned()
            .ok_or(AppError::InvalidOrExpiredInvite)?;

        let inviter_name = inv
            .created_by
            .and_then(|uid| store.users.iter().find(|u| u.id == uid))
            .map(|u| format!("{} {}", u.first_name, u.last_name))
            .unwrap_or_else(|| "Family Administrator".to_string());
        drop(store);

        let settings = state.resolve_instance_settings().await;
        Ok(InvitePreviewResponse {
            email: inv.email,
            invitee_name: inv.invitee_name,
            inviter_name,
            role: inv.role,
            person_id: inv.person_id.map(|id| id.to_string()),
            tree_title: settings.tree_title,
            expires_at: inv.expires_at.to_rfc3339(),
            auth_password_enabled: state.config.is_password_auth_effectively_enabled(),
        })
    }

    pub async fn register_with_invitation(
        state: &AppState,
        req: InviteRegisterRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(SetupCompleteResponse, String), AppError> {
        let token_hash = sha256_hex(req.invite_token.trim());
        let now = Utc::now();

        let inv = {
            let store = state.auth_store.read().await;
            store
                .invitations
                .iter()
                .find(|i| {
                    i.token_hash == token_hash
                        && i.used_at.is_none()
                        && i.revoked_at.is_none()
                        && i.expires_at > now
                })
                .cloned()
                .ok_or(AppError::InvalidOrExpiredInvite)?
        };

        let email = req
            .email
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .or_else(|| inv.email.clone())
            .unwrap_or_else(|| format!("member-{}@family.local", Uuid::new_v4().simple()));

        Self::validate_user_names_and_email(&req.first_name, &req.last_name, &email)?;

        if req.passkey.is_none() && req.password.is_none() {
            let mut fields = HashMap::new();
            fields.insert(
                "auth".to_string(),
                "passkey_or_password_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Either a Passkey or a password is required to register".to_string(),
                Some(fields),
            ));
        }

        let password_hash = match &req.password {
            Some(pw) if !pw.is_empty() => Some(hash_password(pw)?),
            _ => None,
        };

        if let Some(ref pk) = req.passkey {
            let cred_id = pk.credential_id.trim();
            let pub_key = pk.public_key.trim();
            if cred_id.is_empty() || pub_key.is_empty() {
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
            Self::consume_challenge(state, &pk.challenge, pk.client_data_json.as_deref()).await?;
        }

        let user_id = Uuid::new_v4();
        let user_rec = UserRecord {
            id: user_id,
            email: email.clone(),
            first_name: req.first_name.trim().to_string(),
            last_name: req.last_name.trim().to_string(),
            role: inv.role.clone(),
            person_id: inv.person_id,
            is_active: true,
            password_hash,
            locale: Some(state.config.default_locale.clone()),
            theme: None,
            home_person_id: inv.person_id,
            start_page: "dashboard".to_string(),
            digest_opt_in: false,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: Some(now),
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        };

        let generated_codes = generate_recovery_codes();
        let code_records: Vec<RecoveryCodeRecord> = generated_codes
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        let passkey_rec = req.passkey.as_ref().map(|pk| PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: pk.credential_id.trim().to_string(),
            public_key: pk.public_key.trim().to_string(),
            sign_count: 0,
            transports: pk
                .transports
                .clone()
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: pk
                .device_name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Passkey".to_string()),
            aaguid: pk.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        });

        {
            let mut store = state.auth_store.write().await;
            if store.users.iter().any(|u| u.email == email) {
                let mut fields = HashMap::new();
                fields.insert("email".to_string(), "email_already_registered".to_string());
                return Err(AppError::ValidationFailed(
                    "Email address is already registered".to_string(),
                    Some(fields),
                ));
            }
            if let Some(target_inv) = store.invitations.iter_mut().find(|i| i.id == inv.id) {
                target_inv.used_at = Some(now);
            }
            store.users.push(user_rec.clone());
            if let Some(ref pk_rec) = passkey_rec {
                store.passkeys.push(pk_rec.clone());
            }
            store.recovery_codes.extend(code_records.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_user_to_db(pool, &user_rec).await?;
            if let Some(ref pk_rec) = passkey_rec {
                Self::persist_passkey_to_db(pool, pk_rec).await?;
            }
            Self::persist_recovery_codes_to_db(pool, user_id, &code_records).await?;
            let _ = sqlx::query("UPDATE invitations SET used_at = NOW() WHERE id = $1")
                .bind(inv.id)
                .execute(pool)
                .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let plain_codes = generated_codes.into_iter().map(|(c, _)| c).collect();

        Ok((
            SetupCompleteResponse {
                user: user_dto,
                recovery_codes: plain_codes,
                session_token,
            },
            cookie_header,
        ))
    }
}
