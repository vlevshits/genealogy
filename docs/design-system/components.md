# Design System: Reusable UI Components

## 1. Overview & Objective
This document defines the **component library that must exist before any page is implemented**. Every page in [`docs/pages`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages) is assembled only from these components; pages never style raw HTML controls directly.

Visual values (colors, radii, shadows, fonts) come from [theme-and-tokens.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/theme-and-tokens.md). This document adds **component-level tokens**, the **component inventory**, each component's **API contract**, and **where it is used**.

### 1.1 Layers
```
frontend/src/components/
├── ui/        # Layer 1 — Primitives: Button, Input, Select, Dialog, Table, Tabs, …  (no domain knowledge)
├── form/      # Layer 2 — Form system: FormField, Form, GenDateInput, FileDropzone, …
└── domain/    # Layer 3 — Genealogy composites: PersonPicker, PlacePicker, PersonCard, EventRow, …
```
- **Layer 1** knows nothing about genealogy. **Layer 3** composes Layers 1–2 and is reused across pages (e.g. `PersonPicker` is used in quick-add, invite form, merge, face tagging, home-person setting).
- Pages import from `ui/`, `form/`, `domain/` only.

### 1.2 Technical Foundation
| Concern | Library | Notes |
| :--- | :--- | :--- |
| Accessible behavior (focus trap, keyboard, ARIA) | **Radix UI primitives** (shadcn/ui pattern: copied into `ui/`, fully owned) | Dialog, Popover, DropdownMenu, Select, Tabs, Collapsible, Accordion, Tooltip, Checkbox, Switch, RadioGroup, Slider, ScrollArea |
| Variants | `class-variance-authority` + `tailwind-merge` (`cn()` helper) | Every component exposes typed `variant` / `size` props |
| Command palette / combobox list | `cmdk` | Omnibox, Combobox, MultiSelect |
| Tables & virtualization | `@tanstack/react-table` + `@tanstack/react-virtual` | DataTable, long lists |
| Forms & validation | `react-hook-form` + `zod` | Schemas shared with API types where possible |
| Calendar grid | `react-day-picker` | Inside `GenDateInput` popover |
| Toasts | `sonner` | Wrapped by `toast()` helper |
| Icons | `lucide-react` | Single icon set, 16/20 px |

### 1.3 Component API Conventions (apply to every component)
1. **Typed props**, `className` passthrough merged with `cn()`, `ref` forwarded (React 19 `ref` prop).
2. Standard props where applicable: `size: 'sm' | 'md' | 'lg'` (default `md`), `variant`, `disabled`, `loading`, `invalid`.
3. **Controlled & uncontrolled** support for all inputs (`value`/`onChange` and `defaultValue`).
4. **No hardcoded text.** All labels, placeholders, empty states, and ARIA labels are props or come from `t()` keys in `common.json` (e.g. `common.actions.cancel`). See [00-internationalization](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md).
5. **Theme-only colors:** components use semantic tokens (`bg-surface`, `text-secondary`, `border-border`, `ring-focus`) — never raw hex — so light/dark modes work automatically.
6. **Accessibility baseline:** keyboard operable, visible focus ring, correct roles/labels, WCAG 2.2 AA contrast, touch target ≥ 40 px on mobile, respects `prefers-reduced-motion`.
7. **States** every interactive component must render: default, hover, focus-visible, active/pressed, disabled, loading (where async), invalid (inputs), read-only (inputs).

---

## 2. Component Tokens (additions to the theme)

