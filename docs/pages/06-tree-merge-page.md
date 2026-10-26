# Page Specification: Tree Import & Conflict Reconciliation (`/settings/merge`)

## 1. Overview & Objective
The Merge & Reconciliation view guides researchers through safely merging external tree files (GEDCOM / GEDZIP) into the active tree. It features a **Step-by-Step Staged Wizard**, an **Interactive Side-by-Side Diff Comparison**, and **Field-Level Conflict Resolution**.

> [!IMPORTANT] Administrator-Only Boundary:
> The Tree Import & Reconciliation Wizard (`/settings/merge`) is strictly restricted to **Administrators** (`RequireRole::<Admin>`). Because external tree ingestion creates global snapshots and alters genealogical topologies across the entire database, it is blocked for Viewers and Editors (`403 Forbidden`). (In-tree duplicate merging of two existing individual records is handled separately on `/people/merge` for Administrators).

---

## 2. Page Wireframe & Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| STEPPER:  1. Upload (✓)  >  2. Analysis (✓)  >  [3. Resolve Conflicts]  >  4. Commit|
+------------------------------------------------------------------------------------+
| SUMMARY METRIC CARDS:                                                              |
| [ Total Incoming: 1,420 ] [ Definite Matches: 42 ] [ Needs Review: 12 ] [ New: 1,180]
|                                                                                    |
| BATCH ACTION: [ ⚡ Auto-Approve 42 Definite Matches ]       Queue: Match 3 of 12    |
+------------------------------------------------------------------------------------+
| SIDE-BY-SIDE RECONCILIATION CARD:                                                  |
|                                                                                    |
| High Confidence Match (Score: 88%)                                                 |
|                                                                                    |
| Attribute          | Current Tree Record            | Incoming File Record         |
| ------------------ | ------------------------------ | ---------------------------- |
| Full Name          | (•) Ivan Petrov                | ( ) Ivan Petrovich Petrov    |
| Patronymic         | ( ) [Empty]                    | (•) Petrovich (✔ Add)        |
| Birth Date         | ( ) ABT 1892                   | (•) 14 Mar 1892 (✔ Use exact)|
| Birth Place        | (•) Tver, Russia               | ( ) Tver Gubernia            |
| Death Date         | (•) 1943                       | ( ) Unknown                  |
| Father             | Petr Petrov (ID: 104)          | Petr Petrov (Matches ID 104) |
| Attached Media     | [1 photo]                      | [2 photos] (✔ Keep All 3)    |
|                                                                                    |
| [Action Bar]                                                                       |
| [ 🔗 Merge Selected Fields ]    [ ➕ Keep Both as Separate ]    [ ⏭️ Skip Incoming ]|
+------------------------------------------------------------------------------------+
| SAFETY BANNER: Automatic backup snapshot will be created before committing changes. |
+------------------------------------------------------------------------------------+
```

---

## 3. Structural Sections

### 3.1 Wizard Stepper & Progress Header
1. **Upload & Format Check:** Drag-and-drop file uploader with character encoding detection.
2. **Analysis & Matching:** Progress bar while candidate matching scores records.
3. **Resolve Conflicts (Active View):** Step-by-step queue through ambiguous matches.
4. **Commit & Rollback Snapshot:** Confirmation summary before writing to database.

### 3.2 Metrics & Quick Batch Bar
- Clear metric chips showing:
  - Total records found
  - Definite matches ($\ge 90\%$ similarity)
  - Conflicts requiring human review ($65\% - 89\%$)
  - Clean new individuals to be appended ($< 65\%$)
- **"Auto-Approve Definite Matches" Button:** Allows users with large trees to bypass trivial identical matches in one click.

### 3.3 Side-by-Side Candidate Comparator
- Compares one conflicting individual at a time:
  - **Left Column:** Existing database record.
  - **Right Column:** Incoming staged record with highlighting for differences.
  - **Field Selection Radios:** User clicks radio button or checkbox next to the preferred value for:
    - First Name / Patronymic / Surname
    - Birth Date / Birth Place
    - Death Date / Death Place
    - Bio Notes (Choice to overwrite or combine)
    - Photos (Checkbox to merge media libraries)
- **Decision Controls:**
  - `Merge Records`: Combines fields according to user selections.
  - `Keep as Separate`: Creates a new duplicate node in the tree.
  - `Skip Incoming`: Ignores the incoming node completely.

### 3.4 Pre-Commit Safety Checkpoint
- Displays an explicit rollback guarantee banner: *"A complete snapshot will be saved. You can undo this entire merge within 14 days from Admin Settings"*.

---

## 4. Definition of Done (DoD)

- [ ] **Administrator-Only Access:** The merge wizard (`/settings/merge`) is accessible only to Administrators; Viewers, Editors, and unauthenticated visitors receive `403 Forbidden` / redirect to `/login`.
- [ ] **Linear Stepper Progression:** User cannot jump to merge commit without resolving or skipping flagged conflicts.
- [ ] **Field-Level Diff Highlight:** Discrepancies between current and incoming records are clearly highlighted with accessible color contrast.
- [ ] **Atomic Transaction:** Committing merges records, creates pre-merge snapshot, and recalculates relationships inside a single database transaction.
- [ ] **Queue Navigation:** User can move back and forth through the conflict queue before final submission.
