#[allow(unused_imports)]
use super::*;

impl AuthService {
    pub fn to_passkey_dto(pk: &PasskeyRecord) -> PasskeyCredentialDto {
        PasskeyCredentialDto {
            id: pk.id.to_string(),
            credential_id: pk.credential_id.clone(),
            device_name: pk.device_name.clone(),
            transports: pk.transports.clone(),
            sign_count: pk.sign_count,
            aaguid: pk.aaguid.map(|u| u.to_string()),
            created_at: pk.created_at.to_rfc3339(),
            last_used_at: pk.last_used_at.map(|d| d.to_rfc3339()),
        }
    }

    pub fn to_invitation_dto(inv: &InvitationRecord) -> InvitationDto {
        InvitationDto {
            id: inv.id.to_string(),
            email: inv.email.clone(),
            invitee_name: inv.invitee_name.clone(),
            role: inv.role.clone(),
            person_id: inv.person_id.map(|u| u.to_string()),
            created_by: inv.created_by.map(|u| u.to_string()),
            expires_at: inv.expires_at.to_rfc3339(),
            used_at: inv.used_at.map(|d| d.to_rfc3339()),
            revoked_at: inv.revoked_at.map(|d| d.to_rfc3339()),
            created_at: inv.created_at.to_rfc3339(),
        }
    }

    pub fn to_admin_user_summary(u: &UserRecord, passkeys_count: usize) -> AdminUserSummaryDto {
        AdminUserSummaryDto {
            id: u.id.to_string(),
            email: u.email.clone(),
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            display_name: format!("{} {}", u.first_name, u.last_name),
            role: u.role.clone(),
            person_id: u.person_id.map(|p| p.to_string()),
            is_active: u.is_active,
            passkeys_count,
            has_password: u.password_hash.is_some(),
            last_login_at: u.last_login_at.map(|d| d.to_rfc3339()),
            deactivated_at: u.deactivated_at.map(|d| d.to_rfc3339()),
            created_at: u.created_at.to_rfc3339(),
        }
    }

    pub async fn update_admin_user(
        state: &AppState,
        target_user_id: Uuid,
        req: UpdateAdminUserRequest,
    ) -> Result<AdminUserSummaryDto, AppError> {
        let normalized_role = if let Some(ref raw_role) = req.role {
            let r = raw_role.trim().to_ascii_lowercase();
            if !matches!(r.as_str(), "admin" | "editor" | "viewer") {
                let mut fields = HashMap::new();
                fields.insert("role".to_string(), "invalid_role".to_string());
                return Err(AppError::ValidationFailed(
                    "Role must be 'admin', 'editor', or 'viewer'".to_string(),
                    Some(fields),
                ));
            }
            Some(r)
        } else {
            None
        };

        let parsed_person_id = match req.person_id.as_deref() {
            Some("") => Some(None),
            Some(s) => {
                let target = s.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    let id = Uuid::parse_str(&p.id).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "person_id must be a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(Some(id))
                } else {
                    let id = Uuid::parse_str(target).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "person_id must be a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(Some(id))
                }
            }
            None => None,
        };

        let now = Utc::now();
        let (updated_user, passkeys_count, purge_sessions) = {
            let mut store = state.auth_store.write().await;
            let active_admins_count = store
                .users
                .iter()
                .filter(|u| u.role == "admin" && u.is_active)
                .count();

            let target_idx = store
                .users
                .iter()
                .position(|u| u.id == target_user_id)
                .ok_or(AppError::NotFound)?;

            let target_currently_active_admin =
                store.users[target_idx].role == "admin" && store.users[target_idx].is_active;

            let would_lose_admin_role = normalized_role
                .as_deref()
                .map(|r| r != "admin")
                .unwrap_or(false);
            let would_be_deactivated = req.is_active == Some(false);

            // Sole Admin Protection (`02 §5.2 & §7 DoD #10`)
            if target_currently_active_admin
                && (would_lose_admin_role || would_be_deactivated)
                && active_admins_count <= 1
            {
                return Err(AppError::SoleAdminProtected);
            }

            let u = &mut store.users[target_idx];
            if let Some(ref r) = normalized_role {
                u.role = r.clone();
            }
            if let Some(pid_opt) = parsed_person_id {
                u.person_id = pid_opt;
                if u.home_person_id.is_none() {
                    u.home_person_id = pid_opt;
                }
            }

            let mut purge = false;
            if let Some(active) = req.is_active {
                u.is_active = active;
                if !active {
                    u.deactivated_at = Some(now);
                    purge = true;
                } else {
                    u.deactivated_at = None;
                }
            }
            u.updated_at = now;
            let cloned_user = u.clone();

            if purge {
                store.sessions.retain(|s| s.user_id != target_user_id);
            }

            let pk_cnt = store
                .passkeys
                .iter()
                .filter(|p| p.user_id == target_user_id)
                .count();

            (cloned_user, pk_cnt, purge)
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET role = $1, person_id = $2, home_person_id = $3, is_active = $4, deactivated_at = $5, updated_at = NOW()
                 WHERE id = $6",
            )
            .bind(&updated_user.role)
            .bind(updated_user.person_id)
            .bind(updated_user.home_person_id)
            .bind(updated_user.is_active)
            .bind(updated_user.deactivated_at)
            .bind(updated_user.id)
            .execute(pool)
            .await;

