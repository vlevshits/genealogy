pub mod auth;
pub mod health;
pub mod system;

use axum::{
    http::{header, Method},
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    error::{ErrorDetail, ErrorResponse},
    models::gendate::{CalendarType, GenDate, GenDateMod, PartialDate},
    routes::{
        auth::{
            complete_initial_setup, create_add_passkey_challenge, create_admin_invite,
            create_login_challenge, create_register_challenge, create_user_sign_in_link,
            delete_passkey, get_current_user, get_setup_status, list_admin_invites,
            list_admin_users, list_passkeys, list_sessions, login_with_password,
            login_with_recovery_code, logout, preview_invitation, regenerate_recovery_codes,
            rename_passkey, request_magic_link, revoke_admin_invite, revoke_all_user_sessions,
            revoke_other_sessions, revoke_session, set_user_password, update_current_user,
            verify_add_passkey, verify_invite_registration, verify_magic_link_get,
            verify_magic_link_post, verify_passkey_login,
        },
        health::{health_check, HealthResponse},
        system::{
            download_admin_backup, get_admin_settings, get_admin_system, get_system_info,
            list_admin_backups, trigger_admin_backup, update_admin_settings, AdminBackupStatus,
            AdminDatabaseStatus, AdminJobsStatus, AdminStorageStatus, AdminSystemResponse,
            BackupListResponse, SystemInfoResponse, TriggerBackupRequest, TriggerBackupResponse,
        },
    },
    services::{
        auth::{
            AdminSignInLinkResponse, AdminUserListResponse, AdminUserSummaryDto,
            AuthSessionResponse, CreateInviteRequest, InvitationDto, InvitationListResponse,
            InviteCreatedResponse, InvitePreviewResponse, InviteRegisterRequest,
            LoginChallengeRequest, LoginChallengeResponse, MagicLinkRequest,
            MagicLinkRequestResponse, MagicLinkVerifyRequest, PasskeyCredentialDto,
            PasskeyListResponse, PasskeyLoginVerifyRequest, PasskeyRegistrationInput,
            PasswordLoginRequest, RecoveryCodeLoginRequest, RecoveryCodeLoginResponse,
            RegenerateRecoveryCodesResponse, RenamePasskeyRequest, SessionDto, SessionListResponse,
            SetPasswordRequest, SetupCompleteResponse, SetupRequest, SetupStatusResponse,
            UpdateProfileRequest, UserDto, WebAuthnChallengeResponse,
        },
        backup::{BackupArchiveInfo, BackupFileEntry, BackupManifest, BackupStats},
        security::csrf_protection_middleware,
    },
    state::{AppState, InstanceSettingsDto, UpdateInstanceSettingsRequest},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        system::get_system_info,
        system::get_admin_system,
        system::get_admin_settings,
        system::update_admin_settings,
        system::list_admin_backups,
        system::trigger_admin_backup,
        system::download_admin_backup,
        auth::get_setup_status,
        auth::create_register_challenge,
        auth::complete_initial_setup,
        auth::preview_invitation,
        auth::verify_invite_registration,
        auth::create_login_challenge,
        auth::verify_passkey_login,
        auth::login_with_password,
        auth::request_magic_link,
        auth::verify_magic_link_get,
        auth::verify_magic_link_post,
        auth::login_with_recovery_code,
        auth::get_current_user,
        auth::update_current_user,
        auth::logout,
        auth::list_passkeys,
        auth::create_add_passkey_challenge,
        auth::verify_add_passkey,
        auth::rename_passkey,
        auth::delete_passkey,
        auth::regenerate_recovery_codes,
        auth::set_user_password,
        auth::list_sessions,
        auth::revoke_session,
        auth::revoke_other_sessions,
        auth::list_admin_users,
        auth::create_admin_invite,
        auth::list_admin_invites,
        auth::revoke_admin_invite,
        auth::create_user_sign_in_link,
        auth::revoke_all_user_sessions,
    ),
    components(
        schemas(
            HealthResponse,
            ErrorResponse,
            ErrorDetail,
            GenDate,
            GenDateMod,
            CalendarType,
            PartialDate,
            SystemInfoResponse,
            AdminSystemResponse,
            AdminDatabaseStatus,
            AdminStorageStatus,
            AdminBackupStatus,
            AdminJobsStatus,
            InstanceSettingsDto,
            UpdateInstanceSettingsRequest,
            BackupArchiveInfo,
            BackupFileEntry,
            BackupStats,
            BackupManifest,
            BackupListResponse,
            TriggerBackupRequest,
            TriggerBackupResponse,
            SetupStatusResponse,
            WebAuthnChallengeResponse,
            LoginChallengeRequest,
            LoginChallengeResponse,
            PasskeyRegistrationInput,
            SetupRequest,
            InviteRegisterRequest,
            UserDto,
            SetupCompleteResponse,
            AuthSessionResponse,
            PasskeyLoginVerifyRequest,
            PasswordLoginRequest,
            MagicLinkRequest,
            MagicLinkRequestResponse,
            MagicLinkVerifyRequest,
            RecoveryCodeLoginRequest,
            RecoveryCodeLoginResponse,
            RegenerateRecoveryCodesResponse,
            PasskeyCredentialDto,
            PasskeyListResponse,
            RenamePasskeyRequest,
            SetPasswordRequest,
            UpdateProfileRequest,
            SessionDto,
            SessionListResponse,
            CreateInviteRequest,
            InvitationDto,
            InviteCreatedResponse,
            InvitationListResponse,
            InvitePreviewResponse,
            AdminUserSummaryDto,
            AdminUserListResponse,
            AdminSignInLinkResponse,
        )
    ),
    tags(
        (name = "System", description = "System operations and health monitoring"),
        (name = "Auth", description = "Authentication, Passkeys (WebAuthn), sessions, and recovery"),
        (name = "Admin", description = "Instance administration, settings, family members, and backups")
    ),
    info(
        title = "Genealogy Family Tree API",
        version = "0.1.0",
        description = "Lightweight, privacy-first family tree archival API backend"
    )
)]
pub struct ApiDoc;

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
        // Admin Operations, Users & Invitations
        .route("/admin/system", get(get_admin_system))
        .route(
            "/admin/settings",
            get(get_admin_settings).patch(update_admin_settings),
        )
        .route(
            "/admin/backups",
            get(list_admin_backups).post(trigger_admin_backup),
        )
        .route("/admin/backups/{filename}", get(download_admin_backup))
        .route("/admin/users", get(list_admin_users))
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
        ));

    Router::new()
        .nest("/api/v1", api_v1)
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
