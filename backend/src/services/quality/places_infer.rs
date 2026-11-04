#[allow(unused_imports)]
use super::*;

// ============================================================================
// Automated Place Hierarchy Quality & Restructuring Engine (`15 §4`, `12 §3.5`)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProposedHierarchyStepDto {
    pub name: String,
    pub place_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_place_gid: Option<String>,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ParentPlaceCandidateDto {
    pub place_id: String,
    pub gid: String,
    pub name: String,
    pub full_name: String,
    pub place_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    pub confidence: i32,
    pub match_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceHierarchyIssueDto {
    pub id: String,
    pub place_id: String,
    pub place_gid: String,
    pub issue_type: String,
    pub rule_code: String,
    pub severity: String,
    pub confidence: i32,
    pub confidence_tier: String,
    pub current_name: String,
    pub current_full_name: String,
    pub current_place_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_parent_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    pub events_count: u32,
    pub children_count: u32,
    pub proposed_leaf_name: String,
    pub proposed_clean_name: String,
    pub proposed_place_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_parent_name: Option<String>,
    pub proposed_full_name: String,
    pub proposed_chain: Vec<ProposedHierarchyStepDto>,
    pub parent_candidates: Vec<ParentPlaceCandidateDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consolidate_with_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consolidate_with_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserve_alt_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PlaceHierarchyAnalysisCountsDto {
    pub total_places: usize,
    pub total_issues: usize,
    pub high_confidence_count: usize,
    pub medium_confidence_count: usize,
    pub low_confidence_count: usize,
    pub unparsed_paths_count: usize,
    pub orphan_places_count: usize,
    pub type_mismatches_count: usize,
    pub duplicate_nodes_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceHierarchyAnalysisResponseDto {
    pub status: String,
    pub counts: PlaceHierarchyAnalysisCountsDto,
    pub issues: Vec<PlaceHierarchyIssueDto>,
    pub items: Vec<PlaceHierarchyIssueDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct PlaceHierarchyAnalysisQuery {
    #[serde(default)]
    pub issue_type: Option<String>,
    #[serde(default)]
    pub min_confidence: Option<i32>,
    #[serde(default)]
    pub include_dismissed: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct PlaceHierarchyPreviewQuery {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub place_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct FixPlaceHierarchyRequest {
    #[serde(default)]
    pub place_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub parent_path: Option<String>,
    #[serde(default)]
    pub clean_name: Option<String>,
    #[serde(default)]
    pub proposed_leaf_name: Option<String>,
    #[serde(default)]
    pub place_type: Option<String>,
    #[serde(default)]
    pub consolidate_with_place_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FixPlaceHierarchyResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    pub tx_id: String,
    pub place: PlaceSummaryDto,
    pub created_parents: Vec<PlaceSummaryDto>,
    pub consolidated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_source_place_id: Option<String>,
    pub breadcrumbs: Vec<PlaceBreadcrumbDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct BatchFixPlacesRequest {
    #[serde(default)]
    pub min_confidence: Option<i32>,
    #[serde(default)]
    pub place_ids: Option<Vec<String>>,
    #[serde(default)]
    pub issue_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct BatchFixPlacesResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    pub tx_id: String,
    pub fixed_count: usize,
    pub created_parents_count: usize,
    pub consolidated_count: usize,
    pub fixed_place_ids: Vec<String>,
    pub places: Vec<PlaceSummaryDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DismissPlaceIssueRequest {
    #[serde(default)]
    pub place_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub rule_code: Option<String>,
}

/// Computes the great-circle distance in kilometers between two `(lat, lon)` points using the Haversine formula (`15 §4.2.3`).
pub fn haversine_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

pub(crate) fn place_hierarchy_rank(place_type: &str) -> u8 {
    match place_type.trim().to_ascii_lowercase().as_str() {
        "country" => 1,
        "region" => 2,
        "district" => 3,
        "city" | "village" | "parish" => 4,
        "cemetery" => 5,
        _ => 4,
    }
}

/// Infers canonical `place_type` and strips locality abbreviations (`г. `, `с. `, `д. `, etc.) (`15 §4.2.1, §4.2.2`).
pub fn infer_place_type_and_clean_name(raw_segment: &str) -> (String, Option<String>) {
    let trimmed = raw_segment.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    let lower = normalize_russian_orthography_word(trimmed);

    // 1. Exact or keyword Country checks
    let known_countries = [
        "россииская империя",
        "российская империя",
        "россия",
        "российская федерация",
        "рф",
        "ссср",
        "рсфср",
        "союз советских социалистических республик",
        "украина",
        "беларусь",
        "белоруссия",
        "польша",
        "царство польское",
        "литва",
        "латвия",
        "эстония",
        "молдова",
        "молдавия",
        "казахстан",
        "грузия",
        "армения",
        "азербайджан",
        "узбекистан",
        "германия",
        "франция",
        "великобритания",
        "сша",
        "канада",
        "израиль",
        "австрия",
        "австро-венгрия",
        "russian empire",
        "russia",
        "russian federation",
        "ussr",
        "soviet union",
        "ukraine",
        "belarus",
        "poland",
        "lithuania",
        "latvia",
        "estonia",
        "germany",
        "france",
        "united kingdom",
        "england",
        "usa",
        "united states",
        "canada",
        "israel",
        "austria",
        "austro-hungary",
    ];
    if known_countries.contains(&lower.as_str())
        || lower.contains("империя")
        || lower.contains("республика")
        || lower.contains("федерация")
        || lower.contains("королевство")
        || lower.contains("empire")
        || lower.contains("kingdom")
        || lower.contains("republic")
        || lower.contains("federation")
    {
        return (trimmed.to_string(), Some("country".to_string()));
    }

    // 2. Cemetery checks
    if lower.contains("кладбище")
        || lower.contains("погост")
        || lower.contains("некрополь")
        || lower.contains("cemetery")
        || lower.contains("graveyard")
        || lower.contains("burial ground")
    {
        return (trimmed.to_string(), Some("cemetery".to_string()));
    }

    // 3. Parish checks
    if lower.contains("приход") || lower.contains("parish") || lower.contains("churchyard") {
        return (trimmed.to_string(), Some("parish".to_string()));
    }

    // 4. Region checks (`губерния`, `область`, `край`, `воеводство`, `governorate`, `oblast`, `province`)
    for (abbr, full) in [
        (" губ.", " губерния"),
        (" г-я", " губерния"),
        (" обл.", " область"),
    ] {
        if lower.ends_with(abbr) {
            let prefix_len = trimmed.len().saturating_sub(abbr.len());
            let expanded = format!("{}{}", trimmed[..prefix_len].trim(), full);
            return (expanded, Some("region".to_string()));
        }
    }
    let region_keywords = [
        "губерния",
        "область",
        "край",
        "краи",
        "воеводство",
        "наместничество",
        "провинция",
        "governorate",
        "gubernia",
        "guberniya",
        "oblast",
        "province",
        "voivodeship",
        "krai",
    ];
    if region_keywords.iter().any(|kw| {
        lower
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w == *kw)
    }) {
        return (trimmed.to_string(), Some("region".to_string()));
    }

    // 5. District checks (`уезд`, `район`, `волость`, `округ`, `uyezd`, `raion`, `district`, `county`)
    for (abbr, full) in [(" у.", " уезд"), (" р-н", " район"), (" вол.", " волость")]
    {
        if lower.ends_with(abbr) {
            let prefix_len = trimmed.len().saturating_sub(abbr.len());
            let expanded = format!("{}{}", trimmed[..prefix_len].trim(), full);
            return (expanded, Some("district".to_string()));
        }
    }
    let district_keywords = [
        "уезд",
        "район",
        "раион",
        "округ",
        "волость",
        "uyezd",
        "uezd",
        "raion",
        "district",
        "county",
        "township",
        "okrug",
        "volost",
    ];
    if district_keywords.iter().any(|kw| {
        lower
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w == *kw)
    }) {
        return (trimmed.to_string(), Some("district".to_string()));
    }

    // 6. Locality prefixes/suffixes (`г. `, `гор. `, `город `, `с. `, `село `, `д. `, `дер. `, `деревня `, etc.)
    let city_prefixes = [
        "г. ",
        "г.",
        "гор. ",
        "гор.",
        "город ",
        "пгт. ",
        "пгт ",
        "местечко ",
        "м. ",
        "city of ",
        "town of ",
    ];
    let lower_raw = trimmed.to_lowercase();
    for pref in city_prefixes {
        if lower_raw.starts_with(pref) {
            let rest = trimmed[pref.len()..].trim();
            if !rest.is_empty() {
                return (rest.to_string(), Some("city".to_string()));
            }
        }
    }
    for suf in [" город", " г.", " city", " town", " borough"] {
        if lower_raw.ends_with(suf) {
            let rest = trimmed[..trimmed.len() - suf.len()].trim();
            if !rest.is_empty() {
                return (rest.to_string(), Some("city".to_string()));
            }
        }
    }

    let village_prefixes = [
        "с. ",
        "с.",
        "село ",
        "д. ",
        "д.",
        "дер. ",
        "дер.",
        "деревня ",
        "пос. ",
        "пос.",
        "п. ",
        "посёлок ",
        "поселок ",
        "ст. ",
        "станица ",
        "х. ",
        "хут. ",
        "хутор ",
        "слобода ",
        "сл. ",
        "аул ",
        "village of ",
        "hamlet of ",
    ];
    for pref in village_prefixes {
        if lower_raw.starts_with(pref) {
            let rest = trimmed[pref.len()..].trim();
            if !rest.is_empty() {
                return (rest.to_string(), Some("village".to_string()));
            }
        }
    }
    for suf in [
        " село",
        " деревня",
        " хутор",
        " станица",
        " посёлок",
        " поселок",
        " village",
        " hamlet",
        " shtetl",
        " settlement",
    ] {
        if lower_raw.ends_with(suf) {
            let rest = trimmed[..trimmed.len() - suf.len()].trim();
            if !rest.is_empty() {
                return (rest.to_string(), Some("village".to_string()));
            }
        }
    }

    (trimmed.to_string(), None)
}

pub(crate) fn looks_like_rural_locality_name(clean_name: &str) -> bool {
    let lower = normalize_russian_orthography_word(clean_name);
    for suf in [
        "ка",
        "ово",
        "ево",
        "ино",
        "ское",
        "цкое",
        "полье",
        "слобода",
        "хутор",
    ] {
        if lower.ends_with(suf) && lower.chars().count() >= 5 {
            return true;
        }
    }
    false
}

/// Normalizes a place name for duplicate/parent matching (strips prefixes, expands abbreviations, transliterates).
pub fn normalize_place_match_key(raw: &str) -> String {
    let (clean, _) = infer_place_type_and_clean_name(raw);
    let mut words = Vec::new();
    for token in clean
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let norm = normalize_cross_script_token(token);
        if !norm.is_empty() {
            words.push(norm);
        }
    }
    words.join(" ")
}

/// Extracts the root toponymic stem for administrative parent matching (`"Тверской уезд"` -> `"tver"`, `"Тверская губерния"` -> `"tver"`).
pub fn extract_jurisdiction_stem(raw: &str) -> Option<String> {
    let (clean, _) = infer_place_type_and_clean_name(raw);
    let admin_words = [
        "губерния",
        "губ",
        "область",
        "обл",
        "край",
        "краи",
        "уезд",
        "район",
        "раион",
        "округ",
        "волость",
        "воеводство",
        "наместничество",
        "провинция",
        "империя",
        "республика",
        "федерация",
        "governorate",
        "gubernia",
        "guberniya",
        "oblast",
        "province",
        "uyezd",
        "uezd",
        "raion",
        "district",
        "county",
        "township",
        "volost",
        "okrug",
        "krai",
        "city",
        "town",
        "village",
    ];

    let mut core_tokens = Vec::new();
    for word in clean
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let ortho = normalize_russian_orthography_word(word);
        if !admin_words.contains(&ortho.as_str()) {
            core_tokens.push(ortho);
        }
    }

    let first = core_tokens.first()?.as_str();
    if first.chars().count() < 3 {
        return None;
    }

    let mut stem = first.to_string();
    for suffix in [
        "овский",
        "овская",
        "овское",
        "евский",
        "евская",
        "евское",
        "инский",
        "инская",
        "инское",
        "ский",
        "ской",
        "ская",
        "ское",
        "цкий",
        "цкой",
        "цкая",
        "цкое",
        "ск",
        "ов",
        "ев",
        "ин",
        "ь",
        "а",
        "я",
        "ы",
        "и",
    ] {
        if let Some(stripped) = stem.strip_suffix(suffix) {
            if stripped.chars().count() >= 3 {
                stem = stripped.to_string();
                break;
            }
        }
    }

    let latin = normalize_cross_script_token(&stem);
    if latin.len() >= 3 {
        Some(latin)
    } else {
        None
    }
}

pub(crate) fn find_matching_place_in_store<'a>(
    places: &'a [PlaceSummaryDto],
    segment_name: &str,
    expected_parent_id: Option<&str>,
    exclude_place_id: &str,
) -> Option<&'a PlaceSummaryDto> {
    let target_key = normalize_place_match_key(segment_name);
    if target_key.is_empty() {
        return None;
    }

    // 1. Prefer exact parent match + normalized name/alt_name match
    if let Some(found) = places.iter().find(|p| {
        p.id != exclude_place_id
            && p.parent_id.as_deref() == expected_parent_id
            && (normalize_place_match_key(&p.name) == target_key
                || p.alt_names
                    .iter()
                    .any(|a| normalize_place_match_key(&a.name) == target_key))
    }) {
        return Some(found);
    }

    // 2. Fallback: match a top-level jurisdiction (`country` / `region` / `district`) with matching normalized name
    let (_, seg_inferred_type) = infer_place_type_and_clean_name(segment_name);
    places.iter().find(|p| {
        p.id != exclude_place_id
            && !p.name.contains(',')
            && !p.name.contains(';')
            && (expected_parent_id.is_none()
                || p.parent_id.is_none()
                || p.parent_id.as_deref() == expected_parent_id)
            && (seg_inferred_type.is_none()
                || p.place_type == "other"
                || Some(p.place_type.as_str()) == seg_inferred_type.as_deref())
            && (normalize_place_match_key(&p.name) == target_key
                || p.alt_names
                    .iter()
                    .any(|a| normalize_place_match_key(&a.name) == target_key))
    })
}

pub(crate) fn split_jurisdiction_segments(raw_path: &str) -> Vec<String> {
    let mut parts: Vec<String> = raw_path
        .split([',', ';', '/'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if parts.len() >= 2 {
        let (_, first_type) = infer_place_type_and_clean_name(&parts[0]);
        let (_, last_type) = infer_place_type_and_clean_name(parts.last().unwrap());
        // If written top-down (Country -> Region -> District -> Locality), reverse into Leaf -> ... -> Root
        let first_rank = first_type.as_deref().map(place_hierarchy_rank).unwrap_or(4);
        let last_rank = last_type.as_deref().map(place_hierarchy_rank).unwrap_or(4);
        if first_rank < last_rank && first_rank <= 2 {
            parts.reverse();
        }
    }

    parts
}

pub(crate) fn collect_descendant_coordinates(
    places: &[PlaceSummaryDto],
    parent_id: &str,
) -> Vec<(f64, f64)> {
    let mut coords = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(parent_id.to_string());
    visited.insert(parent_id.to_string());

    while let Some(curr) = queue.pop_front() {
        for p in places {
            if p.parent_id.as_deref() == Some(curr.as_str()) && visited.insert(p.id.clone()) {
                if let (Some(lat), Some(lon)) = (p.latitude, p.longitude) {
                    coords.push((lat, lon));
                }
                queue.push_back(p.id.clone());
            }
        }
    }
    coords
}

pub(crate) fn build_ancestor_steps_for_parent(
    places: &[PlaceSummaryDto],
    parent_id: &str,
) -> Vec<ProposedHierarchyStepDto> {
    let mut steps = Vec::new();
    let mut curr = places.iter().find(|p| p.id == parent_id);
    let mut guard = 0usize;
    while let Some(p) = curr {
        let (_, inferred) = infer_place_type_and_clean_name(&p.name);
        let eff_type = if p.place_type == "other" {
            inferred.unwrap_or_else(|| p.place_type.clone())
        } else {
            p.place_type.clone()
        };
        steps.push(ProposedHierarchyStepDto {
            name: p.name.clone(),
            place_type: eff_type,
            existing_place_id: Some(p.id.clone()),
            existing_place_gid: Some(p.gid.clone()),
            is_new: false,
        });
        guard += 1;
        if guard > 16 {
            break;
        }
        curr = p
            .parent_id
            .as_deref()
            .and_then(|pid| places.iter().find(|cand| cand.id == pid));
    }
    steps.reverse();
    steps
}
