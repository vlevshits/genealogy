#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct FanChartQuery {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub focal_id: Option<String>,
    #[serde(default)]
    pub generations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FanChartNodeDto {
    pub ahnentafel: u32,
    pub generation: u32,
    pub quadrant: String,
    pub person: TreePersonNodeDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub father_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mother_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FanChartResponse {
    pub root_id: String,
    pub generations: u32,
    pub total_nodes: usize,
    pub nodes: Vec<FanChartNodeDto>,
}

pub(crate) fn ahnentafel_quadrant(n: u32) -> String {
    if n == 1 {
        return "root".to_string();
    }
    if n == 2 {
        return "paternal".to_string();
    }
    if n == 3 {
        return "maternal".to_string();
    }
    // Walk down to the Gen-3 ancestor (4..=7) by dividing by 2
    let mut curr = n;
    while curr > 7 {
        curr /= 2;
    }
    match curr {
        4 => "paternal_grandfather".to_string(),
        5 => "paternal_grandmother".to_string(),
        6 => "maternal_grandfather".to_string(),
        7 => "maternal_grandmother".to_string(),
        _ => "root".to_string(),
    }
}

/// Computes the Ahnentafel-indexed Fan Chart hierarchy up to `generations` (4..=8, default 5) (`04-tree-visualizations.md §4.3`).
pub fn compute_fan_chart(
    root_id: &str,
    generations: u32,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
) -> FanChartResponse {
    let generations = generations.clamp(1, 8);
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut nodes = Vec::new();
    let mut visited_in_branch: HashSet<(u32, String)> = HashSet::new();
    let mut queue: VecDeque<(u32, u32, String)> = VecDeque::new();

    if person_by_id.contains_key(root_id) {
        queue.push_back((1, 1, root_id.to_string()));
        visited_in_branch.insert((1, root_id.to_string()));
    }

    while let Some((ahnentafel, gen_num, pid)) = queue.pop_front() {
        let Some(&person) = person_by_id.get(pid.as_str()) else {
            continue;
        };

        // Find parents of `pid`
        let mut father_id: Option<String> = None;
        let mut mother_id: Option<String> = None;

        if let Some(parent_fam) = families
            .iter()
            .find(|f| f.children.iter().any(|c| c.child_id == pid))
        {
            let p1 = parent_fam
                .partner1_id
                .as_deref()
                .and_then(|id| person_by_id.get(id).copied());
            let p2 = parent_fam
                .partner2_id
                .as_deref()
                .and_then(|id| person_by_id.get(id).copied());

            match (p1, p2) {
                (Some(a), Some(b)) => {
                    if a.sex.eq_ignore_ascii_case("F") || b.sex.eq_ignore_ascii_case("M") {
                        father_id = Some(b.id.clone());
                        mother_id = Some(a.id.clone());
                    } else {
                        father_id = Some(a.id.clone());
                        mother_id = Some(b.id.clone());
                    }
                }
                (Some(only), None) | (None, Some(only)) => {
                    if only.sex.eq_ignore_ascii_case("F") {
                        mother_id = Some(only.id.clone());
                    } else {
                        father_id = Some(only.id.clone());
                    }
                }
                (None, None) => {}
            }
        }

        if gen_num < generations {
            if let Some(ref fid) = father_id {
                if fid != &pid {
                    queue.push_back((ahnentafel * 2, gen_num + 1, fid.clone()));
                }
            }
            if let Some(ref mid) = mother_id {
                if mid != &pid {
                    queue.push_back((ahnentafel * 2 + 1, gen_num + 1, mid.clone()));
                }
            }
        }

        nodes.push(FanChartNodeDto {
            ahnentafel,
            generation: gen_num,
            quadrant: ahnentafel_quadrant(ahnentafel),
            person: person.clone(),
            father_id,
            mother_id,
        });
    }

    nodes.sort_by_key(|n| n.ahnentafel);
    let total_nodes = nodes.len();

    FanChartResponse {
        root_id: root_id.to_string(),
        generations,
        total_nodes,
        nodes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceMapMarkerPersonDto {
    pub id: String,
    pub gid: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceMapMarkerDto {
    pub id: String,
    pub gid: String,
    pub name: String,
    pub full_name: String,
    pub place_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub events_count: usize,
    pub person_count: usize,
    pub event_types: Vec<String>,
    pub persons: Vec<PlaceMapMarkerPersonDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlacesMapResponse {
    pub items: Vec<PlaceMapMarkerDto>,
    pub total: usize,
}

/// Builds the clustered Places Map markers response (`GET /api/v1/places/map`, `12-places.md`).
pub fn compute_places_map(
    store: &GenealogyPrivacyStore,
    visible_persons: &[TreePersonNodeDto],
) -> PlacesMapResponse {
    let visible_by_id: HashMap<&str, &TreePersonNodeDto> =
        visible_persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut items = Vec::new();

    for place in &store.places {
        let (Some(lat), Some(lng)) = (place.latitude, place.longitude) else {
            continue;
        };

        let mut person_ids: HashSet<String> = HashSet::new();
        let mut event_types_set: HashSet<String> = HashSet::new();
        let mut matched_events_count = 0usize;

        for ev in &store.events {
            let matches_place = ev.place_id.as_deref() == Some(place.id.as_str())
                || ev
                    .place_full_name
                    .as_deref()
                    .map(|pn| {
                        pn.eq_ignore_ascii_case(&place.name)
                            || pn.eq_ignore_ascii_case(&place.full_name)
                    })
                    .unwrap_or(false);
            if matches_place {
                matched_events_count += 1;
                event_types_set.insert(ev.event_type.clone());
                if let Some(pid) = ev.individual_id.as_deref() {
                    if visible_by_id.contains_key(pid) {
                        person_ids.insert(pid.to_string());
                    }
                }
            }
        }

        // Also match person birth/death summary places by name/full_name
        for vp in visible_persons {
            if let Some(bp) = vp.birth.as_ref().and_then(|b| b.place.as_deref()) {
                if bp.eq_ignore_ascii_case(&place.name)
                    || bp.eq_ignore_ascii_case(&place.full_name)
                    || bp.contains(&place.name)
                {
                    person_ids.insert(vp.id.clone());
                    event_types_set.insert("birth".to_string());
                }
            }
            if let Some(dp) = vp.death.as_ref().and_then(|d| d.place.as_deref()) {
                if dp.eq_ignore_ascii_case(&place.name)
                    || dp.eq_ignore_ascii_case(&place.full_name)
                    || dp.contains(&place.name)
                {
                    person_ids.insert(vp.id.clone());
                    event_types_set.insert("death".to_string());
                }
            }
        }

        let events_count = matched_events_count.max(place.events_count as usize);
        let mut persons_list: Vec<PlaceMapMarkerPersonDto> = person_ids
            .iter()
            .filter_map(|pid| visible_by_id.get(pid.as_str()))
            .map(|p| {
                let display_name = [
                    p.primary_name.given_name.trim(),
                    p.primary_name.patronymic.as_deref().unwrap_or("").trim(),
                    p.primary_name.surname.trim(),
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
                PlaceMapMarkerPersonDto {
                    id: p.id.clone(),
                    gid: p.gid.clone(),
                    display_name,
                }
            })
            .collect();
        persons_list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        let person_count = persons_list.len();
        persons_list.truncate(10);

        let mut event_types: Vec<String> = event_types_set.into_iter().collect();
        event_types.sort();

        items.push(PlaceMapMarkerDto {
            id: place.id.clone(),
            gid: place.gid.clone(),
            name: place.name.clone(),
            full_name: place.full_name.clone(),
            place_type: place.place_type.clone(),
            latitude: lat,
            longitude: lng,
            events_count,
            person_count,
            event_types,
            persons: persons_list,
        });
    }

    let total = items.len();
    PlacesMapResponse { items, total }
}
