# Design System: Visual Identity, Themes & Tokens

> Reusable UI controls and components built on these tokens are specified in [components.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md).

## 1. Design Philosophy: Clean, Simple & Archival
The user interface adheres to a **calm, clean, and content-centric design system** inspired by modern Scandinavian design and premium archival products (Apple, Linear, Notion, MacFamilyTree).

### Core Aesthetic Principles:
1. **High Content-to-Chrome Ratio:** The family tree, historical photographs, and life stories take center stage. UI controls are quiet, lightweight, and fade into the background until needed.
2. **Warm Archival Palette:** Pure stark `#000000` and clinical `#FFFFFF` are avoided in favor of slightly warm, museum-quality neutrals that prevent eye fatigue during lengthy research.
3. **Muted Semantic Accents:** Gender, lineage, and generation colors use sophisticated, desaturated tones (dusty indigo, muted rose, antique gold) rather than bright primary blues and pinks.
4. **Typographic Hierarchy:** Modern sans-serif for crisp UI data density, paired with an elegant, historical serif for ancestral titles and printable posters.
5. **Full Bilingual Cyrillic & Latin Support:** All selected typefaces have complete native glyph coverage for English and Russian without font substitution.

---

## 2. Color Palettes (Light & Dark Modes)

### 2.1 Surfaces & Neutrals
All colors are structured as Tailwind CSS v4 CSS variables (`oklch` / `hsl` compatible):

| Token Name | Light Mode Value | Dark Mode Value | Usage Description |
| :--- | :--- | :--- | :--- |
| `--color-bg-canvas` | `#F8F9FA` (Warm Off-White) | `#0E1116` (Deep Charcoal Slate) | Main tree canvas & page background |
| `--color-bg-surface` | `#FFFFFF` (Pure White) | `#171B22` (Elevated Obsidian) | Cards, modals, side drawers, tables |
| `--color-bg-muted` | `#F1F3F5` (Soft Gray) | `#212630` (Muted Slate) | Hover states, pill badges, input backgrounds |
| `--color-border-subtle`| `#E5E7EB` (Slate 200) | `#2E3542` (Slate 750) | Card borders, dividers, subtle separators |
| `--color-border-focus` | `#3B68A0` (Archival Blue) | `#5B8AC4` (Light Archival Blue) | Active inputs, focused card rings |
| `--color-text-primary` | `#111827` (Deep Ink) | `#F3F4F6` (Cool White) | Primary names, headings, body text |
| `--color-text-secondary`| `#4B5563` (Neutral Gray) | `#9CA3AF` (Muted Silver) | Secondary dates, places, metadata labels |
| `--color-text-tertiary`| `#9CA3AF` (Light Gray) | `#6B7280` (Dim Slate) | Placeholders, hotkey badges, timestamps |

---

### 2.2 Semantic & Brand Accents

| Token Name | Light Mode | Dark Mode | Role / Application |
| :--- | :--- | :--- | :--- |
| `--color-brand-primary` | `#2D5584` | `#4F7EB5` | Primary CTA buttons, brand badges, active links |
| `--color-brand-accent` | `#C68A34` | `#D9A04B` | Archival gold for historical milestones & badges |
| `--color-status-success`| `#2E7D32` | `#4CAF50` | Verified sources, successful imports, saved indicators |
| `--color-status-warning`| `#D97706` | `#F59E0B` | GEDCOM warnings, probable matches, unverified facts |
| `--color-status-danger` | `#C53030` | `#EF4444` | Delete actions, merge conflicts, critical alerts |

---

### 2.3 Lineage & Gender Identity Tokens
These tokens are applied to card accent borders, avatar badges, and connector paths:

