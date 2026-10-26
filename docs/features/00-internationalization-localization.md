# Feature Specification: Internationalization (i18n) & Localization (l10n)

## 1. Overview & Objective (Day 0 Level Requirement)
The application must support complete internationalization and localization from **Day 0**. All user interfaces, transactional communications (magic links, invites), date parsers, relationship calculators, and printable PDF exports must provide full parity in **English (en)** and **Russian (ru)**.

In genealogy, localization is not merely translating labels—it fundamentally shapes how names (e.g. Slavic **Patronymics / Отчество**), historical dates (Julian vs. Gregorian calendars), and complex kinship terms are stored, computed, and displayed.

---

## 2. Core Functional Requirements

### 2.1 UI Localization (React & i18next)
- **Languages Supported:**
  - `en` — English (Default fallback)
  - `ru` — Русский (Russian)
- **Automatic Language Detection:**
  - On first visit, detects browser preference via `navigator.language` (e.g. `ru-RU` $\to$ `ru`).
  - Allows manual switching via a language selector in the navbar / user settings.
  - Persists preference in `localStorage` and syncs with the authenticated user profile.
- **Strict Zero-Hardcoding Rule:**
  - All UI text, tooltips, validation messages, aria-labels, and modal dialogs must use translation keys via `useTranslation()` (e.g. `t('auth.login_with_passkey')`, `t('tree.add_father')`).
  - CI lint rule ensures no untranslated hardcoded strings in JSX components.
- **No Icons or Glyphs in Translation Strings:**
  - Translation strings in `en.json` and `ru.json` must contain pure text only and must **never** include leading/trailing icon characters (`+ `, `← `, `→ `, `✓ `, `✕ `) or decorative emojis (`⚙️`, `📥`, `🔐`, `👥`).
  - Visual icons must be rendered exclusively via component props (e.g., `leftIcon={<Plus />}` or `leftIcon={<ArrowLeft />}`) so icons are never duplicated.
  - Automated i18n parity tests (`frontend/tests/i18n/parity.test.ts`) must fail if any translation string starts with icon glyphs (`+ `, `← `, `→ `) or contains decorative emojis.

### 2.2 Slavic Genealogy Name Structure (Patronymics / Отчество)
In Russian and post-Soviet genealogy records, names follow a tripartite structure:
- **Given Name (Имя):** e.g., *Александр*
- **Patronymic (Отчество):** e.g., *Иванович* (son of Иван) / *Ивановна* (daughter of Иван)
- **Surname (Фамилия):** e.g., *Кузнецов*
- **Maiden / Birth Name (Девичья фамилия):** e.g., *Морозова*

#### Schema & GEDCOM Mapping:
- Schema field `patronymic VARCHAR(60) NULL` added to `individuals` table.
- GEDCOM tag mapping: `_PATR` or `NAME Given /Surname/` with standard extension parsing.
- Display formatter:
  - English locale: `Given Name [Patronymic] Surname` (e.g. *Aleksandr Ivanovich Kuznetsov*)
  - Russian locale: `Фамилия Имя Отчество` (e.g. *Кузнецов Александр Иванович*) or `Имя Отчество Фамилия` in cards.

---

### 2.3 Bilingual Kinship Calculator (Степени Родства)
Genealogical kinship terms differ substantially between languages:

| Relationship | English Term | Russian Term (Male / Female) |
| :--- | :--- | :--- |
| **Father / Mother** | Father / Mother | Отец / Мать |
| **Grandfather / Grandmother** | Grandfather / Grandmother | Дедушка / Бабушка |
| **Great-Grandfather** | Great-Grandfather | Прадедушка / Прабабушка |
| **Sibling** | Brother / Sister | Брат / Сестра |
| **First Cousin** | First Cousin | Двоюродный брат / Двоюродная сестра |
| **Second Cousin** | Second Cousin | Троюродный брат / Троюродная сестра |
| **Uncle / Aunt** | Uncle / Aunt | Дядя / Тётя |
| **Great-Uncle / Great-Aunt** | Great-Uncle / Great-Aunt | Двоюродный дедушка / Двоюродная бабушка |
| **Nephew / Niece** | Nephew / Niece | Племянник / Племянница |
| **Cousin Once Removed** | First cousin once removed | Двоюродный племянник / Двоюродная тётя |
| **Father/Mother-in-law** | Father-in-law / Mother-in-law | Свёкор / Свекровь (мужа), Тесть / Тёща (жены) |

The kinship algorithm must return localized strings with correct gender agreement.

---

### 2.4 Flexible Date Parser & Calendar Support
Genealogical dates in Russian records often use historical Russian abbreviations:

