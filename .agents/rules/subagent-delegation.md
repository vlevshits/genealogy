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
| Test plans, automated tests, DoD verification, bug reproduction, regression checks | `qa-analyst` |
| UI/UX review of screens: accessibility, alignment, use of space, visual hierarchy, discoverability, consistency; raising UX tasks for the frontend | `ux-reviewer` |

## Standard Feature Workflow

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
4. **Verify** — `qa-analyst` validates against the Definition of Done,
   including browser scenarios (via `browser_subagent`) for any feature with
   UI scope. Route defects to the owning developer agent, then re-run QA.
   If a subagent reports it cannot launch the browser, the main agent runs
   the listed browser scenarios itself with `browser_subagent`.
5. **UX Review** — **mandatory whenever the phase/change set touched
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
6. **Close** — update `docs/technical-debt.md` and add an implementation
   report in `docs/implementation/` summarizing the outcome, including the
   UX review verdict and link to its report.

## Delegation Guidelines

- Subagents do not see this conversation: include the goal, the relevant spec
  file paths, acceptance criteria, and any prior findings in the prompt.
- Small, trivial edits (typos, a single string) may be done directly without
  delegation.
- Summarize each subagent's final report to the user rather than pasting it.
