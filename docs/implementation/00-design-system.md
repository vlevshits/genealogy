# Implementation Report: Phase 0 — Design System & Component Library

**Date:** 2026-10-26  
**Status:** Completed & Tested  
**Scope:** Phase 0 of [`docs/roadmap.md`](../roadmap.md) — theme tokens, light/dark/system mode, Universal `AppShell` and mobile `BottomNav`, 3-layer reusable component library (`ui/`, `form/`, `domain/`), `/dev/ui` interactive workbench, bilingual EN/RU date parser & shared `GenDate` fixtures across Rust and TypeScript, anti-fluff archival copywriting, and automated Vitest + Rust integration test suites.

---

## 1. Summary of What Was Implemented

### 1.1 Shared Contracts & Backend (`fixtures/`, `backend/`)
- **Canonical `GenDate` Test Fixtures ([`fixtures/gendate_cases.json`](../../fixtures/gendate_cases.json)):**
  - 15 bilingual (EN/RU) test cases covering exact dates (`14 Jul 1923`, `14.07.1923`, `14 июля 1923 г.`), approximate (`about 1890`, `ок. 1890`), bounds (`before 1917`, `до мая 1860`, `after 1945`, `после 1945 г.`), ranges (`between 1914 and 1918`, `между 1914 и 1918 гг.`), calculated (`calculated 1905`, `вычислено 1905 г.`), Julian calendar (`14.07.1905 (ст. ст.)` → Gregorian sort key `19050727`), and unparseable phrase dates (`"в детстве во время войны"`).
- **Rust `GenDate` Domain Model ([`backend/src/models/gendate.rs`](../../backend/src/models/gendate.rs)):**
  - `GenDate`, `GenDateMod`, `CalendarType`, and `PartialDate` structs with `serde` and `utoipa::ToSchema` derives registered in `/api/v1/openapi.json` ([`backend/src/routes/mod.rs`](../../backend/src/routes/mod.rs)).
  - `GenDate::compute_sort_key()` (with Julian-to-Gregorian offset and month/year rollover via `chrono::NaiveDate`, `before` `-1` and `after` `+1` adjustments) and bilingual `GenDate::parse()`.
  - Integration tests in [`backend/tests/gendate_fixtures_test.rs`](../../backend/tests/gendate_fixtures_test.rs).

### 1.2 Theme Tokens, Mobile Ergonomics & Internationalization (`frontend/src/`)
- **Theme Tokens ([`frontend/src/index.css`](../../frontend/src/index.css)):**
  - Control heights (`--size-control-sm: 2rem`, `--size-control-md: 2.75rem` / 44px, `--size-control-lg: 3rem` / 48px), touch targets (`--size-touch-min: 44px`, `--size-touch-lg: 48px`, `@media (pointer: coarse)` `.touch-target-expand`), focus rings (`--color-ring`, `--ring-width`, `--ring-offset`), motion tokens (`--duration-fast`, `--duration-base`, `--duration-slow`, `--ease-out`), z-index layering (`--z-canvas-overlay` through `--z-tooltip`), safe-area insets (`--safe-top`, `--safe-bottom`, `--safe-left`, `--safe-right`), and elevation shadows Level 1–3.
- **Theme Provider ([`frontend/src/lib/theme.tsx`](../../frontend/src/lib/theme.tsx)):**
  - `ThemeProvider` & `useTheme` supporting `'light' | 'dark' | 'system'` with `matchMedia('(prefers-color-scheme: dark)')` listener, `localStorage` persistence, and zero-flicker class toggling.
- **Internationalization & Anti-Fluff Standards ([`frontend/src/i18n/`](../../frontend/src/i18n/)):**
  - Parameter interpolation `t(key, params)` in [`index.tsx`](../../frontend/src/i18n/index.tsx) with 100% key parity between [`en.json`](../../frontend/src/i18n/en.json) and [`ru.json`](../../frontend/src/i18n/ru.json).
  - Purged all promotional buzzwords and tech-stack names from UI copy per [`docs/features/00-internationalization-localization.md §3`](../features/00-internationalization-localization.md).
  - Bilingual date parser ([`gendate-parser.ts`](../../frontend/src/lib/gendate-parser.ts)) and gender-aware Russian/English kinship label engine ([`kinship.ts`](../../frontend/src/lib/kinship.ts)).

