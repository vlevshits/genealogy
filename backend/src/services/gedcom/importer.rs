#[allow(unused_imports)]
use super::*;

/// Imports a GEDCOM 5.5.1 or 7.0 payload into `GenealogyPrivacyStore` (`03 §2.1, §2.2, §3.1, §6 DoD #1–#6`).
pub fn import_gedcom_into_store(
    store: &mut GenealogyPrivacyStore,
    raw_bytes: &[u8],
    mode_raw: &str,
    confirm_text: Option<&str>,
    current_year: i32,
    age_threshold: i32,
) -> Result<GedcomImportResult, AppError> {
    let mode = match mode_raw.trim().to_ascii_lowercase().as_str() {
        "" | "new" => "new".to_string(),
        "replace" => "replace".to_string(),
        "clean_slate" => "clean_slate".to_string(),
        "merge" => "merge".to_string(),
        _ => {
            let mut fields = HashMap::new();
            fields.insert("mode".to_string(), "invalid_mode".to_string());
            return Err(AppError::ValidationFailed(
                "Import mode must be 'new', 'replace', 'clean_slate', or 'merge'".to_string(),
                Some(fields),
            ));
        }
    };
    let is_replace_mode = mode == "replace" || mode == "clean_slate";

    if is_replace_mode && !store.persons.is_empty() && !store.is_demo_seed {
        let confirmed = confirm_text.map(str::trim).is_some_and(|c| {
            c.eq_ignore_ascii_case("ЗАМЕНИТЬ")
                || c.eq_ignore_ascii_case("REPLACE")
                || c.eq_ignore_ascii_case("CLEAN_SLATE")
        });
        if !confirmed {
            let mut fields = HashMap::new();
            fields.insert(
                "confirm_text".to_string(),
                "confirmation_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Type CLEAN_SLATE, ЗАМЕНИТЬ, or REPLACE to confirm replacing the existing family tree"
                    .to_string(),
                Some(fields),
            ));
        }
    }

    let (decoded_text, detected_charset) = decode_gedcom_bytes(raw_bytes);
    let roots = parse_gedcom_tree(&decoded_text)?;

    let snap_reason = if mode == "merge" {
        "merge_import"
    } else {
        "clean_slate_import"
    };
    let created_snap = crate::services::history::create_snapshot(
        store,
        Some(format!("Pre-GEDCOM import ({mode})")),
        snap_reason,
        None,
        None,
        "/tmp/genealogy-backups",
    )?;
    let snapshot_id = Some(created_snap.id);

    if mode == "new" || is_replace_mode || store.is_demo_seed {
        let preserved_snapshots = std::mem::take(&mut store.snapshots);
        *store = GenealogyPrivacyStore::empty();
        store.snapshots = preserved_snapshots;
    }

    let initial_places_count = store.places.len();
    let initial_persons_count = store.persons.len();
    let initial_families_count = store.families.len();
    let initial_events_count = store.events.len();
    let initial_media_count = store.media.len();
    let initial_notes_count = store.notes.len();

    let mut dialect = GedcomDialectSummary {
        source_system: "Unknown".to_string(),
        source_version: None,
        gedcom_version: "5.5.1".to_string(),
        charset: detected_charset,
        ..Default::default()
    };
    let mut warnings: Vec<String> = Vec::new();

    // Parse HEAD metadata
    if let Some(head) = roots.iter().find(|r| r.tag == "HEAD") {
        if let Some(sour) = head.find_child("SOUR") {
            if !sour.value.trim().is_empty() {
                dialect.source_system = sour.value.trim().to_string();
            } else if let Some(name) = sour.child_value("NAME") {
                dialect.source_system = name;
            }
            dialect.source_version = sour.child_value("VERS");
        }
        if let Some(gedc) = head.find_child("GEDC") {
            if let Some(vers) = gedc.child_value("VERS") {
                dialect.gedcom_version = vers;
            }
        }
        if let Some(ch) = head.child_value("CHAR") {
            dialect.charset = ch;
        }
    }

    // Pass 1: Collect top-level NOTE (`0 @Nxxxx@ NOTE`), SOUR (`0 @Sxxxx@ SOUR`), and OBJE (`0 @Oxxxx@ OBJE`)
    let mut top_level_notes: HashMap<String, String> = HashMap::new();
    let mut attached_top_level_notes: HashSet<String> = HashSet::new();
    let mut top_level_sources: HashMap<String, String> = HashMap::new();
    let mut obje_xref_to_media_id: HashMap<String, String> = HashMap::new();

    for node in &roots {
        match node.tag.as_str() {
            "NOTE" => {
                if let Some(ref xref) = node.xref {
                    let key = xref.trim_matches('@').to_uppercase();
                    top_level_notes.insert(key, node.value.trim().to_string());
                }
            }
            "SOUR" => {
                if let Some(ref xref) = node.xref {
                    let key = xref.trim_matches('@').to_uppercase();
                    let title = node
                        .child_value("TITL")
                        .or_else(|| node.child_value("ABBR"))
                        .unwrap_or_else(|| node.value.trim().to_string());
                    top_level_sources.insert(key, title);
                }
            }
            "OBJE" => {
                let raw_xref = node.xref.as_deref();
                let raw_xref_clean = raw_xref
                    .map(|x| x.trim_matches('@').to_uppercase())
                    .unwrap_or_default();

                let file_node = node.find_child("FILE");
                let raw_path = file_node
                    .map(|f| f.value.trim().to_string())
                    .unwrap_or_default();
                let (clean_file_name, was_stripped) = strip_media_directory_path(&raw_path);
                if was_stripped {
                    dialect.stripped_media_paths_count += 1;
                }

                let form_val = file_node
                    .and_then(|f| f.child_value("FORM"))
                    .or_else(|| node.child_value("FORM"));
                let explicit_title = file_node
                    .and_then(|f| f.child_value("TITL"))
                    .or_else(|| node.child_value("TITL"));

                if mode == "merge" {
                    if let Some(existing_media_id) = find_or_enrich_existing_media(
                        store,
                        initial_media_count,
                        &raw_xref_clean,
                        &raw_path,
                        &clean_file_name,
                        explicit_title.as_deref(),
                    ) {
                        if !raw_xref_clean.is_empty() {
                            obje_xref_to_media_id.insert(raw_xref_clean, existing_media_id);
                        }
                        continue;
                    }
                }

                let existing_gids: HashSet<String> =
                    store.media.iter().map(|m| m.gid.clone()).collect();
                let gid =
                    allocate_gid('M', &mut store.next_media_gid_seq, raw_xref, &existing_gids);

                let title_val = explicit_title.unwrap_or_else(|| {
                    if !clean_file_name.is_empty() {
                        clean_file_name.clone()
                    } else {
                        gid.clone()
                    }
                });

                let mime_type = infer_mime_type_from_form_or_path(form_val.as_deref(), &raw_path);
                let media_id = Uuid::new_v4().to_string();

                if !raw_xref_clean.is_empty() {
                    obje_xref_to_media_id.insert(raw_xref_clean, media_id.clone());
                }
                obje_xref_to_media_id.insert(gid.to_uppercase(), media_id.clone());

                store.media.push(MediaAssetSummaryDto {
                    id: media_id.clone(),
                    gid,
                    title: title_val,
                    mime_type,
                    file_url: format!("/api/v1/media/{media_id}/file"),
                    file_name: if clean_file_name.is_empty() {
                        None
                    } else {
                        Some(clean_file_name.clone())
                    },
                    original_file_name: if clean_file_name.is_empty() {
                        None
                    } else {
                        Some(clean_file_name)
                    },
                    file_size_bytes: None,
                    sha256: None,
                    width: None,
                    height: None,
                    description: None,
                    date_taken: None,
                    date_sort: None,
                    place_id: None,
                    place_full_name: None,
                    is_missing: true,
                    external_path: if raw_path.is_empty() {
                        None
                    } else {
                        Some(raw_path)
                    },
                    is_private: false,
                    only_living_persons: false,
                    linked_person_ids: vec![],
                    face_tags: vec![],
                    links: vec![],
                    version: 1,
                });
            }
            _ => {}
        }
    }

    // Pass 2: Parse all `0 @Ixxxx@ INDI` records
    let mut indi_xref_to_person_id: HashMap<String, String> = HashMap::new();
    let mut person_famc_pedi: HashMap<(String, String), String> = HashMap::new();
    let mut pending_associations: Vec<PendingAssociation> = Vec::new();
    let mut merged_individuals: usize = 0;
    let mut matched_by_uid: usize = 0;
    let mut matched_by_name_dates: usize = 0;
    let mut matched_by_kinship: usize = 0;
    let mut matched_keys_preview: Vec<String> = Vec::new();
    let precomputed_plan = if mode == "merge" {
        precompute_person_matches(store, initial_persons_count, initial_families_count, &roots)
    } else {
        PrecomputedMergePlan::default()
    };
    let chronology_conflicts_prevented = precomputed_plan.chronology_conflicts_prevented;

    for node in &roots {
        if node.tag != "INDI" {
            continue;
        }

        let raw_xref = node.xref.as_deref();
        let xref_clean = raw_xref
            .map(|x| x.trim_matches('@').to_uppercase())
            .unwrap_or_default();

        // Extract external IDs (`_UID` / `UID`, `RIN`, `_FSFTID`, `REFN`)
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
                "_FSFTID" => {
                    let fs_val = child.value.trim().to_string();
                    if !fs_val.is_empty() {
                        external_ids.insert("fsftid".to_string(), fs_val);
                    }
                }
                "REFN" => {
                    let refn_val = child.value.trim().to_string();
                    if !refn_val.is_empty() {
                        external_ids.insert("refn".to_string(), refn_val);
                    }
                }
                _ => {}
            }
        }

        // Parse all `1 NAME` blocks (`03 §2.2 #1`)
        let parsed_names: Vec<ParsedGedcomName> = node
            .children
            .iter()
            .filter(|c| c.tag == "NAME")
            .map(parse_gedcom_name_node)
            .collect();

        let mut given_name = String::new();
        let mut patronymic: Option<String> = None;
        let mut surname = String::new();
        let mut birth_surname: Option<String> = None;
        let mut nickname: Option<String> = None;
        let mut name_prefix: Option<String> = None;
        let mut name_suffix: Option<String> = None;
        let mut alt_names: Vec<IndividualAltNameDto> = Vec::new();

        let birth_entry = parsed_names
            .iter()
            .find(|n| matches!(n.name_type.as_deref(), Some("birth" | "maiden")))
            .or_else(|| parsed_names.first());
        let married_entry = parsed_names
            .iter()
            .find(|n| matches!(n.name_type.as_deref(), Some("married")));

        if let Some(b_name) = birth_entry {
            given_name = b_name.given_name.clone();
            patronymic = b_name.patronymic.clone();
            surname = b_name.surname.clone();
            if !b_name.surname.is_empty() {
                birth_surname = Some(b_name.surname.clone());
            }
            if let Some(ref marnm) = b_name.married_surname {
                if !marnm.is_empty() {
                    birth_surname = if !b_name.surname.is_empty() {
                        Some(b_name.surname.clone())
                    } else {
                        None
                    };
                    surname = marnm.clone();
                    dialect.married_names_resolved += 1;
                    alt_names.push(IndividualAltNameDto {
                        id: Uuid::new_v4().to_string(),
                        name_type: "married".to_string(),
                        given_name: Some(given_name.clone()),
                        patronymic: patronymic.clone(),
                        surname: Some(marnm.clone()),
                        lang: None,
                    });
                }
            }
            nickname = b_name.nickname.clone();
            name_prefix = b_name.prefix.clone();
            name_suffix = b_name.suffix.clone();
        }

        if let Some(m_name) = married_entry {
            if given_name.is_empty() && !m_name.given_name.is_empty() {
                given_name = m_name.given_name.clone();
            }
            if patronymic.is_none() && m_name.patronymic.is_some() {
                patronymic = m_name.patronymic.clone();
            }
            if !m_name.surname.is_empty() {
                surname = m_name.surname.clone();
                dialect.married_names_resolved += 1;
                alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: "married".to_string(),
                    given_name: if m_name.given_name.is_empty() {
                        Some(given_name.clone())
                    } else {
                        Some(m_name.given_name.clone())
                    },
                    patronymic: m_name.patronymic.clone().or_else(|| patronymic.clone()),
                    surname: Some(m_name.surname.clone()),
                    lang: None,
                });
            }
        }

        for extra in &parsed_names {
            let t = extra.name_type.as_deref().unwrap_or("");
            if !matches!(t, "" | "birth" | "maiden" | "married") {
                alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: t.to_string(),
                    given_name: if extra.given_name.is_empty() {
                        None
                    } else {
                        Some(extra.given_name.clone())
                    },
                    patronymic: extra.patronymic.clone(),
                    surname: if extra.surname.is_empty() {
                        None
                    } else {
                        Some(extra.surname.clone())
                    },
                    lang: None,
                });
            }
        }

        if let Some(top_nick) = node.child_value("NICK") {
            if nickname.is_none() {
                nickname = Some(top_nick);
            }
        }

        let sex = match node
            .child_value("SEX")
            .unwrap_or_else(|| "U".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "M" | "MALE" => "M".to_string(),
            "F" | "FEMALE" => "F".to_string(),
            _ => "U".to_string(),
        };

        let (birth_year, death_year) = extract_node_vital_years(node);
        let candidate = IncomingPersonCandidate {
            xref_clean: &xref_clean,
            given_name: &given_name,
            patronymic: patronymic.as_deref(),
            surname: &surname,
            birth_surname: birth_surname.as_deref(),
            nickname: nickname.as_deref(),
            name_prefix: name_prefix.as_deref(),
            name_suffix: name_suffix.as_deref(),
            sex: &sex,
            birth_year,
            death_year,
            min_child_birth_year: None,
            max_parent_birth_year: None,
            alt_names: &alt_names,
            external_ids: &external_ids,
        };

        // In merge mode, check if person already exists in `0..initial_persons_count`
        let prematched = if mode == "merge" {
            precomputed_plan.matches_by_xref.get(&xref_clean)
        } else {
            None
        };

        let person_id = if let Some(m) = prematched {
            let idx = m.existing_idx;
            match m.kind {
                PersonMatchKind::Uid => matched_by_uid += 1,
                PersonMatchKind::NameDates => matched_by_name_dates += 1,
                PersonMatchKind::Kinship => matched_by_kinship += 1,
            }
            if matched_keys_preview.len() < 20 {
                matched_keys_preview.push(m.preview_entry.clone());
            }
            let pid = store.persons[idx].id.clone();
            enrich_existing_person(&mut store.persons[idx], &candidate);
            merged_individuals += 1;
            pid
        } else {
            let existing_gids: HashSet<String> =
                store.persons.iter().map(|p| p.gid.clone()).collect();
            let gid = allocate_gid('I', &mut store.next_gid_seq, raw_xref, &existing_gids);
            let new_pid = Uuid::new_v4().to_string();

            store.persons.push(TreePersonNodeDto {
                id: new_pid.clone(),
                gid: gid.clone(),
                sex,
                is_living: true,
                living_override: None,
                is_private: node.child_value("RESN").is_some_and(|r| {
                    r.to_ascii_lowercase().contains("privacy")
                        || r.to_ascii_lowercase().contains("confidential")
                }),
                primary_name: PersonNameDto {
                    given_name,
                    patronymic,
                    surname,
                },
                birth_surname,
                nickname,
                name_prefix,
                name_suffix,
                biography: None,
                birth: None,
                death: None,
                avatar_url: None,
                avatar_media_id: None,
                avatar_crop: None,
                notes: None,
                birth_sort: None,
                death_sort: None,
                has_death_event: false,
                alt_names,
                external_ids,
                version: 1,
                tx_id: None,
                generation: None,
                is_direct_lineage: None,
            });
            new_pid
        };

        if !xref_clean.is_empty() {
            indi_xref_to_person_id.insert(xref_clean.clone(), person_id.clone());
        }

        // Parse child tags on INDI: events, FAMC, ASSO, OBJE, NOTE
        for child in &node.children {
            match child.tag.as_str() {
                "FAMC" => {
                    let fam_xref = child.value.trim().trim_matches('@').to_uppercase();
                    if !fam_xref.is_empty() {
                        let pedi = child
                            .child_value("PEDI")
                            .map(|p| p.to_ascii_lowercase())
                            .unwrap_or_else(|| "birth".to_string());
                        let mapped_pedi = match pedi.as_str() {
                            "adopted" | "adoption" => "adopted",
                            "foster" => "foster",
                            "step" => "step",
                            "unknown" => "unknown",
                            _ => "birth",
                        };
                        person_famc_pedi
                            .insert((person_id.clone(), fam_xref), mapped_pedi.to_string());
                    }
                }
                "ASSO" => {
                    let target_xref = child.value.trim().trim_matches('@').to_uppercase();
                    let rela = child
                        .child_value("RELA")
                        .or_else(|| child.child_value("TYPE"))
                        .unwrap_or_else(|| "Association".to_string());
                    if !target_xref.is_empty() {
                        pending_associations.push(PendingAssociation {
                            person_id: person_id.clone(),
                            target_xref,
                            rela,
                        });
                    }
                }
                "OBJE" => {
                    let obje_xref = child.value.trim().trim_matches('@').to_uppercase();
                    if let Some(media_id) = obje_xref_to_media_id.get(&obje_xref).cloned() {
                        link_media_to_person(store, &media_id, &person_id);
                    }
                }
                "NOTE" => {
                    attach_gedcom_note(
                        store,
                        &child.value,
                        "individual",
                        &person_id,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                    );
                }
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
                        Some(&person_id),
                        None,
                        "custom",
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                "BIRT" | "CHR" | "BAPM" | "DEAT" | "BURI" | "CREM" | "ADOP" | "OCCU" | "EDUC"
                | "GRAD" | "RESI" | "IMMI" | "EMIG" | "NATU" | "CENS" | "MILI" | "RETI" => {
                    let ev_type = match child.tag.as_str() {
                        "BIRT" => "birth",
                        "CHR" | "BAPM" => "baptism",
                        "DEAT" => "death",
                        "BURI" => "burial",
                        "CREM" => "cremation",
                        "ADOP" => "adoption",
                        "OCCU" => "occupation",
                        "EDUC" => "education",
                        "GRAD" => "graduation",
                        "RESI" => "residence",
                        "IMMI" => "immigration",
                        "EMIG" => "emigration",
                        "NATU" => "naturalization",
                        "CENS" => "census",
                        "MILI" => "military_service",
                        "RETI" => "retirement",
                        _ => "custom",
                    };

                    if child.tag == "DEAT" {
                        if let Some(p) = store.persons.iter_mut().find(|p| p.id == person_id) {
                            p.has_death_event = true;
                            p.living_override = Some(false);
                        }
                        let has_date_or_place = child.child_value("DATE").is_some_and(|d| {
                            !d.trim().is_empty() && !d.trim().eq_ignore_ascii_case("Y")
                        }) || child.find_child("PLAC").is_some();
                        if !has_date_or_place {
                            dialect.deceased_flag_only_count += 1;
                        }
                    }

                    parse_and_insert_event(
                        store,
                        child,
                        Some(&person_id),
                        None,
                        ev_type,
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                _ => {}
            }
        }
    }

    let merged_families = import_gedcom_families_and_associations(
        store,
        initial_families_count,
        &roots,
        &mode,
        &indi_xref_to_person_id,
        &person_famc_pedi,
        pending_associations,
        &top_level_notes,
        &mut attached_top_level_notes,
        &top_level_sources,
        &obje_xref_to_media_id,
        &mut dialect,
        &mut warnings,
    );

    // Pass 5: Recompute vitals, living status, place counts, and media `only_living_persons`
    recompute_all_individuals_vitals(store, current_year, age_threshold);
    recompute_all_places_metadata(store);

    let living_map: HashMap<String, bool> = store
        .persons
        .iter()
        .map(|p| (p.id.clone(), p.is_living))
        .collect();
    for media in &mut store.media {
        if !media.linked_person_ids.is_empty() {
            media.only_living_persons = media
                .linked_person_ids
                .iter()
                .all(|pid| living_map.get(pid).copied().unwrap_or(true));
        }
    }

    let individuals_imported = store.persons.len().saturating_sub(initial_persons_count);
    let families_imported = store.families.len().saturating_sub(initial_families_count);
    let events_imported = store.events.len().saturating_sub(initial_events_count);
    let places_imported = store.places.len().saturating_sub(initial_places_count);
    let media_imported = store.media.len().saturating_sub(initial_media_count);
    let media_missing_count = store.media.iter().filter(|m| m.is_missing).count();
    let notes_imported = store.notes.len().saturating_sub(initial_notes_count);

    dialect.coordinates_normalized = dialect.normalized_coordinates_count;
    dialect.uids_preserved = dialect.preserved_uids_count;
    dialect.vendor_upd_filtered = dialect.filtered_upd_timestamps_count;
    dialect.parental_asso_deduplicated = dialect.deduplicated_parental_associations_count;
    dialect.missing_media_stubs = media_missing_count;

    if media_missing_count > 0 && warnings.is_empty() {
        warnings.push(format!(
            "{media_missing_count} external media paths have no bundled binary; media asset stubs will be created for post-import reconciliation."
        ));
    }

    let import_tx_id = Uuid::new_v4().to_string();
    store.change_log.push(ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        tx_id: import_tx_id,
        tx_label: "gedcom.import".to_string(),
        op: "gedcom_import".to_string(),
        action: "gedcom_import".to_string(),
        entity_type: "tree".to_string(),
        entity_id: "root".to_string(),
        entity_label: Some("GEDCOM Import".to_string()),
        actor_id: None,
        actor_name: None,
        summary: format!(
            "Imported GEDCOM ({} individuals, {} families, {} media)",
            individuals_imported, families_imported, media_imported
        ),
        before_json: None,
        after_json: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(GedcomImportResult {
        status: "ok".to_string(),
        operation: "gedcom_import".to_string(),
        message: format!(
            "Imported {} individuals, {} families, and {} media items",
            individuals_imported, families_imported, media_imported
        ),
        mode,
        snapshot_id,
        imported_individuals: individuals_imported,
        imported_families: families_imported,
        merged_individuals,
        merged_families,
        matched_by_uid,
        matched_by_name_dates,
        matched_by_kinship,
        chronology_conflicts_prevented,
        matched_keys_preview,
        imported_events: events_imported,
        imported_places: places_imported,
        imported_media: media_imported,
        missing_media_count: media_missing_count,
        prompt_home_person: individuals_imported > 0,
        individuals_imported,
        families_imported,
        events_imported,
        places_imported,
        media_imported,
        media_missing_count,
        notes_imported,
        dialect,
        warnings,
    })
}
