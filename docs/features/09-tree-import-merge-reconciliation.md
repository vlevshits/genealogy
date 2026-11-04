# Feature Specification: Tree Import, Merging & Conflict Reconciliation

## 1. Overview & Objective
Enable researchers to import external family trees (from relatives, research collaborators, Ancestry, or MyHeritage) and **merge them safely into the active tree** without creating duplicate individuals or overwriting existing research.

Because merging family trees is one of the most error-prone operations in genealogy, this system adopts a **Staged Import Architecture** with a **Probabilistic Record Linkage (Candidate Matching) Engine**, an **Interactive Side-by-Side Reconciliation Interface**, and a **1-Click Rollback Snapshot**.

> [!NOTE] Reused for AI Story Intake:
> This same staged candidate matching and reconciliation workflow powers the **AI Family Story Intake & Subtree Staging Studio** ([19](19-ai-family-story-intake-and-mcp.md)), reconciling narrative-extracted draft subtrees against the live tree with identical kinship guards and atomic undoability.

---

## 2. Staged Merge Workflow

```
[Uploaded Tree File (GEDCOM / GEDZIP)]
                 |
                 v
   +---------------------------+
   |   Isolated Staging Area   |  (Parsed into staging tables; zero impact on live tree)
   +-------------+-------------+
                 |
                 v
   +---------------------------+
   | Candidate Matching Engine |  (Scores similarity between Incoming vs. Live Records)
   +-------------+-------------+
                 |
                 v
   +---------------------------+
   | Interactive Diff & Review |  (User reviews High/Medium confidence matches & resolves conflicts)
   +-------------+-------------+
                 |
                 v
   +---------------------------+
   | Pre-Merge Backup Snapshot |  (Automatic safety checkpoint before writing)
   +-------------+-------------+
                 |
                 v
   +---------------------------+
   | Atomic Merge Commit       |  (Records combined, relationships relinked, media deduplicated)
   +---------------------------+
```

---

## 3. Candidate Matching Algorithm (Record Linkage)

When an incoming tree is staged, the backend runs a multi-factor similarity scoring algorithm comparing each incoming individual against all existing individuals in the database:

### 3.1 Structural Kinship Disqualification & Generational Separation
In genealogical research, names frequently repeat across generations (e.g. fathers and sons named *"Ivan Petrov"* / *"John Smith"*, or siblings named after deceased older brothers). To prevent disastrous accidental merges:
1. **Direct Kinship Disqualification:** Two records that occupy distinct generational tiers within a connected pedigree (e.g., parent and child, grandparent and grandchild) are **strictly disqualified** (`score = 0`) from matching, even if their given name and surname are identical.
2. **Distinct Parent Sets:** If both the incoming record and the live record have known, confirmed biological parents that are completely different individuals (and not candidate matches themselves), matching is blocked or penalized below the review threshold.
3. **Primary Scope of Duplicates:** Duplicates are expected and actively evaluated when:
   - An **additional external tree or branch is imported** that overlaps with existing family lines.
   - The tree contains **detached subtrees or unconnected singletons** whose missing parents/spouses align with records in the main tree.
   - Within a clean single connected component, duplicate detection is suppressed for individuals with distinct confirmed parents and non-overlapping lifespans.

### 3.2 Scoring Factors
1. **Kinship & Structural Context (Weight: 40% — Primary Factor):**
   - Matching parent names (Father and/or Mother), or patronymic matching father's given name: up to 40%.
   - Matching spouse name(s): up to 25%.
   - Matching children set: up to 25%.
   - *Requirement for Definite Matches:* Scores $\ge 80\%$ strictly require confirmed relationship or vital date alignment; name similarity alone cannot bridge the gap.
2. **Date & Place of Birth & Death (Weight: 30%):**
   - Exact date match = 100% of factor.
   - Same year, missing day/month = 85% of factor.
   - Within $\pm 2$ years tolerance (accounting for census/metric book discrepancies) = 60% of factor.
   - Matching birth or death place (or matching parent jurisdiction) boosts confidence.
3. **Name Similarity (Weight: 30%, Capped at 35% without Relational Context):**
   - Exact string match or Levenshtein distance $\le 2$.
   - Phonetic matching (Soundex / Daitch-Mokotoff Soundex / Cyrillic Metaphone for Russian names).
   - Maiden name / alternate name matching.
   - **Name similarity alone is capped at 35%**, guaranteeing that a father and son with identical names can never trigger a false merge recommendation.

