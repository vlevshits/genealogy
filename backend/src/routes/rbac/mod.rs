#![allow(unused_imports)]
pub(crate) use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
pub(crate) use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
pub(crate) use chrono::{Datelike, Utc};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use sha2::{Digest, Sha256};
pub(crate) use std::collections::{HashMap, HashSet, VecDeque};
pub(crate) use std::path::PathBuf;
pub(crate) use utoipa::{IntoParams, ToSchema};
pub(crate) use uuid::Uuid;

pub(crate) use crate::{
    error::{AppError, ErrorResponse},
    models::gendate::GenDate,
    services::{
        auth::{AuthService, UserDto},
        dashboard::{
            build_dashboard_response, compute_tree_stats, compute_upcoming_anniversaries,
            AnniversariesListResponse, AnniversariesQuery, DashboardResponse, TreeStatsResponse,
        },
        gedcom::{
            analyze_gedcom_payload, export_store_to_gedcom, import_gedcom_into_store,
            GedcomAnalysisReport, GedcomAnalyzeRequest, GedcomExportRequest, GedcomImportRequest,
            GedcomImportResult,
        },
        genealogy::{
            allocate_gid, build_place_breadcrumbs, calculate_kinship_relation, compute_fan_chart,
            compute_kinship_path, compute_places_map, ensure_no_ancestor_cycle,
            ensure_no_place_cycle, recompute_all_individuals_vitals, recompute_all_places_metadata,
            resolve_or_create_place_chain, FanChartQuery, FanChartResponse, KinshipQuery,
            KinshipResponse, PlacesMapResponse, ProfileKinshipDto,
        },
        history::{
            create_snapshot, get_entity_version, log_change, parse_if_match_version,
            query_history_feed, query_record_history, restore_change, rollback_snapshot,
            undo_transaction, verify_optimistic_version, CreateSnapshotRequest,
            EntityVersionResponse, HistoryFeedQuery, HistoryFeedResponse, RecordHistoryResponse,
            RollbackSnapshotRequest, RollbackSnapshotResponse, SnapshotListResponse,
            UndoTransactionResponse,
        },
        media_bundle::{
            process_media_bundle_archive, save_media_bytes_to_disk, spool_multipart_to_temp_file,
            MediaBundleQuery, MediaBundleReconciliationReport,
        },
        pdf::{
            calculate_preview_info, generate_printable_pdf, PdfExportPreviewInfo, PdfExportRequest,
        },
        privacy::{
            canonicalize_surname_display, enrich_family_dto_marriage, evaluate_is_living,
            find_family_marriage_facts, fold_slavic_surname_stem,
            normalize_russian_orthography_word, normalize_search_tokens, parse_search_query,
            redact_media_for_caller, redact_person_for_caller, redact_person_for_tree_view,
            suggest_slavic_patronymic, transliterate_to_latin, AvatarCropDto, ChangeLogEntryDto,
            FaceTagDto, FamilyChildDto, FamilyDto, GenealogyPrivacyStore, IndividualAltNameDto,
            LifeEventDto, MediaAssetSummaryDto, MediaLinkDto, NoteDto, PersonLivingEvaluationInput,
            PersonNameDto, PlaceAltNameDto, PlaceBreadcrumbDto, PlaceSummaryDto, TreePersonNodeDto,
            TreeSnapshotDto, VitalEventSummaryDto,
        },
        quality::{
            analyze_place_hierarchies, batch_fix_places, build_quality_summary,
            compute_duplicate_candidates, compute_quality_warnings, dismiss_place_issue_in_store,
            dismiss_warning_in_store, execute_person_merge, fix_place_hierarchy,
            mark_not_duplicate_in_store, match_prospective_person, preview_person_merge,
            preview_place_hierarchy_issue, BatchFixPlacesRequest, BatchFixPlacesResponse,
            DismissPlaceIssueRequest, DismissWarningRequest, DismissWarningResponse,
            DuplicateCandidatesFilterQuery, DuplicateCandidatesListResponse,
            FixPlaceHierarchyRequest, FixPlaceHierarchyResponse, MergePreviewQuery,
            MergePreviewResponse, NotDuplicatePairRequest, NotDuplicatePairResponse,
            PersonMatchQuery, PersonMatchResponse, PersonMergeRequest as QualityPersonMergeRequest,
            PersonMergeResponse, PlaceHierarchyAnalysisQuery, PlaceHierarchyAnalysisResponseDto,
            PlaceHierarchyIssueDto, PlaceHierarchyPreviewQuery, QualitySummaryResponse,
            QualityWarningsFilterQuery, QualityWarningsListResponse,
        },
        rbac::{CallerAccess, PublicOrAuth, RequireAdmin, RequireAuth, RequireEditor},
    },
    state::AppState,
};

pub mod types;
pub use types::*;
pub mod types_search_media;
pub use types_search_media::*;
pub mod onboarding_tree;
pub use onboarding_tree::*;
pub mod tree_view;
pub use tree_view::*;
pub mod person_profile;
pub use person_profile::*;
pub mod person_mutations;
pub use person_mutations::*;
pub mod quick_add;
pub use quick_add::*;
pub mod person_delete_families;
pub use person_delete_families::*;
pub mod events;
pub use events::*;
pub mod notes_media_files;
pub use notes_media_files::*;
pub mod search;
pub use search::*;
pub mod people_directory;
pub use people_directory::*;
pub mod places;
pub use places::*;
pub mod media_list_create;
pub use media_list_create::*;
pub mod media_tags_links;
pub use media_tags_links::*;
pub mod gedcom_pdf;
pub use gedcom_pdf::*;
pub mod history_quality;
pub use history_quality::*;
pub mod dashboard_kinship;
pub use dashboard_kinship::*;
pub mod ai_mcp;
pub mod ai_stories;
pub use ai_stories::*;
pub mod mcp;
pub use mcp::*;
