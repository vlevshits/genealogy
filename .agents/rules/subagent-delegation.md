---
trigger: always_on
---

# Role-Based Subagent Delegation

This workspace defines custom subagents in `.agents/agents/`. When a piece of
work matches a role, delegate it to that subagent via `invoke_subagent`
instead of doing it inline.

| Work type | Subagent |
| :--- | :--- |
| New/changed requirements, requirement analysis, writing/updating feature & page specs, roadmap / implementation plan updates | `spec-writer` |
| Rust/Axum/SQLx code, migrations, API endpoints, backend tests, backend Docker | `backend-developer` |
| React/TypeScript UI, design-system components, Tailwind styles, i18n, API client, Nginx/frontend Docker | `frontend-developer` |
| Feature design/planning, architecture decisions, code review, spec conformance, security review | `architect-reviewer` |
| Playwright E2E tests: user journeys, regression suites, page object models, auth fixtures, responsive & cross-browser flows | `e2e-tester` |
| Test plans, automated test verification (cargo, vitest, playwright), DoD verification, bug reproduction, regression checks | `qa-analyst` |
| UI/UX review of screens: accessibility, alignment, use of space, visual hierarchy, discoverability, consistency; raising UX tasks for the frontend | `ux-reviewer` |
| Memory bank & glossary upkeep: per-feature implementation memory, glossary, architecture/codebase maps, decision log, gotchas, active context; staleness audits | `memory-bank-curator` |

## Standard Feature Workflow

The memory bank (`docs/memory-bank/`, protocol in
`.agents/rules/memory-bank.md`) frames every workflow: agents **recall** from
it at the start and **contribute** `## Memory Notes` at the end; the curator
folds those notes back in at Close.

R. **Recall** — before step 0, the main agent reads
   `docs/memory-bank/active-context.md`, the relevant `features/NN-*.md`,
   `glossary.md` and `gotchas.md`, and builds the *context pack* (see
   Delegation Guidelines) that every subagent prompt in this workflow carries.
0. **Specify** — for any new or changed requirement, `spec-writer` analyses
   impact, creates/updates `docs/features/*.md` / `docs/pages/*.md`, and
   places the work in `docs/roadmap.md`. Resolve its *Open Questions* with
   the user before moving on. Skip when the specs already cover the work.
1. **Design** — `architect-reviewer` (Mode 1) produces the technical plan from
   the relevant `docs/features/*.md` / `docs/pages/*.md` specs. If it finds
   spec gaps or contradictions, route them back to `spec-writer`.
2. **Implement** — `backend-developer` first (API contract + OpenAPI), then
   `frontend-developer`. Independent backend/frontend work may run in parallel.
3. **Review** — `architect-reviewer` (Mode 2) reviews the change set. Send
   blocker/major findings back to the owning developer agent and re-review.
4. **E2E Tests** — **`e2e-tester`** writes and updates automated Playwright E2E
   tests for the feature (`frontend/e2e/`). Runs immediately once feature
   implementation is complete and reviewed to cover real user journeys,
   responsive mobile/desktop viewports, EN/RU localization states, and DoD
   scenarios.
5. **Verify** — `qa-analyst` validates against the Definition of Done by
   running the full automated test battery (`cargo test`, `npm test`,
   `npm run test:e2e`), checking DoD traceability, and validating interactive
   browser scenarios (via `browser_subagent`) for any feature with UI scope.
   Route defects to the owning developer agent, then re-run.
6. **UX Review** — **mandatory whenever the phase/change set touched
   `frontend/`** (skip only for backend-only work). Invoke `ux-reviewer`
   with the phase name, changed files and affected screens/routes. It writes
   a report to `docs/ux-reviews/` with `UX-NNN` tasks.
   - Send all `critical` and `major` tasks to `frontend-developer` (pass the
     report path and task IDs), then re-run `ux-reviewer` on the affected
     screens until none remain open.
   - `minor` / `suggestion` tasks: fix in the same loop if cheap, otherwise
     record them in `docs/technical-debt.md` referencing the `UX-NNN` ID.
   - Tasks tagged `spec-change` go to `spec-writer` first (confirm with the
     user), then to `frontend-developer`.
   - If UX fixes were made, have `qa-analyst` run a quick regression on the
     touched screens.
7. **Close** — update `docs/technical-debt.md` and add an implementation
   report in `docs/implementation/` summarizing the outcome, including the
   UX review verdict and link to its report.
8. **Remember** — **mandatory** after Close: invoke `memory-bank-curator`
   (Mode A) with the phase/feature name, the change set (changed files or
   commit range), the implementation report path, the spec paths and **all
   `## Memory Notes` sections** collected from the subagents' reports. It
   updates the feature memory file, glossary, decisions, gotchas, codebase
   map and active context, and runs `scripts/check-memory-bank.sh`. Route its
   *Drift & Recommendations* to the owning agent (`spec-writer` for spec
   drift, developers for bugs, main agent for TD entries).

Outside the full workflow (bug fixes, refactors, migrations, dependency
upgrades, one-off investigations that uncovered a non-obvious invariant), still
invoke `memory-bank-curator` (Mode A, small scope) when the change alters what
the memory bank says. Run Mode C (Audit) on request, when
`scripts/check-memory-bank.sh` fails, or every few phases.

## Delegation Guidelines

- Subagents do not see this conversation: include the goal, the relevant spec
  file paths, acceptance criteria, and any prior findings in the prompt.
- **Context pack:** every subagent prompt also lists the memory bank files to
  read first — always `docs/memory-bank/active-context.md` and
  `docs/memory-bank/glossary.md`, plus the relevant
  `docs/memory-bank/features/NN-*.md` (and `conventions-and-patterns.md` /
  `gotchas.md` for developers, QA and reviewers; `decisions.md` for the
  architect and spec-writer).
- Every subagent's final report ends with `## Memory Notes`; keep them and
  pass them to `memory-bank-curator` in the Remember step.
- Small, trivial edits (typos, a single string) may be done directly without
  delegation.
- Summarize each subagent's final report to the user rather than pasting it.
