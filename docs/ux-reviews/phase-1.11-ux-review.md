# UX Review — Phase 1.11 — 2026-10-28

## Verdict: OK
**Screens reviewed:**
- Change History Audit Feed: `/history` (`HistoryPage.tsx`, `HistoryFilterBar.tsx`)
- Record Revision Timeline: `RecordHistoryDrawer.tsx`
- 3-Way Conflict Resolution: `ConflictResolutionModal.tsx`
- Tree Snapshot Checkpoints: `TreeSnapshotsCard.tsx`
- Undo & Notifications: `useUndoToast.ts`, `toast.tsx`

---

## Status of Previous Tasks
*From Phase 1.12 & Phase 1.13 review:*
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | **fixed** |
| **UX-002** | Resolve multi-touch pinch-to-zoom race condition and card long-press drag threshold | **fixed** |
| **UX-003** | Wire centralized `formatPersonName` into all UI components and fix fallback | **fixed** |
| **UX-004** | Remove prominent "Living" pill badge from `PersonDrawer` and `PersonBottomSheet` [spec-change] | **fixed** |
| **UX-005** | Update `LifeSpan` component to display genealogical dagger `†` for deceased | **fixed** |
| **UX-006** | Fix PWA offline banner top safe-area inset and consolidate install actions | **fixed** |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | **fixed** |
| **UX-008** | Parametrize family archive instance title in PWA install copy | **fixed** |

---

## Status of Phase 1.11 Tasks
| ID | Title | Status |
| :--- | :--- | :--- |
| **UX-009** | Elevate toast container above mobile bottom navigation with safe-area clearance | **fixed** |
| **UX-010** | Add missing `role="radiogroup"`, visible focus rings, and mobile layout to conflict modal | **fixed** |
| **UX-011** | Fix untranslated `{count}` token in diff toggle buttons | **fixed** |
| **UX-012** | Prevent diff values from overflowing card boundaries on mobile viewports | **fixed** |
| **UX-013** | Format raw database transaction labels with human-friendly descriptions | **fixed** |
| **UX-014** | Optimize dialog sizing and instruction placement in revision & conflict modals | **fixed** |

---

## Re-Review Findings & Verification Detail

### 1. Mobile Toast Clearance & Safe Area (`UX-009`)
- `toast.tsx:147` container positioning is now `bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] sm:bottom-4 left-1/2 -translate-x-1/2 sm:left-auto sm:right-4 sm:translate-x-0`.
- On mobile viewports (~390px), toasts and the 6-second undo toast countdown bar appear clearly above `BottomNav` without obscuring tabs or overlapping the iOS home indicator bar.

### 2. Conflict Resolution Modal Accessibility & Mobile Layout (`UX-010`, `UX-014`)
- **Semantic ARIA Grouping:** Each field's resolution choices are wrapped in an element with `role="radiogroup"` and `aria-label={item.label || item.field}`.
- **Visible Keyboard Focus:** Radio option cards include `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus`, providing unambiguous feedback during Tab navigation.
- **Responsive Stacking:** On viewports `< 640px`, the rigid 3-column table header is hidden (`hidden sm:grid sm:grid-cols-3`) and each card displays an in-card source badge (`sm:hidden text-[10px] font-semibold uppercase tracking-wide text-tertiary`), ensuring clean mobile readability.
- **Instruction Placement:** The field selection instruction banner is moved above the comparison table, guiding users before they choose field values.

### 3. Change Log Diff Toggle & Word Wrapping (`UX-011`, `UX-012`, `UX-013`)
- **Interpolated Counts:** `HistoryPage.tsx` and `RecordHistoryDrawer.tsx` now pass `{ count }` to `t('history.drawer.showDiffsBtn', { count })`, rendering formatted counts (e.g., "Show field changes (3)" / "Показать изменённые поля (3)") with zero raw `{count}` tokens.
- **Mobile Word Wrapping:** Diff value spans in `HistoryPage.tsx` include `break-words max-w-full overflow-hidden text-ellipsis`, preventing horizontal overflow when viewing long notes, places, or URLs.
- **Human-Centric Transaction Labels:** `formatTxLabel` maps dot-notated database keys (`person.add_parent`, `person.update_profile`, etc.) to clear localized action phrases in English and Russian.

### 4. Revision Dialog Sizing (`UX-014`)
- `RecordHistoryDrawer.tsx:411` sets `size="sm"` on the "Restore Version" confirmation dialog, keeping it centered and compact on mobile viewports instead of taking over the full viewport height.

---

## Positive Notes
- **Dignified Concurrency Flow:** The 3-way conflict resolution pattern gives users clear, accessible control over concurrent edits without silent overwrites.
- **Reassuring Safety Rollback:** `TreeSnapshotsCard` requires explicit typed `"ROLLBACK"` confirmation, displays record count summaries, and clearly informs administrators that a pre-rollback snapshot is created automatically.
- **Polished 6-Second Undo:** The countdown bar and accessible undo trigger provide immediate peace of mind for accidental changes.
- **Full Test Conformance:** All 105 vitest tests pass across the entire frontend suite.
