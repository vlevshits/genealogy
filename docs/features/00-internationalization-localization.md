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

#### Magic Link Email Template
- **English (`magic_link_en.html` / text):**
  - **Subject:** `Sign in to {{tree_title}}`
  - **Body Content:**
    > Hello {{user_name}},
    >
    > Click the button below to sign in to {{tree_title}}. This link is valid for **15 minutes** and can only be used once.
    >
    > **[ Sign In to Family Archive ]** ({{magic_link_url}})
    >
    > If you did not request this link, you can safely ignore this email. Your account remains secure.
- **Russian (`magic_link_ru.html` / text):**
  - **Subject:** `Вход в {{tree_title}}`
  - **Body Content:**
    > Здравствуйте, {{user_name}}!
    >
    > Нажмите кнопку ниже для входа в {{tree_title}}. Ссылка действительна в течение **15 минут** и может быть использована только один раз.
    >
    > **[ Войти в семейный архив ]** ({{magic_link_url}})
    >
    > Если вы не запрашивали это письмо, просто проигнорируйте его. Ваша учётная запись в безопасности.

#### Family Invitation Email Template
- **English (`invite_en.html` / text):**
  - **Subject:** `Invitation to join {{tree_title}}`
  - **Body Content:**
    > Hello,
    >
    > {{inviter_name}} has invited you to view and preserve family history in {{tree_title}} as an **{{role}}**.
    >
    > **[ Accept Invitation & Create Account ]** ({{invite_url}})
    >
    > This invitation link is valid for 7 days.
- **Russian (`invite_ru.html` / text):**
  - **Subject:** `Приглашение в {{tree_title}}`
  - **Body Content:**
    > Здравствуйте!
    >
    > {{inviter_name}} приглашает вас исследовать и сохранять семейную историю в архиве {{tree_title}} с правами **{{role}}**.
    >
    > **[ Принять приглашение и войти ]** ({{invite_url}})
    >
    > Ссылка-приглашение действительна в течение 7 дней.

---

### 2.7 Error Code Localization Architecture (API & Network Failures)
All error messages presented to the user—whether arising from browser network drops, HTTP error responses, or structured backend API failures—must be fully localized into the active interface language (`en` or `ru`).

#### Core Zero-Raw-Error Principle:
The application must **never** display raw unlocalized English strings (e.g. `"Failed to fetch"`, `"CSRF or Origin header validation failed"`, `"Not authenticated"`, or backend developer messages) when running in Russian or non-English locales.

#### Error Resolution Architecture:
1. **Network & Connection Failures:**
   - Network drops, offline states, DNS failures, or browser `fetch()` rejections (`TypeError: Failed to fetch`) map to `errors.network_error`:
     - English: *"Unable to connect to the server. Please check your network connection and try again."*
     - Russian: *"Не удалось связаться с сервером. Пожалуйста, проверьте подключение к сети и попробуйте снова."*
2. **Structured Backend Errors (`{ error: { code, message, details, field_errors } }`):**
   - The backend `message` property is strictly for developer logs and telemetry; the frontend must never render it directly in localized UI.
   - The frontend translates the error by resolving `errors.<code_in_snake_case>` from the active translation dictionary.
3. **HTTP Status Fallback:**
   - If an error lacks a recognized machine code, the client resolves the HTTP status code (e.g. `errors.401`, `errors.403`, `errors.404`, `errors.409`, `errors.422`, `errors.429`, `errors.500`, `errors.503`).
4. **Generic Fallback:**
   - If neither code nor status resolves to a known translation key, the client renders `errors.unknown` (English: *"An unexpected error occurred. Please try again."* / Russian: *"Произошла непредвиденная ошибка. Пожалуйста, попробуйте снова."*).

#### Standard Error Catalog (`errors.*` Dictionary):

