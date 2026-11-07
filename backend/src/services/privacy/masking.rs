#[allow(unused_imports)]
use super::*;

/// Applies the RBAC & Living-Person Redaction Contract (`02-rbac-access-control.md §3.2, §3.3, §4.2`).
/// - Returns `None` if `person.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic`: masks living individuals (`given_name: "Living"` / `"Живущий"`,
///   `patronymic: None`, `birth: None`, `death: None`, `avatar_url: None`, `notes: None`, `biography: None`).
/// - For `Viewer`, `Editor`, and `Admin`: returns living individuals unredacted.
pub fn redact_person_for_caller(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
    locale: &str,
) -> Option<TreePersonNodeDto> {
    if person.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic && person.is_living {
        let masked_given = if locale.trim().eq_ignore_ascii_case("ru") {
            "Живущий".to_string()
        } else {
            "Living".to_string()
        };
        return Some(TreePersonNodeDto {
            id: person.id.clone(),
            gid: person.gid.clone(),
            sex: person.sex.clone(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: masked_given,
                patronymic: None,
                surname: person.primary_name.surname.clone(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
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
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: person.version,
            tx_id: None,
            generation: person.generation,
            is_direct_lineage: person.is_direct_lineage,
        });
    }

    Some(person.clone())
}

/// Applies the Tree Canvas Projection Living-Person Redaction Contract (`04-tree-visualizations.md §3.1`, `02-rbac-access-control.md §3.2, §4.2`).
/// - Returns `None` if `person.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic`: masks living individuals to a single given-name initial character with a dot
///   (e.g. `"И."` / `"J."`), strips surnames, patronymics, dates, places, biographies, notes, and avatars.
/// - For `Viewer`, `Editor`, and `Admin`: returns living individuals unredacted.
pub fn redact_person_for_tree_view(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
) -> Option<TreePersonNodeDto> {
    if person.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic && person.is_living {
        let initial = person
            .primary_name
            .given_name
            .trim()
            .chars()
            .next()
            .unwrap_or('?');
        return Some(TreePersonNodeDto {
            id: person.id.clone(),
            gid: person.gid.clone(),
            sex: person.sex.clone(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: format!("{initial}."),
                patronymic: None,
                surname: String::new(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
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
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: person.version,
            tx_id: None,
            generation: person.generation,
            is_direct_lineage: person.is_direct_lineage,
        });
    }

    Some(person.clone())
}

/// Redacts or filters a life event for tree and MCP projections (`02-rbac-access-control.md §3.2, §4.2`, `19-ai-family-story-intake-and-mcp.md §4.2.3`).
/// - Returns `None` if `event.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic` (which includes `Viewer` MCP tokens per `19 §4.2.3`):
///   - If the event belongs to an individual (`individual_id`), returns `None` when the individual is missing, private, or `is_living`.
///   - If the event belongs to a family (`family_id`), returns `None` when the family is missing, private, or has any living partner.
pub fn redact_event_for_tree_view(
    event: &LifeEventDto,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
    access: &CallerAccess,
) -> Option<LifeEventDto> {
    if event.is_private && !access.can_view_private() {
        return None;
    }

    if let Some(ref ind_id) = event.individual_id {
        let person = persons.iter().find(|p| &p.id == ind_id)?;
        redact_person_for_tree_view(person, access)?;
        if *access == CallerAccess::AnonymousPublic && person.is_living {
            return None;
        }
    }

    if let Some(ref fam_id) = event.family_id {
        let fam = families.iter().find(|f| &f.id == fam_id)?;
        if fam.is_private && !access.can_view_private() {
            return None;
        }
        if *access == CallerAccess::AnonymousPublic {
            let p1_living = fam
                .partner1_id
                .as_ref()
                .and_then(|pid| persons.iter().find(|p| &p.id == pid))
                .is_some_and(|p| p.is_living);
            let p2_living = fam
                .partner2_id
                .as_ref()
                .and_then(|pid| persons.iter().find(|p| &p.id == pid))
                .is_some_and(|p| p.is_living);
            if p1_living || p2_living {
                return None;
            }
        }
    }

    Some(event.clone())
}

/// Redacts a media asset for the given caller (`02-rbac-access-control.md §3.2, §7 DoD #3`).
pub fn redact_media_for_caller(
    media: &MediaAssetSummaryDto,
    access: &CallerAccess,
    store: &GenealogyPrivacyStore,
) -> Option<MediaAssetSummaryDto> {
    if media.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic {
        if media.only_living_persons {
            return None;
        }
        let mut cloned = media.clone();
        cloned.face_tags.retain(|ft| !ft.is_living);

        let is_living_person = |pid: &str| -> bool {
            store
                .persons
                .iter()
                .find(|p| p.id == pid || p.gid.eq_ignore_ascii_case(pid))
                .map(|p| p.is_living)
                .unwrap_or(true)
        };

        cloned
            .linked_person_ids
            .retain(|pid| !is_living_person(pid));

        cloned.links.retain(|l| {
            if l.entity_type == "individual" {
                !is_living_person(&l.entity_id)
            } else {
                true
            }
        });

        return Some(cloned);
    }

    Some(media.clone())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DismissedWarningDto {
    pub id: String,
    pub rule_code: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed_by: Option<String>,
    pub dismissed_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftPersonDto {
    pub temp_id: String,
    pub given_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patronymic: Option<String>,
    #[serde(default)]
    pub surname: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maiden_name: Option<String>,
    pub sex: String,
    #[serde(default, alias = "role_to_speaker", alias = "role")]
    pub relation_to_speaker: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "birth_date")]
    pub birth_date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_place: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "death_date")]
    pub death_date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_place: Option<String>,
    #[serde(default = "default_true_bool")]
    pub is_living: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "notes")]
    pub biography_notes: Option<String>,
    #[serde(default, alias = "confidence_score")]
    pub confidence: u8,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "source_snippet"
    )]
    pub grounding_quote: Option<String>,
}

