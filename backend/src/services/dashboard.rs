use chrono::{Datelike, Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use utoipa::ToSchema;

use crate::{
    models::gendate::{CalendarType, GenDate, GenDateMod},
    services::{
        history::{query_history_feed, ChangeLogTransactionGroupDto, HistoryFeedQuery},
        privacy::{
            redact_media_for_caller, redact_person_for_caller, GenealogyPrivacyStore,
            MediaAssetSummaryDto, TreePersonNodeDto,
        },
        quality::{
            build_quality_summary, format_person_display_name, to_warning_person_ref,
            QualityWarningPersonRefDto,
        },
        rbac::CallerAccess,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AnniversariesQuery {
    #[serde(default)]
    pub days: Option<u32>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub home_person_id: Option<String>,
    #[serde(default)]
    pub reference_date: Option<String>,
    #[serde(default)]
    pub include_all_memorials: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AnniversaryItemDto {
    pub id: String,
    pub kind: String,
    pub person: QualityWarningPersonRefDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<QualityWarningPersonRefDto>,
    pub event_date: String,
    pub original_date_text: String,
    pub calendar: String,
    pub month: u32,
    pub day: u32,
    pub next_occurrence_date: String,
    pub days_until: i64,
    pub years_elapsed: i32,
    pub is_milestone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AnniversariesListResponse {
    pub items: Vec<AnniversaryItemDto>,
    pub days_window: u32,
    pub scope: String,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TreeStatsResponse {
    pub total_persons: usize,
    pub living_count: usize,
    pub deceased_count: usize,
    pub male_count: usize,
    pub female_count: usize,
    pub unknown_sex_count: usize,
    pub families_count: usize,
    pub events_count: usize,
    pub places_count: usize,
    pub media_count: usize,
    pub notes_count: usize,
    pub generations_count: usize,
    pub missing_birth_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub earliest_birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_ancestor: Option<QualityWarningPersonRefDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardQualityWidgetDto {
    pub total_open_warnings: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub duplicate_candidates_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DashboardResponse {
    pub is_empty: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_person: Option<TreePersonNodeDto>,
    pub upcoming_anniversaries: Vec<AnniversaryItemDto>,
    pub anniversaries: Vec<AnniversaryItemDto>,
    pub recent_activity: Vec<ChangeLogTransactionGroupDto>,
    pub stats: TreeStatsResponse,
    pub latest_photos: Vec<MediaAssetSummaryDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_summary: Option<DashboardQualityWidgetDto>,
}

fn julian_offset_days(year: i32) -> u64 {
    match year {
        1900..=2099 => 13,
        1800..=1899 => 12,
        1700..=1799 => 11,
        1582..=1699 => 10,
        _ => {
            let century = year.div_euclid(100);
            (century - century.div_euclid(4) - 2).max(0) as u64
        }
    }
}

/// Extracts an exact Gregorian `(year, month, day, calendar_str)` from a raw date string,
/// shifting Julian (`@#DJULIAN@` / `(ст. ст.)` / `(Julian)`) dates into Gregorian (`16-dashboard-home.md §3.2`).
fn extract_exact_gregorian_ymd(raw_date: &str) -> Option<(i32, u32, u32, String)> {
    let trimmed = raw_date.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Handle GEDCOM `@#DJULIAN@` prefix if present
    let (cleaned, explicit_julian) = if let Some(rest) = trimmed.strip_prefix("@#DJULIAN@") {
        (rest.trim(), true)
    } else {
        (trimmed, false)
    };

    let gd = GenDate::parse(cleaned);
    if gd.r#mod != GenDateMod::Exact {
        return None;
    }
    let start = gd.start?;
    let m = u32::from(start.m?);
    let d = u32::from(start.d?);
    let y = start.y;

    let is_julian = explicit_julian || gd.cal == CalendarType::Julian;
    if is_julian {
        let offset = julian_offset_days(y);
        if let Some(shifted) =
            NaiveDate::from_ymd_opt(y, m, d).and_then(|dt| dt.checked_add_days(Days::new(offset)))
        {
            return Some((
                shifted.year(),
                shifted.month(),
                shifted.day(),
                "julian".to_string(),
            ));
        }
    }

    Some((y, m, d, "gregorian".to_string()))
}

fn next_occurrence_from(ref_date: NaiveDate, month: u32, day: u32) -> (NaiveDate, i64) {
    let candidate_for_year = |yr: i32| -> NaiveDate {
        if let Some(dt) = NaiveDate::from_ymd_opt(yr, month, day) {
            dt
        } else if month == 2 && day == 29 {
            // Feb 29 in a non-leap year falls on Feb 28 (`16-dashboard-home.md §3.2`)
            NaiveDate::from_ymd_opt(yr, 2, 28).unwrap_or(ref_date)
        } else {
            NaiveDate::from_ymd_opt(yr, month.clamp(1, 12), 1).unwrap_or(ref_date)
        }
    };

    let this_year_dt = candidate_for_year(ref_date.year());
    let next_dt = if this_year_dt >= ref_date {
        this_year_dt
    } else {
        candidate_for_year(ref_date.year() + 1)
    };
    let days_until = (next_dt - ref_date).num_days();
    (next_dt, days_until)
}

/// Computes the set of person IDs within `max_hops` (default 4) of `focal_id` via BFS across parent/child/spouse edges.
fn close_relatives_within_hops(
    store: &GenealogyPrivacyStore,
    focal_id: &str,
    max_hops: usize,
) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    visited.insert(focal_id.to_string());
    queue.push_back((focal_id.to_string(), 0));

    while let Some((curr_id, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }
        for fam in &store.families {
            let is_member = fam.partner1_id.as_deref() == Some(curr_id.as_str())
                || fam.partner2_id.as_deref() == Some(curr_id.as_str())
                || fam.children.iter().any(|c| c.child_id == curr_id);
            if is_member {
                for neighbor in fam
                    .partner1_id
                    .iter()
                    .chain(fam.partner2_id.iter())
                    .chain(fam.children.iter().map(|c| &c.child_id))
                {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
    }
    visited
}

/// Computes the maximum generation count in the tree (`16-dashboard-home.md §3.4`).
pub fn compute_generations_count(store: &GenealogyPrivacyStore) -> usize {
    if store.persons.is_empty() {
        return 0;
    }

    let mut children_by_parent: HashMap<&str, Vec<&str>> = HashMap::new();
    for fam in &store.families {
        for p_id in [fam.partner1_id.as_deref(), fam.partner2_id.as_deref()]
            .into_iter()
            .flatten()
        {
            let entry = children_by_parent.entry(p_id).or_default();
            for ch in &fam.children {
                entry.push(ch.child_id.as_str());
            }
        }
    }

    let mut max_depth = 1usize;
    for person in &store.persons {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
        visited.insert(person.id.as_str());
        queue.push_back((person.id.as_str(), 1));

        while let Some((curr, depth)) = queue.pop_front() {
            if depth > max_depth {
                max_depth = depth;
            }
            if depth >= 30 {
                continue;
            }
            if let Some(children) = children_by_parent.get(curr) {
                for &ch_id in children {
                    if visited.insert(ch_id) {
                        queue.push_back((ch_id, depth + 1));
                    }
                }
            }
        }
    }

    max_depth
}

/// Computes upcoming birthdays, wedding anniversaries, and round-number memorial anniversaries (`16-dashboard-home.md §3.2`).
pub fn compute_upcoming_anniversaries(
    store: &GenealogyPrivacyStore,
    query: &AnniversariesQuery,
    access: &CallerAccess,
    locale: &str,
) -> AnniversariesListResponse {
    let days_window = query.days.unwrap_or(30).clamp(1, 365);
    let scope = query
        .scope
        .as_deref()
        .unwrap_or("all")
        .trim()
        .to_ascii_lowercase();
    let include_all_memorials = query.include_all_memorials.unwrap_or(false);

    let ref_date = query
        .reference_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
        .unwrap_or_else(|| Utc::now().date_naive());

    let close_set: Option<HashSet<String>> = if scope == "close" {
        query
            .home_person_id
            .as_deref()
            .and_then(|id| store.resolve_person_id(id))
            .map(|hid| close_relatives_within_hops(store, &hid, 4))
    } else {
        None
    };

    let mut items = Vec::new();

    for person in &store.persons {
        if let Some(ref cset) = close_set {
            if !cset.contains(&person.id) {
                continue;
            }
        }

        let Some(visible_person) = redact_person_for_caller(person, access, locale) else {
            continue;
        };

        // For AnonymousPublic, living persons have redacted dates and must not appear in upcoming birthdays (`02-rbac-access-control.md`, `16 §2`)
        if *access == CallerAccess::AnonymousPublic && person.is_living {
            continue;
        }

        // 1. Birthdays (living or deceased with exact birth date)
        let birth_raw = person
            .birth
            .as_ref()
            .and_then(|b| b.date.as_deref())
            .or_else(|| {
                store
                    .events
                    .iter()
                    .find(|e| {
                        e.individual_id.as_deref() == Some(person.id.as_str())
                            && e.event_type.eq_ignore_ascii_case("birth")
                    })
                    .and_then(|e| {
                        e.date
                            .as_ref()
                            .map(|d| d.text.as_str())
                            .or(e.gedcom_date.as_deref())
                    })
            });

        if let Some(b_str) = birth_raw {
            if let Some((by, bm, bd, cal)) = extract_exact_gregorian_ymd(b_str) {
                let (next_dt, days_until) = next_occurrence_from(ref_date, bm, bd);
                if days_until >= 0 && days_until <= i64::from(days_window) {
                    let years_elapsed = (next_dt.year() - by).max(0);
                    items.push(AnniversaryItemDto {
                        id: format!("birthday:{}:{next_dt}", person.id),
                        kind: "birthday".to_string(),
                        person: to_warning_person_ref(&visible_person),
                        partner: None,
                        event_date: format!("{by:04}-{bm:02}-{bd:02}"),
                        original_date_text: b_str.to_string(),
                        calendar: cal,
                        month: bm,
                        day: bd,
                        next_occurrence_date: next_dt.to_string(),
                        days_until,
                        years_elapsed,
                        is_milestone: years_elapsed > 0 && years_elapsed % 5 == 0,
                    });
                }
            }
        }

        // 2. Memorial (Death) Anniversaries (round milestones every 5/10/25/50/100 years, or all if requested)
        let death_raw = person
            .death
            .as_ref()
            .and_then(|d| d.date.as_deref())
            .or_else(|| {
                store
                    .events
                    .iter()
                    .find(|e| {
                        e.individual_id.as_deref() == Some(person.id.as_str())
                            && e.event_type.eq_ignore_ascii_case("death")
                    })
                    .and_then(|e| {
                        e.date
                            .as_ref()
                            .map(|d| d.text.as_str())
                            .or(e.gedcom_date.as_deref())
                    })
            });

        if let Some(d_str) = death_raw {
            if let Some((dy, dm, dd, cal)) = extract_exact_gregorian_ymd(d_str) {
                let (next_dt, days_until) = next_occurrence_from(ref_date, dm, dd);
                let years_elapsed = (next_dt.year() - dy).max(0);
                let is_milestone = years_elapsed > 0 && years_elapsed % 5 == 0;
                if days_until >= 0
                    && days_until <= i64::from(days_window)
                    && (is_milestone || include_all_memorials)
                {
                    items.push(AnniversaryItemDto {
                        id: format!("memorial:{}:{next_dt}", person.id),
                        kind: "memorial".to_string(),
                        person: to_warning_person_ref(&visible_person),
                        partner: None,
                        event_date: format!("{dy:04}-{dm:02}-{dd:02}"),
                        original_date_text: d_str.to_string(),
                        calendar: cal,
                        month: dm,
                        day: dd,
                        next_occurrence_date: next_dt.to_string(),
                        days_until,
                        years_elapsed,
                        is_milestone,
                    });
                }
            }
        }
    }

    // 3. Wedding Anniversaries
    for fam in &store.families {
        let (Some(p1_id), Some(p2_id)) = (fam.partner1_id.as_deref(), fam.partner2_id.as_deref())
        else {
            continue;
        };
        if let Some(ref cset) = close_set {
            if !cset.contains(p1_id) && !cset.contains(p2_id) {
                continue;
            }
        }
        let Some(p1_raw) = store.persons.iter().find(|p| p.id == p1_id) else {
            continue;
        };
        let Some(p2_raw) = store.persons.iter().find(|p| p.id == p2_id) else {
            continue;
        };
        if *access == CallerAccess::AnonymousPublic && (p1_raw.is_living || p2_raw.is_living) {
            continue;
        }
        let Some(p1_vis) = redact_person_for_caller(p1_raw, access, locale) else {
            continue;
        };
        let Some(p2_vis) = redact_person_for_caller(p2_raw, access, locale) else {
            continue;
        };

        let marriage_raw = store
            .events
            .iter()
            .find(|e| {
                e.family_id.as_deref() == Some(fam.id.as_str())
                    && (e.event_type.eq_ignore_ascii_case("marriage")
                        || e.event_type.eq_ignore_ascii_case("marr"))
            })
            .and_then(|e| {
                e.date
                    .as_ref()
                    .map(|d| d.text.as_str())
                    .or(e.gedcom_date.as_deref())
            });

        if let Some(m_str) = marriage_raw {
            if let Some((my, mm, md, cal)) = extract_exact_gregorian_ymd(m_str) {
                let (next_dt, days_until) = next_occurrence_from(ref_date, mm, md);
                if days_until >= 0 && days_until <= i64::from(days_window) {
                    let years_elapsed = (next_dt.year() - my).max(0);
                    items.push(AnniversaryItemDto {
                        id: format!("wedding:{}:{next_dt}", fam.id),
                        kind: "wedding".to_string(),
                        person: to_warning_person_ref(&p1_vis),
                        partner: Some(to_warning_person_ref(&p2_vis)),
                        event_date: format!("{my:04}-{mm:02}-{md:02}"),
                        original_date_text: m_str.to_string(),
                        calendar: cal,
                        month: mm,
                        day: md,
                        next_occurrence_date: next_dt.to_string(),
                        days_until,
                        years_elapsed,
                        is_milestone: years_elapsed > 0 && years_elapsed % 5 == 0,
                    });
                }
            }
        }
    }

    items.sort_by(|a, b| {
        a.days_until
            .cmp(&b.days_until)
            .then_with(|| a.id.cmp(&b.id))
    });
    let total = items.len();

    AnniversariesListResponse {
        items,
        days_window,
        scope,
        total,
    }
}

/// Computes tree-wide statistics (`GET /api/v1/stats` & `dashboard.stats`).
pub fn compute_tree_stats(
    store: &GenealogyPrivacyStore,
    access: &CallerAccess,
    locale: &str,
) -> TreeStatsResponse {
    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, access, locale))
        .collect();

    let total_persons = visible_persons.len();
    let mut living_count = 0;
    let mut deceased_count = 0;
    let mut male_count = 0;
    let mut female_count = 0;
    let mut unknown_sex_count = 0;
    let mut missing_birth_count = 0;
    let mut earliest_birth_year: Option<i32> = None;
    let mut latest_birth_year: Option<i32> = None;
    let mut oldest_person: Option<(&TreePersonNodeDto, i32)> = None;

    for p in &visible_persons {
        if p.is_living {
            living_count += 1;
        } else {
            deceased_count += 1;
        }
        match p.sex.trim().to_ascii_uppercase().as_str() {
            "M" => male_count += 1,
            "F" => female_count += 1,
            _ => unknown_sex_count += 1,
        }

        let by_opt = p.birth_sort.map(|s| s / 10_000).or_else(|| {
            p.birth
                .as_ref()
                .and_then(|b| b.date.as_deref())
                .and_then(|d| GenDate::parse(d).start.map(|st| st.y))
        });

        if let Some(by) = by_opt {
            earliest_birth_year = Some(earliest_birth_year.map_or(by, |cur| cur.min(by)));
            latest_birth_year = Some(latest_birth_year.map_or(by, |cur| cur.max(by)));
            if oldest_person.is_none_or(|(_, cur_y)| by < cur_y) {
                oldest_person = Some((p, by));
            }
        } else {
            missing_birth_count += 1;
        }
    }

    let visible_media_count = store
        .media
        .iter()
        .filter_map(|m| redact_media_for_caller(m, access, store))
        .count();

    let visible_notes_count = store
        .notes
        .iter()
        .filter(|n| !n.is_private || access.can_view_private())
        .count();

    let generations_count = compute_generations_count(store);

    TreeStatsResponse {
        total_persons,
        living_count,
        deceased_count,
        male_count,
        female_count,
        unknown_sex_count,
        families_count: store.families.len(),
        events_count: store.events.len(),
        places_count: store.places.len(),
        media_count: visible_media_count,
        notes_count: visible_notes_count,
        generations_count,
        missing_birth_count,
        earliest_birth_year,
        latest_birth_year,
        oldest_ancestor: oldest_person.map(|(p, _)| to_warning_person_ref(p)),
    }
}

/// Builds the full dashboard response (`GET /api/v1/dashboard`).
pub fn build_dashboard_response(
    store: &GenealogyPrivacyStore,
    access: &CallerAccess,
    locale: &str,
    preferred_home_person_id: Option<&str>,
    anniversary_query: &AnniversariesQuery,
) -> DashboardResponse {
    let is_empty = store.persons.is_empty();

    // Resolve home person: preferred -> oldest ancestor -> first visible person
    let home_person = preferred_home_person_id
        .and_then(|id| store.resolve_person_id(id))
        .and_then(|resolved| store.persons.iter().find(|p| p.id == resolved))
        .and_then(|p| redact_person_for_caller(p, access, locale))
        .or_else(|| {
            store
                .persons
                .iter()
                .filter_map(|p| redact_person_for_caller(p, access, locale))
                .min_by_key(|p| p.birth_sort.unwrap_or(i32::MAX))
        });

    let mut ann_q = anniversary_query.clone();
    if ann_q.home_person_id.is_none() {
        ann_q.home_person_id = home_person.as_ref().map(|p| p.id.clone());
    }
    let anniversaries = compute_upcoming_anniversaries(store, &ann_q, access, locale);

    let recent_activity = match access {
        CallerAccess::AnonymousPublic => vec![],
        CallerAccess::Admin => {
            let feed = query_history_feed(
                store,
                &HistoryFeedQuery {
                    limit: Some(5),
                    ..Default::default()
                },
            );
            feed.transactions
        }
        CallerAccess::Editor | CallerAccess::Viewer => {
            let feed = query_history_feed(
                store,
                &HistoryFeedQuery {
                    limit: Some(25),
                    op: Some("create".to_string()),
                    entity_type: Some("individual".to_string()),
                    ..Default::default()
                },
            );
            feed.transactions
                .into_iter()
                .filter(|tx| {
                    tx.changes.iter().all(|ch| {
                        store
                            .persons
                            .iter()
                            .find(|p| p.id == ch.entity_id)
                            .and_then(|p| redact_person_for_caller(p, access, locale))
                            .is_some()
                    })
                })
                .take(5)
                .collect()
        }
    };

    let stats = compute_tree_stats(store, access, locale);

    let latest_photos = match access {
        CallerAccess::AnonymousPublic => vec![],
        _ => {
            let mut photos: Vec<MediaAssetSummaryDto> = store
                .media
                .iter()
                .filter_map(|m| redact_media_for_caller(m, access, store))
                .collect();
            photos.reverse();
            photos.truncate(6);
            photos
        }
    };

    let quality_summary = if *access == CallerAccess::Admin {
        let q = build_quality_summary(store);
        Some(DashboardQualityWidgetDto {
            total_open_warnings: q.total_open_warnings,
            warning_count: q.warning_count,
            info_count: q.info_count,
            duplicate_candidates_count: q.duplicate_candidates_count,
        })
    } else {
        None
    };

    let _ = format_person_display_name;

    DashboardResponse {
        is_empty,
        home_person,
        upcoming_anniversaries: anniversaries.items.clone(),
        anniversaries: anniversaries.items,
        recent_activity,
        stats,
        latest_photos,
        quality_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::privacy::GenealogyPrivacyStore;

    #[test]
    fn test_latest_photos_suppressed_for_public_anonymous() {
        let mut store = GenealogyPrivacyStore::default();
        store.media.push(MediaAssetSummaryDto {
            id: "m1".to_string(),
            gid: "M0001".to_string(),
            title: "Old Photo".to_string(),
            mime_type: "image/jpeg".to_string(),
            file_url: "/api/v1/media/m1/file".to_string(),
            file_name: None,
            original_file_name: None,
            file_size_bytes: None,
            sha256: None,
            width: None,
            height: None,
            description: None,
            date_taken: None,
            date_sort: None,
            place_id: None,
            place_full_name: None,
            is_missing: false,
            external_path: None,
            is_private: false,
            only_living_persons: false,
            linked_person_ids: vec![],
            face_tags: vec![],
            links: vec![],
            version: 1,
        });

        let query = AnniversariesQuery::default();
        let admin_resp = build_dashboard_response(&store, &CallerAccess::Admin, "en", None, &query);
        assert_eq!(admin_resp.latest_photos.len(), 1);

        let editor_resp =
            build_dashboard_response(&store, &CallerAccess::Editor, "en", None, &query);
        assert_eq!(editor_resp.latest_photos.len(), 1);

        let viewer_resp =
            build_dashboard_response(&store, &CallerAccess::Viewer, "en", None, &query);
        assert_eq!(viewer_resp.latest_photos.len(), 1);

        let guest_resp =
            build_dashboard_response(&store, &CallerAccess::AnonymousPublic, "en", None, &query);
        assert!(
            guest_resp.latest_photos.is_empty(),
            "Guest dashboard must not expose latest_photos"
        );
    }
}
