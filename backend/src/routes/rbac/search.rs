#[allow(unused_imports)]
use super::*;

pub(crate) fn format_person_display_name(person: &TreePersonNodeDto) -> String {
    let mut parts = Vec::new();
    if !person.primary_name.surname.trim().is_empty() {
        if let Some(ref bs) = person.birth_surname {
            let bs_trimmed = bs.trim();
            if !bs_trimmed.is_empty()
                && !bs_trimmed.eq_ignore_ascii_case(&person.primary_name.surname)
            {
                parts.push(format!(
                    "{} ({bs_trimmed})",
                    person.primary_name.surname.trim()
                ));
            } else {
                parts.push(person.primary_name.surname.trim().to_string());
            }
        } else {
            parts.push(person.primary_name.surname.trim().to_string());
        }
    }
    if !person.primary_name.given_name.trim().is_empty() {
        parts.push(person.primary_name.given_name.trim().to_string());
    }
    if let Some(ref pat) = person.primary_name.patronymic {
        if !pat.trim().is_empty() {
            parts.push(pat.trim().to_string());
        }
    }
    if parts.is_empty() {
        person.gid.clone()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn build_person_token_forms(word: &str) -> Vec<String> {
    let ortho = normalize_russian_orthography_word(word);
    let folded = fold_slavic_surname_stem(&ortho);
    let lat_ortho = transliterate_to_latin(&ortho);
    let lat_folded = transliterate_to_latin(&folded);
    let mut out = Vec::with_capacity(4);
    for cand in [ortho, folded, lat_ortho, lat_folded] {
        if !cand.is_empty() && !out.contains(&cand) {
            out.push(cand);
        }
    }
    out
}

pub(crate) fn score_person_for_search(
    person: &TreePersonNodeDto,
    parsed: &crate::services::privacy::ParsedSearchQuery,
) -> Option<i32> {
    // 1. Exact GID match jumps straight to top score
    if let Some(ref gid_q) = parsed.exact_gid {
        if person.gid.eq_ignore_ascii_case(gid_q) {
            return Some(10_000);
        }
    }

    let mut raw_words: Vec<&str> = Vec::new();
    for field in [
        Some(person.primary_name.surname.as_str()),
        Some(person.primary_name.given_name.as_str()),
        person.primary_name.patronymic.as_deref(),
        person.birth_surname.as_deref(),
        person.nickname.as_deref(),
        Some(person.gid.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        for w in field
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
        {
            raw_words.push(w);
        }
    }
    for alt in &person.alt_names {
        for field in [
            alt.given_name.as_deref(),
            alt.patronymic.as_deref(),
            alt.surname.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            for w in field
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
            {
                raw_words.push(w);
            }
        }
    }

    let candidate_word_forms: Vec<Vec<String>> = raw_words
        .iter()
        .map(|w| build_person_token_forms(w))
        .collect();

    let mut total_score: i32 = 0;

    for q_tok in &parsed.text_tokens {
        let q_forms = build_person_token_forms(q_tok);
        let mut best_token_score: i32 = 0;

        for c_forms in &candidate_word_forms {
            for qf in &q_forms {
                for cf in c_forms {
                    if cf == qf {
                        best_token_score = best_token_score.max(120);
                    } else if cf.starts_with(qf) {
                        best_token_score = best_token_score.max(85);
                    } else if cf.contains(qf) && qf.chars().count() >= 2 {
                        best_token_score = best_token_score.max(50);
                    }
                }
            }
        }

        if best_token_score == 0 {
            // Every query text token must match at least one name/gid token
            return None;
        }
        total_score += best_token_score;
    }

    let birth_year = person.birth_sort.map(|bs| bs / 10_000);
    if let Some(target_year) = parsed.year_hint {
        match birth_year {
            Some(by) => {
                let diff = (by - target_year).abs();
                if diff == 0 {
                    total_score += 250;
                } else if diff <= 2 {
                    total_score += 200 - (diff * 20);
                } else if diff <= 5 {
                    total_score += 60;
                } else if parsed.text_tokens.is_empty() {
                    // Pure numeric year search must be within ±2 years
                    return None;
                }
            }
            None => {
                if parsed.text_tokens.is_empty() {
                    return None;
                }
            }
        }
    }

    if total_score > 0 {
        Some(total_score)
    } else {
        None
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Global grouped omnibox search across people, places, media, and role-aware actions", body = SearchResultResponseDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn global_search(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResultResponseDto>, AppError> {
    let start_instant = std::time::Instant::now();
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let types_set: HashSet<String> = query
        .types
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

    let all_types = types_set.is_empty();
    let want_person = all_types || types_set.contains("person") || types_set.contains("people");
    let want_place = all_types || types_set.contains("place") || types_set.contains("places");
    let want_media = all_types || types_set.contains("media");
    let want_action = all_types || types_set.contains("action") || types_set.contains("actions");

    let person_limit = query.limit.unwrap_or(8).clamp(1, 50);
    let secondary_limit = query.limit.unwrap_or(3).clamp(1, 25);

    let q_trimmed = query.q.trim().to_string();
    let parsed = parse_search_query(&q_trimmed);

    // Count family connections per person to break ties (`13 §2.3`)
    let mut connections_count: HashMap<String, usize> = HashMap::new();
    for fam in &store.families {
        let mut members = Vec::new();
        if let Some(ref p1) = fam.partner1_id {
            members.push(p1.clone());
        }
        if let Some(ref p2) = fam.partner2_id {
            members.push(p2.clone());
        }
        for ch in &fam.children {
            members.push(ch.child_id.clone());
        }
        let deg = members.len().saturating_sub(1);
        for m in members {
            *connections_count.entry(m).or_insert(0) += deg;
        }
    }

    let mut people = Vec::new();
    if want_person && !q_trimmed.is_empty() {
        for raw_person in &store.persons {
            if raw_person.is_private && !access.can_view_private() {
                continue;
            }
            // Living-Person Privacy Guard (`13 §2.4, §6 DoD #5`):
            // Anonymous visitors in public mode must never be able to discover living people
            // by full name, surname, or birth year.
            if access == CallerAccess::AnonymousPublic && raw_person.is_living {
                if !state.config.public_search_show_living {
                    continue;
                }
                // Even when `public_search_show_living=true`, only match against the redacted node
                if let Some(masked) = redact_person_for_tree_view(raw_person, &access) {
                    if let Some(score) = score_person_for_search(&masked, &parsed) {
                        people.push(SearchPersonResultDto {
                            id: masked.id.clone(),
                            gid: masked.gid.clone(),
                            display_name: masked.primary_name.given_name.clone(),
                            given_name: masked.primary_name.given_name,
                            patronymic: None,
                            surname: String::new(),
                            sex: masked.sex,
                            birth_year: None,
                            death_year: None,
                            birth_place: None,
                            avatar_url: None,
                            is_living: true,
                            is_private: false,
                            score,
                        });
                    }
                }
                continue;
            }

            if let Some(base_score) = score_person_for_search(raw_person, &parsed) {
                let rel_bonus = connections_count
                    .get(&raw_person.id)
                    .copied()
                    .unwrap_or(0)
                    .min(15) as i32;
                let final_score = base_score + rel_bonus;
                let birth_year = raw_person.birth_sort.map(|bs| bs / 10_000);
                let death_year = raw_person.death_sort.map(|ds| ds / 10_000);
                let birth_place = raw_person.birth.as_ref().and_then(|b| b.place.clone());

                people.push(SearchPersonResultDto {
                    id: raw_person.id.clone(),
                    gid: raw_person.gid.clone(),
                    display_name: format_person_display_name(raw_person),
                    given_name: raw_person.primary_name.given_name.clone(),
                    patronymic: raw_person.primary_name.patronymic.clone(),
                    surname: raw_person.primary_name.surname.clone(),
                    sex: raw_person.sex.clone(),
                    birth_year,
                    death_year,
                    birth_place,
                    avatar_url: raw_person.avatar_url.clone(),
                    is_living: raw_person.is_living,
                    is_private: raw_person.is_private,
                    score: final_score,
                });
            }
        }

        people.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| {
                    a.birth_year
                        .unwrap_or(i32::MAX)
                        .cmp(&b.birth_year.unwrap_or(i32::MAX))
                })
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
        people.truncate(person_limit);
    }

    let mut places = Vec::new();
    if want_place && !q_trimmed.is_empty() {
        let q_forms = build_person_token_forms(&q_trimmed);
        for place in &store.places {
            let place_tokens = normalize_search_tokens(&format!(
                "{} {} {}",
                place.name,
                place.full_name,
                place
                    .alt_names
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
            let matches_gid = parsed
                .exact_gid
                .as_ref()
                .is_some_and(|g| place.gid.eq_ignore_ascii_case(g));
            let matches_text = !q_forms.is_empty()
                && q_forms
                    .iter()
                    .any(|qf| place_tokens.split_whitespace().any(|pt| pt.contains(qf)));
            if matches_gid || matches_text {
                places.push(SearchPlaceResultDto {
                    id: place.id.clone(),
                    gid: place.gid.clone(),
                    name: place.name.clone(),
                    full_name: place.full_name.clone(),
                    place_type: place.place_type.clone(),
                    events_count: place.events_count,
                    has_coordinates: place.latitude.is_some() && place.longitude.is_some(),
                });
            }
        }
        places.sort_by(|a, b| {
            b.events_count
                .cmp(&a.events_count)
                .then_with(|| a.name.cmp(&b.name))
        });
        places.truncate(secondary_limit);
    }

    let mut media = Vec::new();
    if want_media && !q_trimmed.is_empty() {
        let q_forms = build_person_token_forms(&q_trimmed);
        for m in &store.media {
            let Some(visible_m) = redact_media_for_caller(m, &access, &store) else {
                continue;
            };
            let media_tokens =
                normalize_search_tokens(&format!("{} {}", visible_m.title, visible_m.gid));
            let matches = q_forms
                .iter()
                .any(|qf| media_tokens.split_whitespace().any(|mt| mt.contains(qf)));
            if matches {
                media.push(SearchMediaResultDto {
                    id: visible_m.id,
                    gid: visible_m.gid,
                    title: visible_m.title,
                    mime_type: visible_m.mime_type,
                    file_url: visible_m.file_url,
                    is_private: visible_m.is_private,
                });
            }
        }
        media.truncate(secondary_limit);
    }

    let mut actions = Vec::new();
    if want_action && access != CallerAccess::AnonymousPublic {
        let mut catalog = vec![
            SearchActionDto {
                id: "nav-tree".to_string(),
                label: "Interactive Family Tree".to_string(),
                hint: "Open tree canvas".to_string(),
                path: "/tree".to_string(),
                icon: "git-branch".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-people".to_string(),
                label: "People Directory".to_string(),
                hint: "Browse and filter all individuals".to_string(),
                path: "/people".to_string(),
                icon: "users".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-surnames".to_string(),
                label: "Surnames Index".to_string(),
                hint: "Alphabetical list of family names".to_string(),
                path: "/people/surnames".to_string(),
                icon: "book-open".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-places".to_string(),
                label: "Places Directory".to_string(),
                hint: "Historical places and map".to_string(),
                path: "/places".to_string(),
                icon: "map-pin".to_string(),
                required_role: "viewer".to_string(),
            },
        ];

        if access.can_edit() {
            catalog.push(SearchActionDto {
                id: "act-add-person".to_string(),
                label: "Add Person".to_string(),
                hint: "Create a new individual record".to_string(),
                path: "/people?action=add".to_string(),
                icon: "user-plus".to_string(),
                required_role: "editor".to_string(),
            });
        }

        if access.is_admin() {
            catalog.push(SearchActionDto {
                id: "act-import-gedcom".to_string(),
                label: "Import GEDCOM".to_string(),
                hint: "Import a .ged genealogy file".to_string(),
                path: "/admin/gedcom".to_string(),
                icon: "upload".to_string(),
                required_role: "admin".to_string(),
            });
            catalog.push(SearchActionDto {
                id: "nav-settings".to_string(),
                label: "Go to Settings".to_string(),
                hint: "Instance configuration & backups".to_string(),
                path: "/admin/settings".to_string(),
                icon: "settings".to_string(),
                required_role: "admin".to_string(),
            });
        }

        if q_trimmed.is_empty() {
            actions = catalog;
        } else {
            let q_lower = q_trimmed.to_lowercase();
            actions = catalog
                .into_iter()
                .filter(|a| {
                    a.label.to_lowercase().contains(&q_lower)
                        || a.hint.to_lowercase().contains(&q_lower)
                })
                .collect();
        }
    }

    let timing_ms = start_instant.elapsed().as_millis() as u64;

    Ok(Json(SearchResultResponseDto {
        query: q_trimmed,
        people,
        places,
        media,
        actions,
        timing_ms,
    }))
}