### 1.3 Three-Layer Component Library (`frontend/src/components/`)
- **Layer 1 — UI Primitives (`src/components/ui/`, 46 components):**
  - *Actions:* `Button`, `IconButton` (required `label`), `ButtonGroup`, `SegmentedControl`, `CopyButton`.
  - *Feedback:* `Badge`, `Kbd`, `Spinner`, `Skeleton`, `Divider`, `ProgressBar`, `EmptyState`, `Alert`, `Banner`.
  - *Inputs:* `TextInput`, `SearchInput`, `PasswordInput`, `EmailInput`, `Textarea`, `Checkbox`, `CheckboxGroup`, `Switch`, `RadioGroup`, `RadioCardGroup`, `Select` (desktop dropdown + mobile bottom sheet), `Combobox` (150ms debounce, `AbortController`, `onCreate`, `allowFreeText`), `MultiSelect`, `Slider`, `RangeSlider`, `YearInput`, `TypedConfirmInput`, `FileDropzone`.
  - *Overlays:* `Dialog` (`sm | md | xl | full`, mobile full-screen), `ConfirmDialog` (`useConfirm` promise API, impact list, `TypedConfirmInput`), `Drawer` (`right` 480px / `bottom` sheet with safe-area padding), `Popover`, `Tooltip`, `DropdownMenu`, `CommandPalette` (`Cmd/Ctrl+K` omnibox), `ToastProvider` / `useToast` (with 6s Undo countdown).
  - *Navigation & Structure:* `AppShell` (Universal header, breadcrumbs/context bar, `Cmd+K` trigger, EN/RU toggle, Light/Dark/System toggle, mobile drawer), `BottomNav` (fixed 5-tab mobile bar: Home, Tree, People, Media, More bottom sheet with 48×48px tap targets and safe-area inset), `AuthLayout`, `NavLink`, `Breadcrumbs`, `PageHeader`, `Tabs`, `SideNav`, `Stepper`, `Collapsible`, `Accordion`, `TreeView`, `ScrollArea`.
  - *Data Display:* `DataTable` (50,000-row virtualized viewport, sorting, shift-click range selection, column visibility, `mobileRow` card renderer, `FilterBar`, `FilterChip`, `BulkActionBar`), `ComparisonTable`, `Card`, `Avatar`, `AvatarGroup`, `Stat`, `DescriptionList`, `Timeline`, `ActivityFeed`, `MarkdownView`, `Image`.
- **Layer 2 — Form System (`src/components/form/`, 6 components):**
  - `Form`, `useForm`, `FormField` (`zod` validation, server `field_errors` mapping, soft yellow chronology warnings), `FormSection`, `FormActions`, `MarkdownEditor`, `GenDateInput` (live EN/RU parse preview, modifier menu, century/decade calendar popover, Julian `ст. ст.` toggle, phrase fallback).
- **Layer 3 — Genealogy Domain Composites (`src/components/domain/`, 13 components):**
  - `PersonName`, `GenDateText`, `LifeSpan`, `PlaceText`, `PrivacyBadge`, `LivingBadge`, `KinshipLabel`, `PersonListItem`, `PersonPicker` (cross-script Cyrillic/Latin search), `PlacePicker`, `SexSelector`, `EventRow`, `EventForm`, `PersonCard`.
- **Component Workbench (`/dev/ui`, [`DevUiWorkbench.tsx`](../../frontend/src/pages/DevUiWorkbench.tsx)):**
  - Code-split via `React.lazy()` and `<Suspense>` in [`App.tsx`](../../frontend/src/App.tsx); showcases all 10 component groups and interactive test harnesses in Light/Dark/System and EN/RU.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | Zero formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (4/4 tests)** | Health, OpenAPI, 15 shared `GenDate` fixtures, Julian rollover |
| **Frontend Unit & A11y Tests** | `npm test -- --run` | **Passed (33/33 tests)** | 15 `GenDate` fixtures, EN/RU parity & anti-fluff check, 50k virtualized `DataTable`, `BottomNav` 48px targets, zero-hex check |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | `tsc --noEmit && vite build` with code-split `DevUiWorkbench` chunk |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | All 5 Round 1 findings resolved and verified in Round 2 |
| **QA & Browser Verification** | `qa-analyst` + Playwright MCP | **PASS (0 defects)** | All 11 DoD items and 6 interactive browser scenarios verified |

---

## 3. Technical Debt & Tradeoffs Identified

See [`docs/technical-debt.md`](../technical-debt.md) for the full registry:
1. **TD-006 (Offline Sandbox Package Cache & Self-Hosted Font Bundles):** `class-variance-authority`, `zod`, and `vitest` were installed from the local npm cache while UI primitives, virtualized `DataTable`, and form controller were implemented directly in React 19 + TypeScript for zero-network reproducible builds.

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 1 — Deployment & Single-Instance Operations:** [`docs/features/08-deployment-single-instance.md`](../features/08-deployment-single-instance.md) (single `docker compose up`, health check, automatic migrations, exponential DB retry loop resolving **TD-001**, nightly backup + CLI restore).
