#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PersonMatchKind {
    Uid,
    NameDates,
    Kinship,
}

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
    pub(crate) min_child_birth_year: Option<i32>,
    pub(crate) max_parent_birth_year: Option<i32>,
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

pub(crate) fn canonicalize_given_variant(token: &str) -> String {
    match token {
        "софья" => "софия".to_string(),
        "наталия" => "наталья".to_string(),
        "феодор" => "федор".to_string(),
        "данила" | "данило" => "даниил".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn extract_given_tokens(raw: &str) -> Vec<String> {
    let raw_parts: Vec<String> = raw
        .split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | ',' | '/' | ';'))
        .map(normalize_name_token)
        .filter(|t| !t.is_empty() && t != "unknown" && t != "неизвестно" && t != "n" && t != "?")
        .collect();
    if raw_parts.len() <= 1 {
        return raw_parts
            .into_iter()
            .map(|t| canonicalize_given_variant(&t))
            .collect();
    }
    let non_patr: Vec<String> = raw_parts
        .iter()
        .filter(|t| !looks_like_slavic_patronymic(t))
        .map(|t| canonicalize_given_variant(t))
        .collect();
    if non_patr.is_empty() {
        vec![canonicalize_given_variant(&raw_parts[0])]
    } else {
        non_patr
    }
}

pub(crate) fn extract_surname_tokens(raw: &str) -> Vec<String> {
    raw.split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '-' | '/' | ',' | ';'))
        .map(normalize_name_token)
        .filter(|t| !t.is_empty() && t != "unknown" && t != "неизвестно")
        .collect()
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
    let t1 = extract_surname_tokens(s1);
    let t2 = extract_surname_tokens(s2);
    if t1.is_empty() || t2.is_empty() {
        return false;
    }
    t1.iter().any(|n1| {
        t2.iter().any(|n2| {
            if n1 == n2 {
                return true;
            }
            let stem1 = slavic_surname_stem(n1);
            let stem2 = slavic_surname_stem(n2);
            !stem1.is_empty() && stem1.chars().count() >= 3 && stem1 == stem2
        })
    })
}

pub(crate) fn given_names_match(g1: &str, g2: &str) -> bool {
    let t1 = extract_given_tokens(g1);
    let t2 = extract_given_tokens(g2);
    if t1.is_empty() || t2.is_empty() {
        return false;
    }
    t1.iter().any(|a| t2.iter().any(|b| a == b))
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

pub(crate) fn has_chronology_conflict(
    p_birth_year: Option<i32>,
    p_min_child_year: Option<i32>,
    p_max_parent_year: Option<i32>,
    c_birth_year: Option<i32>,
    c_min_child_year: Option<i32>,
    c_max_parent_year: Option<i32>,
) -> bool {
    if let (Some(by), Some(min_cy)) = (c_birth_year, p_min_child_year) {
        if by > min_cy - 12 {
            return true;
        }
    }
    if let (Some(by), Some(min_cy)) = (p_birth_year, c_min_child_year) {
        if by > min_cy - 12 {
            return true;
        }
    }
    if let (Some(by), Some(max_py)) = (c_birth_year, p_max_parent_year) {
        if by < max_py + 12 {
            return true;
        }
    }
    if let (Some(by), Some(max_py)) = (p_birth_year, c_max_parent_year) {
        if by < max_py + 12 {
            return true;
        }
    }
    false
}

pub(crate) fn parse_year_from_date_str(raw: &str) -> Option<i32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("Y") {
        return None;
    }
    let parsed = GenDate::parse(trimmed);
    if let (Some(s), Some(e)) = (parsed.start, parsed.end) {
        return Some((s.y + e.y) / 2);
    }
    parsed
        .start
        .map(|s| s.y)
        .or_else(|| parsed.sort_key().map(|k| k / 10_000))
}