```css
@theme {
  /* Control heights */
  --size-control-sm: 2rem;      /* 32px — toolbars, table row actions, dense filters */
  --size-control-md: 2.5rem;    /* 40px — default inputs & buttons */
  --size-control-lg: 3rem;      /* 48px — auth CTAs, mobile primary actions */

  /* Focus */
  --color-ring: var(--color-border-focus);
  --ring-width: 2px;
  --ring-offset: 2px;

  /* Status (aliases for components) */
  --color-success: var(--color-status-success);
  --color-warning: var(--color-status-warning);
  --color-danger:  var(--color-status-danger);

  /* Motion */
  --duration-fast: 120ms;       /* hover, press */
  --duration-base: 200ms;       /* popovers, tabs, collapsible */
  --duration-slow: 300ms;       /* drawers, dialogs */
  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);

  /* Layering */
  --z-canvas-overlay: 20;       /* floating tree controls */
  --z-header: 30;
  --z-dropdown: 40;             /* menus, popovers, select lists */
  --z-drawer: 50;
  --z-modal: 60;
  --z-toast: 70;
  --z-tooltip: 80;
}
```
- Overlay surfaces (menus, popovers, dialogs, drawers) use `bg-surface` + `border-border` + elevation Level 2 (menus) or Level 3 (dialogs/drawers).
- Frosted overlays (`bg-surface/90 backdrop-blur-md`) are reserved for floating canvas controls and modal backdrops.

---

## 3. Component Inventory

### 3.1 Actions

#### `Button`
| Prop | Values |
| :--- | :--- |
| `variant` | `primary` (brand fill) · `secondary` (surface + border) · `ghost` (no border, muted hover) · `danger` (danger fill) · `link` (inline text) |
| `size` | `sm` · `md` · `lg` |
| `leftIcon` / `rightIcon` | Lucide icon |
| `loading` | Shows spinner, keeps width, sets `aria-busy`, blocks clicks |
| `fullWidth`, `asChild` | `asChild` renders as router `Link` |

Rules: one `primary` button per view region; destructive actions use `danger` and always go through `ConfirmDialog`.

#### `IconButton`
Square button for icon-only actions (`⋯`, zoom `+/-`, close `✕`). **`label` prop is required** (used as `aria-label` and tooltip). Variants: `ghost` (default), `secondary`, `floating` (frosted pill for canvas controls).

#### `ButtonGroup`
Visually joined buttons (zoom bar `+ | 100% | −`, toolbar clusters). Horizontal / vertical.

#### `SegmentedControl`
Single choice among 2–6 short options, shown inline. Keyboard: arrows move selection (Radix ToggleGroup, `type="single"`).
*Used by:* tree view-mode switcher, sex selector, People/Places "List | Tree | Map", theme Light/Dark/System, tiled vs poster mode.

#### `CopyButton`
Copies a value to the clipboard and shows a "Copied" confirmation for 2 s. *Used by:* invite links, sign-in links, recovery codes, `gid`, "Copy link" in profile menu.

---

### 3.2 Form System