| Error Code Key | HTTP | Meaning / Trigger Scenario | English UI Translation | Russian UI Translation |
| :--- | :---: | :--- | :--- | :--- |
| `network_error` | — | Connection drop, backend unreachable, `Failed to fetch` | Unable to connect to the server. Please check your network connection. | Не удалось связаться с сервером. Пожалуйста, проверьте подключение к сети. |
| `not_authenticated` | 401 | Unauthenticated access to protected endpoint | You need to sign in to perform this action. | Для выполнения этого действия необходимо войти в систему. |
| `session_expired` | 401 | User session token expired or invalidated | Your session has expired. Please sign in again. | Срок действия сессии истёк. Пожалуйста, войдите снова. |
| `invalid_credentials` | 401 | Incorrect password during sign-in | Invalid email or password. | Неверный адрес электронной почты или пароль. |
| `passkey_replay_detected`| 401 | WebAuthn signature replay or counter regression | Authentication security check failed. Please try signing in again. | Проверка безопасности аутентификации не пройдена. Попробуйте войти снова. |
| `invalid_or_expired_challenge` | 401 | WebAuthn challenge timeout or mismatch | Authentication challenge expired. Please try again. | Время ожидания подтверждения истекло. Пожалуйста, попробуйте снова. |
| `invalid_or_expired_invite` | 400/404 | Invitation token invalid, revoked, or >7 days old | This invitation link is invalid or has expired. | Эта ссылка-приглашение недействительна или срок её действия истёк. |
| `invalid_or_expired_token` | 400/404 | Magic link or recovery code expired or invalid | The link or code is invalid or has expired. | Ссылка или код недействительны или срок их действия истёк. |
| `last_auth_method` | 400 | Attempted to delete only remaining sign-in method | Cannot remove your only sign-in method. | Нельзя удалить единственный способ входа в систему. |
| `sole_admin_protected` | 403 | Attempted to demote or deactivate sole remaining Admin | Cannot deactivate or demote the only administrator. | Нельзя деактивировать или понизить единственного администратора. |
| `password_auth_disabled` | 403 | Password login attempted when password auth is off | Password authentication is disabled on this instance. | Вход по паролю отключён в этой системе. |
| `csrf_validation_failed` | 403 | Missing `X-Requested-With` header or invalid `Origin` | Security validation failed. Please refresh the page. | Проверка безопасности запроса не пройдена. Пожалуйста, обновите страницу. |
| `forbidden` | 403 | Role lacks required permissions | You do not have permission to perform this action. | У вас недостаточно прав для выполнения этого действия. |
| `settings_locked_from_env` | 403 | Attempt to modify setting locked by `.env` | This setting is managed by environment configuration and cannot be changed here. | Эта настройка заблокирована переменными окружения и не может быть изменена здесь. |
| `not_found` | 404 | Generic resource not found | The requested item was not found. | Запрошенный объект не найден. |
| `person_not_found` | 404 | Individual not found or deleted | Person not found or has been removed. | Запись о персоне не найдена или была удалена. |
| `setup_already_completed` | 400 | Initial setup already executed | Instance setup has already been completed. | Первоначальная настройка системы уже завершена. |
| `already_onboarded` | 400 | Empty tree onboarding already completed | Tree onboarding has already been completed. | Первоначальное заполнение древа уже завершено. |
| `version_conflict` | 409 | Concurrent update collision (optimistic lock) | This record was modified by another user. Please refresh and review. | Эта запись была изменена другим пользователем. Пожалуйста, обновите страницу. |
| `cycle_detected` | 409 | Relationship would form impossible ancestor loop | This relationship would create an impossible ancestor cycle. | Эта связь создаёт циклическую зависимость предков и потомков. |
| `place_in_use` | 409 | Place cannot be deleted because events reference it | This place cannot be deleted because it is associated with life events. | Это место нельзя удалить, так как к нему привязаны исторические события. |
| `rate_limited` | 429 | Rate limit exceeded | Too many requests. Please wait a moment and try again. | Слишком много запросов. Пожалуйста, подождите немного и повторите попытку. |
| `account_locked` | 429 | 5 failed login attempts → 15-minute lockout | Account is temporarily locked due to failed attempts. Try again in 15 minutes. | Учётная запись временно заблокирована из-за частых попыток входа. Попробуйте через 15 минут. |
| `smtp_not_configured` | 503 | Magic link requested when SMTP is disabled or unconfigured | Email delivery is not configured on this server. Please sign in with your passkey, password, or contact an administrator. | Отправка писем не настроена на этом сервере. Пожалуйста, используйте ключ доступа, пароль или обратитесь к администратору. |
| `database_error` | 500 | Database query or connection fault | A database error occurred. Please try again shortly. | Произошла ошибка базы данных. Пожалуйста, повторите попытку позже. |
| `internal_error` | 500 | Unexpected server error | An internal server error occurred. | Произошла внутренняя ошибка сервера. |
| `unknown` | — | Fallback for unmapped errors | An unexpected error occurred. Please try again. | Произошла непредвиденная ошибка. Пожалуйста, попробуйте снова. |

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
- [ ] **Complete Error Code Localization:** All standard error codes (`network_error`, `not_authenticated`, `session_expired`, `invalid_credentials`, `passkey_replay_detected`, `csrf_validation_failed`, `version_conflict`, `rate_limited`, `account_locked`, `database_error`, `internal_error`, `unknown`, etc.) are defined with 100% key parity under `errors.*` in both `en.json` and `ru.json`.
- [ ] **Zero Raw Unlocalized Errors in UI:** Error toasts, inline form banners, and modal dialogs never display raw unlocalized English strings (e.g. "Failed to fetch", "Not authenticated", "CSRF or Origin header validation failed") when the user is in Russian locale.