            if purge_sessions {
                let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                    .bind(target_user_id)
                    .execute(pool)
                    .await;
            }
        }

        Ok(Self::to_admin_user_summary(&updated_user, passkeys_count))
    }

    pub(crate) fn validate_user_names_and_email(
        first_name: &str,
        last_name: &str,
        email: &str,
    ) -> Result<(), AppError> {
        let mut fields = HashMap::new();
        if first_name.trim().is_empty() || first_name.trim().chars().count() > 60 {
            fields.insert("first_name".to_string(), "invalid_length".to_string());
        }
        if last_name.trim().is_empty() || last_name.trim().chars().count() > 60 {
            fields.insert("last_name".to_string(), "invalid_length".to_string());
        }
        let email_trimmed = email.trim();
        if email_trimmed.is_empty() || !email_trimmed.contains('@') || email_trimmed.len() > 255 {
            fields.insert("email".to_string(), "invalid_email".to_string());
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationFailed(
                "Invalid user profile fields".to_string(),
                Some(fields),
            ));
        }
        Ok(())
    }

    pub(crate) async fn consume_emergency_admin_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<(), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        if let Some(pool) = &state.db {
            if let Ok(Some(_)) = sqlx::query(
                "UPDATE emergency_admin_links
                 SET used_at = NOW()
                 WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
                 RETURNING id",
            )
            .bind(&token_hash)
            .fetch_optional(pool)
            .await
            {
                return Ok(());
            }
        }
        Err(AppError::InvalidOrExpiredToken)
    }

    pub(crate) async fn persist_user_to_db(
        pool: &sqlx::PgPool,
        u: &UserRecord,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, email, first_name, last_name, role, person_id, is_active, password_hash, locale, theme, home_person_id, start_page, digest_opt_in, last_login_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        )
        .bind(u.id)
        .bind(&u.email)
        .bind(&u.first_name)
        .bind(&u.last_name)
        .bind(&u.role)
        .bind(u.person_id)
        .bind(u.is_active)
        .bind(&u.password_hash)
        .bind(&u.locale)
        .bind(&u.theme)
        .bind(u.home_person_id)
        .bind(&u.start_page)
        .bind(u.digest_opt_in)
        .bind(u.last_login_at)
        .bind(u.created_at)
        .bind(u.updated_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn persist_passkey_to_db(
        pool: &sqlx::PgPool,
        pk: &PasskeyRecord,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO passkey_credentials (id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(pk.id)
        .bind(pk.user_id)
        .bind(&pk.credential_id)
        .bind(&pk.public_key)
        .bind(pk.sign_count)
        .bind(&pk.transports)
        .bind(&pk.device_name)
        .bind(pk.aaguid)
        .bind(pk.created_at)
        .bind(pk.last_used_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn persist_recovery_codes_to_db(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        codes: &[RecoveryCodeRecord],
    ) -> Result<(), AppError> {
        for rc in codes {
            sqlx::query(
                "INSERT INTO recovery_codes (id, user_id, code_hash, created_at)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(rc.id)
            .bind(user_id)
            .bind(&rc.code_hash)
            .bind(rc.created_at)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