#### `Form` + `FormField` (wrapper used by every input)
`FormField` renders: **label**, optional **"optional"/required marker**, the control, **hint text**, **error message**, and optional **warning** (soft validation, yellow — see [15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md#23-inline-presentation)).
- Binds to `react-hook-form` via `name`; wires `id`, `aria-describedby`, `aria-invalid` automatically.
- Layouts: `vertical` (default), `horizontal` (label left — settings pages).
- `FormSection`: titled group with description (settings panels, person edit form).
- `FormActions`: sticky save/cancel bar with dirty-state tracking; triggers the unsaved-changes guard on navigation.
- Server errors (`field_errors` from the API, see [17 §2.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md#22-error-format)) map onto fields automatically by name.

#### `TextInput`
Single-line input. Props: `size`, `leftIcon`, `rightSlot` (clear button, unit, kbd hint), `invalid`, `readOnly`, `maxLength` with counter. Variants:
- `SearchInput` — search icon, clear `✕`, debounced `onSearch` (default 200 ms), `Esc` clears.
- `PasswordInput` — show/hide toggle, strength hint (min 10 chars).
- `EmailInput` — `type=email`, `autocomplete=email`.

#### `Textarea`
Auto-growing (min 3 / max 12 rows), optional character counter.

#### `MarkdownEditor`
Textarea + **Write / Preview** tabs, small toolbar (bold, italic, link, list). Preview uses the shared sanitizing `MarkdownView`. *Used by:* biography, notes.

#### `Checkbox`
Supports `indeterminate` (table "select all"). Label clickable. `CheckboxGroup` for lists of options.

#### `Switch`
On/off setting that applies immediately (public access, digest opt-in, "show photos" in PDF). Includes label + description.

#### `RadioGroup` & `RadioCardGroup`
- `RadioGroup`: classic vertical list.
- `RadioCardGroup`: each option is a card with title, description, optional icon/preview. *Used by:* GEDCOM import mode (Clean Slate / Merge), PDF tree style, PDF theme, tiled vs poster, merge field choice (§3.5 `ComparisonTable`).

#### `Select` (single choice from a fixed list)
Custom-styled list on Radix Select. For **≤ 15 options without search** (generation depth, sheet size, role, language, relationship type, event type, child relation).
Props: `options: {value, label, icon?, description?, disabled?}[]`, `placeholder`, `size`, `invalid`, option groups.
On mobile (< 768 px) it opens as a bottom sheet.

#### `Combobox` (searchable single choice, sync or async)
Text input + filtered list (cmdk).
| Prop | Purpose |
| :--- | :--- |
| `loadOptions(query)` | Async search (debounced 150 ms, aborts stale requests) |
| `renderOption` | Custom row (avatar, secondary text) |
| `onCreate(query)` | Shows *"+ Create '…'"* row (places) |
| `recent` | Options shown on empty query (recently used) |
| `clearable`, `minChars` | |
States: loading row, empty ("No results"), error with retry. Keyboard: ↑/↓, Enter, Esc, Home/End.
Also used for the event **Source** field (free text with suggestions: `allowFreeText`).

#### `MultiSelect`
Combobox that selects several values, shown as removable **chips** inside the field; Backspace removes the last chip. Sync or async options, optional `max`.
*Used by:* People Directory filters (surname, sex), media filters (people, type), linking photo to several people, history filters (users, entity types).

#### `Slider`
Single value or range, with min/max labels and current value bubble; step support. *Used by:* PDF overlap (5–20 mm). `RangeSlider` variant for birth-year filters, paired with two numeric inputs for exact entry.

#### `GenDateInput` (genealogy date picker) — key control
Free-text date entry with live parsing **plus** an optional calendar popover. Produces the `GenDate` value defined in [10 §3.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#32-genealogical-date-gendate).

```
┌──────────────────────────────────────────┐
│ ок. 1890                         [📅] [▾]│  ← text input, calendar button, modifier menu
└──────────────────────────────────────────┘
  → about 1890 · Julian ☐                     ← live parse preview / phrase warning
```
- **Text first:** accepts every EN/RU format from [00 §2.4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#24-flexible-date-parser--calendar-support) (`14.07.1923`, `abt 1890`, `между 1914 и 1918`, `до мая 1860`).
- **Live preview** below the input shows the interpreted, localized meaning; unparseable input shows *"Will be saved as text"* (phrase date) — never blocks saving.
- **Modifier menu (`▾`):** Exact / About / Estimated / Calculated / Before / After / Between / From–To. Choosing *Between* or *From–To* shows a second date field.
- **Calendar popover (`📅`):** day grid with fast **year and decade navigation** (genealogy dates span centuries — jumping from 2026 to 1850 takes ≤ 3 clicks); allows picking *year only* or *month + year* (partial dates).
- **Calendar toggle:** Julian (ст. ст.) / Gregorian (нов. ст.), shown for dates before 1918 or when Julian is detected; displays the converted date.
- Emits `{ value: GenDate, sortKey, isValid }`; parser shared with backend rules via test fixtures.
*Used by:* every event form, quick-add drawer, media "date taken", start-with-yourself form.

#### `YearInput`
Numeric year field (1000–current year) with ↑/↓ stepping. Used in range filters and generation-related settings.

#### `FileDropzone` / `Uploader`
Drag-and-drop area + "Choose files" button.
- Props: `accept`, `maxSize`, `maxFiles`, `multiple`, `onUpload`.
- Shows a per-file queue: thumbnail, name, size, progress bar, error, retry, remove.
- Rejects wrong type/size on the client with localized messages (server still validates).
*Used by:* media upload (batch), event "Attach scan", GEDCOM import (single file, up to `MAX_IMPORT_SIZE_MB`), restore backup.

#### `TypedConfirmInput`
Field that enables a dangerous action only after typing an exact phrase (e.g. tree title or `DELETE`). Used inside `ConfirmDialog` for Clean-Slate import, snapshot rollback, restore.

---

### 3.3 Overlays

#### `Dialog` (modal)
Sizes from [00-universal-layout §3.4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/00-universal-layout.md#34-modal-dialog-standards): `sm` 400 px · `md` 560 px · `xl` 800 px · `full` 95vw.
Slots: `DialogHeader` (title + description), `DialogBody` (scrollable), `DialogFooter` (actions right-aligned; primary last).
Behavior: focus trap, `Esc` closes (unless `dismissible={false}` during a running action), returns focus to trigger, frosted backdrop, scroll lock. On mobile, `md`+ dialogs become full-screen.

#### `ConfirmDialog`
Preset `sm` dialog: title, consequence text, optional **impact list** (*"Deletes 6 events, unlinks 2 families"*), optional `TypedConfirmInput`, buttons Cancel / Confirm (`danger` when destructive). Promise API: `await confirm({...})`.

#### `Drawer` (slide-over / sheet)
Side `right` (default, 480 px desktop / 100vw mobile) or `bottom` (mobile sheets). Header with title + close, scrollable body, optional sticky footer. Can be **non-modal** (tree stays interactive behind the person quick drawer).
*Used by:* person quick drawer, quick-add relative, record history, media details on mobile, filters on mobile.

#### `Popover`
Anchored floating panel for small interactive content (calendar, quick place create, column visibility). Closes on outside click / `Esc`.

#### `Tooltip`
Hover/focus hint for icon buttons and truncated text, 400 ms delay, never contains interactive content. Disabled on touch devices (use `label` text instead).

#### `DropdownMenu`
Action menu opened from a button (`⋯`, avatar menu, `More ▾`, `+ Add`).
Items: icon + label + optional shortcut (`Kbd`), separators, group labels, **danger items** (red), disabled items with reason tooltip, checkable items (column visibility), **submenus** (one level max).
Role-aware: items the user cannot use are hidden, not disabled (except when explaining *why* is helpful, e.g. "Place in use").

#### `CommandPalette` (omnibox)
`Cmd/Ctrl+K` dialog built on `cmdk`: search input, grouped results (People, Places, Media, Actions), recent items, keyboard navigation, `Enter` / `Shift+Enter` handlers. Results rendered with `PersonListItem`. See [13 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md#3-omnibox-cmdk--ctrlk).

#### `Toast`
Bottom-right (bottom-center on mobile). Types: success, error (with *Retry*), info, **undo** (action button + 6 s countdown, see [14 §3.3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md#33-restore--undo)). Max 3 visible; `aria-live="polite"`.

---

### 3.4 Navigation & Structure

| Component | Description | Used by |
| :--- | :--- | :--- |
| `AppShell` | Header + context bar + content viewport + toast region; mobile hamburger drawer | All non-auth pages |
| `AuthLayout` | Centered card layout with language/theme switches | Auth pages |
| `NavLink` | Top-nav link with active state | Header |
| `Breadcrumbs` | Hierarchy path; middle items collapse into `⋯` menu when long; last item not a link | Profile, places, media, settings |
| `PageHeader` | Title (+ count), subtitle, primary action slot, secondary actions | All list/detail pages |
| `Tabs` | Underlined tabs with optional count badge (`Media (12)`); URL-synced (`?tab=media`); horizontally scrollable on mobile | Profile, data quality, markdown editor |
| `SideNav` | Vertical sub-navigation with icons and active state; becomes `Select`/tabs on mobile | Settings |
| `Stepper` | Linear wizard steps (done / current / upcoming / error), blocks skipping ahead | GEDCOM import, tree merge |
| `Collapsible` | Single show/hide region with chevron trigger; animated height; remembers state optionally (`persistKey`) | Mini-map, advanced options (PDF tiling, GEDCOM export privacy), alternate names, place tree |
| `Accordion` | Group of collapsibles (single or multiple open) | Settings sections on mobile, help/FAQ in empty states, import warnings by category |
| `TreeView` | Expandable hierarchical list with lazy-loaded children, keyboard arrows | Places hierarchy, surnames index groups |
| `ScrollArea` | Styled scroll container | Drawers, dialog bodies, menus |
| `Divider` | Horizontal/vertical, optional label (*"or sign in with email"*) | Auth, forms |

---

### 3.5 Data Display

#### `DataTable` — key component
Generic table on TanStack Table, used by every list page ([08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md)).
| Capability | Details |
| :--- | :--- |
| Columns | Typed column defs; `cell` renderers; alignment; min/max width; sticky first column option |
| Sorting | Server-side (emits `sort` param), indicator in header, `aria-sort` |
| Selection | Row checkboxes, header select-all (indeterminate), shift-click range; selection drives `BulkActionBar` |
| Virtualization | TanStack Virtual for 50k rows; infinite loading with cursor (`onLoadMore`) |
| Column visibility | Toggle via `DropdownMenu` (checkable items); persisted per user/table key |
| Row actions | Hover-revealed `⋯` menu; whole-row click navigates (`Cmd+click` → new tab) |
| Density | `comfortable` (56 px) / `compact` (40 px) |
| States | Loading skeleton rows, empty state slot, error state with retry |
| Responsive | Below 768 px renders rows as `Card`s via a `mobileRow` renderer |
| URL state | Sorting/filters synced to query string via `useTableUrlState()` |

Companions: `FilterBar` (row of filter controls + *Clear*), `FilterChip` (active filter pill with ✕), `BulkActionBar` (sticky bottom bar: *"2 selected"* + actions).

#### `ComparisonTable` (side-by-side diff)
Two (or three) columns comparing records field by field; differing values highlighted; each row can host a radio choice (keep left / keep right / combine). Accessible: differences announced with text, not color only.
*Used by:* person merge ([15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md)), import reconciliation ([09](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/09-tree-import-merge-reconciliation.md)), edit conflict dialog ([14 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md#4-optimistic-concurrency)), history field diffs.

#### Other display components
| Component | Description | Used by |
| :--- | :--- | :--- |
| `Card` | Surface container: header/body/footer slots, `interactive` variant (hover elevation) | Dashboard widgets, mobile table rows, settings panels |
| `Avatar` | Image with crop support, initials fallback, **living-privacy silhouette**, optional gender accent ring, sizes `xs 24 · sm 32 · md 40 · lg 64 · xl 112` | Everywhere people appear |
| `AvatarGroup` | Overlapping avatars with `+N` | Media tiles (tagged people) |
| `Badge` | Small status label: `neutral`, `brand`, `accent`, `success`, `warning`, `danger`, `outline` | Role pill, Living, Private 🔒, counts, match confidence |
| `Kbd` | Keyboard shortcut hint (`⌘K`) | Header search, menus |
| `Stat` | Big number + label + optional link | Dashboard "Tree at a glance", import summary |
| `DescriptionList` | Label/value pairs (horizontal or stacked) | Drawers, media metadata, system info card |
| `Timeline` | Vertical list of dated items with colored dots/icons and connecting line; item slots for title, meta, body, actions | Person life events, history feed, anniversaries |
| `ActivityFeed` | Day-grouped list of entries (avatar + sentence + time + action) | `/history`, dashboard recent changes |
| `ProgressBar` | Determinate/indeterminate, label + percentage, step text | Imports, exports, uploads, PDF generation |
| `Spinner` | Inline loading indicator | Buttons, combobox loading row |
| `Skeleton` | Shape placeholders (`text`, `circle`, `rect`) | All loading states |
| `EmptyState` | Icon/illustration, title, description, primary action | Every list, empty tree welcome |
| `Alert` | Inline message box: `info`, `success`, `warning`, `danger`; optional title, action | Import warnings, insecure-context passkey hint, rollback guarantee |
| `Banner` | Full-width page-level notice (dismissible optional) | Offline, "record changed by Vera", public-mode notice |
| `MarkdownView` | Sanitized Markdown renderer with typography styles | Biography, notes |
| `Image` | Lazy-loaded, aspect-ratio box, blur-up placeholder, error fallback, authorized media URL helper | Media wall, avatars, lightbox |

---

### 3.6 Domain Components (genealogy-specific, reusable)

| Component | Description | Used by |
| :--- | :--- | :--- |
| `PersonName` | Formats name per locale (RU: *Фамилия Имя Отчество*; EN: *Given Patronymic Surname*), birth surname in parentheses, living-mask (*"Living Petrov"*), variants `full` / `short` / `card` | Everywhere |
| `GenDateText` | Renders a `GenDate` localized (*"ок. 1890"*, *"abt 1890"*), optional Julian/Gregorian dual display | Timeline, tables, cards |
| `LifeSpan` | *"1912 – 1989 · 77 years"* with living / unknown handling | Cards, list items, headers |
| `PlaceText` | Place name with hierarchy on hover/expand | Events, tables |
| `PersonListItem` | Avatar + name + life span + birthplace; selectable row | Omnibox, pickers, drawers, family lists |
| `PersonPicker` | `Combobox` preset searching people (cross-script search), rows = `PersonListItem`, optional *"+ Create new person"*, excludes given IDs (e.g. self when picking a parent) | Quick-add "link existing", invite linking, face tagging, home person, merge, kinship finder |
| `PlacePicker` | `Combobox` preset for places: hierarchy line per result, comma-path creation, inline *create with type + parent* popover | Event forms, media "place taken" |
| `SexSelector` | `SegmentedControl` preset (male / female / other / unknown) with lineage colors | Person forms, quick-add |
| `EventRow` | One life event: icon by type, `GenDateText`, `PlaceText`, description, source text, scan indicator, age at event; read and inline-edit modes | Profile timeline, quick drawer |
| `EventForm` | Type `Select`, `GenDateInput`, `PlacePicker`, description, source `Combobox` (free text), attach scan, private toggle | Profile edit, family events |
| `PersonCard` | Tree canvas card (avatar, name, dates, gender accent, living badge, hover quick-add `+` buttons) | Tree canvas, PDF preview |
| `FamilyGroup` | Partners + relationship type + children list with relation badges | Profile family tab, quick drawer |
| `QuickAddForm` | Minimal person form + duplicate suggestion `Alert` | Tree quick-add drawer, empty-tree "Start with yourself" |
| `PrivacyBadge` / `LivingBadge` | Lock / living indicators with tooltip explaining visibility | Cards, profiles, media |
| `MediaTile` | Thumbnail, title, date, tagged `AvatarGroup`, hover overlay | Media wall, profile media tab, dashboard |
| `FaceTagBox` | Draggable/resizable rectangle with label tooltip on an image (normalized coords) | Lightbox face tagging |
| `MapView` | MapLibre/Leaflet wrapper: markers, polylines, clustering, click-to-pick; renders nothing when maps are disabled | Life map, place detail, family map |
| `KinshipLabel` | Localized relationship text with correct gender (*"Двоюродная сестра"*) | Profile hero pill, kinship finder |
| `JobProgress` | Subscribes to job SSE; renders `ProgressBar` + step text + result/error | Import, export, PDF, backup |
| `ConflictDialog` | `Dialog` + `ComparisonTable` for `409 version_conflict` | Any edit form |

---

## 4. Usage Matrix (which pages need what)

| Component | Auth | Home | Tree | People | Profile | Media | Print | Merge | Places | History/Quality | Settings |
| :--- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| Button / IconButton | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| TextInput / FormField | ● |  | ● | ● | ● | ● | ● |  | ● |  | ● |
| GenDateInput |  |  | ● |  | ● | ● |  |  |  |  |  |
| Select | | | ● | ● | ● | ● | ● |  | ● | ● | ● |
| Combobox / PersonPicker / PlacePicker |  |  | ● | ● | ● | ● | ● | ● | ● |  | ● |
| MultiSelect / FilterBar |  |  |  | ● |  | ● |  |  | ● | ● |  |
| Checkbox / Switch / RadioCardGroup |  |  |  | ● | ● | ● | ● | ● |  |  | ● |
| Slider |  |  |  | ● |  |  | ● |  |  |  |  |
| FileDropzone |  | ● |  |  | ● | ● |  |  |  |  | ● |
| Dialog / ConfirmDialog |  | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Drawer |  |  | ● | ● | ● | ● |  |  | ● | ● |  |
| DropdownMenu | ● | ● | ● | ● | ● | ● |  |  | ● | ● | ● |
| Tabs / SideNav / Stepper |  |  |  |  | ● |  |  | ● |  | ● | ● |
| Collapsible / Accordion / TreeView |  |  | ● |  | ● |  | ● | ● | ● |  | ● |
| DataTable |  |  |  | ● |  |  |  |  | ● | ● | ● |
| ComparisonTable |  |  |  | ● |  |  |  | ● |  | ● |  |
| Timeline / ActivityFeed |  | ● |  |  | ● |  |  |  |  | ● |  |
| Toast / Alert / Banner | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |

---

## 5. Component Workbench & Testing
- **Dev-only route `/dev/ui`** (excluded from production build) renders every component in all variants, sizes, and states, in light and dark mode, EN and RU — the visual reference before page work starts.
- **Unit tests** (Vitest + Testing Library) for behavior: keyboard navigation, controlled/uncontrolled, form binding, async loading/abort in `Combobox`.
- **Accessibility tests** (`axe` via `vitest-axe`) on every workbench story.
- **`GenDateInput` parser fixtures** shared with backend tests (same EN/RU input → same `GenDate` JSON).

---

## 6. Implementation Order (before pages)
1. Tokens (§2) + `cn()` + icons + fonts.
2. `Button`, `IconButton`, `Badge`, `Spinner`, `Skeleton`, `Kbd`, `Divider`.
3. Form system: `Form`, `FormField`, `TextInput` (+ variants), `Textarea`, `Checkbox`, `Switch`, `RadioGroup`, `RadioCardGroup`, `Select`.
4. Overlays: `Dialog`, `ConfirmDialog`, `Drawer`, `Popover`, `Tooltip`, `DropdownMenu`, `Toast`.
5. `Combobox`, `MultiSelect`, `Slider`, `YearInput`, `GenDateInput`, `FileDropzone`, `TypedConfirmInput`, `CopyButton`, `SegmentedControl`.
6. Structure: `AppShell`, `AuthLayout`, `PageHeader`, `Breadcrumbs`, `Tabs`, `SideNav`, `Stepper`, `Collapsible`, `Accordion`, `TreeView`.
7. Data display: `DataTable` (+ `FilterBar`, `FilterChip`, `BulkActionBar`), `Card`, `Avatar`, `Timeline`, `ActivityFeed`, `EmptyState`, `Alert`, `Banner`, `ProgressBar`, `DescriptionList`, `Stat`, `MarkdownView`, `ComparisonTable`, `Image`.
8. Domain: `PersonName`, `GenDateText`, `LifeSpan`, `PersonListItem`, `PersonPicker`, `PlacePicker`, `SexSelector`, `EventRow`, `EventForm`, `PersonCard`, then the rest as pages need them.

---

## 7. Definition of Done (DoD)

- [ ] **Single Source:** Every control used in page specs exists in `ui/`, `form/`, or `domain/`; pages contain no custom-styled raw inputs, buttons, or menus.
- [ ] **Workbench Complete:** `/dev/ui` shows every component in all variants, sizes, and states in light/dark and EN/RU.
- [ ] **Keyboard & Screen Reader:** All components pass axe checks and are fully keyboard operable (Tab, arrows, Enter, Esc, Home/End where applicable).
- [ ] **Theme Tokens Only:** No raw hex colors in component code; switching theme updates all components without reload.
- [ ] **No Hardcoded Strings:** All component text comes from props or translation keys.
- [ ] **Mobile Behavior:** `Select` opens as bottom sheet, `Dialog` becomes full-screen, `DataTable` renders cards, touch targets ≥ 40 px.
- [ ] **GenDateInput:** Parses all documented EN/RU formats, supports partial dates, ranges, Julian toggle, and fast decade navigation; unparseable input is kept as a phrase.
- [ ] **Async Pickers:** `Combobox`/`PersonPicker`/`PlacePicker` debounce, cancel stale requests, and show loading/empty/error states.
- [ ] **DataTable Scale:** Sorting, selection, and smooth scrolling with 50k virtualized rows.
- [ ] **Form Integration:** Every input works with `react-hook-form`, displays zod and server `field_errors`, and supports soft warnings.