### 3.3 Confidence Tiers
| Confidence Tier | Score | Default UI Recommendation | Action |
| :--- | :---: | :--- | :--- |
| **Definite Match** | $\ge 90\%$ | Automatic Merge Candidate | Pre-selected to merge; requires structural relational or vital date confirmation. |
| **Probable Match** | $65\% - 89\%$ | Requires Manual Review | Side-by-side comparison presented for user decision. |
| **New Individual** | $< 65\%$ | Add as New | Imported cleanly as a new branch or individual. |

---

## 4. Interactive Side-by-Side Reconciliation UI

The reconciliation screen displays all identified potential matches in an intuitive review queue:

```
+---------------------------------------------------------------------------------------+
| Match 3 of 12: Probable Match (Confidence: 84%)                                       |
+-----------------------------------+-------------------+-------------------------------+
| Attribute                         | Current Tree Node | Incoming Tree Node            |
+-----------------------------------+-------------------+-------------------------------+
| Full Name                         | Ivan Petrov       | Ivan Petrovich Petrov (✔ Use) |
| Sex                               | Male              | Male                          |
| Birth Date                        | ABT 1892          | 14 Mar 1892 (✔ Use)           |
| Birth Place                       | Tver, Russia      | Tver Gubernia (✔ Combine)     |
| Death Date                        | 1943 (Keep)       | Unknown                       |
| Father                            | Petr Petrov       | Petr Petrov                   |
| Photos Attached                   | 1 Photo           | 2 Photos (✔ Merge & Keep All) |
+-----------------------------------+-------------------+-------------------------------+
| [Action: 🔗 Merge Records | ➕ Keep as Separate Individuals | ⏭️ Skip Incoming Record]    |
+---------------------------------------------------------------------------------------+
```

### 4.1 Resolution Choices per Match:
1. **Merge Records (`Merge`):**
   - User picks which conflicting scalar fields to preserve (e.g. keeping the more precise birth date).
   - Non-conflicting events (e.g. military service from the incoming tree) are automatically appended.
   - Notes and event source texts from both records are combined.
2. **Keep Separate (`Keep Both`):**
   - Imports the incoming individual as a new separate node in the tree.
3. **Skip (`Ignore`):**
   - Discards the incoming record and retains the current record unaltered.

### 4.2 Bulk Review Options:
- **"Auto-Merge Definite Matches" Button:** For large imports with dozens of identical matches, the user can batch-approve all $\ge 90\%$ matches with non-conflicting incoming data in one click.

---

## 5. Media & Asset Deduplication

- When importing a bundled tree archive (`.zip` / `.gdz`), media files are inspected before storage.
- The engine calculates the **SHA-256 hash** of each incoming image/document.
- If an identical file already exists in `/data/media`, no duplicate file is saved; the system simply links the existing media asset to the newly imported or merged individual.

---

## 6. Safety Rails & 1-Click Rollback

1. **Pre-Merge Snapshot:**
   - Before executing the final merge transaction, the backend creates an atomic database checkpoint (`tree_snapshots` table or isolated schema clone).
2. **Undo Merge Button:**
   - For 14 days following any merge operation, an Admin can click **"Rollback Merge [Date]"** in the Audit Log.
   - Instantly restores the tree state to immediately before the merge was committed, ensuring zero permanent data corruption.

---

## 7. Definition of Done (DoD)

- [ ] **Isolated Staging:** Uploading an external tree populates staging tables without altering or locking the active tree.
- [ ] **Candidate Matching Engine:** Accurately flags individuals with identical or highly similar names, birth dates, and parent connections.
- [ ] **Structural Kinship Guards:** Father/son pairs or relatives with distinct confirmed generational tiers and distinct parents are never recommended as duplicate matches (`score = 0`).
- [ ] **Phonetic & Cyrillic Matching:** Successfully correlates transliterated or phonetically close Slavic names (e.g., *Petrov* vs *Пётр*, *Aleksandr* vs *Alexander*).
- [ ] **Side-by-Side Review Screen:** User can inspect conflicting fields, choose which values to retain, and review linked family members before committing.
- [ ] **Media Deduplication:** Uploading identical photos across different trees creates only one physical file on disk (verified via SHA-256 checksum).
- [ ] **Atomic Execution:** The entire merge operation executes within a single database transaction; failure at any step aborts the operation cleanly.
- [ ] **1-Click Rollback:** Admin can undo an executed merge, completely reverting all added and merged records to the pre-merge snapshot state.