| Lineage / Identity | Token Name | Light Mode Hex | Dark Mode Hex | Visual Character |
| :--- | :--- | :--- | :--- | :--- |
| **Male / Paternal** | `--color-lineage-male` | `#45678F` (Dusty Slate Indigo) | `#658BB8` | Calm, historical blue-gray tone |
| **Female / Maternal** | `--color-lineage-female` | `#9B566E` (Muted Dusty Rose) | `#BA728C` | Elegant, desaturated plum/rose |
| **Unspecified / Other**| `--color-lineage-neutral` | `#64748B` (Neutral Slate) | `#8896AB` | Neutral architectural gray |
| **Paternal Grandfather** | `--color-fan-1` | `#3B68A0` (Indigo) | `#5B8AC4` | Fan Chart Ring 1 |
| **Paternal Grandmother** | `--color-fan-2` | `#2E7D6C` (Sage Teal) | `#4AA590` | Fan Chart Ring 2 |
| **Maternal Grandfather** | `--color-fan-3` | `#C28432` (Warm Ochre) | `#DB9D4A` | Fan Chart Ring 3 |
| **Maternal Grandmother** | `--color-fan-4` | `#A85252` (Terracotta) | `#C66C6C` | Fan Chart Ring 4 |

---

## 3. Typography System

### 3.1 Primary UI Font: **Plus Jakarta Sans** or **Inter**
- **Characteristics:** Highly legible, geometric proportions, tall x-height, crisp rendering on high-DPI screens and mobile devices.
- **Coverage:** Complete Latin and Cyrillic character sets.
- **Weights Used:**
  - `400 (Regular)`: Body copy, event dates, descriptions.
  - `500 (Medium)`: Form labels, metadata chips, table text.
  - `600 (Semibold)`: Card names, modal titles, button text.
  - `700 (Bold)`: Section headings, stat metrics.

### 3.2 Display & Archival Serif: **Lora** or **Playfair Display**
- **Characteristics:** Elegant, classic book typography that confers respect and historical permanence without feeling archaic.
- **Coverage:** Full Latin and Cyrillic glyphs.
- **Usage:** Person profile header names, biographical narrative quotes, cover titles on printable PDF charts.

### 3.3 Monospace: **JetBrains Mono**
- **Usage:** Exact genealogical dates, map coordinates, GEDCOM tags, single-use recovery codes.

---

## 4. Spacing Scale & Elevation

### 4.1 Spacing (8pt Metric Grid)
- Micro spacing: `4px` (`gap-1`), `8px` (`gap-2`), `12px` (`gap-3`)
- Standard component spacing: `16px` (`p-4`), `24px` (`p-6`)
- Section & layout spacing: `32px` (`p-8`), `48px` (`py-12`)

### 4.2 Radii Scale
- `rounded-lg` (`8px`): Buttons, input fields, dropdown menus, chips.
- `rounded-xl` (`12px`): Person cards in tree view, modal dialogs, slide-over panels.
- `rounded-2xl` (`16px`): Media photo cards, profile hero containers.
- `rounded-full`: Circular avatars, floating camera control pills.

### 4.3 Shadows & Elevation
- **Level 1 (Cards & Canvas Nodes):** `0 1px 3px rgba(0, 0, 0, 0.05), 0 1px 2px rgba(0, 0, 0, 0.03)`
- **Level 2 (Hovered Cards & Dropdowns):** `0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -1px rgba(0, 0, 0, 0.04)`
- **Level 3 (Modals & Slide-overs):** `0 20px 25px -5px rgba(0, 0, 0, 0.15), 0 10px 10px -5px rgba(0, 0, 0, 0.04)`

---

## 5. Tailwind CSS v4 Theme Configuration

```css
@theme {
  --font-sans: 'Plus Jakarta Sans', 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-serif: 'Lora', 'Playfair Display', Georgia, serif;
  --font-mono: 'JetBrains Mono', monospace;

  --color-canvas: var(--color-bg-canvas);
  --color-surface: var(--color-bg-surface);
  --color-surface-muted: var(--color-bg-muted);
  --color-border: var(--color-border-subtle);
  
  --color-brand: var(--color-brand-primary);
  --color-accent: var(--color-brand-accent);

  --color-success: var(--color-status-success);
  --color-warning: var(--color-status-warning);
  --color-danger: var(--color-status-danger);
  
  --color-male: var(--color-lineage-male);
  --color-female: var(--color-lineage-female);
  --color-neutral: var(--color-lineage-neutral);

  /* Component tokens (control sizes, focus ring, motion, z-index): see components.md §2 */
}
```
