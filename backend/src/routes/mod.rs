pub mod auth;
pub mod health;
pub mod rbac;
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
            revoke_other_sessions, revoke_session, set_me_home_person, set_user_password,
            update_admin_user, update_current_user, update_me_preferences, verify_add_passkey,
            verify_invite_registration, verify_magic_link_get, verify_magic_link_post,
            verify_passkey_login,
        },
        health::{health_check, HealthResponse},
        rbac::{
            analyze_gedcom, create_event, create_face_tag, create_family, create_manual_snapshot,
            create_media, create_media_link, create_note, create_person, create_place,
            delete_event, delete_face_tag, delete_family, delete_media, delete_media_link,
            delete_note, delete_person, delete_place, dismiss_quality_warning,
            dismiss_quality_warning_by_id, export_gedcom, export_gedcom_post, export_printable_pdf,
            get_anniversaries, get_audit_history, get_dashboard, get_data_quality_warnings,
            get_family, get_fan_chart, get_kinship, get_media_crop, get_media_detail,
            get_media_file, get_onboarding_status, get_patronymic_suggestion, get_person,
            get_person_delete_impact, get_person_profile, get_place_detail, get_places_map,
            get_quality_summary, get_record_history, get_record_version, get_stats, get_tree,
            get_tree_view, global_search, import_gedcom, list_duplicate_candidates,
            list_event_sources, list_events, list_families, list_media_archive, list_notes,
            list_people_directory, list_people_surnames, list_places_directory,
            list_quality_warnings, list_snapshots, mark_pair_not_duplicate,
            match_people_candidates, merge_duplicate_people, merge_place, preview_merge_people_get,
            preview_merge_people_post, quick_add_relative, restore_deleted_person,
            rollback_to_snapshot, start_with_self, undo_history_transaction, update_event,
            update_face_tag, update_family, update_media, update_note, update_person, update_place,
            AdminToolOperationResponse, CreateEventRequest, CreateFaceTagRequest,
            CreateFamilyRequest, CreateMediaLinkRequest, CreateMediaRequest, CreateNoteRequest,
            CreatePersonRequest, CreatePlaceRequest, DeleteImpactDto, EventFilterQuery,
            EventListResponse, FamilyListResponse, MediaArchiveResponse, MediaCropQuery,
            MediaFilterQuery, MergePlaceRequest, NoteFilterQuery, NoteListResponse,
            OnboardingStatusResponse, PatronymicSuggestionQuery, PatronymicSuggestionResponse,
            PeopleDirectoryItemDto, PeopleDirectoryQuery, PeopleDirectoryResponse,
            PersonMergeRequest, PersonProfileQuery, PersonProfileResponseDto, PlaceDetailDto,
            PlacesDirectoryResponse, PlacesFilterQuery, ProfileCapabilitiesDto,
            ProfileChildSummaryDto, ProfileFamiliesDto, ProfileLifeEventDto,
            ProfileParentFamilyDto, ProfileRelativeSummaryDto, ProfileSourceSummaryDto,
            ProfileSpouseFamilyDto, QuickAddRelativeRequest, QuickAddRelativeResponse,
            SearchActionDto, SearchMediaResultDto, SearchPersonResultDto, SearchPlaceResultDto,
            SearchQuery, SearchResultResponseDto, SourceAutocompleteResponse, SourceSearchQuery,
            StartWithSelfRequest, StartWithSelfResponse, SurnameIndexEntryDto,
            SurnameLetterGroupDto, SurnamesIndexQuery, SurnamesIndexResponse, TreeEdgeDto,
            TreeResponse, TreeViewQuery, TreeViewResponse, UpdateEventRequest,
            UpdateFaceTagRequest, UpdateFamilyRequest, UpdateMediaRequest, UpdateNoteRequest,
            UpdatePersonRequest, UpdatePlaceRequest,
        },
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
            SetHomePersonRequest, SetPasswordRequest, SetupCompleteResponse, SetupRequest,
            SetupStatusResponse, UpdateAdminUserRequest, UpdateProfileRequest, UserDto,
            WebAuthnChallengeResponse,
        },
        backup::{BackupArchiveInfo, BackupFileEntry, BackupManifest, BackupStats},
        dashboard::{
            AnniversariesListResponse, AnniversariesQuery, AnniversaryItemDto,
            DashboardQualityWidgetDto, DashboardResponse, TreeStatsResponse,
        },
        gedcom::{
            GedcomAnalysisReport, GedcomAnalyzeRequest, GedcomDialectSummary, GedcomExportRequest,
            GedcomImportRequest, GedcomImportResult,
        },
        genealogy::{
            FanChartNodeDto, FanChartQuery, FanChartResponse, KinshipEdgeStepDto, KinshipQuery,
            KinshipResponse, PlaceMapMarkerDto, PlaceMapMarkerPersonDto, PlacesMapResponse,
            ProfileKinshipDto,
        },
        history::{
            ChangeLogTransactionGroupDto, CreateSnapshotRequest, EntityVersionResponse,
            FieldDiffDto, HistoryFeedQuery, HistoryFeedResponse, RecordHistoryItemDto,
            RecordHistoryResponse, RollbackSnapshotRequest, RollbackSnapshotResponse,
            SnapshotListResponse, UndoTransactionResponse,
        },
        privacy::{
            AvatarCropDto, ChangeLogEntryDto, DismissedWarningDto, FaceTagDto, FamilyChildDto,
            FamilyDto, IndividualAltNameDto, LifeEventDto, MediaAssetSummaryDto, MediaLinkDto,
            NoteDto, PersonNameDto, PlaceAltNameDto, PlaceBreadcrumbDto, PlaceSummaryDto,
            TreePersonNodeDto, TreeSnapshotDto, TreeSnapshotRecordCountsDto, VitalEventSummaryDto,
        },
        quality::{
            DismissWarningRequest, DismissWarningResponse, DuplicateCandidatePairDto,
            DuplicateCandidatesFilterQuery, DuplicateCandidatesListResponse,
            MergeCombinedCountsDto, MergeFieldConflictDto, MergePreviewQuery, MergePreviewResponse,
            NotDuplicatePairRequest, NotDuplicatePairResponse, PersonMatchCandidateDto,
            PersonMatchQuery, PersonMatchResponse, PersonMergeResponse, QualityCategoryCountsDto,
            QualitySeverityCountsDto, QualitySummaryResponse, QualityWarningDto,
            QualityWarningPersonRefDto, QualityWarningsFilterQuery, QualityWarningsListResponse,
        },
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
        auth::update_me_preferences,
        auth::set_me_home_person,
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
        auth::update_admin_user,
        auth::create_admin_invite,
        auth::list_admin_invites,
        auth::revoke_admin_invite,
        auth::create_user_sign_in_link,
        auth::revoke_all_user_sessions,
        rbac::get_onboarding_status,
        rbac::start_with_self,
        rbac::get_tree,
        rbac::get_tree_view,
        rbac::get_fan_chart,
        rbac::get_kinship,
        rbac::get_patronymic_suggestion,
        rbac::get_person,
        rbac::get_person_profile,
        rbac::create_person,
        rbac::update_person,
        rbac::delete_person,
        rbac::quick_add_relative,
        rbac::get_person_delete_impact,
        rbac::restore_deleted_person,
        rbac::list_families,
        rbac::get_family,
        rbac::create_family,
        rbac::update_family,
        rbac::delete_family,
        rbac::list_events,
        rbac::create_event,
        rbac::update_event,
        rbac::delete_event,
        rbac::list_event_sources,
        rbac::list_notes,
        rbac::create_note,
        rbac::update_note,
        rbac::delete_note,
        rbac::get_media_file,
        rbac::get_media_crop,
        rbac::global_search,
        rbac::list_people_directory,
        rbac::list_people_surnames,
        rbac::list_places_directory,
        rbac::get_places_map,
        rbac::create_place,
        rbac::get_place_detail,
        rbac::update_place,
        rbac::delete_place,
        rbac::merge_place,
        rbac::list_media_archive,
        rbac::create_media,
        rbac::get_media_detail,
        rbac::update_media,
        rbac::delete_media,
        rbac::create_face_tag,
        rbac::update_face_tag,
        rbac::delete_face_tag,
        rbac::create_media_link,
        rbac::delete_media_link,
        rbac::analyze_gedcom,
        rbac::export_gedcom,
        rbac::export_gedcom_post,
        rbac::import_gedcom,
        rbac::export_printable_pdf,
        rbac::get_audit_history,
        rbac::get_record_history,
        rbac::undo_history_transaction,
        rbac::list_snapshots,
        rbac::create_manual_snapshot,
        rbac::rollback_to_snapshot,
        rbac::get_record_version,
        rbac::get_data_quality_warnings,
        rbac::get_quality_summary,
        rbac::list_quality_warnings,
        rbac::dismiss_quality_warning_by_id,
        rbac::dismiss_quality_warning,
        rbac::list_duplicate_candidates,
        rbac::mark_pair_not_duplicate,
        rbac::match_people_candidates,
        rbac::preview_merge_people_get,
        rbac::preview_merge_people_post,
        rbac::merge_duplicate_people,
        rbac::get_dashboard,
        rbac::get_anniversaries,
        rbac::get_stats,
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
            SetHomePersonRequest,
            UpdateProfileRequest,
            SessionDto,
            SessionListResponse,
            CreateInviteRequest,
            InvitationDto,
            InviteCreatedResponse,
            InvitationListResponse,
            InvitePreviewResponse,
            AdminUserSummaryDto,
            UpdateAdminUserRequest,
            AdminUserListResponse,
            AdminSignInLinkResponse,
            PersonNameDto,
            IndividualAltNameDto,
            VitalEventSummaryDto,
            AvatarCropDto,
            TreePersonNodeDto,
            FamilyChildDto,
            FamilyDto,
            LifeEventDto,
            NoteDto,
            FaceTagDto,
            MediaLinkDto,
            MediaAssetSummaryDto,
            PlaceAltNameDto,
            PlaceBreadcrumbDto,
            PlaceSummaryDto,
            ChangeLogEntryDto,
            TreeSnapshotRecordCountsDto,
            TreeSnapshotDto,
            DismissedWarningDto,
            HistoryFeedQuery,
            FieldDiffDto,
            ChangeLogTransactionGroupDto,
            HistoryFeedResponse,
            RecordHistoryItemDto,
            RecordHistoryResponse,
            UndoTransactionResponse,
            CreateSnapshotRequest,
            SnapshotListResponse,
            RollbackSnapshotRequest,
            RollbackSnapshotResponse,
            EntityVersionResponse,
            OnboardingStatusResponse,
            StartWithSelfRequest,
            StartWithSelfResponse,
            TreeResponse,
            TreeViewQuery,
            TreeEdgeDto,
            TreeViewResponse,
            FanChartQuery,
            FanChartNodeDto,
            FanChartResponse,
            KinshipQuery,
            KinshipEdgeStepDto,
            KinshipResponse,
            PersonProfileQuery,
            ProfileKinshipDto,
            ProfileRelativeSummaryDto,
            ProfileChildSummaryDto,
            ProfileParentFamilyDto,
            ProfileSpouseFamilyDto,
            ProfileFamiliesDto,
            ProfileLifeEventDto,
            ProfileSourceSummaryDto,
            ProfileCapabilitiesDto,
            PersonProfileResponseDto,
            CreatePersonRequest,
            UpdatePersonRequest,
            PatronymicSuggestionQuery,
            PatronymicSuggestionResponse,
            QuickAddRelativeRequest,
            QuickAddRelativeResponse,
            DeleteImpactDto,
            CreateFamilyRequest,
            UpdateFamilyRequest,
            FamilyListResponse,
            EventFilterQuery,
            CreateEventRequest,
            UpdateEventRequest,
            EventListResponse,
            SourceSearchQuery,
            SourceAutocompleteResponse,
            NoteFilterQuery,
            CreateNoteRequest,
            UpdateNoteRequest,
            NoteListResponse,
            PlacesFilterQuery,
            CreatePlaceRequest,
            UpdatePlaceRequest,
            MergePlaceRequest,
            PlaceDetailDto,
            PlaceMapMarkerPersonDto,
            PlaceMapMarkerDto,
            PlacesMapResponse,
            SearchQuery,
            SearchPersonResultDto,
            SearchPlaceResultDto,
            SearchMediaResultDto,
            SearchActionDto,
            SearchResultResponseDto,
            PeopleDirectoryQuery,
            PeopleDirectoryItemDto,
            PeopleDirectoryResponse,
            SurnamesIndexQuery,
            SurnameIndexEntryDto,
            SurnameLetterGroupDto,
            SurnamesIndexResponse,
            PlacesDirectoryResponse,
            MediaFilterQuery,
            CreateMediaRequest,
            UpdateMediaRequest,
            MediaCropQuery,
            CreateFaceTagRequest,
            UpdateFaceTagRequest,
            CreateMediaLinkRequest,
            MediaArchiveResponse,
            GedcomDialectSummary,
            GedcomAnalysisReport,
            GedcomAnalyzeRequest,
            GedcomImportRequest,
            GedcomImportResult,
            GedcomExportRequest,
            AdminToolOperationResponse,
            QualityWarningPersonRefDto,
            QualityWarningDto,
            QualityCategoryCountsDto,
            QualitySeverityCountsDto,
            QualitySummaryResponse,
            QualityWarningsFilterQuery,
            QualityWarningsListResponse,
            DismissWarningRequest,
            DismissWarningResponse,
            DuplicateCandidatePairDto,
            DuplicateCandidatesFilterQuery,
            DuplicateCandidatesListResponse,
            NotDuplicatePairRequest,
            NotDuplicatePairResponse,
            PersonMatchQuery,
            PersonMatchCandidateDto,
            PersonMatchResponse,
            MergeCombinedCountsDto,
            MergeFieldConflictDto,
            MergePreviewQuery,
            MergePreviewResponse,
            PersonMergeRequest,
            PersonMergeResponse,
            AnniversariesQuery,
            AnniversaryItemDto,
            AnniversariesListResponse,
            DashboardQualityWidgetDto,
            DashboardResponse,
            TreeStatsResponse,
        )
    ),
    tags(
        (name = "System", description = "System operations and health monitoring"),
        (name = "Auth", description = "Authentication, Passkeys (WebAuthn), sessions, and recovery"),
        (name = "Admin", description = "Instance administration, settings, family members, and backups"),
        (name = "Tree & Privacy", description = "RBAC-enforced tree nodes, living-person privacy masking, and media shield"),
        (name = "Media & Avatars", description = "Media gallery, file/crop streaming, face tagging, primary avatars, and entity linking"),
        (name = "GEDCOM", description = "GEDCOM 5.5.1 and 7.0 pre-import analysis, import, dialect normalization, and export")
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
        .route("/gedcom/analyze", post(analyze_gedcom))
        .route(
            "/gedcom/export",
            get(export_gedcom).post(export_gedcom_post),
        )
        .route("/gedcom/import", post(import_gedcom))
        .route("/export/pdf", post(export_printable_pdf))
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
        ));

    Router::new()
        .nest("/api/v1", api_v1)
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
