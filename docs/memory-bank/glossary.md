# Shared Project Glossary

This glossary defines domain and technical terminology used across specifications, code, UI copy, and agent reports. It ensures strict bilingual naming consistency between English (EN) and Russian (RU).

---

| Term (EN) | RU | Meaning | Code identifier(s) | Defined in |
| :--- | :--- | :--- | :--- | :--- |
| **Admin** | Администратор | User role with full system privileges, backups, and user management. | `UserRole::Admin`, `"admin"` | `backend/src/services/rbac.rs` |
| **AI Enabled** | Включение ИИ | Public instance status flag indicating whether server-side Gemini AI features (transcription and extraction) are available. | `ai_enabled`, `SystemInfoResponse.ai_enabled` | `backend/src/routes/system.rs` |
| **All Nodes Mode** | Все персоны | Tree visualization showing all disjoint components and trees in one canvas. | `"all_nodes"`, `AllNodesLayout` | `frontend/src/lib/tree-layouts/all-nodes-layout.ts` |
| **Ancestor** | Предок | A person from whom an individual is directly descended. | `ancestor`, `ancestors` | `backend/src/services/genealogy/mod.rs` |
| **Anniversary** | Годовщина | Recurring annual milestone (birthday, wedding, death anniversary). | `Anniversary`, `get_anniversaries` | `backend/src/services/dashboard.rs` |
| **Anti-Collinear Clearance** | Устранение коллинеарности | Offset algorithm preventing overlapping connection lines in tree views. | `applyAntiCollinearClearance` | `frontend/src/lib/tree-layouts/hourglass-layout.ts` |
| **Change Log** | Журнал изменений | Transaction-level audit trail tracking all record modifications for rollback. | `change_log`, `AuditEntry` | `backend/src/services/history/mod.rs` |
| **Descendant** | Потомок | A person descended directly from a specified ancestor. | `descendant`, `descendants` | `backend/src/services/genealogy/mod.rs` |
| **Draft Extracted Data** | Извлечённые данные черновика | Pre-extracted entity graph (draft persons, unions, links, events) staged for database tree reconciliation. | `ParseStoryRequest.extracted_data`, `DraftExtractedDataDto` | `backend/src/services/story_intake/types_api.rs` |
| **Duplicate Candidate** | Подозрение на дубликат | Pair of individuals identified by linkage engine as potentially the same person. | `DuplicateCandidate`, `DuplicatePair` | `backend/src/services/quality/mod.rs` |
| **Editor** | Редактор | Authenticated role permitted to create and modify people, events, and media. | `UserRole::Editor`, `"editor"` | `backend/src/services/rbac.rs` |
| **Event** | Событие | Milestone in a person's life (birth, baptism, marriage, death, burial). | `Event`, `LifeEvent`, `EventType` | `backend/src/models/mod.rs` |
| **Face Tag** | Отметка на лице | Bounding box coordinates linking a face in a photo to a person record. | `FaceTag`, `MediaFaceTag` | `backend/src/routes/rbac/media_tags_links.rs` |
| **Family** | Семья | Relationship unit between partners and their shared children. | `Family`, `FamilyRow` | `backend/src/models/mod.rs` |
| **Fan Chart** | Веерная диаграмма | Radial visualization displaying concentric rings of ancestral generations. | `FanChartCanvas`, `get_fan_chart` | `frontend/src/components/tree/FanChartCanvas.tsx` |
| **`FRONTEND_STATIC_PATH`** | Путь к статике фронтенда | Optional environment variable enabling the Axum backend to directly serve the frontend SPA build with `index.html` fallback. | `FRONTEND_STATIC_PATH`, `AppConfig.frontend_static_path` | `backend/src/config.rs` |
| **GEDCOM** | GEDCOM | Standard specification for exchanging genealogical data (5.5.1 / 7.0). | `GedcomImport`, `GedcomExport` | `backend/src/services/gedcom/mod.rs` |
| **Gemini 3.8 Flash** | Gemini 3.8 Flash | Multimodal LLM used server-side (`extract_story_with_gemini`, `transcribe_audio_with_gemini`) via `GEMINI_API_KEY` in `.env` for oral story transcription and genealogical entity extraction, with offline built-in parser fallback. | `"gemini-3.8-flash"`, `extract_story_with_gemini`, `transcribe_audio_with_gemini` | `backend/src/services/story_intake/gemini.rs` |
| **`GEMINI_API_KEY`** | Ключ Gemini API | Environment variable providing Google AI Studio credentials for server-side audio transcription and story parsing. | `GEMINI_API_KEY`, `AppConfig.gemini_api_key` | `backend/src/config.rs` |
| **`GEMINI_MODEL`** | Модель Gemini | Environment variable configuring the Gemini model identifier (defaults to `"gemini-3.8-flash"`). | `GEMINI_MODEL`, `AppConfig.gemini_model` | `backend/src/config.rs` |
| **GenDate** | Генеалогическая дата | Structured date supporting exact, approximate (`ABT`), range, or dual dates. | `GenDate`, `DateModifier` | `backend/src/models/gendate.rs` |
| **Genealogical ID (`gid`)** | Идентификатор персоны | Short alphanumeric identifier (e.g. `I0001`, `F0002`) used in UI and GEDCOM. | `gid`, `person.gid` | `backend/migrations/20261026000005_core_genealogy_model.sql` |
| **Home Person** | Главная персона | User's preferred starting focus person for tree navigation and dashboard stats. | `home_person_id`, `HomePerson` | `backend/src/services/dashboard.rs` |
| **Hourglass Chart** | Песочные часы | Canvas layout showing ancestors above and descendants below the root person. | `HourglassLayout`, `renderHourglass` | `frontend/src/lib/tree-layouts/hourglass-layout.ts` |
| **Kinship** | Степень родства | Calculated relationship between two individuals (e.g. second cousin once removed). | `KinshipResult`, `calculateKinship` | `backend/src/services/genealogy/mod.rs` |
| **Living Person** | Живущий родственник | Person determined to be alive; subject to privacy redaction for public viewers. | `is_living`, `privacy::is_living` | `backend/src/services/privacy/mod.rs` |
| **LXC Deployment** | Развёртывание в LXC | Native deployment on Debian/Ubuntu Linux containers or bare-metal using systemd, precompiled binaries, and `scripts/install.sh`. | `scripts/install.sh`, `genealogy.service` | `scripts/install.sh` |
| **Magic Link** | Ссылка для входа | Single-use authentication link sent via email for passwordless sign-in. | `MagicLink`, `request_magic_link` | `backend/src/services/auth/mod.rs` |
| **Media Bundle** | Архив медиафайлов | ZIP archive containing batch photos and documents matched to GEDCOM stubs. | `media_bundle`, `upload_media_bundle` | `backend/src/services/media_bundle.rs` |
| **Model Context Protocol (MCP)** | Протокол контекста модели | Protected API standard allowing AI agents to query and stage family tree data. | `McpServer`, `mcp_jsonrpc_handler`| `backend/src/services/mcp/mod.rs` |
| **Multi-Trace Kinship** | Многопутевое родство | Algorithm detecting all distinct biological and marital connection paths. | `find_kinship_paths` | `backend/src/services/genealogy/mod.rs` |
| **On-Demand Release** | Релиз по требованию | Production release workflow triggered exclusively by version tags (`v*.*.*`) or manual dispatch, cross-compiling tarballs with checksums. | `release.yml`, `genealogy-release` | `.github/workflows/release.yml` |
| **Optimistic Locking** | Оптимистичная блокировка | Concurrency control rejecting updates if the record `version` changed. | `version`, `AppError::Conflict` | `backend/src/services/history/mod.rs` |
| **Passkey** | Ключ доступа (Passkey) | FIDO2 / WebAuthn biometric or hardware security credential for authentication. | `Passkey`, `verify_passkey_login` | `backend/src/services/auth/mod.rs` |
| **Patronymic** | Отчество | Slavic middle name derived from the father's given name with gendered suffixes. | `patronymic`, `suggest_patronymic`| `backend/src/services/genealogy/mod.rs` |
| **Pedigree** | Восходящее древо | Direct ancestral lineage going back through biological parents and grandparents. | `pedigree`, `get_pedigree` | `backend/src/services/genealogy/mod.rs` |
| **Personal Access Token (PAT)**| Персональный токен доступа | Secret token used by external tools/agents to authenticate with the MCP server. | `mcp_tokens`, `PersonalAccessTokenDto` | `backend/src/services/mcp/mod.rs` |
| **Person Merge** | Объединение персон | Atomic operation fusing two duplicate person records and reconciling relations. | `merge_duplicate_people` | `backend/src/routes/rbac/history_quality.rs` |
| **Place Hierarchy** | Иерархия мест | Historical administrative tree (e.g. Village → Parish → District → Province). | `places`, `parent_place_id` | `backend/src/routes/rbac/places.rs` |
| **Print Studio** | Студия печати | UI workspace configuring single-sheet posters or tiled multi-page PDF exports. | `PrintExportPage`, `PrintConfig` | `frontend/src/pages/PrintExportPage.tsx` |
| **Privacy-404** | Приватная ошибка 404 | Returning 404 instead of 403 on private records to prevent existence enumeration. | `AppError::NotFound` | `backend/src/services/privacy/mod.rs` |
| **Recovery Code** | Код восстановления | Emergency alphanumeric code allowing account recovery when 2FA/passkey is lost. | `recovery_codes`, `verify_code` | `backend/src/services/auth/mod.rs` |
| **Redaction** | Скрытие данных | Replacing private details of living persons and their events/families with generic placeholders or filtering them out. | `redact_person_for_caller`, `redact_person_for_tree_view`, `redact_event_for_tree_view`, `redact_family_for_mcp` | `backend/src/services/privacy/masking.rs` |
| **Story Extraction Engine** | Движок извлечения историй | Extraction method selector between server-side cloud LLM (Gemini 3.8 Flash) and offline local rules parser. | `StoryExtractionEngine`, `'gemini' \| 'builtin'` | `frontend/src/api/gemini-story-extractor.ts` |
| **Story Intake** | Приём семейных историй | Multimodal interface for transcribing audio/text and generating draft subtrees. | `StoryIntake`, `parse_story_into_draft` | `backend/src/services/story_intake/mod.rs` |
| **Tree Snapshot** | Снимок древа | Point-in-time backup state of all genealogical records allowing 1-click restore. | `tree_snapshots`, `rollback_to_snapshot` | `backend/src/services/history/mod.rs` |
| **Viewer** | Наблюдатель | Read-only authenticated family member permitted to view living relatives in the web UI (masked when accessed via MCP token). | `UserRole::Viewer`, `"viewer"` | `backend/src/services/rbac.rs` |

_Last curated: 2026-11-08 (LXC single-script install/update, on-demand release pipeline & release skill)_
