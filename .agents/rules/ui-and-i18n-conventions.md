---
trigger: always_on
---

# UI & Localization (i18n) Quality Rules

## 1. Never Include Icon Symbols or Emojis in Localization Strings
- Translation strings in `frontend/src/i18n/en.json` and `frontend/src/i18n/ru.json` must **never** include leading or trailing icon characters (`+ `, `← `, `→ `, `✓ `, `✕ `) or decorative emojis (`⚙️`, `📥`, `🔐`, `👥`, etc.).
- Icons must always be rendered exclusively by React icon components (e.g., `<Button leftIcon={<Plus className="h-4 w-4" />}>` or `<ArrowLeft />`), never baked into translation strings. Putting `+ ` in `en.json`/`ru.json` alongside `leftIcon={<Plus />}` causes duplicated icons in the UI (e.g., `+ + Пригласить родственника`).
- `frontend/tests/i18n/parity.test.ts` enforces this rule automatically on every test run.

## 2. Reusable Button Components & Modular Page Architecture
- Never use raw `<button>`, `<input>`, or `<select>` HTML elements in pages or feature panels; always use the design-system components (`Button`, `IconButton`, `ThemeToggleButton`, `LanguageToggleButton`, `TextInput`, `Select`, etc.).
- Keep page files lightweight by splitting panels, cards, dialogs, and table rows into small, single-responsibility reusable components under `frontend/src/components/<domain>/`.
