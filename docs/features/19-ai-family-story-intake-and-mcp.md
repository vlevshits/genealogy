# Feature Specification: AI Family Story Intake ("Tell Me About My Family") & Model Context Protocol (MCP) Server

## 1. Overview & Objective
Relatives and family members frequently possess oral memories, informal letters, or unstructured notes (*"I am Anna Volkova, born June 29, 1995 in Moscow. My husband is Alexey, born in 1993. My mother Maria..."*). Manually entering each person, calculating relationships, and linking nodes across multiple nested forms is intimidating and prone to fatigue.

This feature introduces an intuitive, safe AI-assisted intake pipeline combining:
1. **Multimodal Raw Story Input ("Tell Me About My Family" / "Расскажите о своей семье"):** Accepts free-form plain text or spoken voice audio (in-browser recording or audio file upload) anchored to a designated speaker.
2. **Sandbox Staging Area (Staged Draft Subtree):** The AI extracts individuals, relationships, and events into an isolated draft subtree without mutating the live family tree blindly. Users visually inspect and refine the extracted draft or ask for conversational adjustments.
3. **Reconciliation & Atomic Tree Merge:** Reuses the relationship-centric candidate matching engine ([09 §3](09-tree-import-merge-reconciliation.md#3-candidate-matching-algorithm-record-linkage), [15 §3](15-data-quality-duplicates.md#3-duplicate-detection--relationship-centric-person-merge)) to link matching existing relatives, create new nodes, and commit changes in one atomic, undoable `change_log` transaction.
4. **Dual Integration Architecture:**
   - **Built-in AI Provider (BYOK / Local LLM / Whisper / Offline Fallback):** Configurable in Admin Settings with API keys (OpenAI-compatible, Google Gemini, Anthropic) or local endpoints (Ollama / LocalAI / Whisper.cpp), plus a deterministic local rule/regex parser fallback for offline LAN environments without API keys.
   - **Protected Model Context Protocol (MCP) Server (`/api/v1/mcp`):** Exposes standard JSON-RPC MCP tools over Streamable HTTP/SSE with scoped Personal Access Tokens (PAT), enabling users to connect external AI agents of choice (Claude Desktop, Cursor, ChatGPT, Antigravity, custom scripts) to inspect the tree and draft changes under strict RBAC and audit control.

---

## 2. Data Model

Reuses the canonical entities in [10-core-data-model.md](10-core-data-model.md). Only two lightweight staging and authentication tables are added:

```sql
-- Staged Story Intake Drafts (Sandbox before committing to live tree)
CREATE TABLE story_intake_drafts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title          VARCHAR(200) NOT NULL,
    speaker_id     UUID NULL REFERENCES individuals(id) ON DELETE SET NULL, -- anchor relative ("I" / "я")
    raw_text       TEXT NOT NULL,
    audio_path     VARCHAR(500) NULL,                                       -- optional original audio recording in media storage
    status         VARCHAR(24) NOT NULL DEFAULT 'draft',                    -- draft | reconciled | committed | discarded
    extracted_data JSONB NOT NULL DEFAULT '{}',                             -- structured nodes, relationships, and events
    reconciled_at  TIMESTAMPTZ NULL,
    committed_at   TIMESTAMPTZ NULL,
    change_tx_id   UUID NULL,                                               -- transaction ID in change_log for 1-click rollback
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_story_intake_created_by ON story_intake_drafts(created_by);
CREATE INDEX idx_story_intake_status ON story_intake_drafts(status);

-- Scoped Personal Access Tokens for Protected MCP Server and External Integrations
CREATE TABLE personal_access_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,                                      -- e.g. "Claude Desktop MCP", "Cursor Dev Agent"
    token_hash  VARCHAR(64) NOT NULL UNIQUE,                                -- SHA-256 hash of raw token ("mcp_pat_...")
    token_prefix VARCHAR(16) NOT NULL,                                      -- e.g. "mcp_pat_ab12" for identification
    scopes      TEXT[] NOT NULL DEFAULT ARRAY['read:tree', 'write:draft'],  -- read:tree | write:draft | write:direct
    last_used_at TIMESTAMPTZ NULL,
    expires_at  TIMESTAMPTZ NULL,                                           -- NULL = non-expiring until revoked
    revoked_at  TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pat_token_hash ON personal_access_tokens(token_hash);
CREATE INDEX idx_pat_user_id ON personal_access_tokens(user_id);
```

- **Ephemeral Draft Subtree Payload (`extracted_data` JSONB schema):**
  - `draft_people`: Array of temporary objects (`temp_id`, `given_name`, `surname`, `maiden_name`, `patronymic`, `sex`, `birth_date`, `birth_place`, `death_date`, `death_place`, `is_living`, `notes`, `confidence_score`).
  - `draft_unions`: Array of partner unions (`temp_id`, `partner1_temp_id`, `partner2_temp_id`, `relationship_type`, `marriage_date`, `marriage_place`, `children_temp_ids`).
  - `draft_parent_child`: Array of links (`parent_temp_id`, `child_temp_id`, `relation_type`).
  - `draft_events`: Array of vital/historical events tied to `temp_id`.

---

## 3. User Experience & 3-Stage Intake Flow (`/stories/intake`)

```
+---------------------------------------------------------------------------------------------------------+
| [Global Header]                                                                                         |
+---------------------------------------------------------------------------------------------------------+
| Tell Me About My Family  /  Расскажите о своей семье                                                    |
| Step 1: Tell Your Story  ──>  Step 2: Inspect Draft Subtree  ──>  Step 3: Reconcile & Commit            |
+---------------------------------------------------------------------------------------------------------+
| SPEAKER CONTEXT:                                                                                        |
| Who is speaking in this story ("I" / "я")?                                                              |
| [ Combobox: Anna Volkova (I0001) ▾ ]  ( ) New relative not in tree yet                                  |
+---------------------------------------------------------------------------------------------------------+
| INPUT METHOD:                                                                                           |
| [ (•) Type Narrative ]   [ ( ) Speak Voice Recording ]   [ ( ) Upload Audio File ]                     |
|                                                                                                         |
| +-----------------------------------------------------------------------------------------------------+ |
| | "My name is Anna Volkova, born June 29, 1995 in Moscow. My husband is Alexey Smirnov (born 1993 in   | |
| | Tver). We have a daughter, Sophia, born in 2021. My mother is Maria Ivanovna Petrova (born 1968 in  | |
| | Saint Petersburg), and her father was Ivan Petrov..."                                              | |
| +-----------------------------------------------------------------------------------------------------+ |
|                                                                                                         |
| [ ⚡ Extract & Build Draft Subtree ]                                            Language: Auto-detect (RU) |
+---------------------------------------------------------------------------------------------------------+
```

### 3.1 Stage 1: Raw Family Story Input
- **Speaker Anchoring:**
  - The form prompts: *"Who is telling this story? ('I' / 'я')"*.
  - Defaults to the logged-in user's linked person (`user.person_id`) or `home_person_id`.
  - Enables accurate resolution of relative pronoun references (*"my husband"*, *"my maternal grandfather"*, *"our first daughter"*).
- **Text Input Mode:** Free-form multi-paragraph text area with markdown support, character counter, and sample templates.
- **Audio Recording Mode:**
  - Native browser recording via `MediaRecorder` API with a visual amplitude waveform.
  - Recording controls: `● Record`, `❚❚ Pause`, `■ Stop`, `▶ Play preview`.
  - Automatic silence detection and max duration guard (15 minutes).
- **Audio File Upload Mode:**
  - Dropzone supporting `.webm`, `.mp3`, `.wav`, `.m4a` files up to 25 MB.
- **Transcription Pipeline:**
  - Sent to server audio endpoint `POST /api/v1/ai/transcribe`.
  - Handled by configured STT provider (local Whisper.cpp / OpenAI Whisper / Gemini Multimodal) or browser Web Speech API fallback.
  - Raw audio can optionally be saved into media storage as an oral history audio document linked to the committed records.

### 3.2 Stage 2: Analysis & Staged Local Subtree Draft
- The system extracts structured genealogical entities and displays a split-view workspace:
  - **Left / Top:** Interactive **Local Subtree Canvas** rendering only the newly extracted branch using standard node cards (avatar silhouette, full name, dates, relationship lines).
  - **Right / Bottom:** **Entity Breakdown Cards** displaying extracted people, parent-child links, spousal unions, and confidence pills:
    - High Confidence (green): explicit facts (*"born June 29, 1995"*).
    - Medium Confidence (amber): approximate or inferred facts (*"around 1970"*, inferred gender from Russian patronymic).
    - Ambiguities (blue chip): missing dates or unclear marital status.
- **Conversational Clarification Loop:**
  - A persistent chat input beneath the draft allows iterative adjustments:
    - User types: *"Actually, Sophia was born in 2022, not 2021, and Alexey's middle name is Sergeevich."*
    - The AI incrementally patches the draft subtree without restarting from scratch.
  - Manual inline editing: clicking any card opens the lightweight person/event editor to fix spelling or dates directly.

### 3.3 Stage 3: Reconciliation & Tree Merge
- Runs the **Relationship-Centric Candidate Matching Engine** ([09 §3](09-tree-import-merge-reconciliation.md#3-candidate-matching-algorithm-record-linkage), [15 §3.1](15-data-quality-duplicates.md#31-relationship-centric-candidate-matching-engine)):
  - Compares draft nodes against live tree individuals.
  - Applies kinship guards (father/son naming collisions never match; generational separation preserved).
  - Automatically correlates the speaker and shared relatives.
- **Visual Reconciliation Matrix:**
  - For each draft person, presents two distinct choices:
    1. **Link & Enrich Existing Person:** Shown when similarity $\ge 80\%$ (e.g. *"Matches Anna Volkova (I0001)"*). Allows selecting which new facts/notes to merge without overwriting confirmed dates.
    2. **Create New Person & Connect:** Shown for net-new relatives (e.g. *"Alexey Smirnov — Add as new spouse to Anna"*).
- **Atomic Commit Action:**
  - Primary button: *"Commit Subtree to Family Tree ({count} new, {count} linked)"*.
  - Commits all individuals, families, events, and citations inside a single database transaction.
  - Generates a single transaction ID in `change_log`.
  - Displays an immediate success toast with a 1-click **Undo** button reversing the entire intake operation cleanly.

---

## 4. Dual AI Architecture: Built-in Agent & Protected MCP Server

```
+─────────────────────────────────────────────────────────────────────────────+
|                          GENEALOGY SERVER ARCHITECTURE                      |
+─────────────────────────────────────────────────────────────────────────────+
|                                                                             |
|  [ In-Browser UI (/stories/intake) ]        [ External AI Client ]          |
|    (Web Audio / Text / Refinements)           (Claude Desktop / Cursor)     |
|                   │                                     │                   |
|                   ▼                                     ▼                   |
|      Built-in AI Agent Service               Protected MCP Server Endpoint  |
|         (/api/v1/ai/intake)                       (/api/v1/mcp)             |
|                   │                                     │                   |
|         ┌─────────┴─────────┐                           │ (Bearer PAT Auth) |
|         │                   │                           │                   |
|         ▼                   ▼                           ▼                   |
|  [External LLM]     [Local / Offline]       [ Standard MCP Tool Dispatch ]   |
|  Gemini / OpenAI    Ollama / Regex           - search_people                |
|  BYOK API Key       Fallback Parser          - create_story_draft           |
|                                              - add_draft_person             |
|                                              - commit_draft_to_tree         |
|                                                         │                   |
|                   ┌─────────────────────────────────────┘                   |
|                   ▼                                                         |
|  [ Staging Sandbox: story_intake_drafts ]                                   |
|                   │                                                         |
|                   ▼                                                         |
|  [ Relationship-Centric Reconciliation Engine (09 / 15) ]                   |
|                   │                                                         |
|                   ▼                                                         |
|  [ Atomic Tree Transaction + change_log Audit Trail ]                       |
|                                                                             |
+─────────────────────────────────────────────────────────────────────────────+
```

### 4.1 Built-in AI Agent (BYOK, Local LLM & Offline Fallback)
- **Configuration (`/settings/ai` — Admin only):**
  - **Provider:** `OpenAI-Compatible` | `Google Gemini` | `Anthropic` | `Local (Ollama / LocalAI / Whisper)` | `Offline Rule-Based`.
  - **API Key:** Stored encrypted server-side; never exposed to browser clients.
  - **Base URL & Model Name:** Configurable (e.g. `http://localhost:11434/v1` for Ollama, `gemini-2.5-flash`, `gpt-4o-mini`).
- **Structured Tool-Calling Agent:**
  - The backend issues a system prompt instructing the model to parse entities and call structured tools:
    - `propose_person(temp_id, given_name, surname, patronymic, sex, birth_date, birth_place, death_date, death_place, notes)`
    - `propose_relationship(person1_temp_id, person2_temp_id, relation_type)`
    - `search_existing_people(query)`
    - `match_places(place_name)`
- **Deterministic Offline Rule/Regex Fallback Parser:**
  - When no external LLM key is configured (`AI_PROVIDER="offline"`), the server runs a built-in deterministic heuristic parser.
  - Recognizes standard Slavic and English kinship phrases:
    - *"Меня зовут [Имя] [Отчество] [Фамилия], родился [Дата] в [Место]"*
    - *"Мой муж / моя жена [Имя]..."*, *"Наши дети: [Имя], [Имя]..."*, *"Моя мама [Имя] (в девичестве [Фамилия])..."*
    - *"My name is...", "born on...", "My spouse is...", "Our son/daughter..."*
  - Guarantees zero external network dependencies and out-of-the-box functional tests in local/isolated environments.

### 4.2 Protected Model Context Protocol (MCP) Server (`/api/v1/mcp`)
The backend implements a compliant **Model Context Protocol (MCP)** server over Streamable HTTP and Server-Sent Events (SSE), conforming to the open specification.

#### 4.2.1 Authentication & Token Scopes
- Access is authenticated via `Authorization: Bearer mcp_pat_<token>`.
- Token issuance in `/settings/profile` (or Admin `/settings/users`):
  - Tokens are shown exactly once at creation; only the SHA-256 hash is persisted in `personal_access_tokens`.
  - Scopes:
    - `read:tree`: Access to search, profile inspection, subtrees, and kinship calculation.
    - `write:draft`: Ability to create story drafts, add staged nodes, and preview reconciliation (Safe by default).
    - `write:direct`: Direct creation and mutation of live records (restricted to `Editor` and `Admin` users).

#### 4.2.2 Role-Gated MCP Tool Catalog & Discovery Architecture
Both tool discovery (`tools/list`) and tool execution (`tools/call`) are strictly governed by the authenticated user's RBAC role (`Viewer`, `Editor`, `Admin`) and token scopes:
- **Filtered Discovery (`tools/list`):** An external AI model only sees tools permitted for the user's role. For example, a `Viewer` token never sees mutation or administrative tools; an `Editor` token never sees data quality or snapshot rollback tools.
- **Strict Execution Enforcement (`tools/call`):** Any attempt to invoke an unauthorized tool returns a JSON-RPC error response with code `-32600` / `403 Forbidden` (`code: "forbidden", message: "Editor or Admin role required"`).

##### Tier A: Read & Search Tools (Available to `Viewer`, `Editor`, `Admin` — requires scope `read:tree`)
| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `search_tree` | `query: string, types?: string[], limit?: int` | Full-text, cross-script (Latin/Cyrillic), and phonetic search across individuals, surnames, dates, places, and notes. |
| `get_person_profile` | `person_id: string, include?: string[]` | Retrieves complete biographical details, vitals, immediate family (parents, spouses, siblings, children), notes, citations, and media links. |
| `get_tree_subgraph` | `focal_id: string, mode?: "hourglass" \| "pedigree" \| "descendant", generations?: int` | Returns nodes and family edges in the neighborhood of a focal individual. |
| `find_kinship_path` | `person_a: string, person_b: string` | Calculates minimal biological and legal kinship path, relationship terms (EN/RU), degree, and common ancestors. |
| `search_places` | `query: string, parent_id?: string` | Searches the gazetteer, full jurisdiction breadcrumbs, and coordinates. |
| `get_place_details` | `place_id: string` | Returns place hierarchy breadcrumbs, coordinates, alternate names, and list of life events occurring at this location. |
| `get_upcoming_anniversaries`| `days_ahead?: int` | Returns upcoming birth and marriage milestone anniversaries. |
| `get_tree_statistics` | — | Returns archive summary statistics (total individuals, families, places, media assets, generation span). |

##### Tier B: Staged Subtree & Direct Tree Modification Tools (Available to `Editor` and `Admin` — requires scope `write:draft` or `write:direct`)
*Viewers attempting to call these tools receive `403 Forbidden`.*

*1. Staged Subtree / Story Intake Tools (Safe Sandbox — scope `write:draft`):*
| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `create_story_draft` | `title: string, speaker_id?: string, raw_story?: string` | Initializes an isolated story intake sandbox without altering the live family tree. |
| `upsert_draft_person` | `draft_id: string, temp_id: string, person_data: object` | Inserts or updates a proposed individual node in the draft subtree. |
| `upsert_draft_relationship`| `draft_id: string, person1_temp_id: string, person2_temp_id: string, relation_type: string` | Links two draft persons as parent-child or spousal union. |
| `remove_draft_item` | `draft_id: string, item_type: "person" \| "relationship", temp_id: string` | Removes an entity or link from the staging draft. |
| `reconcile_draft_with_tree` | `draft_id: string` | Runs relationship-centric candidate matching against live tree; returns proposed link/create recommendations. |
| `commit_story_draft` | `draft_id: string, resolution_overrides?: object` | Commits approved draft into the live tree in an atomic SQL transaction with undo logging. |

*2. Direct Tree Modification Tools (Direct Mutation — scope `write:direct`):*
| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `create_person` | `person_data: object` | Directly creates an individual record with vital dates, places, gender, and living override. |
| `update_person` | `person_id: string, fields: object` | Updates scalar fields, biography, or living status of an existing person. |
| `quick_add_relative` | `anchor_person_id: string, relationship: "parent" \| "spouse" \| "sibling" \| "child", person_data: object` | Adds a relative directly anchored to an existing person with automatic duplicate verification guard. |
| `create_or_update_family` | `partner1_id: string, partner2_id?: string, relationship_type?: string, children_ids?: string[]` | Creates a family union or adjusts children order and partner links. |
| `add_life_event` | `person_id?: string, family_id?: string, event_type: string, date?: object, place_id?: string, description?: string` | Records a vital event (birth, marriage, death, residence, occupation, etc.). |
| `update_life_event` | `event_id: string, fields: object` | Modifies an existing life event record. |
| `create_or_update_place` | `name: string, place_type?: string, parent_id?: string, latitude?: float, longitude?: float` | Adds or updates a place in the gazetteer. |
| `add_person_note` | `person_id: string, body: string, is_private?: bool` | Attaches a Markdown research note or oral history transcript to a person. |

##### Tier C: Administrative & Maintenance Tools (Available strictly to `Admin` — requires scope `admin:maintenance`)
*Viewers and Editors attempting to call these tools receive `403 Forbidden`.*
| Tool Name | Parameters | Description |
| :--- | :--- | :--- |
| `get_data_quality_report` | `category?: string` | Inspects open soft validation warnings, duplicate candidate pairs, and place hierarchy anomalies. |
| `merge_duplicate_people` | `target_id: string, source_id: string, field_choices?: object` | Executes atomic merge of two duplicate people; unions events/media/families with alias redirection. |
| `auto_fix_place_hierarchy`| `place_ids?: string[], min_confidence?: int, dry_run?: bool` | Analyzes, previews, or applies automated right-to-left place hierarchy decomposition and coordinate linking. |
| `list_change_history` | `limit?: int, entity_type?: string` | Inspects global audit trail of recent transactions, including AI and MCP operations. |
| `undo_transaction` | `tx_id: string` | Reverts any previous database transaction by transaction ID, restoring previous state. |

#### 4.2.3 Audit, Living Person Masking & Undo via MCP
- **Living Person Protection:** When an external MCP client queries records using a token belonging to a `Viewer` or an unauthenticated public integration, living individual data is strictly masked with single-initial dots (`И.` / `J.`) per [02 §4](02-rbac-access-control.md#4-privacy--masking-rules-backend-serialization-layer).
- **Change Log Tracking:** Every write performed via MCP records `created_by = token.user_id`, `source = "mcp"`, and `token_id = token.id` in `change_log`.
- **1-Click Rollback:** Any batch of changes authored by an external AI agent through MCP can be inspected in Recent Changes (`/history`) and undone in 1 click by an Administrator.

---

## 5. Permissions & Privacy

### 5.1 RBAC Matrix

| Capability / Surface | Public Guest | Authenticated Viewer | Authenticated Editor | Administrator |
| :--- | :---: | :---: | :---: | :---: |
| Access Story Intake UI (`/stories/intake`) | No (401) | No (403) | **Yes** | **Yes** |
| Audio Recording & File Upload | No | No | **Yes** | **Yes** |
| Create / Refine Story Drafts | No | No | **Yes** | **Yes** |
| Reconcile & Commit Subtree into Live Tree | No | No | **Yes** | **Yes** |
| Configure Built-in AI Provider & API Keys | No | No | No (403) | **Yes** |
| Generate MCP Personal Access Token | No | **Yes** (`read:tree`) | **Yes** (`read:tree`, `write:draft`, `write:direct`) | **Yes** (All scopes) |
| Access Protected MCP Endpoint (`/api/v1/mcp`) | No (401) | **Yes** (Bearer PAT) | **Yes** (Bearer PAT) | **Yes** (Bearer PAT) |
| MCP Tier A: Read & Search Tools | No | **Yes** (Living masked) | **Yes** (Full living access) | **Yes** (Full access) |
| MCP Tier B: Staged Subtree & Direct Mutation Tools | No | No (403) | **Yes** | **Yes** |
| MCP Tier C: Admin Maintenance & Quality Tools | No | No (403) | No (403) | **Yes** |
| Rollback AI / MCP Transactions | No | No | Self (within 60s) | **Yes** (Anytime from History) |

### 5.2 Self-Hosting Privacy & Secret Boundaries
- **No Unsolicited Outbound Requests:** In default posture, zero audio or text is transmitted externally. The built-in offline regex parser runs locally.
- **Explicit Cloud Consent:** If an Administrator chooses to configure an external cloud LLM (Gemini or OpenAI), a clear security notice explains that submitted narrative text/audio will be processed by that external provider.
- **Local Endpoint Freedom:** Fully supports self-hosted local endpoints (Ollama, Whisper.cpp, vLLM) on the LAN (`http://localhost:11434`), preserving 100% offline self-hosted privacy.

---

## 6. API Endpoints

### 6.1 Story Intake Endpoints

| Method | Endpoint | Access Level | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/v1/ai/transcribe` | Editor & Admin | Uploads audio recording/file; returns transcribed plaintext. |
| `POST` | `/api/v1/ai/parse-story` | Editor & Admin | Analyzes raw story text with speaker anchoring; returns staged draft subtree payload. |
| `POST` | `/api/v1/ai/refine-draft` | Editor & Admin | Applies conversational text clarification to update an existing draft subtree. |
| `GET` | `/api/v1/ai/drafts` | Editor & Admin | Lists active/recent story drafts created by the caller. |
| `GET` | `/api/v1/ai/drafts/:id` | Editor & Admin | Retrieves full draft subtree and candidate reconciliation matrix. |
| `PATCH` | `/api/v1/ai/drafts/:id` | Editor & Admin | Updates draft node fields or reconciliation choices before committing. |
| `POST` | `/api/v1/ai/drafts/:id/commit` | Editor & Admin | Executes atomic database commit; links records, creates new nodes, records change log. |
| `DELETE` | `/api/v1/ai/drafts/:id` | Editor & Admin | Discards draft sandbox. |

### 6.2 Model Context Protocol (MCP) Endpoints

| Method | Endpoint | Access Level | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/v1/mcp` | Bearer PAT Token | Handles standard MCP JSON-RPC 2.0 requests (`tools/list`, `tools/call`, `resources/list`). |
| `GET` | `/api/v1/mcp/sse` | Bearer PAT Token | Opens Server-Sent Events (SSE) stream for bidirectional streaming MCP sessions. |
| `GET` | `/api/v1/mcp/tokens` | Authenticated | Lists caller's active Personal Access Tokens. |
| `POST` | `/api/v1/mcp/tokens` | Authenticated | Creates a new scoped Personal Access Token (returns raw token string once). |
| `DELETE` | `/api/v1/mcp/tokens/:id` | Authenticated | Revokes a Personal Access Token immediately. |

---

## 7. Localization (EN/RU)

- **Domain-Specific Relationship Terminology:**
  - Russian kinship parsing understands rich Slavic kinship and gendered suffixes:
    - *дедушка по материнской линии* (maternal grandfather), *двоюродный брат* (first cousin male), *золовка* (husband's sister), *тесть* (wife's father), *свекровь* (husband's mother).
    - Patronymic derivation: automatically suggests patronymic or extracts father's given name from Russian patronymic (*"Алексей Сергеевич"* $\to$ father is *"Сергей"*).
    - Maiden name markers: *"в девичестве"*, *"урождённая"*, *"девичья фамилия"*.
  - English kinship parsing handles blended family terms (*"step-father"*, *"half-sister"*, *"partner"*, *"ex-husband"*).
- **UI Copywriting:** Adheres strictly to anti-fluff rules ([00 §3](00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards)):
  - Zero promotional buzzwords ("revolutionary AI", "cutting-edge smart parser").
  - Clear, functional archival copy:
    - EN: *"Tell Me About My Family"*, *"Extracting family members and relationships..."*, *"Staged Subtree Preview"*, *"Commit to Tree"*.
    - RU: *"Расскажите о своей семье"*, *"Поиск родственников и связей в тексте..."*, *"Черновой фрагмент древа"*, *"Добавить в семейное древо"*.
- **Icon Cleanliness:** Zero emoji or icon characters baked into `en.json` and `ru.json` translation strings.

---

## 8. Edge Cases & Non-Goals

### 8.1 Conflicting Dates & Partial Vitals
When a story mentions *"Grandfather died during the war around 1943"*, the system generates a `GenDate::about(1943)` and attaches a life event with `description: "During the war"`, rather than guessing an exact date.

### 8.2 Disambiguating Multiple Relatives with the Same Name
If a story says *"My father Ivan and his father Ivan"*, the system recognizes the two distinct generational tiers based on sentence syntax, assigning two separate temporary nodes (`temp_1: Ivan Sr.`, `temp_2: Ivan Jr.`), and strictly prevents self-linking.

### 8.3 Non-Goals
- **Blind Unsupervised Auto-Commit:** An AI agent is never permitted to alter the live tree without staging the draft and passing user reconciliation review.
- **Arbitrary Code Execution via MCP:** The MCP server strictly exposes bounded genealogical tools with strict argument validation; no generic database query or bash execution tools are exposed.

---

## 9. Definition of Done (DoD)

- [ ] **Multimodal Intake UI:** `/stories/intake` provides responsive text input, native in-browser voice recording (MediaRecorder), and audio file upload (`.webm`, `.mp3`, `.wav`, `.m4a`).
- [ ] **Speaker Context Resolution:** First-person pronouns ("my mother", "my partner") resolve accurately relative to the selected speaker person.
- [ ] **Staged Sandbox Isolation:** Story parsing creates an isolated draft record without mutating live tree individuals or families.
- [ ] **Visual Draft Subtree:** The draft preview renders an interactive mini-tree visualizer along with editable entity cards and confidence badges.
- [ ] **Conversational Clarification Loop:** Users can enter follow-up voice or text clarifications to incrementally update the draft before committing.
- [ ] **Relationship-Centric Reconciliation:** Candidate matching correlates draft nodes with existing tree individuals, applying kinship guards and preventing father/son false matches.
- [ ] **Atomic Undoable Commit:** Merging an approved draft executes within a single database transaction logged in `change_log`, with a working 1-click Undo toast.
- [ ] **Built-in Dual Provider Support:** Supports cloud LLM BYOK (OpenAI/Gemini/Anthropic), local endpoints (Ollama), and a deterministic offline regex fallback parser.
- [ ] **Protected MCP Server:** Conforms to MCP specification over `/api/v1/mcp` and `/api/v1/mcp/sse`, authenticating via scoped Personal Access Tokens (`mcp_pat_...`).
- [ ] **Role-Gated MCP Tool Execution:** Both tool discovery (`tools/list`) and execution (`tools/call`) strictly enforce RBAC (Tier A for Viewers, Tier B for Editors/Admins, Tier C for Admins only; unauthorized calls return MCP error code `-32600` / `403 Forbidden`).
- [ ] **Living Person Privacy in MCP:** Read operations executed via MCP enforce living person single-initial dot masking for non-admin/restricted tokens.
- [ ] **Bilingual Parity:** Complete EN and RU localization for all intake prompts, tool schemas, and toast notifications with zero icon glyphs in translation strings.
