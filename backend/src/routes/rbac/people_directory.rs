#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/people",
    params(PeopleDirectoryQuery),
    responses(
        (status = 200, description = "People directory with cross-script search, filters, sorting, and cursor pagination (authenticated users only)", body = PeopleDirectoryResponse),
        (status = 401, description = "Authentication required even when public tree sharing is enabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_people_directory(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<PeopleDirectoryQuery>,
) -> Result<Json<PeopleDirectoryResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    // Precompute which individuals have at least one parent in visible families
    let mut has_parents_set: HashSet<String> = HashSet::new();
    for fam in &store.families {
        if fam.is_private && !access.can_view_private() {
            continue;
        }
        let has_any_parent = fam.partner1_id.is_some() || fam.partner2_id.is_some();
        if has_any_parent {
            for ch in &fam.children {
                has_parents_set.insert(ch.child_id.clone());
            }
        }
    }

    // Precompute latest change_log timestamp per individual
    let mut updated_at_map: HashMap<String, String> = HashMap::new();
    for entry in &store.change_log {
        if entry.entity_type == "individual" {
            updated_at_map.insert(entry.entity_id.clone(), entry.created_at.clone());
        }
    }

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let total_unfiltered = visible_persons.len();

    let parsed_q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_search_query);

    let surname_filter_stem = query
        .surname
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| transliterate_to_latin(&fold_slavic_surname_stem(s)));

    let sex_filter = query
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
        .map(|s| s.to_ascii_uppercase());

    let living_filter = query
        .living
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());

    let mut filtered_items: Vec<PeopleDirectoryItemDto> = Vec::new();

    for p in visible_persons {
        let birth_year = p.birth_sort.map(|bs| bs / 10_000);
        let death_year = p.death_sort.map(|ds| ds / 10_000);
        let birth_date = p.birth.as_ref().and_then(|b| b.date.clone());
        let birth_place = p.birth.as_ref().and_then(|b| b.place.clone());
        let death_date = p.death.as_ref().and_then(|d| d.date.clone());
        let death_place = p.death.as_ref().and_then(|d| d.place.clone());
        let has_parents = has_parents_set.contains(&p.id);
        let has_birth_date = p.birth_sort.is_some() || birth_date.is_some();

        // 1. Free-text query filter (`q`)
        if let Some(ref pq) = parsed_q {
            if score_person_for_search(&p, pq).is_none() {
                continue;
            }
        }

        // 2. Surname filter (gender-folded + cross-script aware)
        if let Some(ref target_stem) = surname_filter_stem {
            let primary_stem =
                transliterate_to_latin(&fold_slavic_surname_stem(&p.primary_name.surname));
            let birth_stem = p
                .birth_surname
                .as_deref()
                .map(|bs| transliterate_to_latin(&fold_slavic_surname_stem(bs)));
            let matches_primary = &primary_stem == target_stem;
            let matches_birth = birth_stem.as_ref() == Some(target_stem);
            if !matches_primary && !matches_birth {
                continue;
            }
        }

        // 3. Birth year range filter (`born_from`, `born_to`)
        if let Some(from_yr) = query.born_from {
            match birth_year {
                Some(by) if by >= from_yr => {}
                _ => continue,
            }
        }
        if let Some(to_yr) = query.born_to {
            match birth_year {
                Some(by) if by <= to_yr => {}
                _ => continue,
            }
        }

        // 4. Sex filter
        if let Some(ref sf) = sex_filter {
            if !p.sex.eq_ignore_ascii_case(sf) {
                continue;
            }
        }

        // 5. Living / deceased filter
        if let Some(ref lf) = living_filter {
            match lf.as_str() {
                "living" if !p.is_living => continue,
                "deceased" if p.is_living => continue,
                _ => {}
            }
        }

        // 6. Has photo filter
        if let Some(want_photo) = query.has_photo {
            let has_avatar = p.avatar_url.as_ref().is_some_and(|u| !u.trim().is_empty());
            if has_avatar != want_photo {
                continue;
            }
        }

        // 7. Incomplete research filter (`!has_birth_date || !has_parents`) (`13 §4.2, §6 DoD #8`)
        if let Some(want_incomplete) = query.incomplete {
            let is_incomplete = !has_birth_date || !has_parents;
            if is_incomplete != want_incomplete {
                continue;
            }
        }

        let display_name = format_person_display_name(&p);
        let updated_at = updated_at_map.get(&p.id).cloned();

        filtered_items.push(PeopleDirectoryItemDto {
            id: p.id,
            gid: p.gid,
            given_name: p.primary_name.given_name,
            patronymic: p.primary_name.patronymic,
            surname: p.primary_name.surname,
            birth_surname: p.birth_surname,
            display_name,
            sex: p.sex,
            birth_date,
            birth_year,
            birth_place,
            death_date,
            death_year,
            death_place,
            avatar_url: p.avatar_url,
            is_living: p.is_living,
            is_private: p.is_private,
            has_parents,
            has_birth_date,
            version: p.version,
            updated_at,
        });
    }

    // Sorting (`name_asc`, `name_desc`, `birth_asc`, `birth_desc`, `death_asc`, `death_desc`, `updated_desc`)
    let sort_mode = query
        .sort
        .as_deref()
        .unwrap_or("name_asc")
        .trim()
        .to_ascii_lowercase();

    match sort_mode.as_str() {
        "name_desc" => {
            filtered_items.sort_by(|a, b| {
                b.display_name
                    .to_lowercase()
                    .cmp(&a.display_name.to_lowercase())
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
        "birth_asc" => {
            filtered_items.sort_by(|a, b| {
                a.birth_year
                    .unwrap_or(i32::MAX)
                    .cmp(&b.birth_year.unwrap_or(i32::MAX))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "birth_desc" => {
            filtered_items.sort_by(|a, b| {
                b.birth_year
                    .unwrap_or(i32::MIN)
                    .cmp(&a.birth_year.unwrap_or(i32::MIN))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "death_asc" => {
            filtered_items.sort_by(|a, b| {
                a.death_year
                    .unwrap_or(i32::MAX)
                    .cmp(&b.death_year.unwrap_or(i32::MAX))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "death_desc" => {
            filtered_items.sort_by(|a, b| {
                b.death_year
                    .unwrap_or(i32::MIN)
                    .cmp(&a.death_year.unwrap_or(i32::MIN))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "updated_desc" => {
            filtered_items.sort_by(|a, b| {
                b.updated_at
                    .as_deref()
                    .unwrap_or("")
                    .cmp(a.updated_at.as_deref().unwrap_or(""))
                    .then_with(|| b.version.cmp(&a.version))
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
        _ => {
            // Default `name_asc` (surname-first display_name)
            filtered_items.sort_by(|a, b| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
    }

    let total = filtered_items.len();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let page_items: Vec<PeopleDirectoryItemDto> = filtered_items
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let next_cursor = if offset + page_items.len() < total {
        Some((offset + page_items.len()).to_string())
    } else {
        None
    };

    Ok(Json(PeopleDirectoryResponse {
        items: page_items,
        next_cursor,
        total,
        total_unfiltered,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/surnames",
    params(SurnamesIndexQuery),
    responses(
        (status = 200, description = "Alphabetical surnames index with gender folding, counts, and year spans (authenticated users only)", body = SurnamesIndexResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_people_surnames(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<SurnamesIndexQuery>,
) -> Result<Json<SurnamesIndexResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let total_people = visible_persons.len();

    let min_count = query.min_count.unwrap_or(1).max(1);
    let letter_filter = query
        .letter
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase());

    // Group by gender-folded surname stem (`fold_slavic_surname_stem`)
    struct SurnameAgg {
        canonical: String,
        variants: Vec<String>,
        count: usize,
        earliest_year: Option<i32>,
        latest_year: Option<i32>,
    }

    let mut agg_map: HashMap<String, SurnameAgg> = HashMap::new();

    for p in &visible_persons {
        let raw_surname = p.primary_name.surname.trim();
        if raw_surname.is_empty() {
            continue;
        }
        let stem_key = fold_slavic_surname_stem(raw_surname);
        let canonical = canonicalize_surname_display(raw_surname);
        let entry = agg_map.entry(stem_key).or_insert_with(|| SurnameAgg {
            canonical,
            variants: Vec::new(),
            count: 0,
            earliest_year: None,
            latest_year: None,
        });

        entry.count += 1;
        if !entry
            .variants
            .iter()
            .any(|v| v.eq_ignore_ascii_case(raw_surname))
        {
            entry.variants.push(raw_surname.to_string());
            entry.variants.sort();
        }

        let by = p.birth_sort.map(|bs| bs / 10_000);
        let dy = p.death_sort.map(|ds| ds / 10_000);
        for yr in [by, dy].into_iter().flatten() {
            entry.earliest_year = Some(entry.earliest_year.map_or(yr, |cur| cur.min(yr)));
            entry.latest_year = Some(entry.latest_year.map_or(yr, |cur| cur.max(yr)));
        }
    }

    let mut entries: Vec<SurnameIndexEntryDto> = agg_map
        .into_values()
        .filter(|e| e.count >= min_count)
        .map(|e| SurnameIndexEntryDto {
            canonical_surname: e.canonical,
            variants: e.variants,
            count: e.count,
            earliest_year: e.earliest_year,
            latest_year: e.latest_year,
        })
        .collect();

    entries.sort_by(|a, b| {
        a.canonical_surname
            .to_lowercase()
            .cmp(&b.canonical_surname.to_lowercase())
    });

    let total_surnames = entries.len();

    let mut letter_groups: Vec<SurnameLetterGroupDto> = Vec::new();
    for item in entries {
        let first_letter = item
            .canonical_surname
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>())
            .unwrap_or_else(|| "#".to_string());

        if let Some(ref lf) = letter_filter {
            if &first_letter != lf {
                continue;
            }
        }

        if let Some(existing_group) = letter_groups.iter_mut().find(|g| g.letter == first_letter) {
            existing_group.items.push(item);
        } else {
            letter_groups.push(SurnameLetterGroupDto {
                letter: first_letter,
                items: vec![item],
            });
        }
    }

    Ok(Json(SurnamesIndexResponse {
        groups: letter_groups,
        total_surnames,
        total_people,
    }))
}
