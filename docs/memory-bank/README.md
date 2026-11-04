# Memory Bank

The memory bank is the project's **long-term, implementation-level memory** for
humans and AI agents. Agents start every conversation without memory of past
work; the memory bank is how they recover *what exists, where it lives, why it
is the way it is, and what to watch out for* — without re-reading 120k lines of
code.

> [!IMPORTANT]
> The memory bank describes **the code as it is now**. Specs describe what the
> product *should* do; the memory bank describes how it *actually* does it.
> When they disagree, the code wins for the memory bank, and the discrepancy is
> recorded as a gotcha / tech-debt candidate.

## 1. How it relates to other docs

| Doc | Answers | Owner | Lifecycle |
| :--- | :--- | :--- | :--- |
| `docs/features/`, `docs/pages/` | *What* should be built (requirements, DoD) | `spec-writer` | Changes with requirements |
| `docs/roadmap.md` | *When* (phases, order) | `spec-writer` | Changes with planning |
| `docs/implementation/` | *What happened* in a phase (narrative, verification) | main agent (Close step) | Append-only history |
| `docs/ux-reviews/` | UX findings per review | `ux-reviewer` | Append-only history |
| `docs/technical-debt.md` | Known shortcuts & tradeoffs | main agent / developers | Living register |
| **`docs/memory-bank/`** | *How / where / why* the code works today; shared vocabulary | **`memory-bank-curator`** | **Living, always current** |

The memory bank **links** to specs, reports and TD items instead of copying them.

## 2. Structure

| File | Purpose | Read when |
| :--- | :--- | :--- |
| [`README.md`](README.md) | This protocol and index | Always first |
| [`project-context.md`](project-context.md) | Product in one page: purpose, principles, actors, stack, deployment | Starting any task |
| [`active-context.md`](active-context.md) | Current phase, recent changes, in-flight work, next steps, open questions | Starting any task |
| [`architecture.md`](architecture.md) | System map: containers, backend layers, frontend layers, request lifecycle, cross-cutting middleware, data flow | Design / cross-cutting work |
| [`codebase-map.md`](codebase-map.md) | Directory/module → responsibility map, hotspots (oversized files), where to put new code | Before touching unfamiliar code |
| [`conventions-and-patterns.md`](conventions-and-patterns.md) | Recurring, *proven* code patterns with file references (error envelope, RBAC guard, privacy filter, optimistic locking, i18n, DS components, tests) | Before writing code |
| [`glossary.md`](glossary.md) | Domain + technical vocabulary (EN / RU / code identifier / where defined) | Whenever a term is unclear; when naming things |
| [`decisions.md`](decisions.md) | Architecture Decision Log (`ADR-NNN`): context, decision, consequences | Before changing an established approach |
| [`gotchas.md`](gotchas.md) | Pitfalls, past bugs and their root causes, non-obvious invariants (`GOT-NNN`) | Before coding / debugging |
| [`features/`](features/) | One file per feature: code map, flows, invariants, decisions, tests, gotchas | Before working on that feature |

Feature files use the same number as the spec: `features/NN-<name>.md` ↔
`docs/features/NN-<name>.md`. Cross-cutting areas without a feature spec use
`features/xx-<name>.md` (e.g. `xx-design-system.md`).

## 3. Read protocol (all agents)

1. Read `README.md`, `active-context.md` and the relevant `features/NN-*.md`.
2. Read `conventions-and-patterns.md` and `gotchas.md` before writing code.
3. Use `glossary.md` terms and identifiers when naming things and writing
   specs, tests, reports and UI copy (EN/RU columns).
4. Read `decisions.md` before proposing to change an established approach;
   superseding an ADR requires an explicit new ADR.
5. Treat the memory bank as a **map, not the territory**: verify the cited code
   before relying on a detail. If something is stale, report it under
   *Memory Notes* (see §4) — do not silently work around it.

## 4. Write protocol

- **Single writer:** only `memory-bank-curator` edits `docs/memory-bank/`
  (the main agent may make trivial one-line fixes). This keeps tone,
  structure and accuracy consistent.
- **Every other agent contributes** by ending its final report with a
  `## Memory Notes` section:

  ```
  ## Memory Notes
  - New/changed terms: <term> — <meaning> (<code identifier>, <file>)
  - Decisions: <decision> — <why> (<alternatives rejected>)
  - Gotchas: <pitfall / root cause / invariant> (<file:line>)
  - Code map changes: <new modules, endpoints, tables, components, i18n namespaces>
  - Stale memory: <memory-bank file/section> — <what is wrong>
  ```

  Write `- none` when there is nothing to add.
- **When the curator runs** (see `.agents/rules/subagent-delegation.md`):
  in the *Close* step of every feature/phase, after any significant bug fix
  or refactor, and on request for an *Audit*.

## 5. Writing rules (curator)

- **Evidence-based:** every claim about code cites a repo-relative path
  (`backend/src/services/privacy.rs`) and, where helpful, a symbol
  (`fn can_view_person`). Avoid line numbers except in `gotchas.md` (they rot).
