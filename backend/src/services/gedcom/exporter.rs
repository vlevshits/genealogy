#[allow(unused_imports)]
use super::*;

/// Exports `GenealogyPrivacyStore` to GEDCOM 7.0 or 5.5.1 text (`03 §2.3, §4, §6 DoD #3–#5`).
pub fn export_store_to_gedcom(
    store: &GenealogyPrivacyStore,
    req: &GedcomExportRequest,
    tree_title: &str,
) -> String {
    let format_str = req
        .version
        .as_deref()
        .or(req.format.as_deref())
        .unwrap_or("gedcom70")
        .trim()
        .to_ascii_lowercase();
    let is_gedcom_70 = !format_str.contains("551") && !format_str.contains("5.5");
    let include_media = req.include_media.unwrap_or(true);
    let redact_living = req.redact_living.unwrap_or(false);
    let include_private_notes = req.include_private_notes.unwrap_or(!redact_living);

    let mut out = String::with_capacity(64 * 1024);

    // 0 HEAD
    out.push_str("0 HEAD\n");
    out.push_str("1 SOUR Genealogy\n");
    out.push_str("2 VERS 1.0.0\n");
    out.push_str(&format!("2 NAME {}\n", tree_title.replace('\n', " ")));
    out.push_str("1 GEDC\n");
    if is_gedcom_70 {
        out.push_str("2 VERS 7.0\n");
    } else {
        out.push_str("2 VERS 5.5.1\n");
        out.push_str("2 FORM LINEAGE-LINKED\n");
        out.push_str("1 CHAR UTF-8\n");
    }

    let place_by_id: HashMap<String, _> = store.places.iter().map(|p| (p.id.clone(), p)).collect();
    let media_by_id: HashMap<String, _> = store.media.iter().map(|m| (m.id.clone(), m)).collect();

    // Filter & optionally redact individuals
    let mut exported_persons: Vec<TreePersonNodeDto> = Vec::new();
    for person in &store.persons {
        if redact_living {
            if let Some(redacted) =
                redact_person_for_tree_view(person, &CallerAccess::AnonymousPublic)
            {
                exported_persons.push(redacted);
            }
        } else {
            exported_persons.push(person.clone());
        }
    }

    let person_id_to_gid: HashMap<String, String> = exported_persons
        .iter()
        .map(|p| (p.id.clone(), p.gid.clone()))
        .collect();
    let person_is_redacted_living: HashSet<String> = if redact_living {
        exported_persons
            .iter()
            .filter(|p| p.is_living)
            .map(|p| p.id.clone())
            .collect()
    } else {
        HashSet::new()
    };

    // Map person_id -> FAMC and FAMS GIDs
    let mut person_famc: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut person_fams: HashMap<String, Vec<String>> = HashMap::new();

    for fam in &store.families {
        if redact_living && fam.is_private {
            continue;
        }
        if let Some(ref p1) = fam.partner1_id {
            if person_id_to_gid.contains_key(p1) {
                person_fams
                    .entry(p1.clone())
                    .or_default()
                    .push(fam.gid.clone());
            }
        }
        if let Some(ref p2) = fam.partner2_id {
            if person_id_to_gid.contains_key(p2) {
                person_fams
                    .entry(p2.clone())
                    .or_default()
                    .push(fam.gid.clone());
            }
        }
        for ch in &fam.children {
            if person_id_to_gid.contains_key(&ch.child_id) {
                person_famc
                    .entry(ch.child_id.clone())
                    .or_default()
                    .push((fam.gid.clone(), ch.rel_to_partner1.clone()));
            }
        }
    }

    // Emit 0 @Ixxxx@ INDI records
    for person in &exported_persons {
        let is_masked = person_is_redacted_living.contains(&person.id);
        out.push_str(&format!("0 @{}@ INDI\n", person.gid));

        let given_with_patr = match &person.primary_name.patronymic {
            Some(patr) if !patr.trim().is_empty() => {
                format!("{} {}", person.primary_name.given_name, patr.trim())
            }
            _ => person.primary_name.given_name.clone(),
        };

        let has_distinct_birth_surname = person
            .birth_surname
            .as_ref()
            .is_some_and(|bs| !bs.is_empty() && *bs != person.primary_name.surname);
        let has_married_alt = person
            .alt_names
            .iter()
            .any(|a| a.name_type.eq_ignore_ascii_case("married"));

        if !is_masked && (has_distinct_birth_surname || has_married_alt) {
            let b_surn = person
                .birth_surname
                .clone()
                .unwrap_or_else(|| person.primary_name.surname.clone());
            out.push_str(&format!("1 NAME {given_with_patr} /{b_surn}/\n"));
            out.push_str("2 TYPE birth\n");
            if !given_with_patr.is_empty() {
                out.push_str(&format!("2 GIVN {given_with_patr}\n"));
            }
            if !b_surn.is_empty() {
                out.push_str(&format!("2 SURN {b_surn}\n"));
            }

            if !person.primary_name.surname.is_empty() && person.primary_name.surname != b_surn {
                out.push_str(&format!(
                    "1 NAME {given_with_patr} /{}/\n",
                    person.primary_name.surname
                ));
                out.push_str("2 TYPE married\n");
                if !given_with_patr.is_empty() {
                    out.push_str(&format!("2 GIVN {given_with_patr}\n"));
                }
                out.push_str(&format!("2 SURN {}\n", person.primary_name.surname));
            }
        } else {
            out.push_str(&format!(
                "1 NAME {given_with_patr} /{}/\n",
                person.primary_name.surname
            ));
            if !is_masked && person.birth_surname.is_some() {
                out.push_str("2 TYPE birth\n");
            }
            if !given_with_patr.is_empty() {
                out.push_str(&format!("2 GIVN {given_with_patr}\n"));
            }
            if !person.primary_name.surname.is_empty() {
                out.push_str(&format!("2 SURN {}\n", person.primary_name.surname));
            }
        }

        if !is_masked {
            for alt in &person.alt_names {
                if alt.name_type.eq_ignore_ascii_case("married") {
                    continue;
                }
                let ag = alt
                    .given_name
                    .clone()
                    .unwrap_or_else(|| given_with_patr.clone());
                let asurn = alt.surname.clone().unwrap_or_default();
                out.push_str(&format!("1 NAME {ag} /{asurn}/\n"));
                out.push_str(&format!("2 TYPE {}\n", alt.name_type));
                if !ag.is_empty() {
                    out.push_str(&format!("2 GIVN {ag}\n"));
                }
                if !asurn.is_empty() {
                    out.push_str(&format!("2 SURN {asurn}\n"));
                }
            }
        }

        out.push_str(&format!("1 SEX {}\n", person.sex));

        if !is_masked {
            // Emit individual events
            for ev in store
                .events
                .iter()
                .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
            {
                if ev.is_private && !include_private_notes {
                    continue;
                }
                emit_gedcom_event(&mut out, ev, &place_by_id, is_gedcom_70);
            }

            // External IDs
            if let Some(rin) = person.external_ids.get("mh_rin") {
                out.push_str(&format!("1 FACT {rin}\n2 TYPE RIN\n"));
            }
            if let Some(uid) = person.external_ids.get("uid") {
                let uid_tag = if is_gedcom_70 { "UID" } else { "_UID" };
                out.push_str(&format!("1 {uid_tag} {uid}\n"));
            }
        }

        if let Some(famc_list) = person_famc.get(&person.id) {
            for (fam_gid, pedi) in famc_list {
                out.push_str(&format!("1 FAMC @{fam_gid}@\n"));
                out.push_str(&format!("2 PEDI {pedi}\n"));
            }
        }

        if let Some(fams_list) = person_fams.get(&person.id) {
            for fam_gid in fams_list {
                out.push_str(&format!("1 FAMS @{fam_gid}@\n"));
            }
        }

        if include_media && !is_masked {
            for media in &store.media {
                if media.is_private && !include_private_notes {
                    continue;
                }
                if media.linked_person_ids.iter().any(|pid| pid == &person.id) {
                    out.push_str(&format!("1 OBJE @{}@\n", media.gid));
                }
            }
        }

        if !is_masked {
            for note in store
                .notes
                .iter()
                .filter(|n| n.entity_type == "individual" && n.entity_id == person.id)
            {
                if note.is_private && !include_private_notes {
                    continue;
                }
                emit_multiline_tag(&mut out, 1, "NOTE", &note.body);
            }
        }
    }

    // Emit 0 @Fxxxx@ FAM records
    for fam in &store.families {
        if redact_living && fam.is_private {
            continue;
        }
        let p1_gid = fam
            .partner1_id
            .as_ref()
            .and_then(|id| person_id_to_gid.get(id));
        let p2_gid = fam
            .partner2_id
            .as_ref()
            .and_then(|id| person_id_to_gid.get(id));
        let visible_children: Vec<&FamilyChildDto> = fam
            .children
            .iter()
            .filter(|c| person_id_to_gid.contains_key(&c.child_id))
            .collect();

        if p1_gid.is_none() && p2_gid.is_none() && visible_children.is_empty() {
            continue;
        }

        out.push_str(&format!("0 @{}@ FAM\n", fam.gid));
        if let Some(g1) = p1_gid {
            out.push_str(&format!("1 HUSB @{g1}@\n"));
        }
        if let Some(g2) = p2_gid {
            out.push_str(&format!("1 WIFE @{g2}@\n"));
        }

        let fam_events: Vec<&LifeEventDto> = store
            .events
            .iter()
            .filter(|e| e.family_id.as_deref() == Some(fam.id.as_str()))
            .filter(|e| !e.is_private || include_private_notes)
            .collect();

        let has_marr_event = fam_events.iter().any(|e| e.event_type == "marriage");
        for ev in &fam_events {
            emit_gedcom_event(&mut out, ev, &place_by_id, is_gedcom_70);
        }
        if fam.relationship_type == "married" && !has_marr_event {
            out.push_str("1 MARR Y\n");
        }

        if let Some(rin) = fam.external_ids.get("mh_rin") {
            out.push_str(&format!("1 FACT {rin}\n2 TYPE RIN\n"));
        }
        if let Some(uid) = fam.external_ids.get("uid") {
            let uid_tag = if is_gedcom_70 { "UID" } else { "_UID" };
            out.push_str(&format!("1 {uid_tag} {uid}\n"));
        }

        for ch in visible_children {
            if let Some(ch_gid) = person_id_to_gid.get(&ch.child_id) {
                out.push_str(&format!("1 CHIL @{ch_gid}@\n"));
            }
        }

        for note in store
            .notes
            .iter()
            .filter(|n| n.entity_type == "family" && n.entity_id == fam.id)
        {
            if note.is_private && !include_private_notes {
                continue;
            }
            emit_multiline_tag(&mut out, 1, "NOTE", &note.body);
        }
    }

    // Emit 0 @Oxxxx@ OBJE records
    if include_media {
        for media in &store.media {
            if media.is_private && !include_private_notes {
                continue;
            }
            if redact_living && media.only_living_persons {
                continue;
            }
            let _ = media_by_id.get(&media.id);
            out.push_str(&format!("0 @{}@ OBJE\n", media.gid));
            let file_path = media
                .external_path
                .as_deref()
                .or(media.original_file_name.as_deref())
                .or(media.file_name.as_deref())
                .unwrap_or(&media.file_url);
            out.push_str(&format!("1 FILE {file_path}\n"));
            let form_ext = match media.mime_type.as_str() {
                "image/png" => "png",
                "image/webp" => "webp",
                "image/gif" => "gif",
                "application/pdf" => "pdf",
                _ => "jpg",
            };
            out.push_str(&format!("2 FORM {form_ext}\n"));
            if !media.title.is_empty() {
                out.push_str(&format!("2 TITL {}\n", media.title.replace('\n', " ")));
            }
        }
    }

    out.push_str("0 TRLR\n");
    out
}

