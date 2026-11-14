# Design System — Implementation Memory

**Specs:** [docs/design-system/theme-and-tokens.md](../../design-system/theme-and-tokens.md) · [docs/design-system/components.md](../../design-system/components.md) · **Status:** implemented

## 1. Summary
A comprehensive, accessible, bilingual design system built specifically for the Genealogy application. Combines Tailwind CSS v4 CSS-first semantic `@theme` design tokens in `frontend/src/index.css` with a robust library of accessible React 19 UI primitives in `frontend/src/components/ui/`. Tested interactively via the built-in `DevUiWorkbench`.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Theme Stylesheet | `frontend/src/index.css` | Tailwind v4 `@theme` block, light/dark color variables (`--color-surface-subtle`, `--color-subtle`, `--color-border-subtle`, `--color-border-strong`), lineage colors, focus rings, `.touch-target-expand` |
| Class Utility | `frontend/src/lib/cn.ts` | `cn(...inputs)` merging `clsx` and `tailwind-merge` |
| Theme Context | `frontend/src/lib/theme.tsx` | `ThemeProvider`, `useTheme`, `light` / `dark` / `system` modes |
| UI Primitives | `frontend/src/components/ui/` | ~60 accessible components: `dialog.tsx` (`getFocusableElements`, `handleDialogFocusTrapKeyDown`, `inert` skipping `[data-toast-region="true"]`, `dialog-scroll-fade`), `toast.tsx` (`data-toast-region="true"`, multi-line `break-words`, `sm:max-w-md`), `segmented-control.tsx` (`whitespace-nowrap`, `scrollIntoView`, `role="radiogroup"`/`"radio"`) |
| Domain Primitives | `frontend/src/components/domain/`| `GenDateInput`, `PlacePicker`, `RoleBadge`, `GenderIcon` |
| Shell & Layout | `frontend/src/components/ui/` | `AppShell.tsx` (`settings.tabs.appearance`), `AuthLayout.tsx`, `PageHeader.tsx` |
| Sandbox Workbench | `frontend/src/pages/DevUiWorkbench.tsx` | Interactive catalog and visual regression testing workbench |
| Tests | `frontend/tests/components/` | `ui-and-a11y.test.tsx` (keyboard, focus, dark mode contrast), `person-merge-and-quality.test.tsx` (`Dialog` focus trap & `SegmentedControl` checks) |

## 3. Component Inventory (`frontend/src/components/ui/`)

| Category | Components |
| :--- | :--- |
| **Actions** | `Button`, `IconButton`, `ButtonGroup`, `CopyButton`, `ThemeToggleButton`, `LanguageToggleButton` |
| **Form Inputs** | `TextInput` (`Input`), `Textarea`, `Select`, `MultiSelect`, `Combobox`, `Checkbox`, `RadioGroup`, `Switch`, `Slider`, `FileDropzone`, `TypedConfirmInput`, `YearInput` |
| **Surfaces & Cards**| `Card`, `Dialog` (`Modal`), `ConfirmDialog`, `Drawer`, `Popover`, `Tooltip`, `DropdownMenu` |
| **Data & Display** | `DataTable`, `ComparisonTable`, `DescriptionList`, `Avatar`, `Badge`, `RoleBadge`, `Stat`, `EmptyState`, `Timeline`, `ActivityFeed`, `MarkdownView`, `ImageView` |
| **Navigation** | `AppShell`, `SideNav`, `BottomNav`, `Breadcrumbs`, `Tabs`, `SegmentedControl`, `NavLink`, `CommandPalette`, `Stepper` |
| **Feedback** | `Alert`, `Banner`, `Toast`, `Spinner`, `ProgressBar`, `Skeleton`, `ErrorBoundary`, `AccessDeniedCard` |
| **Layout & Scroll**| `PageHeader`, `Divider`, `ScrollArea`, `Accordion`, `Collapsible`, `TreeView`, `Kbd` |

## 4. Key Invariants & Rules
- **Design Tokens Only:** All styling must reference semantic tokens (`bg-surface`, `bg-surface-subtle`, `text-secondary`, `border-subtle`, `border-strong`, `primary-600`); never use hardcoded hex values (`#ffffff`, `#121212`).
- **Touch Target Floor & Spacing (`UX-121`, `UX-133`):** `Button size="xs"` and `SegmentedControl` (`sm`/`md`) apply `touch-target-expand`; primary CTAs measure 44px (`size="md"`), `Dialog` close and `SegmentedControl` `md` tabs meet ≥ 40px, and wrapped chip groups maintain `>= 8px` spacing (`gap-2.5`) so expanded touch targets never overlap.
- **Mobile Multi-Button Footers:** Never apply `shrink-0` on multi-button mobile card footers; use `flex-wrap` so controls do not clip on narrow (`390px`) viewports (`GOT-022`).
- **Strictly No Raw Elements:** Pages and features must never use unstyled HTML `<button>`, `<input>`, or `<select>` elements (`ui-and-i18n-conventions.md`).
- **Bilingual & Icon Safety:** No emojis or icon symbols may be embedded in translation strings passed to buttons or badges (`GOT-003`).

## 5. Decisions
- **`ADR-007`: Tailwind CSS v4 CSS-First Variables:** Tokens live directly in `src/index.css` rather than a JavaScript config file, enabling lightning-fast HMR and native CSS custom properties.

## 6. Gotchas & Known Limitations
- **Modal Focus Trap & Toast Exclusions (`UX-127`, `GOT-022`):** `Dialog` moves focus into `[role="dialog"]`, marks outer DOM subtrees `inert` + `aria-hidden="true"` *except* `[data-toast-region="true"]` so Undo toasts stay clickable, traps `Tab`/`Shift+Tab` (`handleDialogFocusTrapKeyDown`), shows `dialog-scroll-fade` when overflowing, and restores focus to the trigger on `Escape` / close.
- **Deferred Mobile Sheet Polish (`TD-QUAL-01` / `UX-137`):** `Dialog` uses full-viewport `< md` width/height dimensions (`w-full h-full md:h-auto`), while bottom-sheet docking + `pb-[env(safe-area-inset-bottom)]` polish is tracked in [technical-debt.md](../../technical-debt.md).

_Last curated: 2026-11-14 (Dialog focus trap/scroll fade, Toast multi-line wrapping, SegmentedControl scroll/wrap & UX-119/127-137)_
