#[allow(unused_imports)]
use super::*;

pub(crate) struct IncomingPersonCandidate<'a> {
    pub(crate) xref_clean: &'a str,
    pub(crate) given_name: &'a str,
    pub(crate) patronymic: Option<&'a str>,
    pub(crate) surname: &'a str,
    pub(crate) birth_surname: Option<&'a str>,
    pub(crate) nickname: Option<&'a str>,
    pub(crate) name_prefix: Option<&'a str>,
    pub(crate) name_suffix: Option<&'a str>,
    pub(crate) sex: &'a str,
    pub(crate) birth_year: Option<i32>,
    pub(crate) death_year: Option<i32>,
    pub(crate) alt_names: &'a [IndividualAltNameDto],
    pub(crate) external_ids: &'a HashMap<String, String>,
}

pub(crate) fn normalize_name_token(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace('ё', "е")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn slavic_surname_stem(norm_surname: &str) -> &str {
    let s = norm_surname.trim();
    for suffix in ["ова", "ева", "ина", "ына", "ская", "цкая", "ая", "яя"] {
        if s.len() > suffix.len() + 4 && s.ends_with(suffix) {
            return &s[..s.len() - suffix.len()];
        }
    }
    for suffix in ["ов", "ев", "ин", "ын", "ский", "цкий", "ой", "ий", "ый"] {
        if s.len() > suffix.len() + 4 && s.ends_with(suffix) {
            return &s[..s.len() - suffix.len()];
        }
    }
    s
}

pub(crate) fn surnames_match_normalized(s1: &str, s2: &str) -> bool {
    let n1 = normalize_name_token(s1);
    let n2 = normalize_name_token(s2);
    if n1.is_empty() || n2.is_empty() {
        return false;
    }
    if n1 == n2 {
        return true;
    }
    let stem1 = slavic_surname_stem(&n1);
    let stem2 = slavic_surname_stem(&n2);
    !stem1.is_empty() && stem1.chars().count() >= 3 && stem1 == stem2
}

pub(crate) fn given_names_match(g1: &str, g2: &str) -> bool {
    let n1 = normalize_name_token(g1);
    let n2 = normalize_name_token(g2);
    if n1.is_empty()
        || n2.is_empty()
        || n1 == "unknown"
        || n2 == "unknown"
        || n1 == "неизвестно"
        || n2 == "неизвестно"
    {
        return false;
    }
    n1 == n2
}

pub(crate) fn person_surnames_match(
    existing: &TreePersonNodeDto,
    cand: &IncomingPersonCandidate<'_>,
) -> bool {
    let mut existing_surnames: Vec<&str> = Vec::new();
    if !existing.primary_name.surname.trim().is_empty() {
        existing_surnames.push(existing.primary_name.surname.trim());
    }
    if let Some(bs) = existing.birth_surname.as_deref() {
        if !bs.trim().is_empty() {
            existing_surnames.push(bs.trim());
        }
    }
    for alt in &existing.alt_names {
        if let Some(s) = alt.surname.as_deref() {
            if !s.trim().is_empty() {
                existing_surnames.push(s.trim());
            }
        }
    }

    let mut cand_surnames: Vec<&str> = Vec::new();
    if !cand.surname.trim().is_empty() {
        cand_surnames.push(cand.surname.trim());
    }
    if let Some(bs) = cand.birth_surname {
        if !bs.trim().is_empty() {
            cand_surnames.push(bs.trim());
        }
    }
    for alt in cand.alt_names {
        if let Some(s) = alt.surname.as_deref() {
            if !s.trim().is_empty() {
                cand_surnames.push(s.trim());
            }
        }
    }

    existing_surnames.iter().any(|es| {
        cand_surnames
            .iter()
            .any(|cs| surnames_match_normalized(es, cs))
    })
}

fn parse_year_from_date_str(raw: &str) -> Option<i32> {
    let parsed = GenDate::parse(raw);
    parsed
        .start
        .map(|s| s.y)
        .or_else(|| parsed.sort_key().map(|k| k / 10_000))
}

pub(crate) fn extract_node_vital_years(node: &GedcomNode) -> (Option<i32>, Option<i32>) {
    let birth_year = node
        .find_child("BIRT")
        .and_then(|n| n.child_value("DATE"))
        .and_then(|d| parse_year_from_date_str(&d))
        .or_else(|| {
            node.find_child("CHR")
                .or_else(|| node.find_child("BAPM"))
                .and_then(|n| n.child_value("DATE"))
                .and_then(|d| parse_year_from_date_str(&d))
        });

    let death_year = node
        .find_child("DEAT")
        .and_then(|n| n.child_value("DATE"))
        .and_then(|d| parse_year_from_date_str(&d))
        .or_else(|| {
            node.find_child("BURI")
                .or_else(|| node.find_child("CREM"))
                .and_then(|n| n.child_value("DATE"))
                .and_then(|d| parse_year_from_date_str(&d))
        });

    (birth_year, death_year)
}

pub(crate) fn existing_person_vital_years(
    person: &TreePersonNodeDto,
    events: &[LifeEventDto],
) -> (Option<i32>, Option<i32>) {
    let birth_year = person
        .birth_sort
        .map(|bs| bs / 10_000)
        .or_else(|| {
            person
                .birth
                .as_ref()
                .and_then(|b| b.date.as_deref())
                .and_then(parse_year_from_date_str)
        })
        .or_else(|| {
            events
                .iter()
                .filter(|e| {
                    e.individual_id.as_deref() == Some(person.id.as_str())
                        && matches!(e.event_type.as_str(), "birth" | "baptism")
                })
                .find_map(|e| {
                    e.date_sort
                        .map(|ds| ds / 10_000)
                        .or_else(|| e.gedcom_date.as_deref().and_then(parse_year_from_date_str))
                })
        });

    let death_year = person
        .death_sort
        .map(|ds| ds / 10_000)
        .or_else(|| {
            person
                .death
                .as_ref()
                .and_then(|d| d.date.as_deref())
                .and_then(parse_year_from_date_str)
        })
        .or_else(|| {
            events
                .iter()
                .filter(|e| {
                    e.individual_id.as_deref() == Some(person.id.as_str())
                        && matches!(e.event_type.as_str(), "death" | "burial" | "cremation")
                })
                .find_map(|e| {
                    e.date_sort
                        .map(|ds| ds / 10_000)
                        .or_else(|| e.gedcom_date.as_deref().and_then(parse_year_from_date_str))
                })
        });

    (birth_year, death_year)
}

pub(crate) fn find_or_enrich_existing_media(
    store: &mut GenealogyPrivacyStore,
    raw_xref_clean: &str,
    raw_path: &str,
    clean_file_name: &str,
    explicit_title: Option<&str>,
) -> Option<String> {
    let match_idx = store.media.iter().position(|m| {
        let by_ext_path = !raw_path.is_empty()
            && m.external_path
                .as_deref()
                .is_some_and(|ep| ep.eq_ignore_ascii_case(raw_path));
        let by_file_name = !clean_file_name.is_empty()
            && (m
                .file_name
                .as_deref()
                .is_some_and(|f| f.eq_ignore_ascii_case(clean_file_name))
                || m.original_file_name
                    .as_deref()
                    .is_some_and(|f| f.eq_ignore_ascii_case(clean_file_name)));
        let by_gid_and_title = !raw_xref_clean.is_empty()
            && m.gid.eq_ignore_ascii_case(raw_xref_clean)
            && explicit_title
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .is_some_and(|t| m.title.eq_ignore_ascii_case(t));
        by_ext_path || by_file_name || by_gid_and_title
    })?;

    let existing = &mut store.media[match_idx];
    let title_is_fallback = existing.title.trim().is_empty()
        || existing.title.eq_ignore_ascii_case(&existing.gid)
        || existing
            .file_name
            .as_deref()
            .is_some_and(|f| existing.title.eq_ignore_ascii_case(f))
        || existing
            .original_file_name
            .as_deref()
            .is_some_and(|f| existing.title.eq_ignore_ascii_case(f));
    if title_is_fallback && explicit_title.is_some_and(|t| !t.trim().is_empty()) {
        existing.title = explicit_title.unwrap().trim().to_string();
    }
    if existing.external_path.as_deref().unwrap_or("").is_empty() && !raw_path.is_empty() {
        existing.external_path = Some(raw_path.to_string());
    }
    if existing.file_name.as_deref().unwrap_or("").is_empty() && !clean_file_name.is_empty() {
        existing.file_name = Some(clean_file_name.to_string());
    }
    if existing
        .original_file_name
        .as_deref()
        .unwrap_or("")
        .is_empty()
        && !clean_file_name.is_empty()
    {
        existing.original_file_name = Some(clean_file_name.to_string());
    }

    Some(existing.id.clone())
}

pub(crate) fn find_matching_person_index(
    store: &GenealogyPrivacyStore,
    cand: &IncomingPersonCandidate<'_>,
) -> Option<usize> {
    let uid_opt = cand.external_ids.get("uid").filter(|v| !v.is_empty());
    let rin_opt = cand.external_ids.get("mh_rin").filter(|v| !v.is_empty());

    // (a) Match by `_UID` / `UID`
    if let Some(uid) = uid_opt {
        if let Some(idx) = store
            .persons
            .iter()
            .position(|p| p.external_ids.get("uid") == Some(uid))
        {
            return Some(idx);
        }
    }

    // (b) Match by `mh_rin` when UID is present or names match
    if let Some(rin) = rin_opt {
        if let Some(idx) = store.persons.iter().position(|p| {
            p.external_ids.get("mh_rin") == Some(rin)
                && (uid_opt.is_some()
                    || given_names_match(&p.primary_name.given_name, cand.given_name)
                    || person_surnames_match(p, cand))
        }) {
            return Some(idx);
        }
    }

    // (c) Match by GID (`@Ixxxx@`) AND normalized given_name match (or surname match if given_name is missing)
    if !cand.xref_clean.is_empty() {
        if let Some(idx) = store.persons.iter().position(|p| {
            if !p.gid.eq_ignore_ascii_case(cand.xref_clean) {
                return false;
            }
            let p_sex = p.sex.trim().to_ascii_uppercase();
            let c_sex = cand.sex.trim().to_ascii_uppercase();
            if (p_sex == "M" || p_sex == "F") && (c_sex == "M" || c_sex == "F") && p_sex != c_sex {
                return false;
            }
            let (p_by, p_dy) = existing_person_vital_years(p, &store.events);
            if let (Some(eb), Some(ib)) = (p_by, cand.birth_year) {
                if (eb - ib).abs() > 1 {
                    return false;
                }
            }
            if let (Some(ed), Some(id)) = (p_dy, cand.death_year) {
                if (ed - id).abs() > 1 {
                    return false;
                }
            }
            let p_has_given = !normalize_name_token(&p.primary_name.given_name).is_empty();
            let c_has_given = !normalize_name_token(cand.given_name).is_empty();
            if p_has_given && c_has_given {
                given_names_match(&p.primary_name.given_name, cand.given_name)
            } else {
                person_surnames_match(p, cand)
            }
        }) {
            return Some(idx);
        }
    }

    // (d) Match by normalized given_name + surname (or birth_surname) + vital years / patronymic
    if !given_names_match(cand.given_name, cand.given_name) {
        return None;
    }

    let matching_name_count = store
        .persons
        .iter()
        .filter(|p| {
            given_names_match(&p.primary_name.given_name, cand.given_name)
                && person_surnames_match(p, cand)
        })
        .count();

    store.persons.iter().position(|p| {
        if !given_names_match(&p.primary_name.given_name, cand.given_name)
            || !person_surnames_match(p, cand)
        {
            return false;
        }

        // Do not merge if explicit sexes conflict
        let p_sex = p.sex.trim().to_ascii_uppercase();
        let c_sex = cand.sex.trim().to_ascii_uppercase();
        if (p_sex == "M" || p_sex == "F") && (c_sex == "M" || c_sex == "F") && p_sex != c_sex {
            return false;
        }

        let (p_birth_year, p_death_year) = existing_person_vital_years(p, &store.events);

        if let (Some(eb), Some(ib)) = (p_birth_year, cand.birth_year) {
            if (eb - ib).abs() > 1 {
                return false;
            }
        }
        if let (Some(ed), Some(id)) = (p_death_year, cand.death_year) {
            if (ed - id).abs() > 1 {
                return false;
            }
        }

        let p_patr_norm = p
            .primary_name
            .patronymic
            .as_deref()
            .map(normalize_name_token)
            .filter(|s| !s.is_empty());
        let c_patr_norm = cand
            .patronymic
            .map(normalize_name_token)
            .filter(|s| !s.is_empty());

        if let (Some(ref ep), Some(ref ip)) = (&p_patr_norm, &c_patr_norm) {
            if ep != ip {
                return false;
            }
        }

        let birth_year_matches = p_birth_year.is_some() && cand.birth_year.is_some();
        let death_year_matches = p_death_year.is_some() && cand.death_year.is_some();
        let patronymic_matches = p_patr_norm.is_some() && c_patr_norm.is_some();
        let unique_full_name = matching_name_count == 1;

        birth_year_matches || death_year_matches || patronymic_matches || unique_full_name
    })
}

pub(crate) fn enrich_existing_person(
    person: &mut TreePersonNodeDto,
    cand: &IncomingPersonCandidate<'_>,
) {
    let existing_given = person.primary_name.given_name.trim();
    if (existing_given.is_empty()
        || existing_given.eq_ignore_ascii_case("Unknown")
        || existing_given.eq_ignore_ascii_case("Неизвестно"))
        && !cand.given_name.trim().is_empty()
    {
        person.primary_name.given_name = cand.given_name.trim().to_string();
    }

    if person
        .primary_name
        .patronymic
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        if let Some(patr) = cand.patronymic.map(str::trim).filter(|s| !s.is_empty()) {
            person.primary_name.patronymic = Some(patr.to_string());
        }
    }

    if person.primary_name.surname.trim().is_empty() && !cand.surname.trim().is_empty() {
        person.primary_name.surname = cand.surname.trim().to_string();
    }

    let existing_bs = person.birth_surname.as_deref().map(str::trim).unwrap_or("");
    let existing_surn = person.primary_name.surname.trim();
    if existing_bs.is_empty() || existing_bs.eq_ignore_ascii_case(existing_surn) {
        if let Some(bs) = cand.birth_surname.map(str::trim).filter(|s| !s.is_empty()) {
            person.birth_surname = Some(bs.to_string());
        }
    }

    if person
        .nickname
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        if let Some(nick) = cand.nickname.map(str::trim).filter(|s| !s.is_empty()) {
            person.nickname = Some(nick.to_string());
        }
    }

    if person
        .name_prefix
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        if let Some(npfx) = cand.name_prefix.map(str::trim).filter(|s| !s.is_empty()) {
            person.name_prefix = Some(npfx.to_string());
        }
    }

    if person
        .name_suffix
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        if let Some(nsfx) = cand.name_suffix.map(str::trim).filter(|s| !s.is_empty()) {
            person.name_suffix = Some(nsfx.to_string());
        }
    }

    let p_sex = person.sex.trim().to_ascii_uppercase();
    let c_sex = cand.sex.trim().to_ascii_uppercase();
    if (p_sex.is_empty() || p_sex == "U" || p_sex == "UNKNOWN") && (c_sex == "M" || c_sex == "F") {
        person.sex = c_sex;
    }

    for inc_alt in cand.alt_names {
        let inc_key = (
            normalize_name_token(inc_alt.given_name.as_deref().unwrap_or("")),
            normalize_name_token(inc_alt.patronymic.as_deref().unwrap_or("")),
            normalize_name_token(inc_alt.surname.as_deref().unwrap_or("")),
        );
        let already_exists = person.alt_names.iter().any(|ea| {
            let ea_key = (
                normalize_name_token(ea.given_name.as_deref().unwrap_or("")),
                normalize_name_token(ea.patronymic.as_deref().unwrap_or("")),
                normalize_name_token(ea.surname.as_deref().unwrap_or("")),
            );
            ea_key == inc_key
        });
        if !already_exists {
            person.alt_names.push(inc_alt.clone());
        }
    }

    for (k, v) in cand.external_ids {
        person
            .external_ids
            .entry(k.clone())
            .or_insert_with(|| v.clone());
    }
}

