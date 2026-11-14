# Design System — Implementation Memory

**Specs:** [docs/design-system/theme-and-tokens.md](../../design-system/theme-and-tokens.md) · [docs/design-system/components.md](../../design-system/components.md) · **Status:** implemented

## 1. Summary
A comprehensive, accessible, bilingual design system built specifically for the Genealogy application. Combines Tailwind CSS v4 CSS-first semantic `@theme` design tokens in `frontend/src/index.css` with a robust library of accessible React 19 UI primitives in `frontend/src/components/ui/`. Tested interactively via the built-in `DevUiWorkbench`.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Theme Stylesheet | `frontend/src/index.css`, `frontend/index.html` | Tailwind v4 `@theme` block, `:root { color-scheme: light }` / `.dark { color-scheme: dark }` + `<meta name="color-scheme" content="light dark" />`, light/dark color tokens, focus rings, `.touch-target-expand` |
| Class Utility | `frontend/src/lib/cn.ts` | `cn(...inputs)` merging `clsx` and `tailwind-merge` |
| Theme Context | `frontend/src/lib/theme.tsx` | `ThemeProvider`, `useTheme`, `light` / `dark` / `system` modes |
| UI Primitives | `frontend/src/components/ui/` | ~60 accessible components: `overlay-stack.ts` (`useModalOverlay`, `getFocusableElements`, `handleDialogFocusTrapKeyDown`, `overlayStack`, toast-exempt `inert`), `dialog.tsx` (uses/re-exports overlay helpers, `dialog-scroll-fade`), `drawer.tsx` (`useModalOverlay`, `role="dialog"`, modal scrim), `dropdown-menu.tsx` (`clampMenuHorizontalOffset`, `aria-haspopup`/`expanded`/`controls`, `itemRefs` focus), `radio-group.tsx` (`RadioChoiceRow`), `switch.tsx` (`touch-target-expand`), `toast.tsx` (`data-toast-region="true"`, `break-words`, `sm:max-w-md`), `segmented-control.tsx`, `bottom-nav.tsx` (Admin-gated More links), `access-denied-card.tsx` (`flex-wrap` header) |
| Domain Primitives | `frontend/src/components/domain/`| `GenDateInput`, `PlacePicker`, `RoleBadge`, `GenderIcon` |
| Shell & Layout | `frontend/src/components/ui/` | `AppShell.tsx` (`settings.tabs.appearance`, Admin-only nav gating), `AuthLayout.tsx`, `PageHeader.tsx` |
| Sandbox Workbench | `frontend/src/pages/DevUiWorkbench.tsx` | Interactive catalog and visual regression testing workbench |
| Tests | `frontend/tests/components/`, `frontend/e2e/` | `ui-and-a11y.test.tsx`, `person-merge-and-quality.test.tsx`, `history-concurrency.test.tsx`, `e2e/history-visualization.spec.ts` |

## 3. Component Inventory (`frontend/src/components/ui/`)

| Category | Components |
| :--- | :--- |
| **Actions** | `Button`, `IconButton`, `ButtonGroup`, `CopyButton`, `ThemeToggleButton`, `LanguageToggleButton` |
| **Form Inputs** | `TextInput` (`Input`), `Textarea`, `Select`, `MultiSelect`, `Combobox`, `Checkbox`, `RadioGroup`, `RadioChoiceRow`, `Switch`, `Slider`, `FileDropzone`, `TypedConfirmInput`, `YearInput` |
| **Surfaces & Cards**| `Card`, `Dialog` (`Modal`), `ConfirmDialog`, `Drawer`, `Popover`, `Tooltip`, `DropdownMenu` |
| **Data & Display** | `DataTable`, `ComparisonTable`, `DescriptionList`, `Avatar`, `Badge`, `RoleBadge`, `Stat`, `EmptyState`, `Timeline`, `ActivityFeed`, `MarkdownView`, `ImageView` |
| **Navigation** | `AppShell`, `SideNav`, `BottomNav`, `Breadcrumbs`, `Tabs`, `SegmentedControl`, `NavLink`, `CommandPalette`, `Stepper` |
| **Feedback** | `Alert`, `Banner`, `Toast`, `Spinner`, `ProgressBar`, `Skeleton`, `ErrorBoundary`, `AccessDeniedCard` |
| **Layout & Scroll**| `PageHeader`, `Divider`, `ScrollArea`, `Accordion`, `Collapsible`, `TreeView`, `Kbd` |