- **Concise and scannable:** tables and bullets; no narrative history (that
  belongs in `docs/implementation/`).
- **Current, not historical:** update or delete outdated statements instead
  of appending "UPDATE:" notes. History lives in git.
- **Link, don't duplicate:** reference specs (`docs/features/NN-*.md`),
  TD IDs (`TD-NNN`), UX IDs (`UX-NNN`), reports.
- **Stable IDs:** `ADR-NNN` in `decisions.md`, `GOT-NNN` in `gotchas.md`;
  never renumber; mark retired entries `~~struck~~` with a reason.
- **Glossary entries** are alphabetical, one row per term, with EN, RU, code
  identifier(s) and the defining file.
- Each file ends with `_Last curated: YYYY-MM-DD (<phase or reason>)_`.
- Run `scripts/check-memory-bank.sh` after editing; it fails on references to
  repo paths that no longer exist.

## 6. Feature file template (`features/NN-<name>.md`)

```markdown
# <Feature name> — Implementation Memory

**Spec:** [docs/features/NN-*.md](../../features/NN-*.md) · **Pages:** … ·
**Reports:** [docs/implementation/…](../../implementation/…) · **Status:** implemented | partial | not started

## 1. Summary
2–5 sentences: what is implemented today, and notable deviations from the spec.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| Migrations | backend/migrations/… | tables, columns, indexes |
| Services | backend/src/services/… | fn … |
| Routes | backend/src/routes/… | handlers |
| API | `METHOD /api/v1/...` | auth / role, notable error codes |
| Frontend pages | frontend/src/pages/… | |
| Components | frontend/src/components/<domain>/… | |
| API client | frontend/src/api/client.ts | fn … |
| i18n | `<namespace>.*` keys | |
| Tests | backend/tests/…, frontend/tests/… | what they cover |

## 3. Key Flows
Numbered steps for the 1–3 most important flows (request → service → DB → response → UI).

## 4. Invariants & Business Rules
Rules the code enforces (privacy, RBAC, validation, concurrency) and where.

## 5. Decisions
Feature-local decisions; cross-cutting ones link to `decisions.md#adr-nnn`.

## 6. Gotchas & Known Limitations
Feature-local pitfalls; link `GOT-NNN`, `TD-NNN`, `UX-NNN`.

## 7. Extension Points
How to add the next thing (new endpoint, new layout, new event type …).

_Last curated: YYYY-MM-DD (<reason>)_
```

## 7. Feature index

| # | Feature | Memory file |
| :--- | :--- | :--- |
| 00 | Internationalization & localization | [features/00-internationalization-localization.md](features/00-internationalization-localization.md) |
| 01 | Authentication & passkeys | [features/01-authentication-passkeys.md](features/01-authentication-passkeys.md) |
| 02 | RBAC & access control / privacy | [features/02-rbac-access-control.md](features/02-rbac-access-control.md) |
| 03 | GEDCOM import / export | [features/03-gedcom-import-export.md](features/03-gedcom-import-export.md) |
| 04 | Tree visualizations | [features/04-tree-visualizations.md](features/04-tree-visualizations.md) |
| 05 | Printable tree PDF | [features/05-printable-tree-pdf.md](features/05-printable-tree-pdf.md) |
| 06 | Media & face tagging | [features/06-media-face-tagging.md](features/06-media-face-tagging.md) |
| 07 | Person & family management | [features/07-person-family-management.md](features/07-person-family-management.md) |
| 08 | Deployment (single instance) | [features/08-deployment-single-instance.md](features/08-deployment-single-instance.md) |
| 09 | Tree import merge & reconciliation | [features/09-tree-import-merge-reconciliation.md](features/09-tree-import-merge-reconciliation.md) |
| 10 | Core data model | [features/10-core-data-model.md](features/10-core-data-model.md) |
| 12 | Places | [features/12-places.md](features/12-places.md) |
| 13 | Search & people directory | [features/13-search-people-directory.md](features/13-search-people-directory.md) |
| 14 | Change history & concurrency | [features/14-change-history-concurrency.md](features/14-change-history-concurrency.md) |
| 15 | Data quality & duplicates | [features/15-data-quality-duplicates.md](features/15-data-quality-duplicates.md) |
| 16 | Dashboard & home | [features/16-dashboard-home.md](features/16-dashboard-home.md) |
| 17 | API security & operations | [features/17-api-security-operations.md](features/17-api-security-operations.md) |
| 18 | PWA & mobile | [features/18-progressive-web-app-mobile.md](features/18-progressive-web-app-mobile.md) |
| 19 | AI story intake & MCP | [features/19-ai-family-story-intake-and-mcp.md](features/19-ai-family-story-intake-and-mcp.md) |
| xx | Design system | [features/xx-design-system.md](features/xx-design-system.md) |

_Last curated: 2026-11-04 (memory bank introduced)_

