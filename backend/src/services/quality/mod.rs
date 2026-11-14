#![allow(unused_imports)]
pub(crate) use chrono::{Datelike, Utc};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::{HashMap, HashSet, VecDeque};
pub(crate) use utoipa::{IntoParams, ToSchema};
pub(crate) use uuid::Uuid;

pub(crate) use crate::{
    error::AppError,
    models::gendate::{GenDate, GenDateMod},
    services::{
        genealogy::{
            allocate_gid, build_place_breadcrumbs, ensure_no_ancestor_cycle, ensure_no_place_cycle,
            normalize_event_type, recompute_all_individuals_vitals, recompute_all_places_metadata,
        },
        history::{create_snapshot, log_change},
        privacy::{
            fold_slavic_surname_stem, normalize_russian_orthography_word, redact_person_for_caller,
            suggest_slavic_patronymic, ChangeLogEntryDto, DismissedWarningDto, FamilyChildDto,
            FamilyDto, GenealogyPrivacyStore, IndividualAltNameDto, LifeEventDto, MediaLinkDto,
            PersonNameDto, PlaceAltNameDto, PlaceBreadcrumbDto, PlaceSummaryDto, TreePersonNodeDto,
            VitalEventSummaryDto,
        },
        rbac::CallerAccess,
    },
};

pub mod types;
pub use types::*;
pub mod warnings;
pub use warnings::*;
pub mod scoring;
pub use scoring::*;
pub mod candidates;
pub use candidates::*;
pub mod merge;
pub use merge::*;
pub mod places_infer;
pub use places_infer::*;
pub mod places_single;
pub use places_single::*;
pub mod places_fix;
pub use places_fix::*;
pub mod places_batch_tests;
pub use places_batch_tests::*;
pub mod events_duplicates_types;
pub use events_duplicates_types::*;
pub mod events_duplicates;
pub use events_duplicates::*;
pub mod events_duplicates_merge;
pub use events_duplicates_merge::*;
pub mod events_duplicates_batch;
pub use events_duplicates_batch::*;