pub(crate) fn emit_gedcom_event(
    out: &mut String,
    ev: &LifeEventDto,
    place_by_id: &HashMap<String, &crate::services::privacy::PlaceSummaryDto>,
    is_gedcom_70: bool,
) {
    let tag = match ev.event_type.as_str() {
        "birth" => "BIRT",
        "baptism" => "CHR",
        "death" => "DEAT",
        "burial" => "BURI",
        "cremation" => "CREM",
        "adoption" => "ADOP",
        "occupation" => "OCCU",
        "education" => "EDUC",
        "graduation" => "GRAD",
        "residence" => "RESI",
        "immigration" => "IMMI",
        "emigration" => "EMIG",
        "naturalization" => "NATU",
        "census" => "CENS",
        "military_service" => "MILI",
        "retirement" => "RETI",
        "marriage" => "MARR",
        "marriage_license" => "MARL",
        "divorce" => "DIV",
        "engagement" => "ENGA",
        _ => "EVEN",
    };

    let date_str = ev
        .gedcom_date
        .clone()
        .or_else(|| ev.date.as_ref().map(|d| d.to_gedcom()))
        .filter(|s| !s.trim().is_empty());

    let has_details = date_str.is_some()
        || ev.place_full_name.is_some()
        || ev.description.is_some()
        || ev.custom_type.is_some();

    if tag == "DEAT" && !has_details {
        out.push_str("1 DEAT Y\n");
        return;
    }

    let inline_desc = ev
        .description
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty() && tag != "DEAT" && !(tag == "RESI" && t.contains('@')));

    if let Some(inline_text) = inline_desc {
        out.push_str(&format!("1 {tag} {}\n", inline_text.replace('\n', " ")));
    } else {
        out.push_str(&format!("1 {tag}\n"));
    }
    if let Some(ref ct) = ev.custom_type {
        if !ct.trim().is_empty() {
            out.push_str(&format!("2 TYPE {}\n", ct.trim()));
        }
    }
    if let Some(ref ds) = date_str {
        out.push_str(&format!("2 DATE {ds}\n"));
    }
    if let Some(ref plac) = ev.place_full_name {
        if !plac.trim().is_empty() {
            out.push_str(&format!("2 PLAC {}\n", plac.trim()));
            if let Some(ref pid) = ev.place_id {
                if let Some(place) = place_by_id.get(pid) {
                    if let (Some(lat), Some(lon)) = (place.latitude, place.longitude) {
                        out.push_str("3 MAP\n");
                        out.push_str(&format!("4 LATI {}\n", format_gedcom_latitude(lat)));
                        out.push_str(&format!("4 LONG {}\n", format_gedcom_longitude(lon)));
                    }
                }
            }
        }
    }

    if let Some(ref desc) = ev.description {
        let trimmed = desc.trim();
        if tag == "RESI" && trimmed.contains('@') {
            let escaped = if is_gedcom_70 {
                trimmed.to_string()
            } else {
                trimmed.replace('@', "@@")
            };
            out.push_str(&format!("2 EMAIL {escaped}\n"));
        } else if tag == "DEAT" {
            if let Some((caus_part, age_part)) = trimmed.rsplit_once(" (возраст: ") {
                let age_clean = age_part.trim_end_matches(')');
                if !caus_part.trim().is_empty() {
                    out.push_str(&format!("2 CAUS {}\n", caus_part.trim()));
                }
                if !age_clean.trim().is_empty() {
                    out.push_str(&format!("2 AGE {}\n", age_clean.trim()));
                }
            } else if let Some(age_only) = trimmed.strip_prefix("возраст: ") {
                out.push_str(&format!("2 AGE {}\n", age_only.trim()));
            } else {
                out.push_str(&format!("2 CAUS {trimmed}\n"));
            }
        }
    }
}

pub(crate) fn emit_multiline_tag(out: &mut String, level: u8, tag: &str, text: &str) {
    let mut lines = text.lines();
    if let Some(first) = lines.next() {
        out.push_str(&format!("{level} {tag} {first}\n"));
        let cont_level = level + 1;
        for line in lines {
            out.push_str(&format!("{cont_level} CONT {line}\n"));
        }
    }
}
