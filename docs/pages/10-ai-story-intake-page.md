# Page Specification: AI Story Intake & Subtree Staging Studio (`/stories/intake`)

## 1. Overview & Objective
The Story Intake Studio (`/stories/intake`) provides a peaceful, guided workspace where family members and oral historians can contribute unstructured oral or written accounts (*"Tell Me About My Family"* / *"Расскажите о своей семье"*). The interface guides the user through a clear 3-stage progression:
1. **Raw Story Input:** Text narrative or spoken voice recording/upload anchored to a designated speaker.
2. **Draft Subtree Staging:** An interactive, isolated visual canvas displaying extracted individuals, families, and events with confidence indicators and an incremental clarification prompt.
3. **Reconciliation & Commit:** Side-by-side matching against the live family tree using relationship-centric candidate linkage, with 1-click atomic commit and instant Undo.

**Access Level:** Authenticated Editors & Administrators (`Editor`, `Admin`). Unauthenticated guests are redirected to `/login`; Viewers receive `403 Forbidden`.

---

## 2. Page Wireframe & Component Layout

```
+-----------------------------------------------------------------------------------------------------------------+
| [Global Header: Logo | Home | Tree | People | Media | More ▾                                (RU/EN) (Theme) [Avatar] ] |
+-----------------------------------------------------------------------------------------------------------------+
| Breadcrumbs: Home > Stories > Tell Me About My Family                                                           |
| Title: Tell Me About My Family                                                                                  |
| Subtitle: Share oral memories or text notes. The assistant organizes relatives into an inspectable draft tree.  |
+-----------------------------------------------------------------------------------------------------------------+
| STEP PROGRESS TRACKER:                                                                                          |
| (1) Tell Your Story [Active]  ───────>  (2) Inspect Draft Subtree  ───────>  (3) Reconcile & Commit Tree       |
+-----------------------------------------------------------------------------------------------------------------+
| SECTION 1: SPEAKER ANCHORING                                                                                    |
| Who is speaking in this story ("I" / "я")?                                                                      |
| [ Combobox: Anna Volkova (I0001) ▾ ]    [ ] This story is told by a new relative not yet in the tree           |
+-----------------------------------------------------------------------------------------------------------------+
| SECTION 2: STORY INPUT TABS                                                                                     |
| [ (•) Type Narrative ]             [ ( ) Record Voice ]             [ ( ) Upload Audio File ]                   |
| +-------------------------------------------------------------------------------------------------------------+ |
| | Text Area:                                                                                                  | |
| | "My name is Anna Volkova, born June 29, 1995 in Moscow. My partner is Alexey Smirnov (born 1993 in Tver).   | |
| | We have a daughter, Sophia, born in 2021. My mother is Maria Ivanovna Petrova (born 1968 in Saint           | |
| | Petersburg), and her father was Ivan Petrov..."                                                             | |
| |                                                                                                             | |
| | [ 234 characters ]                                                                    [ Clear ] [ Load Sample ] |
| +-------------------------------------------------------------------------------------------------------------+ |
| Language: [ Auto-detect (Russian) ▾ ]                                    [ ⚡ Extract & Build Draft Subtree ]     |
+-----------------------------------------------------------------------------------------------------------------+
| SECTION 3: STAGED SUBTREE WORKSPACE (Appears upon extraction)                                                   |
| +-------------------------------------------------------+-----------------------------------------------------+ |
| | DRAFT SUBTREE CANVAS                                  | EXTRACTED ENTITIES (4 people, 2 unions)             | |
| |                                                       |                                                     | |
| |        [Maria Petrova (1968)]                         | [Card: Anna Volkova (Speaker, 1995)]                | |
| |                  │                                    |   - Birth: 29 Jun 1995, Moscow (High 98%)           | |
| |                  ▼                                    | [Card: Alexey Smirnov (Partner, 1993)]              | |
| |    [Anna Volkova] ═══ [Alexey Smirnov]                |   - Birth: 1993, Tver (High 95%)                    | |
| |                  │                                    | [Card: Sophia Smirnova (Daughter, 2021)]            | |
| |                  ▼                                    |   - Birth: 2021 (Medium 82% — check exact date)     | |
| |         [Sophia (2021)]                               | [Card: Maria Petrova (Mother, 1968)]                | |
| |                                                       |   - Birth: 1968, St. Petersburg                     | |
| +-------------------------------------------------------+-----------------------------------------------------+ |
| CLARIFICATION CHAT / ADJUSTMENTS:                                                                               |
| [ e.g. "Sophia was born in 2022, not 2021"                                                ] [ Send Update ]     |
+-----------------------------------------------------------------------------------------------------------------+
| SECTION 4: RECONCILIATION & TREE MERGE MATRIX                                                                   |
| +-------------------------------------------------------------------------------------------------------------+ |
| | Draft Node           Proposed Live Action                     Resolution Status & Details                   | |
| +-------------------------------------------------------------------------------------------------------------+ |
| | Anna Volkova         (•) Link & Enrich Existing Record        Matches Anna Volkova (I0001, 100%)            | |
| |                      ( ) Create as New Separate Person                                                      | |
| | Alexey Smirnov       ( ) Link to Existing Record              Net-New Relative (0 matches in tree)          | |
| |                      (•) Create as New Relative & Link        Add as spouse/partner to Anna (I0001)         | |
| | Sophia Smirnova      ( ) Link to Existing Record              Net-New Relative                              | |
| |                      (•) Create as New Relative & Link        Add as child to Anna & Alexey                 | |
| | Maria Petrova        (•) Link & Enrich Existing Record        Matches Maria Ivanovna Petrova (I0002, 94%)   | |
| +-------------------------------------------------------------------------------------------------------------+ |
| [ Discard Draft ]                                            [ Commit Subtree to Family Tree (2 new, 2 linked) ]|
+-----------------------------------------------------------------------------------------------------------------+
```