fn is_singleton_event_type(event_type: &str) -> bool {
    matches!(
        event_type,
        "birth"
            | "death"
            | "burial"
            | "baptism"
            | "cremation"
            | "marriage"
            | "divorce"
            | "marriage_license"
            | "engagement"
    )
}

pub(crate) fn find_and_enrich_existing_event(
    store: &mut GenealogyPrivacyStore,
    incoming: &LifeEventDto,
) -> bool {
    let is_singleton = is_singleton_event_type(&incoming.event_type);
    let inc_year = incoming.date_sort.map(|ds| ds / 10_000);

    let match_idx = store.events.iter().position(|ev| {
        if ev.individual_id != incoming.individual_id
            || ev.family_id != incoming.family_id
            || ev.event_type != incoming.event_type
        {
            return false;
        }

        let same_date = match (&ev.gedcom_date, &incoming.gedcom_date) {
            (Some(d1), Some(d2)) if d1.trim().eq_ignore_ascii_case(d2.trim()) => true,
            _ => ev.date_sort.is_some() && ev.date_sort == incoming.date_sort,
        };

        if is_singleton {
            let ev_year = ev.date_sort.map(|ds| ds / 10_000);
            let same_year = ev_year.is_some() && ev_year == inc_year;
            same_date || same_year || ev.gedcom_date.is_none() || incoming.gedcom_date.is_none()
        } else {
            let same_place_or_desc = (ev.place_id.is_some() && ev.place_id == incoming.place_id)
                || (ev.place_full_name.is_some() && ev.place_full_name == incoming.place_full_name)
                || (ev.description.is_some() && ev.description == incoming.description);
            same_date && same_place_or_desc
        }
    });

    let Some(idx) = match_idx else {
        return false;
    };

    let existing = &mut store.events[idx];

    // Enrich date if missing or if existing only had a year and incoming has month/day in the same year
    let should_upgrade_date = match (&existing.date, &incoming.date) {
        (None, Some(_)) => true,
        (Some(ed), Some(id)) => {
            if let (Some(es), Some(is)) = (&ed.start, &id.start) {
                es.y == is.y
                    && (es.m.is_none() && is.m.is_some() || es.d.is_none() && is.d.is_some())
            } else {
                false
            }
        }
        _ => false,
    };

    if should_upgrade_date {
        existing.date = incoming.date.clone();
        existing.date_sort = incoming.date_sort;
        existing.gedcom_date = incoming.gedcom_date.clone();
    } else {
        if existing.gedcom_date.is_none() && incoming.gedcom_date.is_some() {
            existing.gedcom_date = incoming.gedcom_date.clone();
        }
        if existing.date_sort.is_none() && incoming.date_sort.is_some() {
            existing.date_sort = incoming.date_sort;
        }
    }

    if existing.place_id.is_none() && incoming.place_id.is_some() {
        existing.place_id = incoming.place_id.clone();
    }
    if existing
        .place_full_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
        && incoming.place_full_name.is_some()
    {
        existing.place_full_name = incoming.place_full_name.clone();
    }
    if existing
        .description
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
        && incoming.description.is_some()
    {
        existing.description = incoming.description.clone();
    }
    if existing
        .custom_type
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
        && incoming.custom_type.is_some()
    {
        existing.custom_type = incoming.custom_type.clone();
    }
    if existing
        .source_text
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
        && incoming.source_text.is_some()
    {
        existing.source_text = incoming.source_text.clone();
    }
    for mid in &incoming.media_ids {
        if !existing.media_ids.contains(mid) {
            existing.media_ids.push(mid.clone());
        }
    }

    true
}

