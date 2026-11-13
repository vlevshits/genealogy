#[allow(unused_imports)]
use super::*;
use crate::services::privacy::default_child_rel;

/// Response of `PATCH /families/{id}`: the resulting family plus the history
/// transaction id usable with `POST /history/tx/{tx_id}/undo`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FamilyMutationResponse {
    #[serde(flatten)]
    pub family: FamilyDto,
    pub tx_id: String,
    /// `true` when the update left an empty single-parent union which was dissolved.
    #[serde(default)]
    pub dissolved: bool,
}

/// Request body of `POST /persons/{id}/move-to-family`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct MoveToFamilyRequest {
    /// Existing target family (UUID or GID). Mutually exclusive with partner ids.
    pub family_id: Option<String>,
    /// Target partners used to find (or create) a family when `family_id` is omitted.
    pub partner1_id: Option<String>,
    pub partner2_id: Option<String>,
    /// Family the child is removed from. Optional when the person has at most one parent family.
    pub from_family_id: Option<String>,
    /// Pedigree relation to partner 1 / 2 (`birth` by default).
    pub rel_to_partner1: Option<String>,
    pub rel_to_partner2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MoveToFamilyResponse {
    pub person_id: String,
    pub target_family: FamilyDto,
    /// `true` when the target family was created by this request.
    pub target_family_created: bool,
    pub source_family_id: Option<String>,
    /// Remaining source family; `null` when none or when it was dissolved.
    pub source_family: Option<FamilyDto>,
    pub source_family_dissolved: bool,
    pub tx_id: String,
}

fn validation_error(field: &str, code: &str, msg: &str) -> AppError {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), code.to_string());
    AppError::ValidationFailed(msg.to_string(), Some(fields))
}

/// A union is orphaned when it has no children and at most one partner.
pub(crate) fn is_orphaned_family(fam: &FamilyDto) -> bool {
    fam.children.is_empty() && (fam.partner1_id.is_none() || fam.partner2_id.is_none())
}

/// Removes a family and its events, logging every deletion in transaction `tx_id`.
/// `before` is the snapshot written to the family's `delete` entry (so undo restores it).
pub(crate) fn dissolve_family(
    store: &mut GenealogyPrivacyStore,
    fam_id: &str,
    tx_id: &str,
    tx_label: &str,
    before: Option<serde_json::Value>,
    actor: (Option<String>, Option<String>),
) -> Option<FamilyDto> {
    let pos = store.families.iter().position(|f| f.id == fam_id)?;
    let fam = store.families.remove(pos);
    let (removed, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut store.events)
        .into_iter()
        .partition(|e| e.family_id.as_deref() == Some(fam.id.as_str()));
    store.events = kept;
    for ev in &removed {
        log_change(
            store,
            Some(tx_id.to_string()),
            tx_label,
            "event",
            &ev.id,
            "delete",
            Some(format!("{} ({})", ev.event_type, ev.gid)),
            format!("Deleted {} event {}", ev.event_type, ev.gid),
            serde_json::to_value(ev).ok(),
            None,
            actor.0.clone(),
            actor.1.clone(),
        );
    }
    log_change(
        store,
        Some(tx_id.to_string()),
        tx_label,
        "family",
        &fam.id,
        "delete",
        Some(fam.gid.clone()),
        format!("Dissolved family union {}", fam.gid),
        before.or_else(|| serde_json::to_value(&fam).ok()),
        None,
        actor.0,
        actor.1,
    );
    recompute_all_places_metadata(store);
    Some(fam)
}

/// Union is *childless* (no children, any number of partners). Used by marriage-event
/// removal (`delete_event`): the marriage fact is what made a childless union exist, so
/// removing it dissolves the union even with two partners. Intentionally different from
/// [`is_orphaned_family`] (no children AND <= 1 partner), which governs unlinking.
pub(crate) fn is_childless_union(fam: &FamilyDto) -> bool {
    fam.children.is_empty()
}

/// Keeps partner slots canonical after one was cleared: a lone partner always sits in
/// slot 1 and every child's relation is remapped accordingly (`rel_to_partner1 :=
/// old rel_to_partner2`), then the unused `rel_to_partner2` is reset to the default.
pub(crate) fn normalize_partner_slots(fam: &mut FamilyDto) {
    if fam.partner1_id.is_none() && fam.partner2_id.is_some() {
        fam.partner1_id = fam.partner2_id.take();
        for ch in &mut fam.children {
            ch.rel_to_partner1 = std::mem::replace(&mut ch.rel_to_partner2, default_child_rel());
        }
    }
    if fam.partner2_id.is_none() {
        for ch in &mut fam.children {
            ch.rel_to_partner2 = default_child_rel();
        }
    }
}

