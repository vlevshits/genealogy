# Feature Specification: Change History, Undo & Concurrent Editing

## 1. Overview & Objective
Several family members editing one shared tree **will** overwrite or delete each other's work. This spec covers:

1. **Change log** — who changed what, when (with before/after).
2. **Restore / undo** of individual changes and deletions.
3. **Optimistic concurrency** so two editors never silently overwrite each other.
4. **Tree snapshots** used by merge rollback and before Clean-Slate import.

---

## 2. Data Model

```sql
CREATE TABLE change_log (
    id           BIGSERIAL PRIMARY KEY,
    tx_id        UUID NOT NULL,                -- groups changes of one user action (e.g. "Add father" = person + family + link)
    tx_label     VARCHAR(200) NOT NULL,        -- localized key + args: "person.add_parent" {name}
    user_id      UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    entity_type  VARCHAR(16) NOT NULL,
    entity_id    UUID NOT NULL,
    entity_label VARCHAR(200) NULL,            -- cached display name at time of change
    op           VARCHAR(8) NOT NULL,          -- create | update | delete
    before       JSONB NULL,                   -- full row (+ owned children for delete)
    after        JSONB NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON change_log (entity_type, entity_id, created_at DESC);
CREATE INDEX ON change_log (created_at DESC);
CREATE INDEX ON change_log (tx_id);

CREATE TABLE tree_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reason      VARCHAR(32) NOT NULL,          -- merge_import | clean_slate_import | bulk_merge | manual
    label       VARCHAR(200) NULL,
    file_path   TEXT NOT NULL,                 -- pg_dump (data-only, genealogy tables) in /app/data/backups/snapshots
    created_by  UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL            -- default +14 days
);
```

- `change_log` is written **in the same DB transaction** as the change (application-level, not triggers, so `tx_label` and user context are reliable).
- Retention: `HISTORY_RETENTION_DAYS` (default `365`; `0` = forever). Bulk import rows are summarized as one entry per import (not 10,000 rows).

---

## 3. User Experience

### 3.1 Recent Changes (`/history`)
- Feed grouped by day, one row per **transaction**:
  *"🟢 Vera added **Пётр Фёдорович Щеглов** as father of Иван — 14:32"*
  *"🟡 Lev edited birth date of **Елена Морозова**: `1915` → `abt 1914`"*
  *"🔴 Lev deleted **Unknown Child** — [Restore]"*
- Filters: user, entity type, date range, "only deletions".
- Visible to Editors and Admins. Viewers do not see history.

### 3.2 Per-Record History
- Every profile/family/place has a *History* entry in its `⋯` menu → slide-over drawer listing that record's changes with field-level diffs.

### 3.3 Restore & Undo
| Action | Who | Behavior |
| :--- | :--- | :--- |
| **Undo toast** | The author | After any save/delete, a 6-second *"Undo"* toast reverts the whole transaction. |
| **Restore deleted record** | Editor+ | Re-creates the record and its owned children/links from `before`. If linked records no longer exist, those links are skipped and reported. |
| **Revert a field change** | Editor+ | Applies `before` values for the changed fields as a **new** change (history is append-only). Blocked with explanation if the field was changed again later. |
| **Rollback snapshot** | Admin | Restores all genealogy tables from a `tree_snapshot`; listed in *Settings → Backups*; requires typed confirmation. A safety snapshot of the current state is taken first. |

### 3.4 Dashboard Integration
The dashboard ([16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md)) shows the latest 5 transactions.

---

## 4. Optimistic Concurrency

- Every editable row carries `version INT`. API reads return it; every `PATCH`/`DELETE` must send `If-Match: <version>` (or `version` in body).
- Update: `UPDATE … SET …, version = version + 1 WHERE id = $1 AND version = $2`. Zero rows → **`409 Conflict`** with the current server record.
- **UI on conflict:** modal *"Vera changed this person 2 minutes ago"* with a field-level comparison of **your changes** vs. **current values**, and buttons: *Keep mine* (re-submit with new version), *Use theirs*, *Merge field by field*.
- Live awareness (lightweight): when an edit form is open, the client polls `GET /api/v1/:type/:id/version` every 30 s and shows a non-blocking banner if it changed.

---

## 5. API Endpoints
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/history?user_id=&entity_type=&op=&before=&limit=` | Transaction feed. |
| `GET` | `/api/v1/history/:entity_type/:entity_id` | History of one record. |
| `POST` | `/api/v1/history/tx/:tx_id/undo` | Undo a whole transaction (author within 10 min, or Editor+). |
| `POST` | `/api/v1/history/:change_id/restore` | Restore a deleted record / revert a field change. |
| `GET` | `/api/v1/snapshots` | List snapshots (Admin). |
| `POST` | `/api/v1/snapshots/:id/rollback` | Roll back to snapshot (Admin). |

---

## 6. Definition of Done (DoD)

- [ ] **Complete Logging:** Every create/update/delete through the API produces a `change_log` entry inside the same transaction.
- [ ] **Readable Feed:** History shows human-readable, localized sentences grouped by user action, not raw rows.
- [ ] **Restore Deleted Person:** A deleted person can be restored with their events, notes, face tags, and family links.
- [ ] **Undo Toast:** Every destructive action offers an immediate undo.
- [ ] **No Silent Overwrites:** Two editors saving the same record → the second receives `409` and a resolution dialog.
- [ ] **Snapshot Rollback:** Admin can roll back a merge/import snapshot; a pre-rollback snapshot is created automatically.
- [ ] **Retention:** Entries older than `HISTORY_RETENTION_DAYS` and expired snapshots are pruned by the nightly job.