---

## 3. Structural Components

### 3.1 Speaker Context Selector
- **Component:** `SpeakerSelectorCard`
- **Purpose:** Anchors first-person references ("I", "my mother", "my spouse", "our son") to a specific individual node.
- **Controls:**
  - Combobox searching individuals in the live tree (pre-populated with logged-in user's `person_id` or `home_person_id`).
  - Radio/Checkbox toggle: *"This narrative is told by someone not yet recorded in the tree"* (treats the speaker as a new individual).
- **Behavior:** Changing the speaker re-evaluates relational pronoun bindings across the narrative text.

### 3.2 Multimodal Story Input Panel
- **Component:** `StoryInputPanel`
- **Tabs:**
  1. **Type Narrative (`SegmentedControl: 'text'`):**
     - Markdown-capable multi-line text area with auto-grow (`min-h-[160px]`, `max-h-[400px]`).
     - Character counter and helper templates (*"Parents and childhood"*, *"Marriage and children"*, *"War and migration"*).
  2. **Record Voice (`SegmentedControl: 'audio_record'`):**
     - Native Web Audio / `MediaRecorder` integration.
     - Visualizer: real-time animated frequency waveform.
     - Timer: `00:00 / 15:00` with pulsing red recording indicator.
     - Action buttons: `Record`, `Pause`, `Stop`, `Play Preview`, `Re-record`.
     - Direct button: *"Transcribe Audio"*.
  3. **Upload Audio File (`SegmentedControl: 'audio_upload'`):**
     - Drag-and-drop zone accepting `.webm`, `.mp3`, `.wav`, `.m4a` files up to 25 MB.
     - Progress bar during upload and transcription.
- **Action Button:** `Extract & Build Draft Subtree` (`Sparkles` icon, primary variant).
- **Loading State:** Skeleton shimmer over draft workspace with animated message: *"Parsing genealogical entities and dates..."*.

### 3.3 Draft Subtree Visual Canvas & Entity Breakdown
- **Component:** `DraftSubtreeCanvas` & `DraftEntityListCard`
- **Left Column (Canvas):**
  - Renders an isolated interactive SVG layout of the draft subtree.
  - Cards display temporary name, vital dates, relationship badges, and confidence tint:
    - Green border: High Confidence ($\ge 85\%$).
    - Amber border: Inferred or approximate date/relationship ($60\% - 84\%$).
    - Blue chip: Missing vital date or ambiguous relationship.
  - Cards support click-to-edit to manually adjust individual fields inline.
- **Right Column (Breakdown):**
  - Accordion list of extracted people, unions, and life events.
  - Shows original narrative snippet from which each fact was derived (explainability/grounding).
  - Delete button (`Trash2` icon) per draft node to prune spurious extractions.

### 3.4 Conversational Clarification & Follow-Up Bar
- **Component:** `DraftClarificationBar`
- **Purpose:** Enables conversational adjustments without resetting the entire draft.
- **Controls:**
  - Input field: *"Enter a correction or additional detail (e.g., 'Sophia was born in 2022, not 2021')"*.
  - Voice button: optional microphone toggle to dictate a quick spoken correction.
  - Action button: `Update Draft` (`Send` icon).
- **Behavior:** The backend parses the delta against the active `story_intake_drafts.id`, patching nodes and updating the visual canvas dynamically.

### 3.5 Reconciliation & Tree Merge Matrix Panel
- **Component:** `ReconciliationMatrixCard`
- **Purpose:** Displays how draft nodes map onto the live tree and gives user final approval before committing.
- **Layout:**
  - Table or card list showing each draft individual:
    - **Draft Person:** Extracted name, vitals, relationship to speaker.
    - **Live Match:** Candidate match card with similarity score badge, kinship alignment, and comparison diff.
    - **Action Selector:** Radio group (`Link & Enrich Existing Record` vs `Create as New Separate Relative`).
    - **Field Enrichment Checkboxes:** When linking to an existing person, checkboxes indicate which incoming fields to append or update.
- **Primary Footer CTA:**
  - `Commit Subtree to Family Tree` (displays exact count: *"{N} new relatives, {M} linked"*).
  - `Discard Draft` (ghost variant, prompts confirmation dialog).

### 3.6 Success & Undo Toast
- Upon successful commit:
  - Navigates user to the updated focal person in `/tree?focal=<id>` or profile `/persons/<id>`.
  - Fires an elevated success toast:
    *"Family subtree successfully added to tree (2 new relatives, 2 linked). [ Undo ]"*.
  - Clicking **Undo** invokes `undo_transaction` reversing the entire commit atomically.

---

## 4. Definition of Done (DoD)

- [ ] **Responsive 3-Stage Layout:** The intake page smoothly guides the user through Input $\to$ Draft Subtree $\to$ Reconciliation with step progression indicators.
- [ ] **Speaker Selector Integration:** User can select an existing anchor relative or toggle "new relative", and pronouns correctly bind to this anchor.
- [ ] **In-Browser Audio Recording:** Real-time microphone recording functions with audio waveform visualizer, pause/resume, playback, and duration timer.
- [ ] **Audio Upload Dropzone:** Drag-and-drop handles `.webm`, `.mp3`, `.wav`, `.m4a` files up to 25 MB with upload progress feedback.
- [ ] **Visual Draft Tree:** The staged subtree renders on an isolated SVG canvas with clear confidence color coding.
- [ ] **Conversational Clarifications:** Sending a follow-up correction updates draft nodes and canvas without reloading the page.
- [ ] **Reconciliation Matrix:** Each extracted individual offers clear "Link & Enrich" vs "Create New" controls with relationship-centric candidate matching.
- [ ] **Atomic Undoable Commit:** Committing updates the live tree and displays an immediate toast with working 1-click Undo.
- [ ] **Mobile Usability (< 768px):** Steps stack vertically without horizontal overflow; draft node inspection opens in a swipeable bottom sheet.
- [ ] **Bilingual Localization:** Complete EN and RU translations for all page copy, helper templates, audio prompts, and error alerts with zero emoji prefixes.