pub(crate) fn find_and_enrich_existing_family(
    store: &mut GenealogyPrivacyStore,
    partner1_id: &Option<String>,
    partner2_id: &Option<String>,
    relationship_type: &str,
    children: &[FamilyChildDto],
    external_ids: &HashMap<String, String>,
) -> Option<String> {
    let uid_opt = external_ids.get("uid").filter(|u| !u.is_empty());
    let inc_child_ids: HashSet<&str> = children.iter().map(|c| c.child_id.as_str()).collect();

    let match_idx = store.families.iter().position(|f| {
        // (a) Match by external UID
        if let Some(uid) = uid_opt {
            if f.external_ids.get("uid") == Some(uid) {
                return true;
            }
        }

        // (b) Both partner1_id and partner2_id are Some and match in either order
        if let (Some(ip1), Some(ip2), Some(fp1), Some(fp2)) = (
            partner1_id.as_deref(),
            partner2_id.as_deref(),
            f.partner1_id.as_deref(),
            f.partner2_id.as_deref(),
        ) {
            if (ip1 == fp1 && ip2 == fp2) || (ip1 == fp2 && ip2 == fp1) {
                return true;
            }
            // Both families have two explicit partners, and the pair differs -> distinct marriage/union
            return false;
        }

        let shared_children_count = f
            .children
            .iter()
            .filter(|c| inc_child_ids.contains(c.child_id.as_str()))
            .count();

        // (c) At least one partner matches (or single-parent families complement each other) AND they share >= 1 child
        let f_partners: Vec<&str> = [f.partner1_id.as_deref(), f.partner2_id.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        let inc_partners: Vec<&str> = [partner1_id.as_deref(), partner2_id.as_deref()]
            .into_iter()
            .flatten()
            .collect();

        let shares_partner = f_partners.iter().any(|fp| inc_partners.contains(fp));
        let complementary_single_parents = f_partners.len() == 1
            && inc_partners.len() == 1
            && ((f.partner1_id.is_some() && partner2_id.is_some())
                || (f.partner2_id.is_some() && partner1_id.is_some()));

        if (shares_partner || complementary_single_parents) && shared_children_count >= 1 {
            return true;
        }

        // (d) Both partners are None on one side and they share >= 2 children (or exact same single child)
        if (f_partners.is_empty() || inc_partners.is_empty())
            && (shared_children_count >= 2
                || (shared_children_count == 1
                    && f.children.len() == 1
                    && inc_child_ids.len() == 1))
        {
            return true;
        }

        false
    })?;

    let existing = &mut store.families[match_idx];

    // Enrich missing partner1_id / partner2_id without creating self-marriage
    if existing.partner1_id.is_none() {
        if let Some(p1) = partner1_id.as_ref() {
            if existing.partner2_id.as_ref() != Some(p1) {
                existing.partner1_id = Some(p1.clone());
            }
        } else if let Some(p2) = partner2_id.as_ref() {
            if existing.partner2_id.as_ref() != Some(p2) {
                existing.partner1_id = Some(p2.clone());
            }
        }
    }
    if existing.partner2_id.is_none() {
        if let Some(p2) = partner2_id.as_ref() {
            if existing.partner1_id.as_ref() != Some(p2) {
                existing.partner2_id = Some(p2.clone());
            }
        } else if let Some(p1) = partner1_id.as_ref() {
            if existing.partner1_id.as_ref() != Some(p1) {
                existing.partner2_id = Some(p1.clone());
            }
        }
    }

    if existing.relationship_type == "unknown" && relationship_type != "unknown" {
        existing.relationship_type = relationship_type.to_string();
    }

    for inc_child in children {
        if !existing
            .children
            .iter()
            .any(|c| c.child_id == inc_child.child_id)
        {
            let mut new_child = inc_child.clone();
            new_child.sort_order = existing.children.len() as i32;
            existing.children.push(new_child);
        }
    }

    for (k, v) in external_ids {
        existing
            .external_ids
            .entry(k.clone())
            .or_insert_with(|| v.clone());
    }

    Some(existing.id.clone())
}
