pub mod auth;
pub mod health;
pub mod openapi;
pub mod rbac;
pub mod system;

pub use openapi::ApiDoc;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{
        auth::{
            complete_initial_setup, create_add_passkey_challenge, create_admin_invite,
            create_login_challenge, create_register_challenge, create_user_sign_in_link,
            delete_passkey, get_current_user, get_setup_status, list_admin_invites,
            list_admin_users, list_passkeys, list_sessions, login_with_password,
            login_with_recovery_code, logout, preview_invitation, regenerate_recovery_codes,
            rename_passkey, request_magic_link, revoke_admin_invite, revoke_all_user_sessions,
            revoke_other_sessions, revoke_session, set_me_home_person, set_user_password,
            update_admin_user, update_current_user, update_me_preferences, verify_add_passkey,
            verify_invite_registration, verify_magic_link_get, verify_magic_link_post,
            verify_passkey_login,
        },
        health::health_check,
        rbac::{
            analyze_gedcom, batch_fix_quality_places, commit_ai_story_draft_handler, create_event,
            create_face_tag, create_family, create_manual_snapshot, create_mcp_token_handler,
            create_media, create_media_link, create_note, create_person, create_place,
            delete_ai_story_draft_handler, delete_event, delete_face_tag, delete_family,
            delete_media, delete_media_link, delete_note, delete_person, delete_place,
            dismiss_quality_place_issue, dismiss_quality_warning, dismiss_quality_warning_by_id,
            export_gedcom, export_gedcom_post, export_printable_pdf, fix_quality_place_hierarchy,
            get_admin_ai_settings_handler, get_ai_story_draft_handler, get_anniversaries,
            get_audit_history, get_dashboard, get_data_quality_warnings, get_family, get_fan_chart,
            get_kinship, get_media_crop, get_media_detail, get_media_file, get_onboarding_status,
            get_patronymic_suggestion, get_pdf_preview_info, get_person, get_person_delete_impact,
            get_person_profile, get_place_detail, get_places_map, get_quality_places_analysis,
            get_quality_summary, get_record_history, get_record_version, get_stats, get_tree,
            get_tree_view, global_search, import_gedcom, list_ai_story_drafts_handler,
            list_duplicate_candidates, list_event_sources, list_events, list_families,
            list_mcp_tokens_handler, list_media_archive, list_notes, list_people_directory,
            list_people_surnames, list_places_directory, list_quality_warnings, list_snapshots,
            mark_pair_not_duplicate, match_people_candidates, mcp_jsonrpc_handler, mcp_sse_handler,
            merge_duplicate_people, merge_place, parse_ai_story_handler, preview_merge_people_get,
            preview_merge_people_post, preview_quality_place_issue, quick_add_relative,
            reconcile_ai_story_draft_handler, reconcile_gedcom_media_bundle,
            refine_ai_story_draft_by_id_handler, refine_ai_story_draft_handler,
            restore_deleted_person, revoke_mcp_token_handler, rollback_to_snapshot,
            start_with_self, transcribe_ai_story_handler, undo_history_transaction,
            update_admin_ai_settings_handler, update_ai_story_draft_handler, update_event,
            update_face_tag, update_family, update_media, update_note, update_person, update_place,
            upload_media_bundle,
        },
        system::{
            download_admin_backup, get_admin_settings, get_admin_system, get_system_info,
            list_admin_backups, trigger_admin_backup, update_admin_settings,
        },
    },
    services::security::{csrf_protection_middleware, request_context_middleware},
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::IF_MATCH,
            header::HeaderName::from_static("x-requested-with"),
        ])
        .allow_origin(Any);

    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .route("/system/info", get(get_system_info))
        // Auth & Passkeys
        .route("/auth/setup/status", get(get_setup_status))
        .route("/auth/setup", post(complete_initial_setup))
        .route("/auth/register/challenge", post(create_register_challenge))
        .route("/auth/register/verify", post(verify_invite_registration))
        .route("/auth/register", post(verify_invite_registration))
        .route("/auth/invites/{token}", get(preview_invitation))
        .route("/auth/login/challenge", post(create_login_challenge))
        .route("/auth/login/verify", post(verify_passkey_login))
        .route("/auth/password/login", post(login_with_password))
        .route("/auth/magic-link/request", post(request_magic_link))
        .route(
            "/auth/magic-link/verify",
            get(verify_magic_link_get).post(verify_magic_link_post),
        )
        .route("/auth/recovery-code", post(login_with_recovery_code))
        .route("/auth/me", get(get_current_user).patch(update_current_user))
        .route("/me", get(get_current_user).patch(update_me_preferences))
        .route("/me/home-person", put(set_me_home_person))
        .route("/auth/logout", post(logout))
        .route("/auth/passkeys", get(list_passkeys))
        .route(
            "/auth/passkeys/add/challenge",
            post(create_add_passkey_challenge),
        )
        .route("/auth/passkeys/add/verify", post(verify_add_passkey))
        .route(
            "/auth/passkeys/{id}",
            patch(rename_passkey).delete(delete_passkey),
        )
        .route(
            "/auth/recovery-codes/regenerate",
            post(regenerate_recovery_codes),
        )
        .route("/auth/password", put(set_user_password))
        .route(
            "/auth/sessions",
            get(list_sessions).delete(revoke_other_sessions),
        )
        .route("/auth/sessions/{id}", delete(revoke_session))
        // Onboarding & Home Person
        .route("/onboarding/status", get(get_onboarding_status))
        .route("/onboarding/start-with-self", post(start_with_self))
        // Dashboard, Anniversaries & Stats
        .route("/dashboard", get(get_dashboard))
        .route("/anniversaries", get(get_anniversaries))
        .route("/stats", get(get_stats))
        // Core Genealogy Data Model, Person/Family/Event Editing, Places & RBAC Endpoints
        .route("/tree", get(get_tree))
        .route("/tree/view", get(get_tree_view))
        .route("/tree/fan", get(get_fan_chart))
        .route("/kinship", get(get_kinship))
        .route(
            "/persons/suggest-patronymic",
            get(get_patronymic_suggestion),
        )
        .route("/persons", post(create_person))
        .route(
            "/persons/{id}",
            get(get_person).patch(update_person).delete(delete_person),
        )
        .route("/persons/{id}/profile", get(get_person_profile))
        .route("/persons/{id}/quick-add", post(quick_add_relative))
        .route("/persons/{id}/delete-impact", get(get_person_delete_impact))
        .route("/persons/{id}/restore", post(restore_deleted_person))
        .route("/families", get(list_families).post(create_family))
        .route(
            "/families/{id}",
            get(get_family).patch(update_family).delete(delete_family),
        )
        .route("/events/sources", get(list_event_sources))
        .route("/events", get(list_events).post(create_event))
        .route("/events/{id}", patch(update_event).delete(delete_event))
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/{id}", patch(update_note).delete(delete_note))
        .route("/media", get(list_media_archive).post(create_media))
        .route(
            "/media/bundle",
            post(upload_media_bundle).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route(
            "/media/{id}",
            get(get_media_detail)
                .patch(update_media)
                .delete(delete_media),
        )
        .route("/media/{id}/file", get(get_media_file))
        .route("/media/{id}/crop", get(get_media_crop))
        .route("/media/{id}/tags", post(create_face_tag))
        .route(
            "/media/{id}/tags/{tag_id}",
            patch(update_face_tag).delete(delete_face_tag),
        )
        .route("/media/{id}/links", post(create_media_link))
        .route("/media/{id}/links/{link_id}", delete(delete_media_link))
        .route("/search", get(global_search))
        .route("/people", get(list_people_directory))
        .route("/people/surnames", get(list_people_surnames))
        .route("/people/match", get(match_people_candidates))
        .route(
            "/people/merge/preview",
            get(preview_merge_people_get).post(preview_merge_people_post),
        )
        .route("/people/merge/not-duplicate", post(mark_pair_not_duplicate))
        .route("/people/merge", post(merge_duplicate_people))
        .route("/places", get(list_places_directory).post(create_place))
        .route("/places/map", get(get_places_map))
        .route(
            "/places/{id}",
            get(get_place_detail)
                .patch(update_place)
                .delete(delete_place),
        )
        .route("/places/{id}/merge", post(merge_place))
        .route(
            "/gedcom/analyze",
            post(analyze_gedcom).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route(
            "/gedcom/export",
            get(export_gedcom).post(export_gedcom_post),
        )
        .route(
            "/gedcom/import",
            post(import_gedcom).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route(
            "/gedcom/media-bundle",
            post(reconcile_gedcom_media_bundle).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/export/pdf", post(export_printable_pdf))
        .route("/export/pdf/preview-info", post(get_pdf_preview_info))
        .route("/history", get(get_audit_history))
        .route(
            "/history/{entity_type}/{entity_id}",
            get(get_record_history),
        )
        .route("/history/tx/{tx_id}/undo", post(undo_history_transaction))
        .route("/history/{id}/restore", post(restore_deleted_person))
        .route(
            "/snapshots",
            get(list_snapshots).post(create_manual_snapshot),
        )
        .route("/snapshots/{id}/rollback", post(rollback_to_snapshot))
        .route("/{entity_type}/{id}/version", get(get_record_version))
        .route("/quality", get(get_data_quality_warnings))
        .route("/quality/summary", get(get_quality_summary))
        .route("/quality/warnings", get(list_quality_warnings))
        .route("/quality/warnings/dismiss", post(dismiss_quality_warning))
        .route(
            "/quality/warnings/{id}/dismiss",
            post(dismiss_quality_warning_by_id),
        )
        .route("/quality/duplicates", get(list_duplicate_candidates))
        .route(
            "/quality/duplicates/not-duplicate",
            post(mark_pair_not_duplicate),
        )
        .route("/quality/places/analysis", get(get_quality_places_analysis))
        .route("/quality/places/preview", get(preview_quality_place_issue))
        .route("/quality/places/fix", post(fix_quality_place_hierarchy))
        .route("/quality/places/batch-fix", post(batch_fix_quality_places))
        .route("/quality/places/dismiss", post(dismiss_quality_place_issue))
        // AI Family Story Intake & Protected MCP Server
        .route(
            "/ai/transcribe",
            post(transcribe_ai_story_handler).layer(DefaultBodyLimit::max(20 * 1024 * 1024)),
        )
        .route("/ai/parse-story", post(parse_ai_story_handler))
        .route("/ai/refine-draft", post(refine_ai_story_draft_handler))
        .route("/ai/drafts", get(list_ai_story_drafts_handler))
        .route(
            "/ai/drafts/{id}",
            get(get_ai_story_draft_handler)
                .patch(update_ai_story_draft_handler)
                .delete(delete_ai_story_draft_handler),
        )
        .route(
            "/ai/drafts/{id}/refine",
            post(refine_ai_story_draft_by_id_handler),
        )
        .route(
            "/ai/drafts/{id}/reconcile",
            post(reconcile_ai_story_draft_handler),
        )
        .route(
            "/ai/drafts/{id}/commit",
            post(commit_ai_story_draft_handler),
        )
        .route(
            "/mcp/tokens",
            get(list_mcp_tokens_handler).post(create_mcp_token_handler),
        )
        .route("/mcp/tokens/{id}", delete(revoke_mcp_token_handler))
        .route(
            "/auth/mcp-tokens",
            get(list_mcp_tokens_handler).post(create_mcp_token_handler),
        )
        .route("/auth/mcp-tokens/{id}", delete(revoke_mcp_token_handler))
        .route("/mcp", post(mcp_jsonrpc_handler))
        .route("/mcp/sse", get(mcp_sse_handler))
        // Admin Operations, Users & Invitations
        .route("/admin/system", get(get_admin_system))
        .route(
            "/admin/settings",
            get(get_admin_settings).patch(update_admin_settings),
        )
        .route(
            "/admin/ai-settings",
            get(get_admin_ai_settings_handler).patch(update_admin_ai_settings_handler),
        )
        .route(
            "/admin/backups",
            get(list_admin_backups).post(trigger_admin_backup),
        )
        .route("/admin/backups/{filename}", get(download_admin_backup))
        .route("/admin/users", get(list_admin_users))
        .route("/admin/users/{id}", patch(update_admin_user))
        .route(
            "/admin/users/{id}/sign-in-link",
            post(create_user_sign_in_link),
        )
        .route(
            "/admin/users/{id}/sessions",
            delete(revoke_all_user_sessions),
        )
        .route(
            "/admin/invites",
            get(list_admin_invites).post(create_admin_invite),
        )
        .route("/admin/invites/{id}", delete(revoke_admin_invite))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_protection_middleware,
        ))
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024));

    let mut root_router = Router::new()
        .nest("/api/v1", api_v1)
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()));

    if let Some(ref static_path) = state.config.frontend_static_path {
        let path = std::path::PathBuf::from(static_path);
        if path.exists() {
            let sw_file = path.join("sw.js");
            let manifest_file = path.join("manifest.webmanifest");
            let index_file = path.join("index.html");

            if sw_file.exists() {
                root_router = root_router.route(
                    "/sw.js",
                    get(move || serve_static_no_cache(sw_file.clone(), "application/javascript")),
                );
            }
            if manifest_file.exists() {
                root_router = root_router.route(
                    "/manifest.webmanifest",
                    get(move || {
                        serve_static_no_cache(manifest_file.clone(), "application/manifest+json")
                    }),
                );
            }

            if index_file.exists() {
                let serve_dir = ServeDir::new(&path).fallback(ServeFile::new(index_file));
                root_router = root_router.fallback_service(serve_dir);
            }
        }
    }

    root_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_context_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn serve_static_no_cache(
    path: std::path::PathBuf,
    content_type: &'static str,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match tokio::fs::read(&path).await {
        Ok(bytes) => axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, content_type)
            .header(
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            )
            .body(axum::body::Body::from(bytes))
            .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}