| English Input | Russian Input | Standardized Meaning / GEDCOM |
| :--- | :--- | :--- |
| `14 Jul 1923` | `14 июля 1923 г.` / `14.07.1923` | Exact Date (`14 JUL 1923`) |
| `about 1890` / `abt 1890` | `около 1890 г.` / `ок. 1890` | Approximate (`ABT 1890`) |
| `before 1917` / `bef 1917` | `до 1917 г.` | Upper bound (`BEF 1917`) |
| `after 1945` / `aft 1945` | `после 1945 г.` | Lower bound (`AFT 1945`) |
| `between 1914 and 1918` | `между 1914 и 1918 гг.` | Range (`BET 1914 AND 1918`) |
| `calculated 1905` | `вычислено 1905 г.` | Calculated (`CAL 1905`) |

#### Historical Calendar Toggle (Julian vs. Gregorian):
- Russian civil records transitioned from the **Julian Calendar (Old Style)** to the **Gregorian Calendar (New Style)** on **February 14, 1918**.
- Date editor allows marking dates as `(ст. ст.)` (Old Style) or `(нов. ст.)` (New Style) with automatic +13 day offset calculation for 20th century records.

---

### 2.5 Bilingual Printable PDF Generation
- PDF generation engine dynamically selects language based on user's export preferences.
- **Font Subsetting & Cyrillic Support:**
  - PDF layout engine bundles open-source Unicode fonts with full Cyrillic support (e.g. **Inter**, **PT Sans**, **Playfair Display Cyrillic**, or **Noto Serif**).
  - Guarantees zero missing glyphs or "tofu" squares when rendering names like *Щеглов, Пётр Фёдорович*.
- All headers, banners, sheet labels (*"Лист 2 из 9 [Ряд 1, Колонка 2]"*), cut markers, and dates format in Russian when exported in Russian.

---

### 2.6 Backend Email & Notification Templates
Transactional emails (invitations, magic links) are rendered bilingual based on the recipient's language preference or the instance default:
- English: *"Sign in to Levshits Family Tree — click the link below"*
- Russian: *"Вход в Семейное Древо Левшиц — нажмите ссылку ниже"*

---

## 3. UI Copywriting, Tone of Voice & Anti-Fluff Standards

### 3.1 Core Anti-Fluff Principle
The user interface is a private, enduring family archive, intended to be used by multiple generations—from curious children to non-technical grandparents. It is **never** a developer portfolio, a SaaS marketing pitch, or a tech demo. All UI copy must be calm, functional, respectful, and strictly focused on family history, lineage, and archival preservation.

### 3.2 Forbidden Marketing & Promotional Buzzwords
User-facing strings, page headings, buttons, badges, empty states, and dialogs must **never** contain marketing hype or promotional SaaS jargon. Prohibited phrases include, but are not limited to:
- *"Next-Gen"* / *"Next-Generation"* (e.g. forbidden: *"Next-Gen Family Archive Platform"*)
- *"Cutting-edge"*, *"State-of-the-art"*, *"Revolutionary"*, *"World-class"*
- *"Blazing-fast"*, *"High-speed"*, *"Ultra-fast"*
- *"Enterprise-grade"*, *"Industry-leading"*
- *"All-in-one platform"*, *"Unleash"*, *"Supercharge"*
- Promotional feature badges (e.g. *"Zero-Cloud Dependency"* as a boast; if conveying privacy, state simply: *"Stored locally on your server"* or *"Private family archive"*).

### 3.3 Strict Invisibility of the Tech Stack
The underlying software architecture and technology stack are internal engineering details. Non-technical relatives must never see framework names or developer bragging on web pages:
- **Strictly forbidden in UI:** Mentions of `Rust`, `Axum`, `React`, `React 19`, `PostgreSQL`, `Tailwind`, `Tailwind CSS v4`, `Docker`, `Nginx`, `SQLx`, `Tokio`, `Vite`, etc.
- **No Stack Badges or Footers:** Footers must never display tech slogans like `"Axum + React 19 + Tailwind v4 + PostgreSQL 16"`.
- **No Developer Tool Links in User UI:** Interactive OpenAPI explorer (`/swagger-ui`) or component workbench (`/dev/ui`) must never appear in end-user navigation, cards, or footers.