pub(crate) fn default_true_bool() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftUnionDto {
    pub temp_id: String,
    pub partner1_temp_id: String,
    #[serde(default)]
    pub partner2_temp_id: String,
    #[serde(default, alias = "relationship_type")]
    pub union_type: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "marriage_date"
    )]
    pub marriage_date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marriage_place: Option<String>,
    #[serde(default, alias = "confidence_score")]
    pub confidence: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftParentChildDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub union_temp_id: Option<String>,
    pub parent_temp_id: String,
    pub child_temp_id: String,
    #[serde(default = "default_pedigree_birth", alias = "relation_type")]
    pub pedigree: String,
}

pub(crate) fn default_pedigree_birth() -> String {
    "birth".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftEventDto {
    pub temp_id: String,
    pub person_temp_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "place_name")]
    pub place_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "source_snippet"
    )]
    pub grounding_quote: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftExtractedDataDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_temp_id: Option<String>,
    #[serde(default, alias = "draft_people", alias = "people")]
    pub persons: Vec<DraftPersonDto>,
    #[serde(default, alias = "draft_unions")]
    pub unions: Vec<DraftUnionDto>,
    #[serde(default, alias = "draft_parent_child", alias = "parent_child")]
    pub parent_child_links: Vec<DraftParentChildDto>,
    #[serde(default, alias = "draft_events")]
    pub events: Vec<DraftEventDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftMatchCandidateDto {
    pub person_id: String,
    pub person_gid: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_place: Option<String>,
    pub confidence: u8,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DraftReconciliationMatchDto {
    pub draft_temp_id: String,
    pub recommended_action: String,
    pub selected_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_person_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_person_gid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_person_name: Option<String>,
    pub confidence: u8,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub candidates: Vec<DraftMatchCandidateDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct StoryIntakeDraftDto {
    pub id: String,
    pub created_by: String,
    pub title: String,
    pub raw_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_media_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_person_id: Option<String>,
    #[serde(default)]
    pub is_new_speaker: bool,
    pub status: String,
    pub extracted_data: DraftExtractedDataDto,
    #[serde(default)]
    pub reconciliation_matches: Vec<DraftReconciliationMatchDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_tx_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PersonalAccessTokenDto {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token_prefix: String,
    #[serde(skip_serializing, default)]
    pub token_hash: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AiInstanceConfigDto {
    pub llm_provider: String,
    pub llm_base_url: String,
    pub llm_model: String,
    #[serde(skip_serializing, default)]
    pub llm_api_key: Option<String>,
    pub has_api_key: bool,
    pub stt_provider: String,
    pub stt_model: String,
    pub mcp_enabled: bool,
}

impl Default for AiInstanceConfigDto {
    fn default() -> Self {
        Self {
            llm_provider: "deterministic".to_string(),
            llm_base_url: "http://localhost:11434/v1".to_string(),
            llm_model: "llama3.1:8b".to_string(),
            llm_api_key: None,
            has_api_key: false,
            stt_provider: "whisper-local".to_string(),
            stt_model: "whisper-large-v3-turbo".to_string(),
            mcp_enabled: true,
        }
    }
}