/// Persists an edited family inside transaction `tx_id`: either dissolves it (logging the
/// pre-edit snapshot so undo restores it) or logs a regular `update` entry.
#[allow(clippy::too_many_arguments)]
pub(crate) fn commit_family_edit(
    store: &mut GenealogyPrivacyStore,
    before: Option<serde_json::Value>,
    updated: &FamilyDto,
    dissolve: bool,
    tx_id: &str,
    tx_label: &str,
    summary: String,
    actor: (Option<String>, Option<String>),
) {
    if dissolve {
        dissolve_family(store, &updated.id, tx_id, tx_label, before, actor);
    } else {
        log_change(
            store,
            Some(tx_id.to_string()),
            tx_label,
            "family",
            &updated.id,
            "update",
            Some(updated.gid.clone()),
            summary,
            before,
            serde_json::to_value(updated).ok(),
            actor.0,
            actor.1,
        );
    }
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
pub struct FamilyDetachQuery {
    /// Optional optimistic-lock version of the family (`If-Match` is also accepted).
    pub version: Option<i32>,
}

enum Detach {
    Child,
    Partner,
}

async fn detach_from_family(
    actor: (Option<String>, Option<String>),
    state: &AppState,
    id: &str,
    person_ref: &str,
    version: Option<i32>,
    kind: Detach,
) -> Result<Json<FamilyMutationResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let fam_idx = store
        .families
        .iter()
        .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;
    verify_optimistic_version(
        store.families[fam_idx].version,
        version,
        &format!("Family {}", store.families[fam_idx].gid),
        serde_json::to_value(&store.families[fam_idx]).unwrap_or_default(),
    )?;
    let person_id =
        resolve_person_id_in_store(&store.persons, person_ref).ok_or(AppError::PersonNotFound)?;

    let before = serde_json::to_value(&store.families[fam_idx]).ok();
    let fam = &mut store.families[fam_idx];
    let summary = match kind {
        Detach::Child => {
            if !fam.children.iter().any(|c| c.child_id == person_id) {
                return Err(AppError::ChildNotFound);
            }
            fam.children.retain(|c| c.child_id != person_id);
            format!("Detached child from family union {}", fam.gid)
        }
        Detach::Partner => {
            if fam.partner1_id.as_deref() == Some(person_id.as_str()) {
                fam.partner1_id = None;
            } else if fam.partner2_id.as_deref() == Some(person_id.as_str()) {
                fam.partner2_id = None;
            } else {
                return Err(AppError::PartnerNotFound);
            }
            normalize_partner_slots(fam);
            format!("Detached partner from family union {}", fam.gid)
        }
    };
    fam.version += 1;
    let updated = fam.clone();

    let tx_id = Uuid::new_v4().to_string();
    let dissolved = is_orphaned_family(&updated);
    commit_family_edit(
        &mut store,
        before,
        &updated,
        dissolved,
        &tx_id,
        "family.detach",
        summary,
        actor,
    );
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(Json(FamilyMutationResponse {
        family: updated,
        tx_id,
        dissolved,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/families/{id}/children/{child_id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID"),
        ("child_id" = String, Path, description = "Child person UUID or GID"),
        FamilyDetachQuery
    ),
    responses(
        (status = 200, description = "Child detached; family dissolved when left with no children and <= 1 partner", body = FamilyMutationResponse),
        (status = 404, description = "Family or person not found, or `child_not_found`", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn detach_family_child(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, child_id)): Path<(String, String)>,
    Query(q): Query<FamilyDetachQuery>,
    headers: HeaderMap,
) -> Result<Json<FamilyMutationResponse>, AppError> {
    let version = q.version.or_else(|| parse_if_match_version(&headers));
    let actor = (
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    detach_from_family(actor, &state, &id, &child_id, version, Detach::Child).await
}

#[utoipa::path(
    delete,
    path = "/api/v1/families/{id}/partners/{person_id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID"),
        ("person_id" = String, Path, description = "Partner person UUID or GID"),
        FamilyDetachQuery
    ),
    responses(
        (status = 200, description = "Partner detached (remaining partner promoted to slot 1 with child relations remapped); family dissolved when childless", body = FamilyMutationResponse),
        (status = 404, description = "Family or person not found, or `partner_not_found`", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn detach_family_partner(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, person_id)): Path<(String, String)>,
    Query(q): Query<FamilyDetachQuery>,
    headers: HeaderMap,
) -> Result<Json<FamilyMutationResponse>, AppError> {
    let version = q.version.or_else(|| parse_if_match_version(&headers));
    let actor = (
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    detach_from_family(actor, &state, &id, &person_id, version, Detach::Partner).await
}

fn same_partner_set(fam: &FamilyDto, p1: &Option<String>, p2: &Option<String>) -> bool {
    (fam.partner1_id == *p1 && fam.partner2_id == *p2)
        || (fam.partner1_id == *p2 && fam.partner2_id == *p1)
}

#[utoipa::path(
    post,
    path = "/api/v1/persons/{id}/move-to-family",
    params(
        ("id" = String, Path, description = "Child person UUID or GID")
    ),
    request_body = MoveToFamilyRequest,
    responses(
        (status = 200, description = "Child moved atomically; `tx_id` can be undone via /history/tx/{tx_id}/undo", body = MoveToFamilyResponse),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse),
        (status = 403, description = "Editor role required", body = ErrorResponse),
        (status = 404, description = "Person or family not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn move_person_to_family(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MoveToFamilyRequest>,
) -> Result<Json<MoveToFamilyResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let actor = (
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    let mut store = state.privacy_store.write().await;
    let child_id = resolve_person_id_in_store(&store.persons, &id).ok_or(AppError::NotFound)?;

    let opt_trim = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let find_family = |store: &GenealogyPrivacyStore, raw: &str| {
        store
            .families
            .iter()
            .position(|f| f.id == raw || f.gid.eq_ignore_ascii_case(raw))
    };

    // Resolve target partners / family.
    let target_family_ref = opt_trim(&req.family_id);
    let mut p1 = None;
    let mut p2 = None;
    for (raw, slot) in [
        (opt_trim(&req.partner1_id), &mut p1),
        (opt_trim(&req.partner2_id), &mut p2),
    ] {
        if let Some(raw) = raw {
            *slot =
                Some(resolve_person_id_in_store(&store.persons, &raw).ok_or(AppError::NotFound)?);
        }
    }
    let target_idx = match target_family_ref {
        Some(raw) => Some(find_family(&store, &raw).ok_or(AppError::NotFound)?),
        None => {
            if p1.is_none() && p2.is_none() {
                return Err(validation_error(
                    "family_id",
                    "target_required",
                    "family_id or at least one partner id is required",
                ));
            }
            if p1.is_some() && p1 == p2 {
                return Err(validation_error(
                    "partner2_id",
                    "duplicate_partner",
                    "partner1_id and partner2_id must be distinct individuals",
                ));
            }
            store
                .families
                .iter()
                .position(|f| same_partner_set(f, &p1, &p2))
        }
    };
    let (t_p1, t_p2) = match target_idx {
        Some(i) => (
            store.families[i].partner1_id.clone(),
            store.families[i].partner2_id.clone(),
        ),
        None => (p1.clone(), p2.clone()),
    };

    // Child cannot be a partner nor an ancestor of a partner.
    for parent in [t_p1.as_deref(), t_p2.as_deref()].into_iter().flatten() {
        ensure_no_ancestor_cycle(&store.families, parent, &child_id)?;
    }

    // Resolve source family.
    let source_idx = match opt_trim(&req.from_family_id) {
        Some(raw) => {
            let i = find_family(&store, &raw).ok_or(AppError::NotFound)?;
            if !store.families[i]
                .children
                .iter()
                .any(|c| c.child_id == child_id)
            {
                return Err(validation_error(
                    "from_family_id",
                    "not_a_child_of_family",
                    "person is not a child of from_family_id",
                ));
            }
            Some(i)
        }
        None => {
            let candidates: Vec<usize> = store
                .families
                .iter()
                .enumerate()
                .filter(|(i, f)| {
                    Some(*i) != target_idx && f.children.iter().any(|c| c.child_id == child_id)
                })
                .map(|(i, _)| i)
                .collect();
            match candidates.as_slice() {
                [] => None,
                [one] => Some(*one),
                _ => {
                    return Err(validation_error(
                        "from_family_id",
                        "ambiguous_source_family",
                        "person belongs to several parent families; specify from_family_id",
                    ))
                }
            }
        }
    };
    if source_idx.is_some() && source_idx == target_idx {
        return Err(validation_error(
            "family_id",
            "same_family",
            "person is already a child of the target family",
        ));
    }
    if let Some(ti) = target_idx {
        if store.families[ti]
            .children
            .iter()
            .any(|c| c.child_id == child_id)
        {
            return Err(validation_error(
                "family_id",
                "already_child_of_family",
                "person is already a child of the target family",
            ));
        }
    }

    let tx_id = Uuid::new_v4().to_string();
    let tx_label = "person.move_to_family";
    let source_id = source_idx.map(|i| store.families[i].id.clone());
    let target_fam_id = target_idx.map(|i| store.families[i].id.clone());

    // 1. Remove from the source family (dissolving it when orphaned).
    let mut source_family = None;
    let mut source_dissolved = false;
    if let Some(si) = source_idx {
        let before = serde_json::to_value(&store.families[si]).ok();
        let fam = &mut store.families[si];
        fam.children.retain(|c| c.child_id != child_id);
        fam.version += 1;
        let updated = fam.clone();
        if is_orphaned_family(&updated) {
            dissolve_family(
                &mut store,
                &updated.id,
                &tx_id,
                tx_label,
                before,
                actor.clone(),
            );
            source_dissolved = true;
        } else {
            log_change(
                &mut store,
                Some(tx_id.clone()),
                tx_label,
                "family",
                &updated.id,
                "update",
                Some(updated.gid.clone()),
                format!("Removed child from family union {}", updated.gid),
                before,
                serde_json::to_value(&updated).ok(),
                actor.0.clone(),
                actor.1.clone(),
            );
            source_family = Some(updated);
        }
    }

    // 2. Add to the target family (creating it if needed).
    let rel1 = normalize_pedigree_type(req.rel_to_partner1.as_deref().unwrap_or("birth"));
    let rel2 = normalize_pedigree_type(req.rel_to_partner2.as_deref().unwrap_or("birth"));
    let mut created = false;
    let target_pos = match target_fam_id.as_deref() {
        Some(tid) => store.families.iter().position(|f| f.id == tid),
        None => None,
    };
    let target_family = match target_pos {
        Some(ti) => {
            let before = serde_json::to_value(&store.families[ti]).ok();
            let fam = &mut store.families[ti];
            let next_order = fam
                .children
                .iter()
                .map(|c| c.sort_order)
                .max()
                .map_or(0, |m| m + 1);
            fam.children.push(FamilyChildDto {
                child_id: child_id.clone(),
                rel_to_partner1: rel1,
                rel_to_partner2: rel2,
                sort_order: next_order,
            });
            fam.version += 1;
            let updated = fam.clone();
            log_change(
                &mut store,
                Some(tx_id.clone()),
                tx_label,
                "family",
                &updated.id,
                "update",
                Some(updated.gid.clone()),
                format!("Added child to family union {}", updated.gid),
                before,
                serde_json::to_value(&updated).ok(),
                actor.0.clone(),
                actor.1.clone(),
            );
            updated
        }
        None => {
            created = true;
            let existing_gids: HashSet<String> =
                store.families.iter().map(|f| f.gid.clone()).collect();
            let gid = allocate_gid('F', &mut store.next_family_gid_seq, None, &existing_gids);
            let fam = FamilyDto {
                id: Uuid::new_v4().to_string(),
                gid,
                partner1_id: p1.clone(),
                partner2_id: p2.clone(),
                relationship_type: "unknown".to_string(),
                is_private: false,
                children: vec![FamilyChildDto {
                    child_id: child_id.clone(),
                    rel_to_partner1: rel1,
                    rel_to_partner2: rel2,
                    sort_order: 0,
                }],
                external_ids: HashMap::new(),
                version: 1,
                marriage_date: None,
                marriage_place: None,
            };
            store.families.push(fam.clone());
            log_change(
                &mut store,
                Some(tx_id.clone()),
                tx_label,
                "family",
                &fam.id,
                "create",
                Some(fam.gid.clone()),
                format!("Created family union {}", fam.gid),
                None,
                serde_json::to_value(&fam).ok(),
                actor.0.clone(),
                actor.1.clone(),
            );
            fam
        }
    };

    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(Json(MoveToFamilyResponse {
        person_id: child_id,
        target_family,
        target_family_created: created,
        source_family_id: source_id,
        source_family,
        source_family_dissolved: source_dissolved,
        tx_id,
    }))
}