### 3.4 Permitted Exceptions & Functional Terminology
1. **Genealogical & Archival Formats:** Standard industry file format names necessary for user action and export are permitted (e.g., `GEDCOM 5.5.1`, `GEDCOM 7.0`, `GEDZIP`, `PDF`, `CSV`, `ZIP`).
2. **User-Facing Biometrics & Authentication:** Standard operating-system authentication terms are permitted for clarity (e.g., `Passkey`, `Touch ID`, `Face ID`, `Windows Hello`, `Security Key`).
3. **Admin Settings Operational Status:** In Settings (`/settings` or `/settings/system`), operational status must use plain, functional terminology:
   - Use **"Database: Connected"** / **"База данных: Подключена"** instead of *"PostgreSQL 16 (pg_trgm, unaccent)"*.
   - Use **"Server: Healthy"** / **"Сервер: В норме"** instead of *"Axum 0.8 (Rust)"*.
   - Use **"Application Version: v0.1.0"** / **"Версия приложения: v0.1.0"** instead of framework versions.
   - Use **"Archive Storage: 1.2 GB used"** / **"Хранилище: 1.2 ГБ занято"** instead of internal volume labels.

### 3.5 Microcopy Transformation Reference (EN / RU)

| Context | ❌ Forbidden (Marketing / Tech Bragging) | ✅ Required (Clean, Archival & Functional) |
| :--- | :--- | :--- |
| **Hero / Title Badge** | `Next-Gen Family Archive Platform` | `[Tree Title]` (e.g. *Levshits Family Archive*) / *Семейный архив Левшиц* |
| **App Subtitle** | `Built with high-speed Rust Axum API backend, React 19 canvas visualizations, and single-instance Docker deployment.` | `A private home for your family history.` / *Частный архив истории вашей семьи.* |
| **App Footer** | `Axum + React 19 + Tailwind v4 + PostgreSQL 16` | `[Tree Title] © 2026 • Private Family Archive` / *Семейный архив © 2026* |
| **Status Card Title** | `Single-Instance Microservices Verification` | `System Status` / *Статус системы* |
| **Backend Label** | `Axum 0.8 (Rust)` | `API Server: Online` / *Сервер API: Подключён* |
| **Database Label** | `PostgreSQL: PostgreSQL 16 (pg_trgm, unaccent)` | `Database: Connected` / *База данных: Подключена* |
| **Backup Download CTA**| `Download Postgres dump + media assets` | `Download Full Archive (Database & Media)` / *Скачать полный архив (База данных и медиа)* |
| **Empty Onboarding** | `Experience the power of next-gen genealogy visualization` | `Let's start your family tree: start with yourself or import a file.` / *Начните составление семейного древа: добавьте себя или импортируйте файл.* |

---

## 4. Directory Layout & Translation Bundles

```
frontend/
├── src/
│   ├── locales/
│   │   ├── en/
│   │   │   ├── common.json
│   │   │   ├── auth.json
│   │   │   ├── tree.json
│   │   │   ├── person.json
│   │   │   ├── media.json
│   │   │   └── pdf.json
│   │   └── ru/
│   │       ├── common.json
│   │       ├── auth.json
│   │       ├── tree.json
│   │       ├── person.json
│   │       ├── media.json
│   │       └── pdf.json
│   └── i18n.ts
backend/
└── resources/
    └── emails/
        ├── magic_link_en.html
        ├── magic_link_ru.html
        ├── invite_en.html
        └── invite_ru.html
```

---

## 5. Definition of Done (DoD)

- [ ] **Instant Language Switching:** Language toggle changes all visible UI text instantaneously without requiring a full page reload.
- [ ] **100% Translation Parity:** Both `en` and `ru` translation files cover all keys; no fallback warnings in development console.
- [ ] **Zero Icon Glyphs in Translation Strings:** No translation value in en.json or ru.json starts with icon symbols (+ , ← , → ) or contains decorative emojis that duplicate rendered SVG icons.
- [ ] **Anti-Fluff & Tech-Agnostic UI Copy:** All user-facing web pages, cards, tooltips, dialogs, footers, and translation files (`en.json`, `ru.json`) are completely free of marketing buzzwords (*"next-gen"*, *"cutting-edge"*, SaaS promotional slogans) and tech stack mentions (*Rust*, *Axum*, *React*, *PostgreSQL*, *Tailwind*, etc.), using clean, functional, family-centered copy.
- [ ] **Patronymic Support:** Individuals have an optional `patronymic` field that displays properly in person cards and profiles in both languages.
- [ ] **Russian Date Parser:** Natural phrases like *"ок. 1895"*, *"до мая 1917"*, and *"между 1914 и 1918 гг."* parse into correct GEDCOM date records.
- [ ] **Cyrillic PDF Rendering:** Printable PDF posters and tiled sheets render Russian names and text cleanly without garbled characters or missing glyphs.
- [ ] **Localized Kinship Engine:** Relationship calculator displays accurate, natural Russian kinship terms (e.g., *"Двоюродный брат"*, *"Троюродная сестра"*).
- [ ] **Localized Emails:** Magic link and invitation emails are delivered in the selected recipient language.
