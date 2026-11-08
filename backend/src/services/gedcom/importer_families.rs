#[allow(unused_imports)]
use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn import_gedcom_families_and_associations(
    store: &mut GenealogyPrivacyStore,
    roots: &[GedcomNode],
    mode: &str,
    indi_xref_to_person_id: &HashMap<String, String>,
    person_famc_pedi: &HashMap<(String, String), String>,
    pending_associations: Vec<PendingAssociation>,
    top_level_notes: &HashMap<String, String>,
    attached_top_level_notes: &mut HashSet<String>,
    top_level_sources: &HashMap<String, String>,
    obje_xref_to_media_id: &HashMap<String, String>,
    dialect: &mut GedcomDialectSummary,
    warnings: &mut Vec<String>,
) -> usize {
    let mut merged_families: usize = 0;

    // Pass 3: Parse all `0 @Fxxxx@ FAM` records
    for node in roots {
        if node.tag != "FAM" {
            continue;
        }

        let raw_xref = node.xref.as_deref();
        let fam_xref_clean = raw_xref
            .map(|x| x.trim_matches('@').to_uppercase())
            .unwrap_or_default();

        let mut external_ids: HashMap<String, String> = HashMap::new();
        for child in &node.children {
            match child.tag.as_str() {
                "_UID" | "UID" => {
                    let uid_val = child.value.trim().to_string();
                    if !uid_val.is_empty() {
                        external_ids.insert("uid".to_string(), uid_val);
                        dialect.preserved_uids_count += 1;
                    }
                }
                "RIN" => {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                "FACT"
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("RIN")) =>
                {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                _ => {}
            }
        }

        let partner1_id = node
            .child_value("HUSB")
            .map(|x| x.trim_matches('@').to_uppercase())
            .and_then(|x| indi_xref_to_person_id.get(&x).cloned());
        let partner2_id = node
            .child_value("WIFE")
            .map(|x| x.trim_matches('@').to_uppercase())
            .and_then(|x| indi_xref_to_person_id.get(&x).cloned());

        let mut children: Vec<FamilyChildDto> = Vec::new();
        for (idx, ch_node) in node.children.iter().filter(|c| c.tag == "CHIL").enumerate() {
            let ch_xref = ch_node.value.trim().trim_matches('@').to_uppercase();
            if let Some(child_id) = indi_xref_to_person_id.get(&ch_xref).cloned() {
                let pedi = person_famc_pedi
                    .get(&(child_id.clone(), fam_xref_clean.clone()))
                    .cloned()
                    .unwrap_or_else(|| "birth".to_string());
                children.push(FamilyChildDto {
                    child_id,
                    sort_order: idx as i32,
                    rel_to_partner1: pedi.clone(),
                    rel_to_partner2: pedi,
                });
            }
        }

        let has_div = node.find_child("DIV").is_some();
        let has_marr = node.find_child("MARR").is_some() || node.find_child("MARL").is_some();
        let relationship_type = if has_div {
            "divorced".to_string()
        } else if has_marr || (partner1_id.is_some() && partner2_id.is_some()) {
            "married".to_string()
        } else {
            "unknown".to_string()
        };

        let existing_fam_id = if mode == "merge" {
            find_and_enrich_existing_family(
                store,
                &partner1_id,
                &partner2_id,
                &relationship_type,
                &children,
                &external_ids,
            )
        } else {
            None
        };

        let fam_id = if let Some(existing_id) = existing_fam_id {
            merged_families += 1;
            existing_id
        } else {
            let existing_gids: HashSet<String> =
                store.families.iter().map(|f| f.gid.clone()).collect();
            let fam_gid = allocate_gid(
                'F',
                &mut store.next_family_gid_seq,
                raw_xref,
                &existing_gids,
            );
            let new_fam_id = Uuid::new_v4().to_string();

            store.families.push(FamilyDto {
                id: new_fam_id.clone(),
                gid: fam_gid,
                partner1_id,
                partner2_id,
                relationship_type,
                is_private: false,
                children,
                external_ids,
                version: 1,
                marriage_date: None,
                marriage_place: None,
            });
            new_fam_id
        };

        for child in &node.children {
            match child.tag.as_str() {
                "_UPD" => {
                    dialect.filtered_upd_timestamps_count += 1;
                }
                "EVEN" => {
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("_UPD"))
                    {
                        dialect.filtered_upd_timestamps_count += 1;
                        continue;
                    }
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        "custom",
                        dialect,
                        top_level_notes,
                        attached_top_level_notes,
                        top_level_sources,
                        obje_xref_to_media_id,
                    );
                }
                "MARL" => {
                    dialect.marriage_licenses_count += 1;
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        "marriage_license",
                        dialect,
                        top_level_notes,
                        attached_top_level_notes,
                        top_level_sources,
                        obje_xref_to_media_id,
                    );
                }
                "MARR" => {
                    let is_flag_only =
                        child.value.trim().eq_ignore_ascii_case("Y") && child.children.is_empty();
                    if !is_flag_only {
                        parse_and_insert_event(
                            store,
                            child,
                            None,
                            Some(&fam_id),
                            "marriage",
                            dialect,
                            top_level_notes,
                            attached_top_level_notes,
                            top_level_sources,
                            obje_xref_to_media_id,
                        );
                    }
                }
                "DIV" | "ENGA" | "ANUL" => {
                    let ev_type = match child.tag.as_str() {
                        "DIV" => "divorce",
                        "ENGA" => "engagement",
                        _ => "custom",
                    };
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        ev_type,
                        dialect,
                        top_level_notes,
                        attached_top_level_notes,
                        top_level_sources,
                        obje_xref_to_media_id,
                    );
                }
                "NOTE" => {
                    attach_gedcom_note(
                        store,
                        &child.value,
                        "family",
                        &fam_id,
                        top_level_notes,
                        attached_top_level_notes,
                    );
                }
                _ => {}
            }
        }
    }

    // Pass 4: Resolve `ASSO` associations (`03 §2.2 #7`)
    for assoc in pending_associations {
        let Some(target_person_id) = indi_xref_to_person_id.get(&assoc.target_xref).cloned() else {
            warnings.push(format!("Unresolved ASSO target @{}@", assoc.target_xref));
            continue;
        };

        let rela_lower = assoc.rela.trim().to_lowercase();
        let is_parental = matches!(
            rela_lower.as_str(),
            "отец" | "мать" | "родитель" | "father" | "mother" | "parent"
        );

        if is_parental {
            let already_linked = store.families.iter().any(|f| {
                let is_child = f.children.iter().any(|c| c.child_id == assoc.person_id);
                let is_parent = f.partner1_id.as_deref() == Some(target_person_id.as_str())
                    || f.partner2_id.as_deref() == Some(target_person_id.as_str());
                is_child && is_parent
            });

            if already_linked {
                dialect.deduplicated_parental_associations_count += 1;
            } else {
                // Synthesize or attach to a single-parent family
                let target_sex = store
                    .persons
                    .iter()
                    .find(|p| p.id == target_person_id)
                    .map(|p| p.sex.clone())
                    .unwrap_or_else(|| "U".to_string());
                let existing_gids: HashSet<String> =
                    store.families.iter().map(|f| f.gid.clone()).collect();
                let fam_gid =
                    allocate_gid('F', &mut store.next_family_gid_seq, None, &existing_gids);
                let (p1, p2) =
                    if target_sex == "F" || rela_lower == "мать" || rela_lower == "mother" {
                        (None, Some(target_person_id))
                    } else {
                        (Some(target_person_id), None)
                    };
                store.families.push(FamilyDto {
                    id: Uuid::new_v4().to_string(),
                    gid: fam_gid,
                    partner1_id: p1,
                    partner2_id: p2,
                    relationship_type: "unknown".to_string(),
                    is_private: false,
                    children: vec![FamilyChildDto {
                        child_id: assoc.person_id,
                        sort_order: 0,
                        rel_to_partner1: "birth".to_string(),
                        rel_to_partner2: "birth".to_string(),
                    }],
                    external_ids: HashMap::new(),
                    version: 1,
                    marriage_date: None,
                    marriage_place: None,
                });
                dialect.synthesized_single_parent_families_count += 1;
            }
        } else {
            let target_info = store
                .persons
                .iter()
                .find(|p| p.id == target_person_id)
                .map(|p| {
                    format!(
                        "@{}@ {} {}",
                        p.gid, p.primary_name.given_name, p.primary_name.surname
                    )
                    .trim()
                    .to_string()
                })
                .unwrap_or_else(|| format!("@{}@", assoc.target_xref));

            let note_body = format!("Связь ({}): {}", assoc.rela.trim(), target_info);
            let already_has_note = store.notes.iter().any(|n| {
                n.entity_type == "individual"
                    && n.entity_id == assoc.person_id
                    && n.body.trim() == note_body.trim()
            });
            if !already_has_note {
                store.notes.push(NoteDto {
                    id: Uuid::new_v4().to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: assoc.person_id,
                    body: note_body,
                    is_private: false,
                    sort_order: store.notes.len() as i32,
                    version: 1,
                });
                dialect.social_associations_notes_count += 1;
            }
        }
    }

    merged_families
}

