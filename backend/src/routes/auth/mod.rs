#![allow(unused_imports)]
pub(crate) use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
pub(crate) use serde::Deserialize;
pub(crate) use uuid::Uuid;

pub(crate) use crate::{
    error::{AppError, ErrorResponse},
    services::{
        auth::{
            AdminSignInLinkResponse, AdminUserListResponse, AdminUserSummaryDto, AuthService,
            AuthSessionResponse, CreateInviteRequest, InvitationListResponse,
            InviteCreatedResponse, InvitePreviewResponse, InviteRegisterRequest,
            LoginChallengeRequest, LoginChallengeResponse, MagicLinkRequest,
            MagicLinkRequestResponse, MagicLinkVerifyRequest, PasskeyCredentialDto,
            PasskeyListResponse, PasskeyLoginVerifyRequest, PasskeyRegistrationInput,
            PasswordLoginRequest, RecoveryCodeLoginRequest, RecoveryCodeLoginResponse,
            RegenerateRecoveryCodesResponse, RenamePasskeyRequest, SessionDto, SessionListResponse,
            SessionRecord, SetHomePersonRequest, SetPasswordRequest, SetupCompleteResponse,
            SetupRequest, SetupStatusResponse, UpdateAdminUserRequest, UpdateProfileRequest,
            UserDto, UserRecord, WebAuthnChallengeResponse,
        },
        rbac::RequireAdmin,
        security::RequestContext,
    },
    state::AppState,
};

pub mod login;
pub use login::*;
pub mod passkeys_sessions;
pub use passkeys_sessions::*;
pub mod admin;
pub use admin::*;
