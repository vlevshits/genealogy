#[allow(unused_imports)]
use super::*;
use chrono::Datelike;

impl AuthService {
    pub async fn is_initialized(state: &AppState) -> bool {
        let store = state.auth_store.read().await;
        if !store.users.is_empty() {
            return true;
        }
        drop(store);

        if let Some(pool) = &state.db {
            if let Ok(row) = sqlx::query("SELECT COUNT(*)::BIGINT AS cnt FROM users")
                .fetch_one(pool)
                .await
            {
                let cnt = row.try_get::<i64, _>("cnt").unwrap_or(0);
                if cnt > 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<UserRecord, sqlx::Error> {
        Ok(UserRecord {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            role: row.try_get("role")?,
            person_id: row.try_get("person_id").ok().flatten(),
            is_active: row.try_get("is_active")?,
            password_hash: row.try_get("password_hash").ok().flatten(),
            locale: row.try_get("locale").ok().flatten(),
            theme: row.try_get("theme").ok().flatten(),
            home_person_id: row.try_get("home_person_id").ok().flatten(),
            start_page: row
                .try_get("start_page")
                .unwrap_or_else(|_| "tree".to_string()),
            digest_opt_in: row.try_get("digest_opt_in").unwrap_or(false),
            failed_login_attempts: row.try_get("failed_login_attempts").unwrap_or(0),
            locked_until: row.try_get("locked_until").ok().flatten(),
            last_login_at: row.try_get("last_login_at").ok().flatten(),
            deactivated_at: row.try_get("deactivated_at").ok().flatten(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub fn row_to_passkey(row: &sqlx::postgres::PgRow) -> Result<PasskeyRecord, sqlx::Error> {
        let transports: Option<Vec<String>> = row.try_get("transports").ok().flatten();
        Ok(PasskeyRecord {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            credential_id: row.try_get("credential_id")?,
            public_key: row.try_get("public_key")?,
            sign_count: row.try_get("sign_count").unwrap_or(0),
            transports: transports.unwrap_or_default(),
            device_name: row.try_get("device_name")?,
            aaguid: row.try_get("aaguid").ok().flatten(),
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at").ok().flatten(),
        })
    }

    pub async fn hydrate_from_db(state: &AppState) -> Result<(), AppError> {
        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(()),
        };

        // 1. users
        let user_rows = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users",
        )
        .fetch_all(pool)
        .await?;

        let mut users = Vec::with_capacity(user_rows.len());
        for r in &user_rows {
            if let Ok(u) = Self::row_to_user(r) {
                users.push(u);
            }
        }

        // 2. passkey_credentials
        let passkey_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials",
        )
        .fetch_all(pool)
        .await?;

        let mut passkeys = Vec::with_capacity(passkey_rows.len());
        for r in &passkey_rows {
            if let Ok(pk) = Self::row_to_passkey(r) {
                passkeys.push(pk);
            }
        }

        // 3. recovery_codes
        let recovery_rows = sqlx::query(
            "SELECT id, user_id, code_hash, used_at, created_at \
             FROM recovery_codes",
        )
        .fetch_all(pool)
        .await?;

        let recovery_codes: Vec<RecoveryCodeRecord> = recovery_rows
            .into_iter()
            .filter_map(|row| {
                Some(RecoveryCodeRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    code_hash: row.try_get("code_hash").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        // 4. sessions (load active unexpired sessions)
        let session_rows = sqlx::query(
            "SELECT id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at \
             FROM sessions \
             WHERE expires_at > NOW()",
        )
        .fetch_all(pool)
        .await?;

        let sessions: Vec<SessionRecord> = session_rows
            .into_iter()
            .filter_map(|row| {
                Some(SessionRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    user_agent: row.try_get("user_agent").ok().flatten(),
                    ip_address: row.try_get("ip_address").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                    last_seen_at: row.try_get("last_seen_at").ok()?,
                    expires_at: row.try_get("expires_at").ok()?,
                })
            })
            .collect();

        // 5. invitations
        let invitation_rows = sqlx::query(
            "SELECT id, token_hash, role, person_id, email, invitee_name, created_by, expires_at, used_at, revoked_at, created_at \
             FROM invitations",
        )
        .fetch_all(pool)
        .await?;

        let invitations: Vec<InvitationRecord> = invitation_rows
            .into_iter()
            .filter_map(|row| {
                Some(InvitationRecord {
                    id: row.try_get("id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    role: row.try_get("role").ok()?,
                    person_id: row.try_get("person_id").ok().flatten(),
                    email: row.try_get("email").ok().flatten(),
                    invitee_name: row.try_get("invitee_name").ok().flatten(),
                    created_by: row.try_get("created_by").ok().flatten(),
                    expires_at: row.try_get("expires_at").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    revoked_at: row.try_get("revoked_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        // 6. magic_links
        let magic_rows = sqlx::query(
            "SELECT id, user_id, token_hash, expires_at, used_at, created_at \
             FROM magic_links \
             WHERE expires_at > NOW() AND used_at IS NULL",
        )
        .fetch_all(pool)
        .await?;

        let magic_links: Vec<MagicLinkRecord> = magic_rows
            .into_iter()
            .filter_map(|row| {
                Some(MagicLinkRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    expires_at: row.try_get("expires_at").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        let admin_user_id_str = users
            .iter()
            .find(|u| u.role == "admin" && u.is_active)
            .or_else(|| users.first())
            .map(|u| u.id.to_string());

        let mut store = state.auth_store.write().await;
        store.users = users;
        store.passkeys = passkeys;
        store.recovery_codes = recovery_codes;
        store.sessions = sessions;
        store.invitations = invitations;
        store.magic_links = magic_links;

        tracing::info!(
            users = store.users.len(),
            passkeys = store.passkeys.len(),
            sessions = store.sessions.len(),
            "Auth memory store hydrated from PostgreSQL"
        );
        drop(store);

        if state.config.app_env != "test" {
            let backup_dir = std::path::Path::new(&state.config.backup_path);
            let live_store_path = backup_dir.join("live-store.json");
            let current_year = Utc::now().year();
            let age_threshold = state.config.living_person_age_threshold as i32;

            let mut hydrated_store: Option<crate::services::privacy::GenealogyPrivacyStore> = None;
            if live_store_path.exists() {
                if let Ok(bytes) = std::fs::read(&live_store_path) {
                    if let Ok(mut loaded) = serde_json::from_slice::<
                        crate::services::privacy::GenealogyPrivacyStore,
                    >(&bytes)
                    {
                        loaded.is_demo_seed = false;
                        crate::services::genealogy::recompute_all_places_metadata(&mut loaded);
                        crate::services::genealogy::recompute_all_individuals_vitals(
                            &mut loaded,
                            current_year,
                            age_threshold,
                        );
                        hydrated_store = Some(loaded);
                    }
                }
            }

            if hydrated_store.is_none() {
                let snapshots_dir = backup_dir.join("snapshots");
                if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let fname = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default();
                        if fname.starts_with("snapshot-") && fname.ends_with(".json") {
                            if let Ok(meta) = entry.metadata() {
                                if let Ok(mtime) = meta.modified() {
                                    if newest.as_ref().is_none_or(|(best_t, _)| mtime > *best_t) {
                                        newest = Some((mtime, path));
                                    }
                                }
                            }
                        }
                    }
                    if let Some((_, snap_path)) = newest {
                        if let Ok(bytes) = std::fs::read(&snap_path) {
                            if let Ok(payload) = serde_json::from_slice::<
                                crate::services::history::StoreSnapshotPayload,
                            >(&bytes)
                            {
                                let mut s =
                                    crate::services::privacy::GenealogyPrivacyStore::empty();
                                s.persons = payload.persons;
                                s.families = payload.families;
                                s.events = payload.events;
                                s.notes = payload.notes;
                                s.media = payload.media;
                                s.places = payload.places;
                                s.next_gid_seq = payload.next_gid_seq;
                                s.next_family_gid_seq = payload.next_family_gid_seq;
                                s.next_event_gid_seq = payload.next_event_gid_seq;
                                s.next_place_gid_seq = payload.next_place_gid_seq;
                                s.next_media_gid_seq = payload.next_media_gid_seq;
                                s.is_demo_seed = false;
                                crate::services::genealogy::recompute_all_places_metadata(&mut s);
                                crate::services::genealogy::recompute_all_individuals_vitals(
                                    &mut s,
                                    current_year,
                                    age_threshold,
                                );
                                hydrated_store = Some(s);
                            }
                        }
                    }
                }
            }

            let hydrated_from_disk = hydrated_store.is_some();
            let mut s = match hydrated_store {
                Some(loaded) => loaded,
                None => state.privacy_store.read().await.clone(),
            };

            let file_tokens = crate::services::mcp::load_mcp_tokens_from_dir(backup_dir);
            if !file_tokens.is_empty() {
                crate::services::mcp::merge_mcp_tokens_into_store(&mut s, file_tokens);
            }

            let db_tokens = crate::services::mcp::load_mcp_tokens_from_db(pool).await;
            if !db_tokens.is_empty() {
                crate::services::mcp::merge_mcp_tokens_into_store(&mut s, db_tokens);
            }

            if let Some(seed_pat) =
                crate::services::mcp::load_seed_mcp_token(backup_dir, admin_user_id_str.as_deref())
            {
                crate::services::mcp::merge_mcp_tokens_into_store(&mut s, vec![seed_pat]);
            }

            for pat in &s.personal_access_tokens {
                if !pat.token_hash.trim().is_empty() {
                    crate::services::mcp::upsert_mcp_token_in_db(pool, pat).await;
                }
            }

            let seed_merge_path = backup_dir.join("seed-merge.ged");
            if seed_merge_path.exists() {
                if let Ok(ged_bytes) = std::fs::read(&seed_merge_path) {
                    let _ = crate::services::gedcom::import_gedcom_into_store(
                        &mut s,
                        &ged_bytes,
                        "merge",
                        None,
                        current_year,
                        age_threshold,
                    );
                }
                let _ = std::fs::remove_file(&seed_merge_path);
            }

            if hydrated_from_disk || !s.personal_access_tokens.is_empty() || !s.persons.is_empty() {
                s.persist_to_dir(backup_dir);
                tracing::info!(
                    persons = s.persons.len(),
                    families = s.families.len(),
                    events = s.events.len(),
                    places = s.places.len(),
                    pats = s.personal_access_tokens.len(),
                    "GenealogyPrivacyStore hydrated from backup storage and PostgreSQL"
                );
                *state.privacy_store.write().await = s;
            }
        }

        Ok(())
    }

    pub async fn ensure_user_by_id(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Option<UserRecord>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(u) = store.users.iter().find(|u| u.id == user_id && u.is_active) {
                return Ok(Some(u.clone()));
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let row_opt = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE id = $1 AND is_active = TRUE",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row_opt else {
            return Ok(None);
        };

        let user = Self::row_to_user(&row)?;

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        Ok(Some(user))
    }

    pub async fn ensure_user_and_passkeys_by_email(
        state: &AppState,
        email: &str,
    ) -> Result<Option<UserRecord>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(u) = store
                .users
                .iter()
                .find(|u| u.email.eq_ignore_ascii_case(email) && u.is_active)
            {
                return Ok(Some(u.clone()));
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let row_opt = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE LOWER(email) = LOWER($1) AND is_active = TRUE",
        )
        .bind(email.trim())
        .fetch_optional(pool)
        .await?;

        let Some(row) = row_opt else {
            return Ok(None);
        };

        let user = Self::row_to_user(&row)?;

        let pk_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_all(pool)
        .await?;

        let passkeys: Vec<PasskeyRecord> = pk_rows
            .iter()
            .filter_map(|r| Self::row_to_passkey(r).ok())
            .collect();

        let rc_rows = sqlx::query(
            "SELECT id, user_id, code_hash, used_at, created_at \
             FROM recovery_codes \
             WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_all(pool)
        .await?;

        let recovery_codes: Vec<RecoveryCodeRecord> = rc_rows
            .into_iter()
            .filter_map(|r| {
                Some(RecoveryCodeRecord {
                    id: r.try_get("id").ok()?,
                    user_id: r.try_get("user_id").ok()?,
                    code_hash: r.try_get("code_hash").ok()?,
                    used_at: r.try_get("used_at").ok().flatten(),
                    created_at: r.try_get("created_at").ok()?,
                })
            })
            .collect();

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        for pk in passkeys {
            if let Some(pos) = store.passkeys.iter().position(|p| p.id == pk.id) {
                store.passkeys[pos] = pk;
            } else {
                store.passkeys.push(pk);
            }
        }

        for rc in recovery_codes {
            if let Some(pos) = store.recovery_codes.iter().position(|r| r.id == rc.id) {
                store.recovery_codes[pos] = rc;
            } else {
                store.recovery_codes.push(rc);
            }
        }

        Ok(Some(user))
    }

    pub async fn ensure_passkey_and_user_by_credential_id(
        state: &AppState,
        cred_id: &str,
    ) -> Result<Option<(PasskeyRecord, UserRecord)>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(pk) = store.passkeys.iter().find(|p| p.credential_id == cred_id) {
                if let Some(u) = store
                    .users
                    .iter()
                    .find(|u| u.id == pk.user_id && u.is_active)
                {
                    return Ok(Some((pk.clone(), u.clone())));
                }
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let pk_row = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE credential_id = $1",
        )
        .bind(cred_id.trim())
        .fetch_optional(pool)
        .await?;

        let Some(pk_row) = pk_row else {
            return Ok(None);
        };

        let pk = Self::row_to_passkey(&pk_row)?;

        let user_row = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE id = $1 AND is_active = TRUE",
        )
        .bind(pk.user_id)
        .fetch_optional(pool)
        .await?;

        let Some(user_row) = user_row else {
            return Ok(None);
        };

        let user = Self::row_to_user(&user_row)?;

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.passkeys.iter().position(|p| p.id == pk.id) {
            store.passkeys[pos] = pk.clone();
        } else {
            store.passkeys.push(pk.clone());
        }

        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        Ok(Some((pk, user)))
    }

    pub async fn ensure_passkeys_by_user_id(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<PasskeyRecord>, AppError> {
        let pool = match &state.db {
            Some(p) => p,
            None => {
                let store = state.auth_store.read().await;
                return Ok(store
                    .passkeys
                    .iter()
                    .filter(|p| p.user_id == user_id)
                    .cloned()
                    .collect());
            }
        };

        let pk_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let passkeys: Vec<PasskeyRecord> = pk_rows
            .iter()
            .filter_map(|r| Self::row_to_passkey(r).ok())
            .collect();

        let mut store = state.auth_store.write().await;
        store.passkeys.retain(|p| p.user_id != user_id);
        store.passkeys.extend(passkeys.clone());

        Ok(passkeys)
    }

    pub async fn create_challenge(
        state: &AppState,
        user_id: Option<Uuid>,
    ) -> Result<String, AppError> {
        let challenge = generate_random_token_base64url();
        let expires_at = Utc::now() + ChronoDuration::minutes(5);

        let mut store = state.auth_store.write().await;
        store.challenges.retain(|_, c| c.expires_at > Utc::now());
        store.challenges.insert(
            challenge.clone(),
            ChallengeRecord {
                challenge_token: challenge.clone(),
                user_id,
                expires_at,
            },
        );
        drop(store);

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO auth_challenges (challenge_token, user_id, expires_at)
                 VALUES ($1, $2, $3)",
            )
            .bind(&challenge)
            .bind(user_id)
            .bind(expires_at)
            .execute(pool)
            .await;
        }

        Ok(challenge)
    }

    pub(crate) fn decode_base64_flexible(input: &str) -> Option<Vec<u8>> {
        let trimmed = input.trim();
        let unpadded = trimmed.trim_end_matches('=');
        URL_SAFE_NO_PAD
            .decode(unpadded)
            .or_else(|_| URL_SAFE.decode(trimmed))
            .or_else(|_| STANDARD_NO_PAD.decode(unpadded))
            .or_else(|_| STANDARD.decode(trimmed))
            .ok()
    }

    pub async fn consume_challenge(
        state: &AppState,
        challenge: &str,
        client_data_json: Option<&str>,
    ) -> Result<Option<Uuid>, AppError> {
        let normalized_challenge = challenge.trim().trim_end_matches('=');
        if normalized_challenge.is_empty() {
            return Err(AppError::InvalidOrExpiredChallenge);
        }

        // If client_data_json is provided (base64 or base64url, with or without padding),
        // validate that its embedded challenge matches BEFORE consuming/deleting the challenge.
        // Browsers construct clientDataJSON.challenge as base64url(challengeBytes):
        // - If frontend decoded the base64url challenge string into raw bytes, embedded_challenge == challenge.
        // - If frontend encoded the challenge string as UTF-8 bytes, base64url_decode(embedded_challenge) == challenge.as_bytes().
        if let Some(cdj_b64) = client_data_json {
            let cdj_trimmed = cdj_b64.trim();
            if !cdj_trimmed.is_empty() {
                let decoded = Self::decode_base64_flexible(cdj_trimmed)
                    .ok_or(AppError::InvalidOrExpiredChallenge)?;
                let json_val = serde_json::from_slice::<serde_json::Value>(&decoded)
                    .map_err(|_| AppError::InvalidOrExpiredChallenge)?;
                let embedded_challenge = json_val
                    .get("challenge")
                    .and_then(|v| v.as_str())
                    .ok_or(AppError::InvalidOrExpiredChallenge)?;

                let embedded_norm = embedded_challenge.trim().trim_end_matches('=');
                let direct_match = embedded_norm == normalized_challenge;
                let utf8_double_encoded_match = Self::decode_base64_flexible(embedded_challenge)
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .map(|decoded_str| {
                        decoded_str.trim().trim_end_matches('=') == normalized_challenge
                    })
                    .unwrap_or(false);

                if !direct_match && !utf8_double_encoded_match {
                    return Err(AppError::InvalidOrExpiredChallenge);
                }
            }
        }

        let now = Utc::now();
        let mut store = state.auth_store.write().await;
        if let Some(rec) = store.challenges.remove(challenge) {
            if rec.expires_at < now {
                return Err(AppError::InvalidOrExpiredChallenge);
            }
            drop(store);
            if let Some(pool) = &state.db {
                let _ = sqlx::query("DELETE FROM auth_challenges WHERE challenge_token = $1")
                    .bind(challenge)
                    .execute(pool)
                    .await;
            }
            return Ok(rec.user_id);
        }
        drop(store);

        if let Some(pool) = &state.db {
            if let Ok(Some(row)) = sqlx::query(
                "DELETE FROM auth_challenges
                 WHERE challenge_token = $1 AND expires_at > NOW()
                 RETURNING user_id",
            )
            .bind(challenge)
            .fetch_optional(pool)
            .await
            {
                let uid: Option<Uuid> = row.try_get("user_id").ok().flatten();
                return Ok(uid);
            }
        }

        Err(AppError::InvalidOrExpiredChallenge)
    }
}
