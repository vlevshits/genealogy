#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GenealogyPrivacyStore {
    pub persons: Vec<TreePersonNodeDto>,
    pub families: Vec<FamilyDto>,
    pub events: Vec<LifeEventDto>,
    pub notes: Vec<NoteDto>,
    pub media: Vec<MediaAssetSummaryDto>,
    pub places: Vec<PlaceSummaryDto>,
    pub change_log: Vec<ChangeLogEntryDto>,
    pub snapshots: Vec<TreeSnapshotDto>,
    pub dismissed_warnings: Vec<DismissedWarningDto>,
    pub not_duplicate_pairs: Vec<(String, String)>,
    pub gid_aliases: HashMap<String, String>,
    pub story_drafts: Vec<StoryIntakeDraftDto>,
    pub personal_access_tokens: Vec<PersonalAccessTokenDto>,
    pub ai_config: AiInstanceConfigDto,
    pub next_gid_seq: u32,
    pub next_family_gid_seq: u32,
    pub next_event_gid_seq: u32,
    pub next_place_gid_seq: u32,
    pub next_media_gid_seq: u32,
    pub is_demo_seed: bool,
}

impl GenealogyPrivacyStore {
    pub fn empty() -> Self {
        Self {
            persons: vec![],
            families: vec![],
            events: vec![],
            notes: vec![],
            media: vec![],
            places: vec![],
            change_log: vec![],
            snapshots: vec![],
            dismissed_warnings: vec![],
            not_duplicate_pairs: vec![],
            gid_aliases: HashMap::new(),
            story_drafts: vec![],
            personal_access_tokens: vec![],
            ai_config: AiInstanceConfigDto::default(),
            next_gid_seq: 100,
            next_family_gid_seq: 100,
            next_event_gid_seq: 100,
            next_place_gid_seq: 100,
            next_media_gid_seq: 100,
            is_demo_seed: false,
        }
    }

    /// Persists the live store to `{BACKUP_PATH}/live-store.json` when running as the server binary (`BACKUP_PATH` set).
    pub fn persist_if_production(&self) {
        if cfg!(test) || std::env::var("APP_ENV").as_deref() == Ok("test") {
            return;
        }
        let Ok(backup_dir) = std::env::var("BACKUP_PATH") else {
            return;
        };
        if backup_dir.trim().is_empty() {
            return;
        }
        if self.is_demo_seed || (self.persons.is_empty() && self.personal_access_tokens.is_empty())
        {
            return;
        }
        let dir_path = std::path::Path::new(&backup_dir);
        let _ = std::fs::create_dir_all(dir_path);
        let tmp_path = dir_path.join("live-store.json.tmp");
        let final_path = dir_path.join("live-store.json");
        if let Ok(bytes) = serde_json::to_vec(self) {
            if std::fs::write(&tmp_path, bytes).is_ok() {
                let _ = std::fs::rename(&tmp_path, &final_path);
            }
        }
    }

    /// Resolves a person UUID or GID (including merged losing GIDs via `gid_aliases`) to the canonical person UUID.
    pub fn resolve_person_id(&self, id_or_gid: &str) -> Option<String> {
        let trimmed = id_or_gid.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(p) = self
            .persons
            .iter()
            .find(|p| p.id == trimmed || p.gid.eq_ignore_ascii_case(trimmed))
        {
            return Some(p.id.clone());
        }
        for (alias_gid, target_id) in &self.gid_aliases {
            if alias_gid.eq_ignore_ascii_case(trimmed)
                && self.persons.iter().any(|p| p.id == *target_id)
            {
                return Some(target_id.clone());
            }
        }
        None
    }

    /// Resolves a place UUID or GID (including merged losing GIDs via `gid_aliases`) to the canonical place UUID.
    pub fn resolve_place_id(&self, id_or_gid: &str) -> Option<String> {
        let trimmed = id_or_gid.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(p) = self
            .places
            .iter()
            .find(|p| p.id == trimmed || p.gid.eq_ignore_ascii_case(trimmed))
        {
            return Some(p.id.clone());
        }
        for (alias_gid, target_id) in &self.gid_aliases {
            if alias_gid.eq_ignore_ascii_case(trimmed)
                && self.places.iter().any(|p| p.id == *target_id)
            {
                return Some(target_id.clone());
            }
        }
        None
    }

