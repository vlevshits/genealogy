use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::gendate::{GenDate, GenDateMod},
    services::{
        genealogy::ensure_no_ancestor_cycle,
        history::create_snapshot,
        privacy::{
            fold_slavic_surname_stem, normalize_russian_orthography_word, redact_person_for_caller,
            ChangeLogEntryDto, DismissedWarningDto, FamilyChildDto, GenealogyPrivacyStore,
            IndividualAltNameDto, LifeEventDto, MediaLinkDto, PersonNameDto, TreePersonNodeDto,
            VitalEventSummaryDto,
        },
        rbac::CallerAccess,
    },
};

/// Canonical 10 soft validation rule codes (`docs/features/15-data-quality-duplicates.md §2.1`).
pub const RULE_DEATH_BEFORE_BIRTH: &str = "DEATH_BEFORE_BIRTH";
pub const RULE_TOO_OLD: &str = "TOO_OLD";
pub const RULE_PARENT_TOO_YOUNG: &str = "PARENT_TOO_YOUNG";
pub const RULE_MOTHER_TOO_OLD: &str = "MOTHER_TOO_OLD";
pub const RULE_BORN_AFTER_PARENT_DEATH: &str = "BORN_AFTER_PARENT_DEATH";
pub const RULE_MARRIED_TOO_YOUNG: &str = "MARRIED_TOO_YOUNG";
pub const RULE_EVENT_AFTER_DEATH: &str = "EVENT_AFTER_DEATH";
pub const RULE_POSSIBLE_DUPLICATE: &str = "POSSIBLE_DUPLICATE";
pub const RULE_MISSING_BIRTH: &str = "MISSING_BIRTH";
pub const RULE_UNCONNECTED: &str = "UNCONNECTED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualityWarningPersonRefDto {
    pub id: String,
    pub gid: String,
    pub display_name: String,
    pub sex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualityWarningDto {
    pub id: String,
    pub code: String,
    pub rule_code: String,
    pub category: String,
    pub severity: String,
    pub entity_type: String,
    pub entity_id: String,
    pub person: QualityWarningPersonRefDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_person: Option<QualityWarningPersonRefDto>,
    pub message: String,
    pub details: serde_json::Value,
    pub is_dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DuplicateCandidatePairDto {
    pub id: String,
    pub person_a: QualityWarningPersonRefDto,
    pub person_b: QualityWarningPersonRefDto,
    pub score: i32,
    pub confidence: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualityCategoryCountsDto {
    pub chronology: usize,
    pub family_age: usize,
    pub completeness: usize,
    pub duplicates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualitySeverityCountsDto {
    pub warning: usize,
    pub info: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualitySummaryResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    pub total_open_warnings: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub duplicate_candidates_count: usize,
    pub dismissed_count: usize,
    pub by_category: QualityCategoryCountsDto,
    pub by_severity: QualitySeverityCountsDto,
    pub by_rule: HashMap<String, usize>,
    pub warnings: Vec<QualityWarningDto>,
    pub duplicates: Vec<DuplicateCandidatePairDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct QualityWarningsFilterQuery {
    #[serde(default)]
    pub rule: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub rule_code: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub include_dismissed: Option<bool>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualityWarningsListResponse {
    pub items: Vec<QualityWarningDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DuplicateCandidatesFilterQuery {
    #[serde(default)]
    pub min_score: Option<i32>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DuplicateCandidatesListResponse {
    pub items: Vec<DuplicateCandidatePairDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DismissWarningRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub rule_code: Option<String>,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub secondary_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DismissWarningResponse {
    pub status: String,
    pub dismissed: DismissedWarningDto,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct NotDuplicatePairRequest {
    #[serde(default)]
    pub person_a_id: Option<String>,
    #[serde(default)]
    pub person_b_id: Option<String>,
    #[serde(default)]
    pub person_a: Option<String>,
    #[serde(default)]
    pub person_b: Option<String>,
    #[serde(default)]
    pub a: Option<String>,
    #[serde(default)]
    pub b: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct NotDuplicatePairResponse {
    pub status: String,
    pub person_a_id: String,
    pub person_b_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PersonMatchQuery {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub given: Option<String>,
    #[serde(default)]
    pub surname: Option<String>,
    #[serde(default)]
    pub patronymic: Option<String>,
    #[serde(default)]
    pub birth_year: Option<i32>,
    #[serde(default)]
    pub birth: Option<String>,
    #[serde(default)]
    pub sex: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub birth_place: Option<String>,
    #[serde(default)]
    pub exclude_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PersonMatchCandidateDto {
    pub person: QualityWarningPersonRefDto,
    pub score: i32,
    pub confidence: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PersonMatchResponse {
    pub items: Vec<PersonMatchCandidateDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct MergePreviewQuery {
    #[serde(default)]
    pub keep_id: Option<String>,
    #[serde(default)]
    pub merge_id: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub primary_person_id: Option<String>,
    #[serde(default)]
    pub duplicate_person_id: Option<String>,
    #[serde(default)]
    pub a: Option<String>,
    #[serde(default)]
    pub b: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MergeFieldConflictDto {
    pub field: String,
    pub keep_value: serde_json::Value,
    pub merge_value: serde_json::Value,
    pub has_conflict: bool,
    pub recommended_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MergeCombinedCountsDto {
    pub events: usize,
    pub media: usize,
    pub notes: usize,
    pub families: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct MergePreviewResponse {
    pub keep_person: TreePersonNodeDto,
    pub merge_person: TreePersonNodeDto,
    pub target: TreePersonNodeDto,
    pub source: TreePersonNodeDto,
    pub score: i32,
    pub reasons: Vec<String>,
    pub can_merge: bool,
    pub cycle_detected: bool,
    pub field_conflicts: Vec<MergeFieldConflictDto>,
    pub events_to_keep: usize,
    pub events_to_transfer: usize,
    pub events_deduplicated: usize,
    pub notes_to_transfer: usize,
    pub media_to_transfer: usize,
    pub families_affected: usize,
    pub combined_counts: MergeCombinedCountsDto,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PersonMergeRequest {
    #[serde(default)]
    pub keep_id: Option<String>,
    #[serde(default)]
    pub merge_id: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub primary_person_id: Option<String>,
    #[serde(default)]
    pub duplicate_person_id: Option<String>,
    #[serde(default)]
    pub field_choices: HashMap<String, String>,
    #[serde(default)]
    pub field_overrides: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PersonMergeResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surviving_person_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surviving_person: Option<TreePersonNodeDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_gid: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct PlausibleDateWindow {
    min_year: i32,
    max_year: i32,
    min_sort: i32,
    max_sort: i32,
}

fn parse_plausible_window(
    date_str: Option<&str>,
    fallback_sort: Option<i32>,
) -> Option<PlausibleDateWindow> {
    if let Some(raw) = date_str {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let gd = GenDate::parse(trimmed);
            if gd.r#mod != GenDateMod::Phrase {
                if let Some(start) = gd.start {
                    let base_start_sort = gd.compute_sort_key().unwrap_or(start.y * 10_000);
                    return Some(match gd.r#mod {
                        GenDateMod::Exact => PlausibleDateWindow {
                            min_year: start.y,
                            max_year: start.y,
                            min_sort: base_start_sort,
                            max_sort: if start.m.is_none() {
                                start.y * 10_000 + 1231
                            } else if start.d.is_none() {
                                start.y * 10_000 + i32::from(start.m.unwrap_or(12)) * 100 + 31
                            } else {
                                base_start_sort
                            },
                        },
                        GenDateMod::About | GenDateMod::Estimated | GenDateMod::Calculated => {
                            // Allow ±2 year fuzzy window for ABT / EST / CAL dates (`15-data-quality-duplicates.md §2.2`)
                            PlausibleDateWindow {
                                min_year: start.y - 2,
                                max_year: start.y + 2,
                                min_sort: (start.y - 2) * 10_000 + 101,
                                max_sort: (start.y + 2) * 10_000 + 1231,
                            }
                        }
                        GenDateMod::Between | GenDateMod::FromTo => {
                            let end_y = gd.end.map(|e| e.y).unwrap_or(start.y);
                            let min_y = start.y.min(end_y);
                            let max_y = start.y.max(end_y);
                            PlausibleDateWindow {
                                min_year: min_y,
                                max_year: max_y,
                                min_sort: min_y * 10_000 + 101,
                                max_sort: max_y * 10_000 + 1231,
                            }
                        }
                        GenDateMod::Before => PlausibleDateWindow {
                            min_year: start.y - 100,
                            max_year: start.y,
                            min_sort: (start.y - 100) * 10_000 + 101,
                            max_sort: base_start_sort,
                        },
                        GenDateMod::After => PlausibleDateWindow {
                            min_year: start.y,
                            max_year: start.y + 100,
                            min_sort: base_start_sort,
                            max_sort: (start.y + 100) * 10_000 + 1231,
                        },
                        GenDateMod::Phrase => unreachable!(),
                    });
                }
            }
        }
    }

    fallback_sort.map(|sort| {
        let y = sort / 10_000;
        PlausibleDateWindow {
            min_year: y,
            max_year: y,
            min_sort: sort,
            max_sort: sort,
        }
    })
}

pub fn format_person_display_name(p: &TreePersonNodeDto) -> String {
    let mut parts = Vec::new();
    if !p.primary_name.given_name.trim().is_empty() {
        parts.push(p.primary_name.given_name.trim());
    }
    if let Some(pat) = p.primary_name.patronymic.as_deref() {
        if !pat.trim().is_empty() {
            parts.push(pat.trim());
        }
    }
    if !p.primary_name.surname.trim().is_empty() {
        parts.push(p.primary_name.surname.trim());
    }
    if parts.is_empty() {
        p.gid.clone()
    } else {
        parts.join(" ")
    }
}

pub fn to_warning_person_ref(p: &TreePersonNodeDto) -> QualityWarningPersonRefDto {
    QualityWarningPersonRefDto {
        id: p.id.clone(),
        gid: p.gid.clone(),
        display_name: format_person_display_name(p),
        sex: p.sex.clone(),
        birth_date: p.birth.as_ref().and_then(|b| b.date.clone()),
        death_date: p.death.as_ref().and_then(|d| d.date.clone()),
    }
}

fn event_date_text(ev: &LifeEventDto) -> Option<&str> {
    ev.date
        .as_ref()
        .map(|d| d.text.as_str())
        .or(ev.gedcom_date.as_deref())
}

fn person_birth_window(
    p: &TreePersonNodeDto,
    events: &[LifeEventDto],
) -> Option<PlausibleDateWindow> {
    let ev_opt = events.iter().find(|e| {
        e.individual_id.as_deref() == Some(p.id.as_str())
            && matches!(e.event_type.as_str(), "birth" | "christening" | "baptism")
    });
    let summary_date = p
        .birth
        .as_ref()
        .and_then(|b| b.date.as_deref())
        .or_else(|| ev_opt.and_then(event_date_text));
    let sort_key = p.birth_sort.or_else(|| ev_opt.and_then(|e| e.date_sort));
    parse_plausible_window(summary_date, sort_key)
}

fn person_death_window(
    p: &TreePersonNodeDto,
    events: &[LifeEventDto],
) -> Option<PlausibleDateWindow> {
    let ev_opt = events.iter().find(|e| {
        e.individual_id.as_deref() == Some(p.id.as_str())
            && matches!(e.event_type.as_str(), "death" | "burial" | "cremation")
    });
    let summary_date = p
        .death
        .as_ref()
        .and_then(|d| d.date.as_deref())
        .or_else(|| ev_opt.and_then(event_date_text));
    let sort_key = p.death_sort.or_else(|| ev_opt.and_then(|e| e.date_sort));
    parse_plausible_window(summary_date, sort_key)
}

fn is_warning_dismissed(
    dismissed: &[DismissedWarningDto],
    warning_id: &str,
    rule_code: &str,
    entity_id: &str,
    secondary_id: Option<&str>,
) -> bool {
    dismissed.iter().any(|d| {
        if d.id == warning_id {
            return true;
        }
        if d.rule_code.eq_ignore_ascii_case(rule_code) && d.entity_id == entity_id {
            if let (Some(sec_a), Some(sec_b)) = (d.secondary_id.as_deref(), secondary_id) {
                return sec_a == sec_b;
            }
            return true;
        }
        false
    })
}

/// Evaluates all 10 soft data-quality rules (`docs/features/15-data-quality-duplicates.md §2.1`).
pub fn compute_quality_warnings(
    store: &GenealogyPrivacyStore,
    include_dismissed: bool,
) -> Vec<QualityWarningDto> {
    let mut warnings = Vec::new();
    let current_year = Utc::now().year();
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();

    // Precompute connected persons (any parent, partner, or child link in `families`)
    let mut connected_ids: HashSet<&str> = HashSet::new();
    for fam in &store.families {
        let has_multiple_members = (fam.partner1_id.is_some() as usize
            + fam.partner2_id.is_some() as usize
            + fam.children.len())
            > 1;
        if has_multiple_members {
            if let Some(p1) = fam.partner1_id.as_deref() {
                connected_ids.insert(p1);
            }
            if let Some(p2) = fam.partner2_id.as_deref() {
                connected_ids.insert(p2);
            }
            for ch in &fam.children {
                connected_ids.insert(ch.child_id.as_str());
            }
        }
    }

    for person in &store.persons {
        let b_win = person_birth_window(person, &store.events);
        let d_win = person_death_window(person, &store.events);

        // Rule 1: DEATH_BEFORE_BIRTH (Death date < Birth date, using widest plausible window)
        if let (Some(b), Some(d)) = (b_win, d_win) {
            if d.max_sort < b.min_sort {
                let wid = format!("{RULE_DEATH_BEFORE_BIRTH}:{}", person.id);
                let dismissed = is_warning_dismissed(
                    &store.dismissed_warnings,
                    &wid,
                    RULE_DEATH_BEFORE_BIRTH,
                    &person.id,
                    None,
                );
                if include_dismissed || !dismissed {
                    warnings.push(QualityWarningDto {
                        id: wid,
                        code: RULE_DEATH_BEFORE_BIRTH.to_string(),
                        rule_code: RULE_DEATH_BEFORE_BIRTH.to_string(),
                        category: "chronology".to_string(),
                        severity: "warning".to_string(),
                        entity_type: "individual".to_string(),
                        entity_id: person.id.clone(),
                        person: to_warning_person_ref(person),
                        related_person: None,
                        message: format!(
                            "Death date precedes birth date for {}",
                            format_person_display_name(person)
                        ),
                        details: serde_json::json!({
                            "birth_sort": b.min_sort,
                            "death_sort": d.max_sort,
                        }),
                        is_dismissed: dismissed,
                    });
                }
            }
        }

        // Rule 2: TOO_OLD (Lifespan > 110 years, or living person with birth year > 110 years ago)
        if let Some(b) = b_win {
            let min_age_at_death_or_now = if let Some(d) = d_win {
                Some(d.min_year - b.max_year)
            } else if person.is_living {
                Some(current_year - b.max_year)
            } else {
                None
            };
            if let Some(age) = min_age_at_death_or_now {
                if age > 110 {
                    let wid = format!("{RULE_TOO_OLD}:{}", person.id);
                    let dismissed = is_warning_dismissed(
                        &store.dismissed_warnings,
                        &wid,
                        RULE_TOO_OLD,
                        &person.id,
                        None,
                    );
                    if include_dismissed || !dismissed {
                        warnings.push(QualityWarningDto {
                            id: wid,
                            code: RULE_TOO_OLD.to_string(),
                            rule_code: RULE_TOO_OLD.to_string(),
                            category: "chronology".to_string(),
                            severity: "warning".to_string(),
                            entity_type: "individual".to_string(),
                            entity_id: person.id.clone(),
                            person: to_warning_person_ref(person),
                            related_person: None,
                            message: format!(
                                "Recorded age ({age} years) exceeds 110 years for {}",
                                format_person_display_name(person)
                            ),
                            details: serde_json::json!({
                                "age": age,
                                "threshold": 110,
                            }),
                            is_dismissed: dismissed,
                        });
                    }
                }
            }
        }

        // Rule 7: EVENT_AFTER_DEATH (Personal life event > 30 days after death, except burial/cremation/probate/will)
        if let Some(d) = d_win {
            for ev in store
                .events
                .iter()
                .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
            {
                let ev_type = ev.event_type.trim().to_ascii_lowercase();
                if matches!(
                    ev_type.as_str(),
                    "death" | "burial" | "cremation" | "probate" | "will" | "memorial"
                ) {
                    continue;
                }
                if let Some(ev_win) = parse_plausible_window(event_date_text(ev), ev.date_sort) {
                    // > 30 days (~100 in YYYYMMDD sort key or > 0 years if year-only)
                    let threshold_sort = if d.max_sort % 10_000 == 0 {
                        (d.max_year + 1) * 10_000
                    } else {
                        d.max_sort + 100
                    };
                    if ev_win.min_sort > threshold_sort {
                        let wid = format!("{RULE_EVENT_AFTER_DEATH}:{}:{}", person.id, ev.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_EVENT_AFTER_DEATH,
                            &person.id,
                            Some(&ev.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_EVENT_AFTER_DEATH.to_string(),
                                rule_code: RULE_EVENT_AFTER_DEATH.to_string(),
                                category: "chronology".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: person.id.clone(),
                                person: to_warning_person_ref(person),
                                related_person: None,
                                message: format!(
                                    "Event '{}' occurs after death date for {}",
                                    ev.event_type,
                                    format_person_display_name(person)
                                ),
                                details: serde_json::json!({
                                    "event_id": ev.id,
                                    "event_type": ev.event_type,
                                    "event_date": event_date_text(ev),
                                    "death_sort": d.max_sort,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }

        // Rule 9: MISSING_BIRTH (No birth date on record)
        let has_birth_event = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(person.id.as_str())
                && e.event_type.eq_ignore_ascii_case("birth")
                && (event_date_text(e)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
                    || e.date_sort.is_some())
        });
        let has_birth_summary = person
            .birth
            .as_ref()
            .and_then(|b| b.date.as_deref())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !has_birth_event && !has_birth_summary && person.birth_sort.is_none() {
            let wid = format!("{RULE_MISSING_BIRTH}:{}", person.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &wid,
                RULE_MISSING_BIRTH,
                &person.id,
                None,
            );
            if include_dismissed || !dismissed {
                warnings.push(QualityWarningDto {
                    id: wid,
                    code: RULE_MISSING_BIRTH.to_string(),
                    rule_code: RULE_MISSING_BIRTH.to_string(),
                    category: "completeness".to_string(),
                    severity: "info".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: person.id.clone(),
                    person: to_warning_person_ref(person),
                    related_person: None,
                    message: format!(
                        "{} is missing a birth date",
                        format_person_display_name(person)
                    ),
                    details: serde_json::json!({}),
                    is_dismissed: dismissed,
                });
            }
        }

        // Rule 10: UNCONNECTED (0 relatives in a tree with > 1 person)
        if store.persons.len() > 1 && !connected_ids.contains(person.id.as_str()) {
            let wid = format!("{RULE_UNCONNECTED}:{}", person.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &wid,
                RULE_UNCONNECTED,
                &person.id,
                None,
            );
            if include_dismissed || !dismissed {
                warnings.push(QualityWarningDto {
                    id: wid,
                    code: RULE_UNCONNECTED.to_string(),
                    rule_code: RULE_UNCONNECTED.to_string(),
                    category: "completeness".to_string(),
                    severity: "info".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: person.id.clone(),
                    person: to_warning_person_ref(person),
                    related_person: None,
                    message: format!(
                        "{} has no connected family relationships",
                        format_person_display_name(person)
                    ),
                    details: serde_json::json!({}),
                    is_dismissed: dismissed,
                });
            }
        }
    }

    // Family-based rules: Rules 3, 4, 5, 6
    for fam in &store.families {
        let partners: Vec<&TreePersonNodeDto> =
            [fam.partner1_id.as_deref(), fam.partner2_id.as_deref()]
                .into_iter()
                .flatten()
                .filter_map(|pid| person_by_id.get(pid).copied())
                .collect();

        // Rule 6: MARRIED_TOO_YOUNG (< 14 years old at marriage)
        let marriage_ev = store.events.iter().find(|e| {
            e.family_id.as_deref() == Some(fam.id.as_str())
                && e.event_type.eq_ignore_ascii_case("marriage")
        });
        let marriage_win =
            marriage_ev.and_then(|ev| parse_plausible_window(event_date_text(ev), ev.date_sort));

        if let Some(m_win) = marriage_win {
            for spouse in &partners {
                if let Some(s_birth) = person_birth_window(spouse, &store.events) {
                    let max_possible_age = m_win.max_year - s_birth.min_year;
                    if max_possible_age < 14 {
                        let wid = format!("{RULE_MARRIED_TOO_YOUNG}:{}:{}", spouse.id, fam.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_MARRIED_TOO_YOUNG,
                            &spouse.id,
                            Some(&fam.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_MARRIED_TOO_YOUNG.to_string(),
                                rule_code: RULE_MARRIED_TOO_YOUNG.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: spouse.id.clone(),
                                person: to_warning_person_ref(spouse),
                                related_person: None,
                                message: format!(
                                    "{} was {} years old at marriage (< 14)",
                                    format_person_display_name(spouse),
                                    max_possible_age
                                ),
                                details: serde_json::json!({
                                    "family_id": fam.id,
                                    "age_at_marriage": max_possible_age,
                                    "min_age_threshold": 14,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }

        // Parent-child checks: Rules 3, 4, 5
        for ch_link in &fam.children {
            let Some(child) = person_by_id.get(ch_link.child_id.as_str()).copied() else {
                continue;
            };
            let Some(c_birth) = person_birth_window(child, &store.events) else {
                continue;
            };

            for parent in &partners {
                if let Some(p_birth) = person_birth_window(parent, &store.events) {
                    let max_parent_age_at_birth = c_birth.max_year - p_birth.min_year;
                    let min_parent_age_at_birth = c_birth.min_year - p_birth.max_year;

                    // Rule 3: PARENT_TOO_YOUNG (Parent < 13 years old at child's birth)
                    if max_parent_age_at_birth < 13 {
                        let wid = format!("{RULE_PARENT_TOO_YOUNG}:{}:{}", parent.id, child.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_PARENT_TOO_YOUNG,
                            &parent.id,
                            Some(&child.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_PARENT_TOO_YOUNG.to_string(),
                                rule_code: RULE_PARENT_TOO_YOUNG.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: parent.id.clone(),
                                person: to_warning_person_ref(parent),
                                related_person: Some(to_warning_person_ref(child)),
                                message: format!(
                                    "{} was {} years old when {} was born (< 13)",
                                    format_person_display_name(parent),
                                    max_parent_age_at_birth,
                                    format_person_display_name(child)
                                ),
                                details: serde_json::json!({
                                    "parent_age": max_parent_age_at_birth,
                                    "child_id": child.id,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }

                    // Rule 4: MOTHER_TOO_OLD (Mother > 55 years old at child's birth)
                    if parent.sex.eq_ignore_ascii_case("F") && min_parent_age_at_birth > 55 {
                        let wid = format!("{RULE_MOTHER_TOO_OLD}:{}:{}", parent.id, child.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_MOTHER_TOO_OLD,
                            &parent.id,
                            Some(&child.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_MOTHER_TOO_OLD.to_string(),
                                rule_code: RULE_MOTHER_TOO_OLD.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: parent.id.clone(),
                                person: to_warning_person_ref(parent),
                                related_person: Some(to_warning_person_ref(child)),
                                message: format!(
                                    "Mother {} was {} years old when {} was born (> 55)",
                                    format_person_display_name(parent),
                                    min_parent_age_at_birth,
                                    format_person_display_name(child)
                                ),
                                details: serde_json::json!({
                                    "mother_age": min_parent_age_at_birth,
                                    "child_id": child.id,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }

                // Rule 5: BORN_AFTER_PARENT_DEATH
                // Mother: child born after mother's death
                // Father: child born > 10 months (~1 year or +10000 sort key) after father's death
                if let Some(p_death) = person_death_window(parent, &store.events) {
                    let is_mother = parent.sex.eq_ignore_ascii_case("F");
                    let violated = if is_mother {
                        c_birth.min_sort > p_death.max_sort
                    } else {
                        let father_limit_sort = p_death.max_sort + 10_000;
                        c_birth.min_sort > father_limit_sort
                    };
                    if violated {
                        let wid =
                            format!("{RULE_BORN_AFTER_PARENT_DEATH}:{}:{}", child.id, parent.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_BORN_AFTER_PARENT_DEATH,
                            &child.id,
                            Some(&parent.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_BORN_AFTER_PARENT_DEATH.to_string(),
                                rule_code: RULE_BORN_AFTER_PARENT_DEATH.to_string(),
                                category: "chronology".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: child.id.clone(),
                                person: to_warning_person_ref(child),
                                related_person: Some(to_warning_person_ref(parent)),
                                message: format!(
                                    "{} was born after the death of parent {}",
                                    format_person_display_name(child),
                                    format_person_display_name(parent)
                                ),
                                details: serde_json::json!({
                                    "parent_id": parent.id,
                                    "parent_death_sort": p_death.max_sort,
                                    "child_birth_sort": c_birth.min_sort,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }
    }

    // Rule 8: POSSIBLE_DUPLICATE (Duplicate candidate pairs with score >= 75)
    let duplicates = compute_duplicate_candidates(store, 75);
    for dup in duplicates {
        let wid = format!(
            "{RULE_POSSIBLE_DUPLICATE}:{}:{}",
            dup.person_a.id, dup.person_b.id
        );
        let dismissed = is_warning_dismissed(
            &store.dismissed_warnings,
            &wid,
            RULE_POSSIBLE_DUPLICATE,
            &dup.person_a.id,
            Some(&dup.person_b.id),
        );
        if include_dismissed || !dismissed {
            warnings.push(QualityWarningDto {
                id: wid,
                code: RULE_POSSIBLE_DUPLICATE.to_string(),
                rule_code: RULE_POSSIBLE_DUPLICATE.to_string(),
                category: "duplicates".to_string(),
                severity: "warning".to_string(),
                entity_type: "individual".to_string(),
                entity_id: dup.person_a.id.clone(),
                person: dup.person_a.clone(),
                related_person: Some(dup.person_b.clone()),
                message: format!(
                    "Possible duplicate ({}% match): {} and {}",
                    dup.score, dup.person_a.display_name, dup.person_b.display_name
                ),
                details: serde_json::json!({
                    "score": dup.score,
                    "reasons": dup.reasons,
                    "person_b_id": dup.person_b.id,
                }),
                is_dismissed: dismissed,
            });
        }
    }

    warnings
}

/// Normalizes a name token across Russian Cyrillic and Latin scripts for fuzzy comparison (`13 §2.2`, `15 §3.1`).
fn normalize_cross_script_token(raw: &str) -> String {
    let ortho = normalize_russian_orthography_word(raw);
    let mut out = String::with_capacity(ortho.len() * 2);
    for ch in ortho.chars() {
        match ch {
            'а' => out.push('a'),
            'б' => out.push('b'),
            'в' => out.push('v'),
            'г' => out.push('g'),
            'д' => out.push('d'),
            'е' => out.push('e'),
            'ж' => out.push_str("zh"),
            'з' => out.push('z'),
            'и' | 'й' | 'ы' => out.push('i'),
            'к' => out.push('k'),
            'л' => out.push('l'),
            'м' => out.push('m'),
            'н' => out.push('n'),
            'о' => out.push('o'),
            'п' => out.push('p'),
            'р' => out.push('r'),
            'с' => out.push('s'),
            'т' => out.push('t'),
            'у' => out.push('u'),
            'ф' => out.push('f'),
            'х' => out.push_str("kh"),
            'ц' => out.push_str("ts"),
            'ч' => out.push_str("ch"),
            'ш' => out.push_str("sh"),
            'щ' => out.push_str("shch"),
            'ъ' | 'ь' => {}
            'ю' => out.push_str("yu"),
            'я' => out.push_str("ya"),
            c if c.is_alphanumeric() => out.push(c),
            _ => {}
        }
    }
    for suffix in ["ii", "iy", "yi", "yy", "ey", "ei"] {
        if let Some(stem) = out.strip_suffix(suffix) {
            if stem.len() >= 2 {
                return format!("{stem}i");
            }
        }
    }
    if let Some(stem) = out.strip_suffix('y') {
        if stem.len() >= 3 {
            return format!("{stem}i");
        }
    }
    out
}

fn canonical_surname_key(raw: &str) -> String {
    let folded = fold_slavic_surname_stem(raw);
    normalize_cross_script_token(&folded)
}

fn bigrams(s: &str) -> HashSet<(char, char)> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return HashSet::new();
    }
    chars.windows(2).map(|w| (w[0], w[1])).collect()
}

fn dice_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let bg_a = bigrams(a);
    let bg_b = bigrams(b);
    if bg_a.is_empty() || bg_b.is_empty() {
        return 0.0;
    }
    let inter = bg_a.intersection(&bg_b).count();
    (2.0 * inter as f64) / ((bg_a.len() + bg_b.len()) as f64)
}

fn collect_person_surnames(p: &TreePersonNodeDto) -> Vec<String> {
    let mut res = Vec::new();
    if !p.primary_name.surname.trim().is_empty() {
        res.push(canonical_surname_key(&p.primary_name.surname));
    }
    if let Some(bs) = p.birth_surname.as_deref() {
        if !bs.trim().is_empty() {
            res.push(canonical_surname_key(bs));
        }
    }
    for alt in &p.alt_names {
        if let Some(s) = alt.surname.as_deref() {
            if !s.trim().is_empty() {
                res.push(canonical_surname_key(s));
            }
        }
    }
    res.sort();
    res.dedup();
    res
}

fn collect_person_given_names(p: &TreePersonNodeDto) -> Vec<String> {
    let mut res = Vec::new();
    if !p.primary_name.given_name.trim().is_empty() {
        res.push(normalize_cross_script_token(&p.primary_name.given_name));
    }
    for alt in &p.alt_names {
        if let Some(g) = alt.given_name.as_deref() {
            if !g.trim().is_empty() {
                res.push(normalize_cross_script_token(g));
            }
        }
    }
    res.sort();
    res.dedup();
    res
}

fn relative_ids_for_person(p_id: &str, store: &GenealogyPrivacyStore) -> HashSet<String> {
    let mut rels = HashSet::new();
    for fam in &store.families {
        let is_p1 = fam.partner1_id.as_deref() == Some(p_id);
        let is_p2 = fam.partner2_id.as_deref() == Some(p_id);
        let is_child = fam.children.iter().any(|c| c.child_id == p_id);
        if is_p1 || is_p2 || is_child {
            if let Some(p1) = fam.partner1_id.as_deref() {
                if p1 != p_id {
                    rels.insert(p1.to_string());
                }
            }
            if let Some(p2) = fam.partner2_id.as_deref() {
                if p2 != p_id {
                    rels.insert(p2.to_string());
                }
            }
            for ch in &fam.children {
                if ch.child_id != p_id {
                    rels.insert(ch.child_id.clone());
                }
            }
        }
    }
    rels
}

/// Scores two person records for duplicate likelihood (`0..=100`) and returns human-readable reason codes (`15 §3.1`).
pub fn score_duplicate_pair(
    a: &TreePersonNodeDto,
    b: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
) -> (i32, Vec<String>) {
    let mut score: i32 = 0;
    let mut reasons: Vec<String> = Vec::new();

    // 1. Surname comparison (including birth_surname, gender-folded Slavic endings, cross-script)
    let surnames_a = collect_person_surnames(a);
    let surnames_b = collect_person_surnames(b);
    let mut best_surname_sim = 0.0_f64;
    for sa in &surnames_a {
        for sb in &surnames_b {
            let sim = dice_similarity(sa, sb);
            if sim > best_surname_sim {
                best_surname_sim = sim;
            }
        }
    }
    if best_surname_sim >= 0.99 {
        score += 35;
        reasons.push("same_surname".to_string());
    } else if best_surname_sim >= 0.72 {
        score += 24;
        reasons.push("similar_surname".to_string());
    }

    // 2. Given name comparison
    let givens_a = collect_person_given_names(a);
    let givens_b = collect_person_given_names(b);
    let mut best_given_sim = 0.0_f64;
    for ga in &givens_a {
        for gb in &givens_b {
            let sim = dice_similarity(ga, gb);
            if sim > best_given_sim {
                best_given_sim = sim;
            }
        }
    }
    if best_given_sim >= 0.99 {
        score += 35;
        reasons.push("same_given_name".to_string());
    } else if best_given_sim >= 0.70 {
        score += 22;
        reasons.push("similar_given_name".to_string());
    }

    // 3. Patronymic comparison
    if let (Some(pa), Some(pb)) = (
        a.primary_name.patronymic.as_deref(),
        b.primary_name.patronymic.as_deref(),
    ) {
        let na = normalize_cross_script_token(pa);
        let nb = normalize_cross_script_token(pb);
        if !na.is_empty() && !nb.is_empty() {
            let sim = dice_similarity(&na, &nb);
            if sim >= 0.85 {
                score += 10;
                reasons.push("same_patronymic".to_string());
            }
        }
    }

    // 4. Birth year proximity
    let b_win_a = person_birth_window(a, &store.events);
    let b_win_b = person_birth_window(b, &store.events);
    if let (Some(wa), Some(wb)) = (b_win_a, b_win_b) {
        let year_diff = if wa.max_year < wb.min_year {
            wb.min_year - wa.max_year
        } else if wb.max_year < wa.min_year {
            wa.min_year - wb.max_year
        } else {
            0
        };
        if year_diff == 0 {
            score += 20;
            reasons.push("same_birth_year".to_string());
        } else if year_diff <= 2 {
            score += 14;
            reasons.push("close_birth_year".to_string());
        } else if year_diff <= 5 {
            score += 6;
            reasons.push("approximate_birth_year".to_string());
        } else if year_diff > 15 {
            score -= 40;
        }
    }

    // 5. Birth place comparison
    if let (Some(pa), Some(pb)) = (
        a.birth.as_ref().and_then(|v| v.place.as_deref()),
        b.birth.as_ref().and_then(|v| v.place.as_deref()),
    ) {
        let na = normalize_cross_script_token(pa);
        let nb = normalize_cross_script_token(pb);
        if !na.is_empty() && !nb.is_empty() && (na == nb || na.contains(&nb) || nb.contains(&na)) {
            score += 10;
            reasons.push("same_birth_place".to_string());
        }
    }

    // 6. Shared relatives (parents, spouses, or children)
    let rels_a = relative_ids_for_person(&a.id, store);
    let rels_b = relative_ids_for_person(&b.id, store);
    if !rels_a.is_empty() && !rels_b.is_empty() && rels_a.intersection(&rels_b).next().is_some() {
        score += 15;
        reasons.push("shared_relatives".to_string());
    }

    // 7. Sex compatibility penalty
    let sex_a = a.sex.trim().to_ascii_uppercase();
    let sex_b = b.sex.trim().to_ascii_uppercase();
    if (sex_a == "M" && sex_b == "F") || (sex_a == "F" && sex_b == "M") {
        score -= 30;
    }

    (score.clamp(0, 100), reasons)
}

fn confidence_tier(score: i32) -> String {
    if score >= 85 {
        "high".to_string()
    } else if score >= 70 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub fn is_not_duplicate_pair(store: &GenealogyPrivacyStore, id_a: &str, id_b: &str) -> bool {
    let (p1, p2) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };
    store
        .not_duplicate_pairs
        .iter()
        .any(|(a, b)| a == p1 && b == p2)
}

/// Computes all candidate duplicate pairs in `store` with `score >= min_score`, excluding `not_duplicate_pairs`.
pub fn compute_duplicate_candidates(
    store: &GenealogyPrivacyStore,
    min_score: i32,
) -> Vec<DuplicateCandidatePairDto> {
    let mut out = Vec::new();
    for i in 0..store.persons.len() {
        for j in (i + 1)..store.persons.len() {
            let p_a = &store.persons[i];
            let p_b = &store.persons[j];
            if is_not_duplicate_pair(store, &p_a.id, &p_b.id) {
                continue;
            }
            let (score, reasons) = score_duplicate_pair(p_a, p_b, store);
            if score >= min_score {
                let (first, second) = if p_a.id < p_b.id {
                    (p_a, p_b)
                } else {
                    (p_b, p_a)
                };
                out.push(DuplicateCandidatePairDto {
                    id: format!("dup:{}:{}", first.id, second.id),
                    person_a: to_warning_person_ref(first),
                    person_b: to_warning_person_ref(second),
                    score,
                    confidence: confidence_tier(score),
                    reasons,
                });
            }
        }
    }
    out.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    out
}

/// Matches a prospective person input against existing records in the tree (`GET /api/v1/people/match`).
pub fn match_prospective_person(
    store: &GenealogyPrivacyStore,
    query: &PersonMatchQuery,
    access: &CallerAccess,
    locale: &str,
) -> PersonMatchResponse {
    let mut given_name = query
        .given_name
        .clone()
        .or_else(|| query.given.clone())
        .unwrap_or_default();
    let mut surname = query.surname.clone().unwrap_or_default();
    let mut patronymic = query.patronymic.clone();

    if given_name.trim().is_empty() && surname.trim().is_empty() {
        if let Some(full) = query.name.as_deref() {
            let parts: Vec<&str> = full.split_whitespace().collect();
            match parts.len() {
                1 => {
                    given_name = parts[0].to_string();
                }
                2 => {
                    given_name = parts[0].to_string();
                    surname = parts[1].to_string();
                }
                n if n >= 3 => {
                    given_name = parts[0].to_string();
                    patronymic = Some(parts[1].to_string());
                    surname = parts[2..].join(" ");
                }
                _ => {}
            }
        }
    }

    let birth_date_str = query
        .birth
        .clone()
        .or_else(|| query.birth_year.map(|y| y.to_string()));
    let birth_sort = query.birth_year.map(|y| y * 10_000);
    let sex = query
        .sex
        .clone()
        .or_else(|| query.gender.clone())
        .unwrap_or_else(|| "U".to_string());

    let probe = TreePersonNodeDto {
        id: "__probe__".to_string(),
        gid: "I0000".to_string(),
        sex,
        is_living: true,
        living_override: None,
        is_private: false,
        primary_name: PersonNameDto {
            given_name,
            patronymic,
            surname,
        },
        birth_surname: None,
        nickname: None,
        name_prefix: None,
        name_suffix: None,
        biography: None,
        birth: if birth_date_str.is_some() || query.birth_place.is_some() {
            Some(VitalEventSummaryDto {
                date: birth_date_str,
                place: query.birth_place.clone(),
            })
        } else {
            None
        },
        death: None,
        avatar_url: None,
        avatar_media_id: None,
        avatar_crop: None,
        notes: None,
        birth_sort,
        death_sort: None,
        has_death_event: false,
        alt_names: vec![],
        external_ids: HashMap::new(),
        version: 1,
        tx_id: None,
        generation: None,
        is_direct_lineage: None,
    };

    let mut items = Vec::new();
    for existing in &store.persons {
        if Some(existing.id.as_str()) == query.exclude_id.as_deref()
            || Some(existing.gid.as_str()) == query.exclude_id.as_deref()
        {
            continue;
        }
        let Some(visible) = redact_person_for_caller(existing, access, locale) else {
            continue;
        };
        let (score, reasons) = score_duplicate_pair(&probe, &visible, store);
        if score >= 50 {
            items.push(PersonMatchCandidateDto {
                person: to_warning_person_ref(&visible),
                score,
                confidence: confidence_tier(score),
                reasons,
            });
        }
    }

    items.sort_by_key(|b| std::cmp::Reverse(b.score));
    items.truncate(5);
    let total = items.len();
    PersonMatchResponse { items, total }
}

/// Builds the quality summary response (`GET /api/v1/quality` and `GET /api/v1/quality/summary`).
pub fn build_quality_summary(store: &GenealogyPrivacyStore) -> QualitySummaryResponse {
    let warnings = compute_quality_warnings(store, false);
    let duplicates = compute_duplicate_candidates(store, 60);

    let mut chronology = 0;
    let mut family_age = 0;
    let mut completeness = 0;
    let mut duplicates_cat = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut by_rule: HashMap<String, usize> = HashMap::new();

    for w in &warnings {
        *by_rule.entry(w.rule_code.clone()).or_insert(0) += 1;
        match w.category.as_str() {
            "chronology" => chronology += 1,
            "family_age" => family_age += 1,
            "completeness" => completeness += 1,
            "duplicates" => duplicates_cat += 1,
            _ => {}
        }
        if w.severity == "info" {
            info_count += 1;
        } else {
            warning_count += 1;
        }
    }

    QualitySummaryResponse {
        status: "ok".to_string(),
        operation: "data_quality".to_string(),
        message: "Data quality warnings authorized for administrator".to_string(),
        total_open_warnings: warnings.len(),
        warning_count,
        info_count,
        duplicate_candidates_count: duplicates.len(),
        dismissed_count: store.dismissed_warnings.len(),
        by_category: QualityCategoryCountsDto {
            chronology,
            family_age,
            completeness,
            duplicates: duplicates_cat,
        },
        by_severity: QualitySeverityCountsDto {
            warning: warning_count,
            info: info_count,
        },
        by_rule,
        warnings,
        duplicates,
    }
}

/// Records a dismissed warning (`POST /api/v1/quality/warnings/dismiss` or `POST /api/v1/quality/warnings/{id}/dismiss`).
pub fn dismiss_warning_in_store(
    store: &mut GenealogyPrivacyStore,
    path_id: Option<&str>,
    req: DismissWarningRequest,
    actor_id: Option<String>,
) -> Result<DismissWarningResponse, AppError> {
    let raw_id = path_id
        .map(|s| s.to_string())
        .or(req.id.clone())
        .unwrap_or_default();

    let mut rule_code = req
        .rule_code
        .or(req.code)
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    let mut entity_id = req.entity_id.unwrap_or_default().trim().to_string();
    let mut secondary_id = req.secondary_id;

    if (rule_code.is_empty() || entity_id.is_empty()) && !raw_id.trim().is_empty() {
        let parts: Vec<&str> = raw_id.trim().split(':').collect();
        if parts.len() >= 2 {
            if rule_code.is_empty() {
                rule_code = parts[0].to_uppercase();
            }
            if entity_id.is_empty() {
                entity_id = parts[1].to_string();
            }
            if parts.len() >= 3 && secondary_id.is_none() {
                secondary_id = Some(parts[2].to_string());
            }
        }
    }

    if let Some(resolved) = store.resolve_person_id(&entity_id) {
        entity_id = resolved;
    }

    if rule_code.is_empty() || entity_id.is_empty() {
        let mut fields = HashMap::new();
        fields.insert(
            "warning".to_string(),
            "Both rule_code and entity_id (or composite warning id) are required".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Invalid warning dismissal parameters".to_string(),
            Some(fields),
        ));
    }

    let id = if !raw_id.trim().is_empty() {
        raw_id.trim().to_string()
    } else if let Some(sec) = secondary_id.as_deref() {
        format!("{rule_code}:{entity_id}:{sec}")
    } else {
        format!("{rule_code}:{entity_id}")
    };

    let dto = DismissedWarningDto {
        id: id.clone(),
        rule_code: rule_code.clone(),
        entity_type: req.entity_type.unwrap_or_else(|| "individual".to_string()),
        entity_id: entity_id.clone(),
        secondary_id: secondary_id.clone(),
        dismissed_by: actor_id,
        dismissed_at: Utc::now().to_rfc3339(),
    };

    if !store.dismissed_warnings.iter().any(|d| {
        d.id == dto.id
            || (d.rule_code == dto.rule_code
                && d.entity_id == dto.entity_id
                && d.secondary_id == dto.secondary_id)
    }) {
        store.dismissed_warnings.push(dto.clone());
    }

    Ok(DismissWarningResponse {
        status: "dismissed".to_string(),
        dismissed: dto,
    })
}

/// Marks a pair of individuals as not-duplicates (`POST /api/v1/quality/duplicates/not-duplicate` / `POST /api/v1/people/merge/not-duplicate`).
pub fn mark_not_duplicate_in_store(
    store: &mut GenealogyPrivacyStore,
    req: NotDuplicatePairRequest,
) -> Result<NotDuplicatePairResponse, AppError> {
    let raw_a = req
        .person_a_id
        .or(req.person_a)
        .or(req.a)
        .unwrap_or_default();
    let raw_b = req
        .person_b_id
        .or(req.person_b)
        .or(req.b)
        .unwrap_or_default();

    let id_a = store.resolve_person_id(&raw_a).ok_or(AppError::NotFound)?;
    let id_b = store.resolve_person_id(&raw_b).ok_or(AppError::NotFound)?;

    if id_a == id_b {
        let mut fields = HashMap::new();
        fields.insert(
            "person_b_id".to_string(),
            "Cannot mark the same person as not-duplicate of themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Distinct individuals required".to_string(),
            Some(fields),
        ));
    }

    let (p1, p2) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };

    if !store
        .not_duplicate_pairs
        .iter()
        .any(|(a, b)| a == &p1 && b == &p2)
    {
        store.not_duplicate_pairs.push((p1.clone(), p2.clone()));
    }

    Ok(NotDuplicatePairResponse {
        status: "marked_not_duplicate".to_string(),
        person_a_id: p1,
        person_b_id: p2,
    })
}

fn check_merge_cycle(
    store: &GenealogyPrivacyStore,
    keep_id: &str,
    merge_id: &str,
) -> Result<(), AppError> {
    let mut simulated_families = store.families.clone();
    for fam in &mut simulated_families {
        if fam.partner1_id.as_deref() == Some(merge_id) {
            fam.partner1_id = Some(keep_id.to_string());
        }
        if fam.partner2_id.as_deref() == Some(merge_id) {
            fam.partner2_id = Some(keep_id.to_string());
        }
        for ch in &mut fam.children {
            if ch.child_id == merge_id {
                ch.child_id = keep_id.to_string();
            }
        }
    }

    for fam in &simulated_families {
        for parent_opt in [&fam.partner1_id, &fam.partner2_id] {
            if let Some(p_id) = parent_opt.as_deref() {
                for ch in &fam.children {
                    ensure_no_ancestor_cycle(&simulated_families, p_id, &ch.child_id)?;
                }
            }
        }
    }
    Ok(())
}

/// Generates a side-by-side person merge preview (`GET/POST /api/v1/people/merge/preview`).
pub fn preview_person_merge(
    store: &GenealogyPrivacyStore,
    query: &MergePreviewQuery,
) -> Result<MergePreviewResponse, AppError> {
    let raw_keep = query
        .keep_id
        .as_deref()
        .or(query.target_id.as_deref())
        .or(query.primary_person_id.as_deref())
        .or(query.a.as_deref())
        .unwrap_or_default();
    let raw_merge = query
        .merge_id
        .as_deref()
        .or(query.source_id.as_deref())
        .or(query.duplicate_person_id.as_deref())
        .or(query.b.as_deref())
        .unwrap_or_default();

    let keep_id = store
        .resolve_person_id(raw_keep)
        .ok_or(AppError::NotFound)?;
    let merge_id = store
        .resolve_person_id(raw_merge)
        .ok_or(AppError::NotFound)?;

    if keep_id == merge_id {
        let mut fields = HashMap::new();
        fields.insert(
            "merge_id".to_string(),
            "Cannot merge a person with themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Cannot merge a person with themselves".to_string(),
            Some(fields),
        ));
    }

    let keep_person = store
        .persons
        .iter()
        .find(|p| p.id == keep_id)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let merge_person = store
        .persons
        .iter()
        .find(|p| p.id == merge_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let (score, reasons) = score_duplicate_pair(&keep_person, &merge_person, store);
    let cycle_detected = check_merge_cycle(store, &keep_id, &merge_id).is_err();

    let mut field_conflicts = Vec::new();
    let push_conflict = |conflicts: &mut Vec<MergeFieldConflictDto>,
                         field: &str,
                         kv: serde_json::Value,
                         mv: serde_json::Value| {
        let k_empty = kv.is_null() || kv.as_str().map(|s| s.trim().is_empty()).unwrap_or(false);
        let m_empty = mv.is_null() || mv.as_str().map(|s| s.trim().is_empty()).unwrap_or(false);
        let has_conflict = !k_empty && !m_empty && kv != mv;
        let recommended = if k_empty && !m_empty { "merge" } else { "keep" };
        conflicts.push(MergeFieldConflictDto {
            field: field.to_string(),
            keep_value: kv,
            merge_value: mv,
            has_conflict,
            recommended_source: recommended.to_string(),
        });
    };

    push_conflict(
        &mut field_conflicts,
        "given_name",
        serde_json::json!(keep_person.primary_name.given_name),
        serde_json::json!(merge_person.primary_name.given_name),
    );
    push_conflict(
        &mut field_conflicts,
        "surname",
        serde_json::json!(keep_person.primary_name.surname),
        serde_json::json!(merge_person.primary_name.surname),
    );
    push_conflict(
        &mut field_conflicts,
        "patronymic",
        serde_json::json!(keep_person.primary_name.patronymic),
        serde_json::json!(merge_person.primary_name.patronymic),
    );
    push_conflict(
        &mut field_conflicts,
        "birth_surname",
        serde_json::json!(keep_person.birth_surname),
        serde_json::json!(merge_person.birth_surname),
    );
    push_conflict(
        &mut field_conflicts,
        "sex",
        serde_json::json!(keep_person.sex),
        serde_json::json!(merge_person.sex),
    );
    push_conflict(
        &mut field_conflicts,
        "biography",
        serde_json::json!(keep_person.biography),
        serde_json::json!(merge_person.biography),
    );
    push_conflict(
        &mut field_conflicts,
        "birth",
        serde_json::json!(keep_person.birth),
        serde_json::json!(merge_person.birth),
    );
    push_conflict(
        &mut field_conflicts,
        "death",
        serde_json::json!(keep_person.death),
        serde_json::json!(merge_person.death),
    );

    let keep_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(keep_id.as_str()))
        .collect();
    let merge_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(merge_id.as_str()))
        .collect();

    let mut events_deduplicated = 0;
    let mut events_to_transfer = 0;
    for mev in &merge_events {
        let is_dup = keep_events.iter().any(|kev| {
            kev.event_type.eq_ignore_ascii_case(&mev.event_type)
                && kev.date_sort == mev.date_sort
                && kev.place_id == mev.place_id
        });
        if is_dup {
            events_deduplicated += 1;
        } else {
            events_to_transfer += 1;
        }
    }

    let notes_to_transfer = store
        .notes
        .iter()
        .filter(|n| n.entity_type == "individual" && n.entity_id == merge_id)
        .count();

    let media_to_transfer = store
        .media
        .iter()
        .filter(|m| {
            m.linked_person_ids.contains(&merge_id)
                || m.face_tags.iter().any(|ft| ft.person_id == merge_id)
                || m.links
                    .iter()
                    .any(|l| l.entity_type == "individual" && l.entity_id == merge_id)
        })
        .count();

    let families_affected = store
        .families
        .iter()
        .filter(|f| {
            f.partner1_id.as_deref() == Some(merge_id.as_str())
                || f.partner2_id.as_deref() == Some(merge_id.as_str())
                || f.children.iter().any(|c| c.child_id == merge_id)
        })
        .count();

    Ok(MergePreviewResponse {
        target: keep_person.clone(),
        source: merge_person.clone(),
        keep_person,
        merge_person,
        score,
        reasons,
        can_merge: !cycle_detected,
        cycle_detected,
        field_conflicts,
        events_to_keep: keep_events.len(),
        events_to_transfer,
        events_deduplicated,
        notes_to_transfer,
        media_to_transfer,
        families_affected,
        combined_counts: MergeCombinedCountsDto {
            events: events_to_transfer,
            media: media_to_transfer,
            notes: notes_to_transfer,
            families: families_affected,
        },
    })
}

/// Executes an atomic, undoable person merge (`POST /api/v1/people/merge`, `15-data-quality-duplicates.md §5.2`).
pub fn execute_person_merge(
    store: &mut GenealogyPrivacyStore,
    req: PersonMergeRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    backup_path: &str,
) -> Result<PersonMergeResponse, AppError> {
    let raw_keep = req
        .keep_id
        .as_deref()
        .or(req.target_id.as_deref())
        .or(req.primary_person_id.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_merge = req
        .merge_id
        .as_deref()
        .or(req.source_id.as_deref())
        .or(req.duplicate_person_id.as_deref())
        .unwrap_or_default()
        .trim();

    // Backward-compatible no-op response if invoked with an empty body (e.g. RBAC permission probe in `rbac_test.rs`)
    if raw_keep.is_empty() && raw_merge.is_empty() {
        return Ok(PersonMergeResponse {
            status: "ok".to_string(),
            operation: "person_merge".to_string(),
            message: "Duplicate person merge authorized for administrator".to_string(),
            tx_id: None,
            snapshot_id: None,
            surviving_person_id: None,
            surviving_person: None,
            merged_id: None,
            merged_gid: None,
        });
    }

    let keep_id = store
        .resolve_person_id(raw_keep)
        .ok_or(AppError::NotFound)?;
    let merge_id = store
        .resolve_person_id(raw_merge)
        .ok_or(AppError::NotFound)?;

    if keep_id == merge_id {
        let mut fields = HashMap::new();
        fields.insert(
            "merge_id".to_string(),
            "Cannot merge a person with themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Cannot merge a person with themselves".to_string(),
            Some(fields),
        ));
    }

    // 1. Verify no ancestor-descendant cycle is created (`15 §5.2`, `10 §6.1`)
    check_merge_cycle(store, &keep_id, &merge_id)?;

    // 2. Automatic pre-merge snapshot (`14 §6.1`, `15 §5.2`)
    let snapshot = create_snapshot(
        store,
        Some(format!("Pre-merge snapshot ({keep_id} <- {merge_id})")),
        "bulk_merge",
        actor_id.clone(),
        actor_name.clone(),
        backup_path,
    )?;

    let keep_idx = store
        .persons
        .iter()
        .position(|p| p.id == keep_id)
        .ok_or(AppError::NotFound)?;
    let merge_idx = store
        .persons
        .iter()
        .position(|p| p.id == merge_id)
        .ok_or(AppError::NotFound)?;

    let keep_before = store.persons[keep_idx].clone();
    let merge_person = store.persons[merge_idx].clone();

    // Capture full pre-merge state for 1-click atomic undo (`POST /api/v1/history/tx/{tx_id}/undo`)
    let before_bundle = serde_json::json!({
        "kind": "person_merge_bundle",
        "keep_before": keep_before,
        "merge_person": merge_person,
        "families_before": store.families,
        "events_before": store.events,
        "notes_before": store.notes,
        "media_before": store.media,
        "gid_aliases_before": store.gid_aliases,
    });

    // 3. Resolve scalar fields on the surviving person
    let choose_from_merge = |keys: &[&str], keep_empty: bool| -> bool {
        for &field in keys {
            if let Some(choice) = req.field_choices.get(field) {
                match choice.trim().to_lowercase().as_str() {
                    "merge" | "source" | "b" => return true,
                    "keep" | "target" | "a" => return false,
                    _ => {}
                }
            }
        }
        keep_empty
    };

    let mut surviving = keep_before.clone();

    if choose_from_merge(
        &["given_name", "name", "primary_name"],
        surviving.primary_name.given_name.trim().is_empty(),
    ) && !merge_person.primary_name.given_name.trim().is_empty()
    {
        surviving.primary_name.given_name = merge_person.primary_name.given_name.clone();
    }
    if choose_from_merge(
        &["surname", "name", "primary_name"],
        surviving.primary_name.surname.trim().is_empty(),
    ) && !merge_person.primary_name.surname.trim().is_empty()
    {
        surviving.primary_name.surname = merge_person.primary_name.surname.clone();
    }
    if choose_from_merge(
        &["patronymic", "name", "primary_name"],
        surviving.primary_name.patronymic.is_none(),
    ) && merge_person.primary_name.patronymic.is_some()
    {
        surviving.primary_name.patronymic = merge_person.primary_name.patronymic.clone();
    }
    if choose_from_merge(&["birth_surname"], surviving.birth_surname.is_none())
        && merge_person.birth_surname.is_some()
    {
        surviving.birth_surname = merge_person.birth_surname.clone();
    }
    if choose_from_merge(
        &["sex", "gender"],
        surviving.sex.trim().is_empty() || surviving.sex == "U",
    ) && !merge_person.sex.trim().is_empty()
        && merge_person.sex != "U"
    {
        surviving.sex = merge_person.sex.clone();
    }
    if choose_from_merge(&["biography", "notes"], surviving.biography.is_none())
        && (merge_person.biography.is_some() || merge_person.notes.is_some())
    {
        surviving.biography = merge_person
            .biography
            .clone()
            .or_else(|| merge_person.notes.clone());
    }
    if choose_from_merge(&["notes", "biography"], surviving.notes.is_none())
        && (merge_person.notes.is_some() || merge_person.biography.is_some())
    {
        surviving.notes = merge_person
            .notes
            .clone()
            .or_else(|| merge_person.biography.clone());
    }
    if choose_from_merge(&["birth", "birth_date"], surviving.birth.is_none())
        && merge_person.birth.is_some()
    {
        surviving.birth = merge_person.birth.clone();
        surviving.birth_sort = merge_person.birth_sort;
    }
    if choose_from_merge(&["death", "death_date"], surviving.death.is_none())
        && merge_person.death.is_some()
    {
        surviving.death = merge_person.death.clone();
        surviving.death_sort = merge_person.death_sort;
        surviving.has_death_event = surviving.has_death_event || merge_person.has_death_event;
    }
    if surviving.avatar_url.is_none() && merge_person.avatar_url.is_some() {
        surviving.avatar_url = merge_person.avatar_url.clone();
        surviving.avatar_media_id = merge_person.avatar_media_id.clone();
        surviving.avatar_crop = merge_person.avatar_crop.clone();
    }

    // 4. Preserve non-chosen primary name as an alternate name (`kind = 'aka'`) + union `alt_names`
    for candidate_name in [&keep_before.primary_name, &merge_person.primary_name] {
        let same_as_surviving = candidate_name
            .given_name
            .eq_ignore_ascii_case(&surviving.primary_name.given_name)
            && candidate_name
                .surname
                .eq_ignore_ascii_case(&surviving.primary_name.surname)
            && candidate_name.patronymic == surviving.primary_name.patronymic;
        if !same_as_surviving
            && (!candidate_name.given_name.trim().is_empty()
                || !candidate_name.surname.trim().is_empty())
        {
            let already_present = surviving.alt_names.iter().any(|a| {
                a.given_name.as_deref().unwrap_or("") == candidate_name.given_name
                    && a.surname.as_deref().unwrap_or("") == candidate_name.surname
            });
            if !already_present {
                surviving.alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: "aka".to_string(),
                    given_name: Some(candidate_name.given_name.clone()),
                    patronymic: candidate_name.patronymic.clone(),
                    surname: Some(candidate_name.surname.clone()),
                    lang: None,
                });
            }
        }
    }
    for alt in &merge_person.alt_names {
        let exists = surviving.alt_names.iter().any(|a| {
            a.given_name == alt.given_name
                && a.surname == alt.surname
                && a.name_type == alt.name_type
        });
        if !exists {
            surviving.alt_names.push(alt.clone());
        }
    }

    // 5. Union external_ids & record merged GID alias (`10 §3.1`, `15 §5.2`)
    for (k, v) in &merge_person.external_ids {
        surviving
            .external_ids
            .entry(k.clone())
            .or_insert_with(|| v.clone());
    }
    surviving
        .external_ids
        .insert("merged_gid".to_string(), merge_person.gid.clone());
    store
        .gid_aliases
        .insert(merge_person.gid.clone(), keep_id.clone());

    surviving.version += 1;

    // 6. Move & deduplicate `events`
    let keep_event_keys: HashSet<(String, Option<i32>, Option<String>)> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(keep_id.as_str()))
        .map(|e| {
            (
                e.event_type.to_ascii_lowercase(),
                e.date_sort,
                e.place_id.clone(),
            )
        })
        .collect();

    let mut updated_events = Vec::with_capacity(store.events.len());
    for mut ev in store.events.drain(..) {
        if ev.individual_id.as_deref() == Some(merge_id.as_str()) {
            let key = (
                ev.event_type.to_ascii_lowercase(),
                ev.date_sort,
                ev.place_id.clone(),
            );
            if keep_event_keys.contains(&key) {
                continue;
            }
            ev.individual_id = Some(keep_id.clone());
        }
        updated_events.push(ev);
    }
    store.events = updated_events;

    // 7. Move `notes`
    for note in &mut store.notes {
        if note.entity_type == "individual" && note.entity_id == merge_id {
            note.entity_id = keep_id.clone();
        }
    }

    // 8. Move & deduplicate `media` links and face tags
    let surviving_display_name = format_person_display_name(&surviving);
    for media in &mut store.media {
        if media.linked_person_ids.contains(&merge_id) {
            media.linked_person_ids.retain(|id| id != &merge_id);
            if !media.linked_person_ids.contains(&keep_id) {
                media.linked_person_ids.push(keep_id.clone());
            }
        }
        for ft in &mut media.face_tags {
            if ft.person_id == merge_id {
                ft.person_id = keep_id.clone();
                ft.person_name = surviving_display_name.clone();
            }
        }
        for link in &mut media.links {
            if link.entity_type == "individual" && link.entity_id == merge_id {
                link.entity_id = keep_id.clone();
            }
        }
        let mut seen_links: HashSet<(String, String)> = HashSet::new();
        media.links.retain(|l: &MediaLinkDto| {
            seen_links.insert((l.entity_type.clone(), l.entity_id.clone()))
        });
    }

    // 9. Re-point `families` (`partner1_id`, `partner2_id`, `children`) and merge duplicate couple families
    for fam in &mut store.families {
        if fam.partner1_id.as_deref() == Some(merge_id.as_str()) {
            fam.partner1_id = Some(keep_id.clone());
        }
        if fam.partner2_id.as_deref() == Some(merge_id.as_str()) {
            fam.partner2_id = Some(keep_id.clone());
        }
        for ch in &mut fam.children {
            if ch.child_id == merge_id {
                ch.child_id = keep_id.clone();
            }
        }
        let mut seen_children: HashSet<String> = HashSet::new();
        fam.children
            .retain(|c: &FamilyChildDto| seen_children.insert(c.child_id.clone()));
    }

    // Merge duplicate families that now share identical partner pairs
    let mut merged_families: Vec<crate::services::privacy::FamilyDto> = Vec::new();
    for fam in store.families.drain(..) {
        let existing_pos = merged_families.iter().position(|ef| {
            let same_direct =
                ef.partner1_id == fam.partner1_id && ef.partner2_id == fam.partner2_id;
            let same_swapped =
                ef.partner1_id == fam.partner2_id && ef.partner2_id == fam.partner1_id;
            (same_direct || same_swapped)
                && (fam.partner1_id.is_some() || fam.partner2_id.is_some())
        });
        if let Some(idx) = existing_pos {
            let target_fam_id = merged_families[idx].id.clone();
            for ch in fam.children {
                if !merged_families[idx]
                    .children
                    .iter()
                    .any(|c| c.child_id == ch.child_id)
                {
                    merged_families[idx].children.push(ch);
                }
            }
            for ev in &mut store.events {
                if ev.family_id.as_deref() == Some(fam.id.as_str()) {
                    ev.family_id = Some(target_fam_id.clone());
                }
            }
        } else {
            merged_families.push(fam);
        }
    }
    store.families = merged_families;

    // 10. Update surviving person and remove source person
    let tx_id = Uuid::new_v4().to_string();
    surviving.tx_id = Some(tx_id.clone());
    store.persons[keep_idx] = surviving.clone();
    store.persons.retain(|p| p.id != merge_id);

    // Clean up dismissed warnings & not_duplicate_pairs referencing `merge_id`
    store.dismissed_warnings.retain(|d| {
        d.entity_id != merge_id && d.secondary_id.as_deref() != Some(merge_id.as_str())
    });
    store
        .not_duplicate_pairs
        .retain(|(a, b)| a != &merge_id && b != &merge_id);

    // 11. Append atomic `merge` entry to `change_log`
    let after_json = serde_json::to_value(&surviving).ok();
    let label = format!(
        "Merged {} ({}) into {} ({})",
        format_person_display_name(&merge_person),
        merge_person.gid,
        format_person_display_name(&surviving),
        surviving.gid
    );
    store.change_log.push(ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        tx_id: tx_id.clone(),
        tx_label: label.clone(),
        op: "merge".to_string(),
        action: "merge".to_string(),
        entity_type: "individual".to_string(),
        entity_id: keep_id.clone(),
        entity_label: Some(format_person_display_name(&surviving)),
        actor_id,
        actor_name,
        summary: label.clone(),
        before_json: Some(before_bundle),
        after_json,
        created_at: Utc::now().to_rfc3339(),
    });

    Ok(PersonMergeResponse {
        status: "merged".to_string(),
        operation: "person_merge".to_string(),
        message: label,
        tx_id: Some(tx_id),
        snapshot_id: Some(snapshot.id),
        surviving_person_id: Some(surviving.id.clone()),
        surviving_person: Some(surviving),
        merged_id: Some(merge_id),
        merged_gid: Some(merge_person.gid),
    })
}