pub(crate) fn extract_node_vital_years(node: &GedcomNode) -> (Option<i32>, Option<i32>) {
    let birth_year = ["BIRT", "CHR", "BAPM"].into_iter().find_map(|tag| {
        node.find_child(tag)
            .and_then(|n| n.child_value("DATE"))
            .and_then(|d| parse_year_from_date_str(&d))
    });
    let death_year = ["DEAT", "BURI", "CREM"].into_iter().find_map(|tag| {
        node.find_child(tag)
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
        .birth
        .as_ref()
        .and_then(|b| b.date.as_deref())
        .and_then(parse_year_from_date_str)
        .or_else(|| {
            events
                .iter()
                .filter(|e| {
                    e.individual_id.as_deref() == Some(person.id.as_str())
                        && matches!(e.event_type.as_str(), "birth" | "baptism")
                })
                .find_map(|e| {
                    e.gedcom_date
                        .as_deref()
                        .and_then(parse_year_from_date_str)
                        .or_else(|| e.date_sort.map(|ds| ds / 10_000))
                })
        })
        .or_else(|| person.birth_sort.map(|bs| bs / 10_000));

    let death_year = person
        .death
        .as_ref()
        .and_then(|d| d.date.as_deref())
        .and_then(parse_year_from_date_str)
        .or_else(|| {
            events
                .iter()
                .filter(|e| {
                    e.individual_id.as_deref() == Some(person.id.as_str())
                        && matches!(e.event_type.as_str(), "death" | "burial" | "cremation")
                })
                .find_map(|e| {
                    e.gedcom_date
                        .as_deref()
                        .and_then(parse_year_from_date_str)
                        .or_else(|| e.date_sort.map(|ds| ds / 10_000))
                })
        })
        .or_else(|| person.death_sort.map(|ds| ds / 10_000));

    (birth_year, death_year)
}

pub(crate) fn find_or_enrich_existing_media(
    store: &mut GenealogyPrivacyStore,
    initial_media_count: usize,
    raw_xref_clean: &str,
    raw_path: &str,
    clean_file_name: &str,
    explicit_title: Option<&str>,
) -> Option<String> {
    let bound = initial_media_count.min(store.media.len());
    let match_idx = store.media[..bound].iter().position(|m| {
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn find_matching_person_index(
    store: &GenealogyPrivacyStore,
    initial_persons_count: usize,
    cand: &IncomingPersonCandidate<'_>,
    reserved_uid_person_indices: &HashMap<String, usize>,
    reserved_person_indices: &HashSet<usize>,
    already_matched_person_indices: &HashSet<usize>,
    existing_family_bounds: &[(Option<i32>, Option<i32>)],
    chronology_conflicts_prevented: &mut usize,
) -> Option<(usize, PersonMatchKind, String)> {
    let bound = initial_persons_count.min(store.persons.len());
    if bound == 0 {
        return None;
    }

    let uid_opt = cand.external_ids.get("uid").filter(|v| !v.is_empty());

    // (a) Exact match by `_UID` / `UID` in `0..bound`
    if let Some(uid) = uid_opt {
        if let Some(&idx) = reserved_uid_person_indices.get(uid) {
            if idx < bound && !already_matched_person_indices.contains(&idx) {
                let short_uid: String = uid.chars().take(8).collect();
                return Some((idx, PersonMatchKind::Uid, format!("UID {short_uid}")));
            }
        }
        if let Some(idx) = (0..bound).find(|&idx| {
            !already_matched_person_indices.contains(&idx)
                && store.persons[idx]
                    .external_ids
                    .get("uid")
                    .is_some_and(|pu| pu.eq_ignore_ascii_case(uid))
        }) {
            let short_uid: String = uid.chars().take(8).collect();
            return Some((idx, PersonMatchKind::Uid, format!("UID {short_uid}")));
        }
    }

    // (b) Fallback matching when at least one side lacks `_UID`
    if !given_names_match(cand.given_name, cand.given_name) {
        return None;
    }

    let cand_canonical_gid =
        (!cand.xref_clean.is_empty()).then(|| canonicalize_gid('I', cand.xref_clean));

    for require_gid in [true, false] {
        for idx in 0..bound {
            if already_matched_person_indices.contains(&idx)
                || reserved_person_indices.contains(&idx)
            {
                continue;
            }
            let p = &store.persons[idx];

            let p_uid = p.external_ids.get("uid").filter(|v| !v.is_empty());
            if let (Some(pu), Some(cu)) = (p_uid, uid_opt) {
                if !pu.eq_ignore_ascii_case(cu) {
                    continue;
                }
            }

            if !given_names_match(&p.primary_name.given_name, cand.given_name)
                || !person_surnames_match(p, cand)
            {
                continue;
            }

            let p_sex = p.sex.trim().to_ascii_uppercase();
            let c_sex = cand.sex.trim().to_ascii_uppercase();
            if (p_sex == "M" || p_sex == "F") && (c_sex == "M" || c_sex == "F") && p_sex != c_sex {
                continue;
            }

            let (p_birth_year, p_death_year) = existing_person_vital_years(p, &store.events);
            let (p_min_child_year, p_max_parent_year) = existing_family_bounds
                .get(idx)
                .copied()
                .unwrap_or((None, None));

            if has_chronology_conflict(
                p_birth_year,
                p_min_child_year,
                p_max_parent_year,
                cand.birth_year,
                cand.min_child_birth_year,
                cand.max_parent_birth_year,
            ) {
                if require_gid {
                    *chronology_conflicts_prevented += 1;
                }
                continue;
            }

            if let (Some(eb), Some(ib)) = (p_birth_year, cand.birth_year) {
                if (eb - ib).abs() > 1 {
                    if require_gid && (eb - ib).abs() >= 12 {
                        *chronology_conflicts_prevented += 1;
                    }
                    continue;
                }
            }
            if let (Some(ed), Some(id)) = (p_death_year, cand.death_year) {
                if (ed - id).abs() > 1 {
                    continue;
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
                    continue;
                }
            }

            let gid_matches = cand_canonical_gid
                .as_deref()
                .is_some_and(|cg| canonicalize_gid('I', &p.gid) == cg);
            let birth_year_matches = p_birth_year.is_some() && cand.birth_year.is_some();
            let death_year_matches = p_death_year.is_some() && cand.death_year.is_some();
            let both_undated = p_birth_year.is_none()
                && cand.birth_year.is_none()
                && p_death_year.is_none()
                && cand.death_year.is_none();

            let matched = if require_gid {
                gid_matches && (birth_year_matches || death_year_matches || both_undated)
            } else {
                birth_year_matches || death_year_matches
            };

            if matched {
                let reason = if let Some(by) = cand.birth_year.or(p_birth_year) {
                    if birth_year_matches {
                        format!("Name + Birth {by}")
                    } else if let Some(dy) = cand.death_year.or(p_death_year) {
                        format!("Name + Death {dy}")
                    } else {
                        format!("Name + GID {}", p.gid)
                    }
                } else if let Some(dy) = cand.death_year.or(p_death_year) {
                    format!("Name + Death {dy}")
                } else {
                    format!("Name + GID {}", p.gid)
                };
                return Some((idx, PersonMatchKind::NameDates, reason));
            }
        }
    }

    None
}

pub(crate) fn enrich_existing_person(
    person: &mut TreePersonNodeDto,
    cand: &IncomingPersonCandidate<'_>,
) {
    let existing_given = person.primary_name.given_name.trim();
    if (existing_given.is_empty()
        || existing_given.eq_ignore_ascii_case("Unknown")
        || existing_given.eq_ignore_ascii_case("Неизвестно")
        || looks_like_cp1251_mojibake(existing_given))
        && !cand.given_name.trim().is_empty()
        && !looks_like_cp1251_mojibake(cand.given_name)
    {
        person.primary_name.given_name = cand.given_name.trim().to_string();
    }

    let existing_patr = person
        .primary_name
        .patronymic
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    if existing_patr.is_empty() || looks_like_cp1251_mojibake(existing_patr) {
        if let Some(patr) = cand
            .patronymic
            .map(str::trim)
            .filter(|s| !s.is_empty() && !looks_like_cp1251_mojibake(s))
        {
            person.primary_name.patronymic = Some(patr.to_string());
        }
    }

    let cand_surn = cand.surname.trim();
    if !cand_surn.is_empty() && !looks_like_cp1251_mojibake(cand_surn) {
        if person.primary_name.surname.trim().is_empty()
            || looks_like_cp1251_mojibake(&person.primary_name.surname)
        {
            person.primary_name.surname = cand_surn.to_string();
        } else if let Some(cand_bs) = cand
            .birth_surname
            .map(str::trim)
            .filter(|s| !s.is_empty() && !looks_like_cp1251_mojibake(s))
        {
            let existing_surn = person.primary_name.surname.trim();
            let existing_bs = person.birth_surname.as_deref().map(str::trim).unwrap_or("");
            let existing_only_has_maiden =
                existing_bs.is_empty() || surnames_match_normalized(existing_bs, existing_surn);
            if existing_only_has_maiden
                && surnames_match_normalized(existing_surn, cand_bs)
                && !surnames_match_normalized(cand_bs, cand_surn)
            {
                person.birth_surname = Some(existing_surn.to_string());
                person.primary_name.surname = cand_surn.to_string();
            }
        }
    }

    let existing_bs = person.birth_surname.as_deref().map(str::trim).unwrap_or("");
    let existing_surn = person.primary_name.surname.trim();
    if existing_bs.is_empty()
        || existing_bs.eq_ignore_ascii_case(existing_surn)
        || looks_like_cp1251_mojibake(existing_bs)
    {
        if let Some(bs) = cand
            .birth_surname
            .map(str::trim)
            .filter(|s| !s.is_empty() && !looks_like_cp1251_mojibake(s))
        {
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
            true
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

    // Enrich date if missing or if existing only had a year/range and incoming has month/day in the same year
    let should_upgrade_date = match (&existing.date, &incoming.date) {
        (None, Some(_)) => true,
        (Some(ed), Some(id)) => {
            if let (Some(es), Some(is)) = (&ed.start, &id.start) {
                (es.y == is.y
                    && (es.m.is_none() && is.m.is_some() || es.d.is_none() && is.d.is_some()))
                    || (ed.end.is_some() && id.end.is_none() && is.m.is_some())
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