    pub fn demo() -> Self {
        let deceased_ancestor = TreePersonNodeDto {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            gid: "I0001".to_string(),
            sex: "M".to_string(),
            is_living: false,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: "Mikhail".to_string(),
                patronymic: Some("Isaakovich".to_string()),
                surname: "Levshits".to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: Some("Archival engineer and family chronicler.".to_string()),
            birth: Some(VitalEventSummaryDto {
                date: Some("1912-03-19".to_string()),
                place: Some("Vitebsk, Belarus".to_string()),
            }),
            death: Some(VitalEventSummaryDto {
                date: Some("1988-11-04".to_string()),
                place: Some("Leningrad, USSR".to_string()),
            }),
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000101/file".to_string()),
            avatar_media_id: Some("00000000-0000-0000-0000-000000000101".to_string()),
            avatar_crop: None,
            notes: Some("Archival engineer and family chronicler.".to_string()),
            birth_sort: Some(19120319),
            death_sort: Some(19881104),
            has_death_event: true,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
            generation: None,
            is_direct_lineage: None,
        };

        let living_relative = TreePersonNodeDto {
            id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
            gid: "I0042".to_string(),
            sex: "F".to_string(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: "Elena".to_string(),
                patronymic: Some("Alexandrovna".to_string()),
                surname: "Levshits".to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: Some("Graduated from Saint Petersburg State University.".to_string()),
            birth: Some(VitalEventSummaryDto {
                date: Some("1994-06-14".to_string()),
                place: Some("Saint Petersburg, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string()),
            avatar_media_id: Some("00000000-0000-0000-0000-000000000102".to_string()),
            avatar_crop: None,
            notes: Some("Graduated from Saint Petersburg State University.".to_string()),
            birth_sort: Some(19940614),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
            generation: None,
            is_direct_lineage: None,
        };

        let private_record = TreePersonNodeDto {
            id: "00000000-0000-0000-0000-000000000099".to_string(),
            gid: "I0099".to_string(),
            sex: "M".to_string(),
            is_living: true,
            living_override: Some(true),
            is_private: true,
            primary_name: PersonNameDto {
                given_name: "Arkady".to_string(),
                patronymic: Some("Borisovich".to_string()),
                surname: "Levshits".to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: Some(VitalEventSummaryDto {
                date: Some("1989-11-02".to_string()),
                place: Some("Moscow, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string()),
            avatar_media_id: Some("00000000-0000-0000-0000-000000000103".to_string()),
            avatar_crop: None,
            notes: Some("Confidential adoption research note.".to_string()),
            birth_sort: Some(19891102),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
            generation: None,
            is_direct_lineage: None,
        };

        let deceased_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000101".to_string(),
            gid: "M0001".to_string(),
            title: "Mikhail Levshits Archival Portrait (1955)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000101/file".to_string(),
            file_name: Some("m0001.webp".to_string()),
            original_file_name: Some("mikhail_1955.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: Some(800),
            height: Some(1000),
            description: Some("Archival portrait taken in Leningrad, 1955.".to_string()),
            date_taken: Some("1955".to_string()),
            date_sort: Some(19550000),
            place_id: Some("00000000-0000-0000-0000-000000000201".to_string()),
            place_full_name: Some("Saint Petersburg, Russia".to_string()),
            is_missing: false,
            external_path: None,
            is_private: false,
            only_living_persons: false,
            linked_person_ids: vec![
                "00000000-0000-0000-0000-000000000001".to_string(),
                "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
            ],
            face_tags: vec![
                FaceTagDto {
                    id: "ft-01".to_string(),
                    person_id: "00000000-0000-0000-0000-000000000001".to_string(),
                    person_name: "Mikhail Levshits".to_string(),
                    is_living: false,
                    x: 0.25,
                    y: 0.30,
                    width: 0.20,
                    height: 0.25,
                    is_primary_avatar: true,
                },
                FaceTagDto {
                    id: "ft-02".to_string(),
                    person_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                    person_name: "Elena Levshits".to_string(),
                    is_living: true,
                    x: 0.60,
                    y: 0.35,
                    width: 0.18,
                    height: 0.22,
                    is_primary_avatar: false,
                },
            ],
            links: vec![
                MediaLinkDto {
                    id: "link-01".to_string(),
                    media_id: "00000000-0000-0000-0000-000000000101".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: "00000000-0000-0000-0000-000000000001".to_string(),
                    role: Some("primary_avatar".to_string()),
                },
                MediaLinkDto {
                    id: "link-02".to_string(),
                    media_id: "00000000-0000-0000-0000-000000000101".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                    role: Some("depicted".to_string()),
                },
            ],
            version: 1,
        };

        let living_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000102".to_string(),
            gid: "M0002".to_string(),
            title: "Elena Levshits Portrait (2024)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string(),
            file_name: Some("m0002.webp".to_string()),
            original_file_name: Some("elena_2024.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: Some(800),
            height: Some(1000),
            description: None,
            date_taken: Some("2024".to_string()),
            date_sort: Some(20240000),
            place_id: None,
            place_full_name: None,
            is_missing: false,
            external_path: None,
            is_private: false,
            only_living_persons: true,
            linked_person_ids: vec!["c7a8b9d0-1234-5678-90ab-cdef12345678".to_string()],
            face_tags: vec![FaceTagDto {
                id: "ft-03".to_string(),
                person_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                person_name: "Elena Levshits".to_string(),
                is_living: true,
                x: 0.40,
                y: 0.25,
                width: 0.25,
                height: 0.30,
                is_primary_avatar: true,
            }],
            links: vec![MediaLinkDto {
                id: "link-03".to_string(),
                media_id: "00000000-0000-0000-0000-000000000102".to_string(),
                entity_type: "individual".to_string(),
                entity_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                role: Some("primary_avatar".to_string()),
            }],
            version: 1,
        };

        let private_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000103".to_string(),
            gid: "M0003".to_string(),
            title: "Confidential Archival Certificate".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string(),
            file_name: Some("m0003.webp".to_string()),
            original_file_name: Some("certificate.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: None,
            height: None,
            description: None,
            date_taken: None,
            date_sort: None,
            place_id: None,
            place_full_name: None,
            is_missing: false,
            external_path: None,
            is_private: true,
            only_living_persons: true,
            linked_person_ids: vec!["00000000-0000-0000-0000-000000000099".to_string()],
            face_tags: vec![],
            links: vec![MediaLinkDto {
                id: "link-04".to_string(),
                media_id: "00000000-0000-0000-0000-000000000103".to_string(),
                entity_type: "individual".to_string(),
                entity_id: "00000000-0000-0000-0000-000000000099".to_string(),
                role: Some("primary_avatar".to_string()),
            }],
            version: 1,
        };

        let places = vec![
            PlaceSummaryDto {
                id: "00000000-0000-0000-0000-000000000201".to_string(),
                gid: "P0001".to_string(),
                name: "Saint Petersburg".to_string(),
                place_type: "city".to_string(),
                parent_id: None,
                full_name: "Saint Petersburg, Russia".to_string(),
                country: Some("Russia".to_string()),
                latitude: Some(59.9343),
                longitude: Some(30.3351),
                events_count: 4,
                children_count: 0,
                alt_names: vec![
                    PlaceAltNameDto {
                        id: "pan-01".to_string(),
                        name: "Leningrad".to_string(),
                        lang: Some("en".to_string()),
                        note: Some("1924–1991 Soviet name".to_string()),
                    },
                    PlaceAltNameDto {
                        id: "pan-02".to_string(),
                        name: "Санкт-Петербург".to_string(),
                        lang: Some("ru".to_string()),
                        note: None,
                    },
                ],
                version: 1,
            },
            PlaceSummaryDto {
                id: "00000000-0000-0000-0000-000000000202".to_string(),
                gid: "P0002".to_string(),
                name: "Vitebsk".to_string(),
                place_type: "city".to_string(),
                parent_id: None,
                full_name: "Vitebsk, Belarus".to_string(),
                country: Some("Belarus".to_string()),
                latitude: Some(55.1904),
                longitude: Some(30.2049),
                events_count: 2,
                children_count: 0,
                alt_names: vec![PlaceAltNameDto {
                    id: "pan-03".to_string(),
                    name: "Витебск".to_string(),
                    lang: Some("ru".to_string()),
                    note: None,
                }],
                version: 1,
            },
        ];

        Self {
            persons: vec![deceased_ancestor, living_relative, private_record],
            families: vec![],
            events: vec![],
            notes: vec![],
            media: vec![deceased_media, living_media, private_media],
            places,
            change_log: vec![],
            snapshots: vec![],
            dismissed_warnings: vec![],
            not_duplicate_pairs: vec![],
            gid_aliases: HashMap::new(),
            story_drafts: vec![],
            personal_access_tokens: vec![],
            ai_config: AiInstanceConfigDto::default(),
            next_gid_seq: 100,
            next_family_gid_seq: 100,
            next_event_gid_seq: 100,
            next_place_gid_seq: 100,
            next_media_gid_seq: 100,
            is_demo_seed: true,
        }
    }
}

impl Default for GenealogyPrivacyStore {
    fn default() -> Self {
        Self::empty()
    }
}
