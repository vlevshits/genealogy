# UX Review — Milestone 2.4: Public Archive Navigation & Safe Return URL Redirection — 2026-10-28

## Verdict: OK WITH ISSUES
Screens reviewed:
- `/login` (`LoginCard.tsx`, `AuthPages.tsx`, `auth-layout.tsx`):
  - Public guest egress button ("Browse Public Archive" / "Перейти к публичному архиву").
  - Contextual tree return button ("Back to Family Tree" / "Вернуться к семейному древу" when `returnUrl` is `/tree` or `/persons/:id`).
  - Strict suppression of the egress button in private mode (`enable_public_access === false`).
  - Safe fallback for restricted or open-redirect return URLs.
  - Header brand logo and title link (`AuthLayout.tsx`) routing to `/`.
- Public navigation triggers:
  - Desktop header Sign In button (`AppShell.tsx`).
  - Mobile drawer Sign In button (`AppShell.tsx`).
  - Mobile bottom navigation bar Sign In tab (`BottomNav.tsx`).
- Navigation defense rails: `frontend/src/lib/navigation-utils.ts` (`sanitizeReturnUrl`, `isPublicGuestRoute`, `resolveGuestEgressTarget`, `buildLoginUrl`).

Viewports & Themes Verified:
- Desktop (1440px) and Mobile (390px / 360px).
- Light and Dark modes.
- English (`en`) and Russian (`ru`) locales.

Recordings / Visual Evidence:
- `auth_login_desktop_1440_ru_dark.png`
- `auth_login_desktop_1440_ru_light.png`
- `auth_login_desktop_1440_en_light.png`
- `auth_login_mobile_390_en_light.png`
- `auth_login_mobile_390_ru_light.png`
- `auth_login_mobile_390_ru_dark.png`
- `login_private_desktop_1440_ru.png`
- Automated test suites: `tests/components/auth-navigation.test.tsx` (23 tests passing), `tests/lib/navigation-utils.test.ts` (17 tests passing), `tests/i18n/parity.test.ts` (10 tests passing), full frontend test suite (168 tests passing).

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-022** | DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation | open |
| **UX-023** | Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management | open |
| **UX-024** | Sub-40px touch targets and raw `<button>` elements in inputs and media controls | open |
| **UX-025** | Misleading "Person record not found" alert on network or server failure | open |
| **UX-026** | Header title truncation and right action crowding on narrow mobile viewports | open |
| **UX-027** | Missing keyboard navigation (ArrowLeft / ArrowRight) in Media Lightbox Studio | open |
| **UX-028** | Theme switcher button width crowding in mobile "More" drawer | suggestion (open) |
| **UX-029** | Tree canvas pointer capture swallows left clicks on person cards; keyboard activation missing | open |
| **UX-030** | Mobile `PersonBottomSheet` missing authoring actions (Add Mother, Add Spouse, Add Event, Delete) | open |
| **UX-031** | Face tags on photograph canvas keyboard inaccessible; missing focus rings, tooltip disclosure, and delete | open |
| **UX-032** | Non-admin authorized users (Editors & Viewers) locked out of Profile and Passkeys in mobile "More" drawer | open |
| **UX-033** | Invitation creation modal lacks 'Admin' role selection option | open |
| **UX-034** | Authenticated mobile header button congestion and title squeeze | open |
| **UX-035** | Destructive "Delete Person" button in `PersonDrawer` lacks visual danger treatment | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |
| **UX-037** | Duplicate counts in `MediaUploadDialog` post-ingestion stat cards and missing pluralization resilience | fixed |
| **UX-038** | Live progress announcement missing `aria-live` and `ProgressBar` lacks accessible localized label | fixed |
| **UX-039** | Misleading imperative verb phrase in static header badge of `GedcomMediaReconcileCard` | fixed |
| **UX-040** | Raw `<button>` with sub-40px touch target and missing keyboard focus on scrollable unmatched files list | fixed |
| **UX-041** | Missing focus management on post-upload reconciliation completion | fixed |
| **UX-042** | Lack of card-level visual hierarchy between direct lineage and collateral branches | open |
| **UX-043** | Mobile toolbar vertical height reduction and icon assistance for Wide/Compact toggle | suggestion (open) |
| **UX-044** | Misleading divider label and dead action button when SMTP is disabled | fixed |
| **UX-045** | Inaccessible and skipped SMTP disabled hint for screen reader and keyboard users | fixed |
| **UX-046** | Missing keyboard focus management and long email text wrapping in `magic_sent` confirmation view | fixed |
| **UX-047** | Multi-line Emergency Recovery button icon alignment on mobile viewports | fixed |

---

## Tasks for frontend-developer