## 4. Key Invariants & Rules
- **Design Tokens Only:** All styling must reference semantic tokens (`bg-surface`, `bg-surface-subtle`, `text-secondary`, `border-subtle`, `border-strong`, `primary-600`); never use hardcoded hex values (`#ffffff`, `#121212`). Pair light/dark CSS tokens with `color-scheme: light` / `.dark { color-scheme: dark }` (`frontend/src/index.css`) and `<meta name="color-scheme" content="light dark" />` (`frontend/index.html`) so native form pickers (e.g. `<input type="date">`) match dark mode.
- **Touch Target Floor & Spacing (`UX-121`, `UX-133`):** `Button size="xs"`, `Switch`, and `SegmentedControl` (`sm`/`md`) apply `touch-target-expand`; primary CTAs measure 44px (`size="md"`), `Dialog` close and `SegmentedControl` `md` tabs meet ≥ 40px, `DropdownMenu` items enforce `min-h-10`, and wrapped chip groups maintain `>= 8px` spacing (`gap-2.5`) so expanded touch targets never overlap.
- **Mobile Multi-Button Footers:** Never apply `shrink-0` on multi-button mobile card footers; use `flex-wrap` so controls do not clip on narrow (`390px`) viewports (`GOT-022`), and wrap badge headers (`AccessDeniedCard` `flex-wrap`).
- **Strictly No Raw Elements:** Pages and features must never use unstyled HTML `<button>`, `<input>`, or `<select>` elements (`ui-and-i18n-conventions.md`).
- **Bilingual & Icon Safety:** No emojis or icon symbols may be embedded in translation strings passed to buttons or badges (`GOT-003`).

## 5. Decisions
- **`ADR-007`: Tailwind CSS v4 CSS-First Variables:** Tokens live directly in `src/index.css` rather than a JavaScript config file, enabling lightning-fast HMR and native CSS custom properties.
- **`ADR-018`: Shared `useModalOverlay` Stack (`overlay-stack.ts`):** Coordinates stacked `Dialog` and `Drawer` instances with refcounted `inert` locking and top-of-stack `Escape` handling.

## 6. Gotchas & Known Limitations
- **Stacked Overlay Focus Trap & Toast Exclusions (`UX-127`, `UX-138`, `GOT-022`, `GOT-025`):** `Dialog` and `Drawer` share `useModalOverlay` (`overlay-stack.ts`), which skips `[data-toast-region]` so Undo toasts stay clickable, handles `Escape` only on `isTopOverlay`, traps `Tab`/`Shift+Tab`, and releases `inert`/`aria-hidden` **before** restoring trigger focus so closing nested overlays works reliably. `Dialog` additionally shows `dialog-scroll-fade` when overflowing.
- **Menu Viewport Clamping & Keyboard Focus (`GOT-025`):** `DropdownMenu` (`dropdown-menu.tsx`) applies `clampMenuHorizontalOffset` inside `max-w-[calc(100vw-1rem)]`, wires `aria-haspopup`/`aria-expanded`/`aria-controls`, moves DOM focus across `itemRefs`, and restores trigger focus on `Escape`. `BottomNav` mirrors `AppShell` by hiding `/export/print`, `/history`, `/quality`, and `/settings` behind `isAdmin`.
- **Deferred Mobile Sheet Polish (`TD-QUAL-01` / `UX-137`):** `Dialog` uses full-viewport `< md` width/height dimensions (`w-full h-full md:h-auto`), while bottom-sheet docking + `pb-[env(safe-area-inset-bottom)]` polish is tracked in [technical-debt.md](../../technical-debt.md).

_Last curated: 2026-11-14 (Overlay stack, Drawer focus trap, DropdownMenu clamping/ARIA, RadioChoiceRow & UX-138–UX-150)_
