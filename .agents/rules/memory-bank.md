---
trigger: always_on
---

# Memory Bank Protocol

`docs/memory-bank/` is the project's long-term implementation memory (see
[`docs/memory-bank/README.md`](../../docs/memory-bank/README.md)). Agents have no
memory between conversations — the memory bank is how they recall *what exists,
where it lives, why it is that way, and what to avoid*.

## 1. Recall before acting (every agent, every non-trivial task)
- Read `docs/memory-bank/README.md`, `active-context.md` and the relevant
  `features/NN-*.md` **before** planning, specifying, coding, reviewing or testing.
- Read `conventions-and-patterns.md` and `gotchas.md` before writing code;
  `decisions.md` before changing an established approach (superseding an ADR
  needs an explicit new ADR, confirmed with the user).
- Use the vocabulary and identifiers in `glossary.md` (EN + RU) for names,
  specs, tests, reports and UI copy. Do not coin a synonym for an existing term.
- The memory bank is a map, not the territory: verify cited code before relying
  on a detail; report stale entries instead of silently working around them.

## 2. Contribute after acting (every subagent)
End every final report with a `## Memory Notes` section (new/changed terms,
decisions, gotchas, code map changes, stale memory — or `- none`). The format
is defined in `docs/memory-bank/README.md` §4.

## 3. Single writer
Only the `memory-bank-curator` subagent edits `docs/memory-bank/`. Other agents
never edit it (the main agent may fix a trivial typo/one-liner directly).

## 4. When the curator runs (main agent's responsibility)
- **Close step of every feature/phase** — Mode A, with the change set, the
  implementation report path and all collected `## Memory Notes`.
- After a significant bug fix, refactor, migration or dependency change, even
  outside the standard workflow.
- When a new domain term appears in a requirement → glossary update (can be
  batched with the next Close).
- On request or when `scripts/check-memory-bank.sh` fails → Mode C (Audit).

## 5. Delegation context packs
When invoking any subagent, include in its prompt the memory bank files it
should read first (at minimum `active-context.md`, the relevant
`features/NN-*.md`, and `glossary.md`), alongside the spec paths.