pub(crate) fn link_media_to_person(
    store: &mut GenealogyPrivacyStore,
    media_id: &str,
    person_id: &str,
) {
    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id) {
        if !media.linked_person_ids.iter().any(|id| id == person_id) {
            media.linked_person_ids.push(person_id.to_string());
        }
        if !media
            .links
            .iter()
            .any(|l| l.entity_type == "individual" && l.entity_id == person_id)
        {
            media.links.push(MediaLinkDto {
                id: Uuid::new_v4().to_string(),
                media_id: media_id.to_string(),
                entity_type: "individual".to_string(),
                entity_id: person_id.to_string(),
                role: Some("gallery".to_string()),
            });
        }
    }

    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        if person.avatar_media_id.is_none() {
            person.avatar_media_id = Some(media_id.to_string());
            person.avatar_url = Some(format!("/api/v1/media/{media_id}/file"));
        }
    }
}

pub(crate) fn attach_gedcom_note(
    store: &mut GenealogyPrivacyStore,
    raw_value: &str,
    entity_type: &str,
    entity_id: &str,
    top_level_notes: &HashMap<String, String>,
    attached_top_level_notes: &mut HashSet<String>,
) {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return;
    }

    let body = if trimmed.starts_with('@') && trimmed.ends_with('@') && trimmed.len() > 2 {
        let key = trimmed.trim_matches('@').to_uppercase();
        if !attached_top_level_notes.insert(key.clone()) {
            // Avoid duplicating shared top-level boilerplate notes across dozens of records
            return;
        }
        let Some(resolved) = top_level_notes.get(&key) else {
            return;
        };
        resolved.clone()
    } else {
        trimmed.to_string()
    };

    if body.is_empty() {
        return;
    }

    if store.notes.iter().any(|n| {
        n.entity_type == entity_type && n.entity_id == entity_id && n.body.trim() == body.trim()
    }) {
        return;
    }

    store.notes.push(NoteDto {
        id: Uuid::new_v4().to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        body,
        is_private: false,
        sort_order: store.notes.len() as i32,
        version: 1,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn parse_and_insert_event(
    store: &mut GenealogyPrivacyStore,
    ev_node: &GedcomNode,
    individual_id: Option<&str>,
    family_id: Option<&str>,
    event_type: &str,
    dialect: &mut GedcomDialectSummary,
    top_level_notes: &HashMap<String, String>,
    attached_top_level_notes: &mut HashSet<String>,
    top_level_sources: &HashMap<String, String>,
    obje_xref_to_media_id: &HashMap<String, String>,
) {
    let raw_date = ev_node.child_value("DATE");
    let parsed_date = raw_date.as_deref().map(GenDate::parse);
    let date_sort = parsed_date.as_ref().and_then(|d| d.sort_key());
    let gedcom_date = raw_date
        .clone()
        .or_else(|| parsed_date.as_ref().map(|d| d.to_gedcom()));

    // Place & directional coordinates (`03 §2.2 #2, #3`)
    let mut place_id: Option<String> = None;
    let mut place_full_name: Option<String> = None;
    if let Some(plac_node) = ev_node.find_child("PLAC") {
        let raw_plac = plac_node.value.trim();
        if !raw_plac.is_empty() {
            let mut lat: Option<f64> = None;
            let mut lon: Option<f64> = None;

            if let Some(map_node) = plac_node.find_child("MAP") {
                if let Some(lati_str) = map_node.child_value("LATI") {
                    if let Some((v, had_dir)) = parse_gedcom_coordinate(&lati_str) {
                        lat = Some(v);
                        if had_dir {
                            dialect.normalized_coordinates_count += 1;
                        }
                    }
                }
                if let Some(long_str) = map_node.child_value("LONG") {
                    if let Some((v, had_dir)) = parse_gedcom_coordinate(&long_str) {
                        lon = Some(v);
                        if had_dir {
                            dialect.normalized_coordinates_count += 1;
                        }
                    }
                }
            }

            if let Ok(resolved_place) =
                resolve_or_create_place_chain(store, raw_plac, None, lat, lon, None)
            {
                place_id = Some(resolved_place.id);
                place_full_name = Some(raw_plac.to_string());
            } else {
                place_full_name = Some(raw_plac.to_string());
            }
        }
    }

    // Description, `CAUS` + `AGE` (`03 §2.2 #9`), and `EMAIL` (`03 §2.2 #10`)
    let mut desc_parts: Vec<String> = Vec::new();
    let inline_val = ev_node.value.trim();
    if !inline_val.is_empty() && !inline_val.eq_ignore_ascii_case("Y") {
        desc_parts.push(inline_val.to_string());
    }

    let caus_opt = ev_node.child_value("CAUS");
    let age_opt = ev_node.child_value("AGE");
    match (caus_opt, age_opt) {
        (Some(c), Some(a)) => desc_parts.push(format!("{c} (возраст: {a})")),
        (Some(c), None) => desc_parts.push(c),
        (None, Some(a)) => desc_parts.push(format!("возраст: {a}")),
        (None, None) => {}
    }

    if let Some(email) = ev_node.child_value("EMAIL") {
        desc_parts.push(email);
    }

    let description = if desc_parts.is_empty() {
        None
    } else {
        Some(desc_parts.join("; "))
    };

    let custom_type = if event_type == "custom" {
        ev_node.child_value("TYPE")
    } else {
        None
    };

    // Source citation text
    let source_text = ev_node.find_child("SOUR").and_then(|s_node| {
        let s_val = s_node.value.trim();
        let page = s_node.child_value("PAGE");
        let base = if s_val.starts_with('@') && s_val.ends_with('@') {
            let key = s_val.trim_matches('@').to_uppercase();
            top_level_sources
                .get(&key)
                .cloned()
                .unwrap_or_else(|| s_val.to_string())
        } else {
            s_val.to_string()
        };
        match (base.is_empty(), page) {
            (false, Some(p)) => Some(format!("{base}, {p}")),
            (false, None) => Some(base),
            (true, Some(p)) => Some(p),
            (true, None) => None,
        }
    });

    // Event-level NOTE
    if let Some(note_node) = ev_node.find_child("NOTE") {
        if let Some(pid) = individual_id {
            attach_gedcom_note(
                store,
                &note_node.value,
                "individual",
                pid,
                top_level_notes,
                attached_top_level_notes,
            );
        } else if let Some(fid) = family_id {
            attach_gedcom_note(
                store,
                &note_node.value,
                "family",
                fid,
                top_level_notes,
                attached_top_level_notes,
            );
        }
    }

    // Event-level OBJE
    let mut media_ids: Vec<String> = Vec::new();
    for ch in ev_node.children.iter().filter(|c| c.tag == "OBJE") {
        let obje_xref = ch.value.trim().trim_matches('@').to_uppercase();
        if let Some(mid) = obje_xref_to_media_id.get(&obje_xref).cloned() {
            media_ids.push(mid);
        }
    }

    let mut candidate_ev = LifeEventDto {
        id: Uuid::new_v4().to_string(),
        gid: String::new(),
        individual_id: individual_id.map(str::to_string),
        family_id: family_id.map(str::to_string),
        event_type: event_type.to_string(),
        custom_type,
        date: parsed_date,
        date_sort,
        gedcom_date,
        place_id,
        place_full_name,
        description,
        source_text,
        media_ids,
        is_private: false,
        version: 1,
    };

    if find_and_enrich_existing_event(store, &candidate_ev) {
        return;
    }

    let existing_gids: HashSet<String> = store.events.iter().map(|e| e.gid.clone()).collect();
    candidate_ev.gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_gids);
    store.events.push(candidate_ev);
}
