# Feature Specification: Internationalization (i18n) & Localization (l10n)

## 1. Overview & Objective (Day 0 Level Requirement)
The platform must support complete internationalization and localization from **Day 0**. All user interfaces, transactional communications (magic links, invites), date parsers, relationship calculators, and printable PDF exports must provide full parity in **English (en)** and **Russian (ru)**.

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

## 3. Directory Layout & Translation Bundles

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

## 4. Definition of Done (DoD)

- [ ] **Instant Language Switching:** Language toggle changes all visible UI text instantaneously without requiring a full page reload.
- [ ] **100% Translation Parity:** Both `en` and `ru` translation files cover all keys; no fallback warnings in development console.
- [ ] **Patronymic Support:** Individuals have an optional `patronymic` field that displays properly in person cards and profiles in both languages.
- [ ] **Russian Date Parser:** Natural phrases like *"ок. 1895"*, *"до мая 1917"*, and *"между 1914 и 1918 гг."* parse into correct GEDCOM date records.
- [ ] **Cyrillic PDF Rendering:** Printable PDF posters and tiled sheets render Russian names and text cleanly without garbled characters or missing glyphs.
- [ ] **Localized Kinship Engine:** Relationship calculator displays accurate, natural Russian kinship terms (e.g., *"Двоюродный брат"*, *"Троюродная сестра"*).
- [ ] **Localized Emails:** Magic link and invitation emails are delivered in the selected recipient language.