### UX-048 [minor] Header brand logo link in `AuthLayout` below 40px touch target height on mobile viewports
- **Screen / viewport / theme / lang:** `/login`, `/register`, `/setup`, `/magic-link/verify` (`AuthLayout.tsx:42-54`), mobile viewports (≤ 390px), all themes and languages.
- **Problem:** In `AuthLayout.tsx`, the brand logo header link is rendered as:
  ```tsx
  <a
    href="/"
    onClick={handleNavigateHome}
    className="flex items-center gap-2.5 rounded-lg focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-focus hover:opacity-90 transition-opacity"
    aria-label={resolvedTitle}
  >
  ```
  The interactive bounding box has no vertical padding or minimum height, resulting in a physical height of only 32px (the height of the icon container `h-8 w-8`). While this technically passes WCAG 2.2 SC 2.5.8 (which sets a 24×24px minimum), our project's senior accessibility design guidelines specify: *"Touch targets ≥ 40×40px with enough spacing"*. Older family members tapping the brand logo or instance name on small mobile touchscreens have a narrower target area than adjacent controls.
- **Evidence:** `frontend/src/components/ui/auth-layout.tsx:42-54` and mobile screenshots (`auth_login_mobile_390_ru_light.png`).
- **Proposed fix:** Add `min-h-10 py-1` (or apply the project's standard `touch-target-expand` class) to the `<a ...>` container in `AuthLayout.tsx`. This expands the clickable/tappable area to 40px height without altering visual baseline alignment with the language and theme toggle buttons.
- **Acceptance criteria:**
  1. The interactive bounding box of the brand header anchor measures at least 40px in height.
  2. Vertical baseline alignment between the brand logo/title and the right-hand toggle buttons (`LanguageToggleButton`, `ThemeToggleButton`) remains visually centered and undistorted across all viewports.
  3. Keyboard focus ring correctly outlines the expanded bounds.
- **Tags:** a11y | layout | consistency

### UX-049 [minor] Hardcoded English `aria-label="Main navigation"` on `<nav>` in `AppShell` and `BottomNav`
- **Screen / viewport / theme / lang:** All pages wrapped by `AppShell` (`app-shell.tsx:105`) and mobile bottom navigation (`bottom-nav.tsx:146`), Russian locale (`ru`), screen reader assistive technologies.
- **Problem:** Both `AppShell.tsx:105` (`<nav aria-label="Main navigation" ...>`) and `BottomNav.tsx:146` (`<nav aria-label="Main navigation" ...>`) hardcode the English string `"Main navigation"`. When an unauthenticated or authenticated user navigates using a Russian screen reader (VoiceOver, NVDA, TalkBack), the landmark navigation container is announced in English rather than the localized Russian equivalent (e.g., *"Основная навигация"* or *"Нижняя навигация"*).
- **Evidence:** `frontend/src/components/ui/app-shell.tsx:105` and `frontend/src/components/ui/bottom-nav.tsx:146`.
- **Proposed fix:**
  1. Add localized keys to `frontend/src/i18n/en.json` and `ru.json` under `nav`:
     - `nav.mainNav`: `"Main navigation"` / `"Основная навигация"`
     - `nav.bottomNav`: `"Bottom navigation"` / `"Нижняя навигация"`
  2. Update `app-shell.tsx:105` to use `aria-label={t('nav.mainNav')}`.
  3. Update `bottom-nav.tsx:146` to use `aria-label={t('nav.bottomNav')}`.
- **Acceptance criteria:**
  1. No hardcoded English strings in `aria-label` attributes within `app-shell.tsx` and `bottom-nav.tsx`.
  2. In `ru` locale, screen reader announces landmark as *"Основная навигация"* or *"Нижняя навигация"*.
  3. `tests/i18n/parity.test.ts` passes with 100% key parity.
- **Tags:** a11y | copy | consistency

---

## Positive Notes
1. **Flawless Guest Egress Experience:** The secondary egress button on `LoginCard` provides a crystal-clear, calm exit path for visitors who arrived at `/login` unintentionally or simply want to explore the tree without logging in.
2. **Context-Aware Labeling & Iconography:** Dynamically switching between *"Browse Public Archive"* (with `Compass` icon) and *"Back to Family Tree"* (with `ArrowLeft` icon) when `returnUrl` points to `/tree` or `/persons/:id` gives users precise spatial awareness of where they will land.
3. **Rigorous Defense in Depth Against Redirect Loops:** When a guest attempts to visit a restricted route (e.g., `/settings` or `/history`) and is redirected to `/login`, the egress button intelligently detects that the destination is restricted for unauthenticated visitors and falls back to `/`. This completely eliminates infinite redirect bounce loops.
4. **Strict Private Mode Confinement:** When `enable_public_access: false`, the egress button is unconditionally omitted from the DOM, guaranteeing that private instances never present false public routes to unauthenticated visitors.
5. **Open Redirect Immunity:** `sanitizeReturnUrl` comprehensively neutralizes protocol-relative exploits (`//evil.com`), pseudo-schemes (`javascript:`, `data:`), backslash confusion (`\evil.com`), CRLF injection, and internal auth redirect loops.
6. **Polished Brand Header:** The brand logo and instance name in `AuthLayout` now render as a semantic `<a>` link to `/` with visible focus rings (`focus-visible:ring-focus focus-visible:ring-2`) and hover opacity, respecting browser conventions (Ctrl/Cmd+click opens in new tab).
7. **Clean Responsive & Bilingual Balance:** All views render cleanly on desktop (1440px) and mobile (390px) viewports with zero horizontal scrolling, no truncated labels, and proper contrast across light and dark themes.
