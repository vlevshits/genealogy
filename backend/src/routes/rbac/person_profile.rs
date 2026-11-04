#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/persons/suggest-patronymic",
    params(PatronymicSuggestionQuery),
    responses(
        (status = 200, description = "Suggested Slavic patronymic for child", body = PatronymicSuggestionResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_patronymic_suggestion(
    RequireAuth(_auth): RequireAuth,
    Query(query): Query<PatronymicSuggestionQuery>,
) -> Result<Json<PatronymicSuggestionResponse>, AppError> {
    let patronymic = suggest_slavic_patronymic(&query.father_given_name, &query.sex);
    Ok(Json(PatronymicSuggestionResponse { patronymic }))
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GEDCOM-style GID (e.g. I0042)")
    ),
    responses(
        (status = 200, description = "Person record redacted per caller RBAC tier", body = TreePersonNodeDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Person not found or private", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let resolved_id = store.resolve_person_id(target).ok_or(AppError::NotFound)?;
    let person = store
        .persons
        .iter()
        .find(|p| p.id == resolved_id)
        .ok_or(AppError::NotFound)?;

    let redacted = redact_person_for_caller(person, &access, &settings.default_locale)
        .ok_or(AppError::NotFound)?;

    Ok(Json(redacted))
}

pub(crate) fn build_profile_relative_summary(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
) -> Option<ProfileRelativeSummaryDto> {
    let redacted = redact_person_for_tree_view(person, access)?;
    Some(ProfileRelativeSummaryDto {
        id: redacted.id,
        gid: redacted.gid,
        given_name: redacted.primary_name.given_name,
        patronymic: redacted.primary_name.patronymic,
        surname: redacted.primary_name.surname,
        sex: redacted.sex,
        is_living: redacted.is_living,
        birth_year: redacted.birth_sort.map(|bs| bs / 10_000),
        death_year: redacted.death_sort.map(|ds| ds / 10_000),
        avatar_url: redacted.avatar_url,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}/profile",
    params(
        ("id" = String, Path, description = "Person UUID or GEDCOM-style GID (e.g. I0042)"),
        PersonProfileQuery
    ),
    responses(
        (status = 200, description = "Aggregated person profile dossier including kinship, life events, family links, media, notes, sources, and capabilities", body = PersonProfileResponseDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person_profile(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PersonProfileQuery>,
) -> Result<Json<PersonProfileResponseDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let resolved_id = store
        .resolve_person_id(target)
        .ok_or(AppError::PersonNotFound)?;
    let raw_person = store
        .persons
        .iter()
        .find(|p| p.id == resolved_id)
        .ok_or(AppError::PersonNotFound)?;

    let redacted_person =
        redact_person_for_tree_view(raw_person, &access).ok_or(AppError::PersonNotFound)?;

    let include_set: HashSet<String> = query
        .include
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .map(|part| part.trim().to_ascii_lowercase())
                .filter(|part| !part.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let include_all = include_set.is_empty();
    let want_kinship = include_all || include_set.contains("kinship");
    let want_events = include_all || include_set.contains("events");
    let want_families = include_all || include_set.contains("families");
    let want_media = include_all || include_set.contains("media");
    let want_notes = include_all || include_set.contains("notes");
    let want_sources = include_all || include_set.contains("sources");

    let (resolved_home_person_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let capabilities = ProfileCapabilitiesDto {
        can_edit: access.can_edit(),
        can_delete: access.can_edit(),
        can_merge: access.is_admin(),
        is_home_person: resolved_home_person_id.as_deref() == Some(raw_person.id.as_str()),
    };

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();
    let visible_person_ids: HashSet<String> =
        visible_persons.iter().map(|p| p.id.clone()).collect();
    let person_map: HashMap<String, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.clone(), p)).collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter_map(|f| {
            filter_family_for_visible_persons(f, access.can_view_private(), &visible_person_ids)
        })
        .collect();

    // Build families section (always respecting living/private redaction for each relative)
    let mut parent_families = Vec::new();
    let mut spouse_families = Vec::new();
    let mut person_spouse_family_ids: HashSet<String> = HashSet::new();
    let mut family_spouse_map: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

    for fam in &visible_families {
        // Parent family where target person is a child
        if fam.children.iter().any(|c| c.child_id == raw_person.id) && want_families {
            let p1 = fam
                .partner1_id
                .as_ref()
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));
            let p2 = fam
                .partner2_id
                .as_ref()
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));

            // Assign father/mother by sex when known, otherwise partner1 -> father, partner2 -> mother
            let (father, mother) = match (&p1, &p2) {
                (Some(a), Some(b)) if a.sex == "F" && b.sex == "M" => {
                    (Some(b.clone()), Some(a.clone()))
                }
                (Some(a), None) if a.sex == "F" => (None, Some(a.clone())),
                (None, Some(b)) if b.sex == "M" => (Some(b.clone()), None),
                _ => (p1, p2),
            };

            let mut siblings = Vec::new();
            for ch in &fam.children {
                if ch.child_id == raw_person.id {
                    continue;
                }
                if let Some(sib_p) = person_map.get(&ch.child_id) {
                    if let Some(sib_summary) = build_profile_relative_summary(sib_p, &access) {
                        siblings.push(sib_summary);
                    }
                }
            }

            parent_families.push(ProfileParentFamilyDto {
                family_id: fam.id.clone(),
                gid: fam.gid.clone(),
                father,
                mother,
                siblings,
                version: fam.version,
            });
        }

        // Spouse family where target person is partner1 or partner2
        let is_p1 = fam.partner1_id.as_deref() == Some(raw_person.id.as_str());
        let is_p2 = fam.partner2_id.as_deref() == Some(raw_person.id.as_str());
        if is_p1 || is_p2 {
            person_spouse_family_ids.insert(fam.id.clone());
            let other_id = if is_p1 {
                fam.partner2_id.as_ref()
            } else {
                fam.partner1_id.as_ref()
            };
            let spouse_summary = other_id
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));

            if let Some(ref sp) = spouse_summary {
                let full_name = format!(
                    "{} {} {}",
                    sp.given_name,
                    sp.patronymic.as_deref().unwrap_or(""),
                    sp.surname
                )
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
                family_spouse_map.insert(fam.id.clone(), (Some(sp.id.clone()), Some(full_name)));
            } else {
                family_spouse_map.insert(fam.id.clone(), (None, None));
            }

            if want_families {
                let marriage_ev = store.events.iter().find(|e| {
                    e.family_id.as_deref() == Some(fam.id.as_str())
                        && e.event_type == "marriage"
                        && (!e.is_private || access.can_view_private())
                });
                let is_masked_union = access == CallerAccess::AnonymousPublic
                    && (raw_person.is_living
                        || spouse_summary.as_ref().is_some_and(|s| s.is_living));

                let marriage_date = if is_masked_union {
                    None
                } else {
                    marriage_ev.and_then(|e| {
                        e.date
                            .as_ref()
                            .map(|d| d.text.clone())
                            .or_else(|| e.gedcom_date.clone())
                    })
                };
                let marriage_place = if is_masked_union {
                    None
                } else {
                    marriage_ev.and_then(|e| e.place_full_name.clone())
                };

                let mut children = Vec::new();
                for ch in &fam.children {
                    if let Some(ch_p) = person_map.get(&ch.child_id) {
                        if let Some(ch_summary) = build_profile_relative_summary(ch_p, &access) {
                            children.push(ProfileChildSummaryDto {
                                person: ch_summary,
                                sort_order: ch.sort_order,
                                rel_to_partner1: ch.rel_to_partner1.clone(),
                                rel_to_partner2: ch.rel_to_partner2.clone(),
                            });
                        }
                    }
                }
                children.sort_by_key(|c| c.sort_order);

                spouse_families.push(ProfileSpouseFamilyDto {
                    family_id: fam.id.clone(),
                    gid: fam.gid.clone(),
                    relationship_type: fam.relationship_type.clone(),
                    spouse: spouse_summary,
                    marriage_date,
                    marriage_place,
                    children,
                    version: fam.version,
                });
            }
        }
    }

    let families = ProfileFamiliesDto {
        parent_families,
        spouse_families,
    };

    // Living-person public shield (`03-person-profile-page.md §1, §3.1, §4 DoD #3`):
    // When caller is AnonymousPublic and target person is_living, mask events, media, notes, sources, and kinship.
    if access == CallerAccess::AnonymousPublic && raw_person.is_living {
        return Ok(Json(PersonProfileResponseDto {
            person: redacted_person,
            kinship: None,
            events: vec![],
            families,
            media: vec![],
            notes: vec![],
            sources_summary: vec![],
            capabilities,
        }));
    }

    let kinship = if want_kinship {
        resolved_home_person_id.as_deref().and_then(|home_id| {
            calculate_kinship_relation(home_id, &raw_person.id, &visible_persons, &visible_families)
        })
    } else {
        None
    };

    let places_map: HashMap<String, &PlaceSummaryDto> =
        store.places.iter().map(|p| (p.id.clone(), p)).collect();
    let birth_year = raw_person.birth_sort.map(|bs| bs / 10_000);

    // Collect relevant life events (individual + spouse family events)
    let mut raw_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| {
            if e.is_private && !access.can_view_private() {
                return false;
            }
            let is_ind = e.individual_id.as_deref() == Some(raw_person.id.as_str());
            let is_fam = e
                .family_id
                .as_ref()
                .is_some_and(|fid| person_spouse_family_ids.contains(fid));
            is_ind || is_fam
        })
        .collect();

    raw_events.sort_by(|a, b| {
        let sa = a.date_sort.unwrap_or(i32::MAX);
        let sb = b.date_sort.unwrap_or(i32::MAX);
        sa.cmp(&sb).then_with(|| a.gid.cmp(&b.gid))
    });

    let mut events = Vec::new();
    let mut source_groups: Vec<(String, Vec<String>)> = Vec::new();

    for ev in &raw_events {
        if want_sources {
            if let Some(src) = ev
                .source_text
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                if let Some(existing) = source_groups
                    .iter_mut()
                    .find(|(s, _)| s.eq_ignore_ascii_case(src))
                {
                    if !existing.1.contains(&ev.id) {
                        existing.1.push(ev.id.clone());
                    }
                } else {
                    source_groups.push((src.to_string(), vec![ev.id.clone()]));
                }
            }
        }

        if want_events {
            let ev_year = ev.date_sort.map(|ds| ds / 10_000);
            let is_post_death = matches!(
                ev.event_type.as_str(),
                "burial" | "cremation" | "probate" | "will"
            );
            let age_at_event = if ev.event_type == "birth" {
                Some(0)
            } else if is_post_death {
                None
            } else if let (Some(by), Some(ey)) = (birth_year, ev_year) {
                if ey >= by {
                    Some(ey - by)
                } else {
                    None
                }
            } else {
                None
            };

            let resolved_place = ev.place_id.as_ref().and_then(|pid| places_map.get(pid));
            let latitude = resolved_place.and_then(|p| p.latitude);
            let longitude = resolved_place.and_then(|p| p.longitude);
            let place_full_name = ev
                .place_full_name
                .clone()
                .or_else(|| resolved_place.map(|p| p.full_name.clone()));

            let (spouse_id, spouse_name) = ev
                .family_id
                .as_ref()
                .and_then(|fid| family_spouse_map.get(fid))
                .cloned()
                .unwrap_or((None, None));

            events.push(ProfileLifeEventDto {
                id: ev.id.clone(),
                gid: ev.gid.clone(),
                individual_id: ev.individual_id.clone(),
                family_id: ev.family_id.clone(),
                event_type: ev.event_type.clone(),
                custom_type: ev.custom_type.clone(),
                date: ev.date.clone(),
                date_sort: ev.date_sort,
                gedcom_date: ev.gedcom_date.clone(),
                place_id: ev.place_id.clone(),
                place_full_name,
                description: ev.description.clone(),
                source_text: ev.source_text.clone(),
                media_ids: ev.media_ids.clone(),
                is_private: ev.is_private,
                version: ev.version,
                age_at_event,
                latitude,
                longitude,
                scans_count: ev.media_ids.len(),
                spouse_id,
                spouse_name,
            });
        }
    }

    let sources_summary: Vec<ProfileSourceSummaryDto> = if want_sources {
        source_groups
            .into_iter()
            .map(|(source_text, event_ids)| {
                let events_count = event_ids.len();
                ProfileSourceSummaryDto {
                    source_text,
                    event_ids,
                    events_count,
                }
            })
            .collect()
    } else {
        vec![]
    };

    let media: Vec<MediaAssetSummaryDto> = if want_media {
        let event_media_ids: HashSet<String> = raw_events
            .iter()
            .flat_map(|e| e.media_ids.iter().cloned())
            .collect();
        store
            .media
            .iter()
            .filter(|m| {
                m.linked_person_ids.contains(&raw_person.id)
                    || m.face_tags.iter().any(|ft| ft.person_id == raw_person.id)
                    || event_media_ids.contains(&m.id)
            })
            .filter_map(|m| redact_media_for_caller(m, &access, &store))
            .collect()
    } else {
        vec![]
    };

    let notes: Vec<NoteDto> = if want_notes {
        let mut person_notes: Vec<NoteDto> = store
            .notes
            .iter()
            .filter(|n| {
                n.entity_type == "individual"
                    && n.entity_id == raw_person.id
                    && (!n.is_private || access.can_view_private())
            })
            .cloned()
            .collect();
        person_notes.sort_by_key(|n| n.sort_order);
        person_notes
    } else {
        vec![]
    };

    Ok(Json(PersonProfileResponseDto {
        person: redacted_person,
        kinship,
        events,
        families,
        media,
        notes,
        sources_summary,
        capabilities,
    }))
}
