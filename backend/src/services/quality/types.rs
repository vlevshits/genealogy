#[allow(unused_imports)]
use super::*;

/// Canonical soft validation rule codes (`docs/features/15-data-quality-duplicates.md §2.2`).
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
pub const RULE_PLACE_UNPARSED_PATH: &str = "PLACE_UNPARSED_PATH";
pub const RULE_PLACE_ORPHAN_HIERARCHY: &str = "PLACE_ORPHAN_HIERARCHY";
pub const RULE_PLACE_TYPE_MISMATCH: &str = "PLACE_TYPE_MISMATCH";
pub const RULE_PLACE_DUPLICATE_CANDIDATE: &str = "PLACE_DUPLICATE_CANDIDATE";

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_detached: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualityCategoryCountsDto {
    pub chronology: usize,
    pub family_age: usize,
    pub completeness: usize,
    pub duplicates: usize,
    #[serde(default)]
    pub places: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualitySeverityCountsDto {
    #[serde(default)]
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct QualitySummaryResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    pub total_open_warnings: usize,
    #[serde(default)]
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub duplicate_candidates_count: usize,
    #[serde(default)]
    pub duplicate_events_count: usize,
    #[serde(default)]
    pub place_hierarchy_issues: usize,
    #[serde(default)]
    pub place_hierarchy_high_confidence: usize,
    pub dismissed_count: usize,
    pub by_category: QualityCategoryCountsDto,
    pub by_severity: QualitySeverityCountsDto,
    pub by_rule: HashMap<String, usize>,
    pub warnings: Vec<QualityWarningDto>,
    pub duplicates: Vec<DuplicateCandidatePairDto>,
    #[serde(default)]
    pub duplicate_events: Vec<DuplicateEventGroupDto>,
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
pub(crate) struct PlausibleDateWindow {
    pub(crate) min_year: i32,

    pub(crate) max_year: i32,

    pub(crate) min_sort: i32,

    pub(crate) max_sort: i32,
}

pub(crate) fn parse_plausible_window(
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

pub(crate) fn event_date_text(ev: &LifeEventDto) -> Option<&str> {
    ev.date
        .as_ref()
        .map(|d| d.text.as_str())
        .or(ev.gedcom_date.as_deref())
}

pub(crate) fn person_birth_window(
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

pub(crate) fn parse_exact_date(date_str: Option<&str>) -> Option<chrono::NaiveDate> {
    let raw = date_str?.trim();
    if raw.is_empty() {
        return None;
    }
    let gd = GenDate::parse(raw);
    gd.exact_naive_date()
}

pub(crate) fn person_exact_birth_date(
    p: &TreePersonNodeDto,
    events: &[LifeEventDto],
) -> Option<chrono::NaiveDate> {
    let ev_opt = events.iter().find(|e| {
        e.individual_id.as_deref() == Some(p.id.as_str())
            && matches!(e.event_type.as_str(), "birth" | "christening" | "baptism")
    });
    parse_exact_date(p.birth.as_ref().and_then(|b| b.date.as_deref()))
        .or_else(|| parse_exact_date(ev_opt.and_then(event_date_text)))
}

pub(crate) fn person_death_window(
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

pub(crate) fn is_warning_dismissed(
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
