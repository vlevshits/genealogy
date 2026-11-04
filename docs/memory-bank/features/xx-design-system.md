# Design System — Implementation Memory

**Specs:** [docs/design-system/theme-and-tokens.md](../../design-system/theme-and-tokens.md) · [docs/design-system/components.md](../../design-system/components.md) · **Status:** implemented

## 1. Summary
A comprehensive, accessible, bilingual design system built specifically for the Genealogy application. Combines Tailwind CSS v4 CSS-first semantic `@theme` design tokens in `frontend/src/index.css` with a robust library of accessible React 19 UI primitives in `frontend/src/components/ui/`. Tested interactively via the built-in `DevUiWorkbench`.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Theme Stylesheet | `frontend/src/index.css` | Tailwind v4 `@theme` block, light/dark color variables, lineage colors, focus rings |
| Class Utility | `frontend/src/lib/cn.ts` | `cn(...inputs)` merging `clsx` and `tailwind-merge` |
| Theme Context | `frontend/src/lib/theme.tsx` | `ThemeProvider`, `useTheme`, `light` / `dark` / `system` modes |
| UI Primitives | `frontend/src/components/ui/` | ~60 accessible components (see Component Inventory) |
| Domain Primitives | `frontend/src/components/domain/`| `GenDateInput`, `PlacePicker`, `RoleBadge`, `GenderIcon` |
| Shell & Layout | `frontend/src/components/ui/` | `AppShell.tsx`, `AuthLayout.tsx`, `PageHeader.tsx` |
| Sandbox Workbench | `frontend/src/pages/DevUiWorkbench.tsx` | Interactive catalog and visual regression testing workbench |
| Tests | `frontend/tests/components/` | `ui-and-a11y.test.tsx` (keyboard, focus, dark mode contrast) |

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
- **Design Tokens Only:** All styling must reference semantic tokens (`bg-surface`, `text-secondary`, `border-subtle`, `primary-600`); never use hardcoded hex values (`#ffffff`, `#121212`).
- **Strictly No Raw Elements:** Pages and features must never use unstyled HTML `<button>`, `<input>`, or `<select>` elements (`ui-and-i18n-conventions.md`).
- **Bilingual & Icon Safety:** No emojis or icon symbols may be embedded in translation strings passed to buttons or badges (`GOT-003`).

## 5. Decisions
- **`ADR-007`: Tailwind CSS v4 CSS-First Variables:** Tokens live directly in `src/index.css` rather than a JavaScript config file, enabling lightning-fast HMR and native CSS custom properties.

## 6. Gotchas & Known Limitations
- Modals and drawers must trap focus while open and restore focus to the triggering element upon closure.

_Last curated: 2026-11-04 (memory bank bootstrap)_
