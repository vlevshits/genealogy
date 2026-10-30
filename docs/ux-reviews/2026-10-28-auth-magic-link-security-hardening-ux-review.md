# UX Review — Milestone 2.3: Magic Link Security Hardening & SMTP Configuration States — 2026-10-28

## Verdict: OK
Screens reviewed:
- `/login` (`LoginCard.tsx`, `AuthPages.tsx`, `auth-layout.tsx`):
  - Passkey primary CTA, password fallback, magic link request, and emergency recovery views.
  - SMTP enabled state vs SMTP disabled state (`smtpEnabled === false`).
  - Confirmation view (`magic_sent`) with zero token exposure.
- Viewports verified: Desktop (1440px), Mobile (390px, 360px).
- Themes verified: Light and Dark mode.
- Locales verified: English (`en`) and Russian (`ru`).

Recordings / Visual Evidence:
- `login_desktop_1440_ru_dark.png`
- `login_desktop_1440_ru_light_actual.png`
- `login_desktop_1440_en_light.png`
- `login_desktop_1440_en_dark.png`
- `login_mobile_390_en_light.png`
- `login_mobile_390_ru_light.png`
- `login_mobile_390_en_dark.png`
- Automated test suite: `tests/components/auth-and-modular-ui.test.tsx` (all 9 tests passing), `tests/i18n/parity.test.ts` (all 10 tests passing).

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
| **UX-037** | Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px) | open |
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | open |
| **UX-039** | Low non-text contrast (< 3:1) for collateral connector edges in dark mode | open |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile | open |
| **UX-041** | Missing tooltip / explanatory subtitle for "Wide tree" / "Compact tree" toggle | open |
| **UX-042** | Lack of card-level visual hierarchy between direct lineage and collateral branches | open |
| **UX-043** | Mobile toolbar vertical height reduction and icon assistance for Wide/Compact toggle | suggestion (open) |
| **UX-044** | Misleading divider label and dead action button when SMTP is disabled | **fixed** (verified: dynamic divider, dead button removed) |
| **UX-045** | Inaccessible and skipped SMTP disabled hint for screen reader and keyboard users | **fixed** (verified: accessible Alert role="status" tabIndex={0} linked via aria-describedby) |
| **UX-046** | Missing keyboard focus management and long email text wrapping in `magic_sent` confirmation view | **fixed** (verified: focus management into/out of view, break-all, bold font-medium email) |
| **UX-047** | Multi-line Emergency Recovery button icon alignment on mobile viewports | **fixed** (verified: items-start leading-snug icon aligned with line 1 on 390px EN/RU) |

---

## Tasks for frontend-developer (All Fixed & Verified)

### UX-044 [major] [FIXED] Misleading divider label and dead action button when SMTP is disabled (`smtpEnabled === false`)
- **Status:** **FIXED** (Verified in `LoginCard.tsx:265-271, 347-360` and browser screenshots).
- **Screen / viewport / theme / lang:** `/login` (`LoginCard.tsx`), all viewports, light & dark themes, EN & RU.
- **Problem:**
  1. In `LoginCard.tsx`, the divider above the form fields was hardcoded to `<Divider label={t('auth.login.orEmailDivider')} />` ("or sign in with email link" / "или получить ссылку на почту"). When `smtpEnabled === false`, email delivery is not configured on the instance. The divider literally invited the user to sign in with an email link, only for the user to discover the "Send Magic Link" button below was disabled.
  2. The "Send Magic Link" button was rendered as a permanently disabled full-width button when SMTP was unconfigured, creating visual clutter and confusing users.
- **Resolution:**
  - Divider dynamically checks `authPasswordEnabled` and `smtpEnabled`. When SMTP is disabled and password is enabled, it renders `t('auth.login.orPasswordDivider')` ("or sign in with password" / "или войти по паролю"). The email link label is never shown when SMTP is disabled.
  - The "Send Magic Link" button is completely removed when `smtpEnabled === false`, eliminating the dead control.
- **Tags:** hierarchy, copy, consistency, layout

---

### UX-045 [major] [FIXED] Inaccessible and skipped SMTP disabled hint for screen reader and keyboard users
- **Status:** **FIXED** (Verified in `LoginCard.tsx:315, 362-371, 375-384`, `en.json`, `ru.json`).
- **Screen / viewport / theme / lang:** `/login` (`LoginCard.tsx`), all viewports, all themes, EN & RU.
- **Problem:**
  1. Hint text was rendered as a plain unassociated paragraph below a disabled button.
  2. Keyboard tab navigation skipped past the notice.
  3. Notice lacked accessible role (`role="status"`) and was not linked via `aria-describedby`.
  4. Inaccurate copy when `authPasswordEnabled === false` (told user to use a password when passwords were not supported).
- **Resolution:**
  - Implemented `<Alert id="smtp-disabled-hint" variant="info" role="status" tabIndex={0}>` with visible focus ring.
  - Programmatically associated with `EmailInput` via `aria-describedby={!smtpEnabled ? 'smtp-disabled-hint' : undefined}`.
  - Added conditional token `t('auth.login.smtpDisabledNoPasswordHint')` ("Email sign-in is not configured on this server. Sign in with a passkey or emergency recovery code." / "Вход по ссылке недоступен, так как почта не настроена на сервере. Войдите с помощью ключа доступа или резервного кода.") when `authPasswordEnabled === false`.
- **Tags:** a11y, copy, states

---

### UX-046 [minor] [FIXED] Missing keyboard focus management and long email text wrapping in `magic_sent` confirmation view
- **Status:** **FIXED** (Verified in `LoginCard.tsx:67-82, 166-193`).
- **Screen / viewport / theme / lang:** `/login` (`LoginCard.tsx`), viewports ≤ 390px, all themes, EN & RU.
- **Problem:**
  1. Unmounting and remounting views reset focus to `document.body`.
  2. Long email addresses lacked `break-all` / `break-words` styling and could overflow on mobile viewports.
  3. Recipient email lacked bold/visual contrast.
- **Resolution:**
  - Added programmatic focus management: upon entering `magic_sent` mode, `backToSignInRef.current?.focus()` (and `autoFocus` on the button) ensures immediate focus on the primary action; upon clicking "Back to sign-in options", focus is smoothly restored to the email input (`emailInputRef.current?.focus()`).
  - Added `<p className="break-words">` container with `<span className="font-medium text-primary break-all">{targetEmail}</span>` for robust wrapping and typographic emphasis.
- **Tags:** a11y, layout, hierarchy

---

### UX-047 [minor] [FIXED] Multi-line Emergency Recovery button icon alignment on mobile viewports
- **Status:** **FIXED** (Verified in `LoginCard.tsx:387-399` and browser screenshots at 390px in EN & RU).
- **Screen / viewport / theme / lang:** `/login` (`LoginCard.tsx`), mobile viewports ≤ 390px, light & dark themes, EN & RU.
- **Problem:**
  - On narrow viewports (≤ 390px), the recovery link copy wraps to two lines in both English and Russian.
  - The key icon was vertically centered across both lines, floating in the empty vertical space between lines.
- **Resolution:**
  - Applied `inline-flex items-start text-left leading-snug whitespace-normal max-w-full` on the button and `shrink-0 mt-0.5` on the `<KeyRound>` icon.
  - The icon now aligns with line 1 of the text block across mobile viewports in English and Russian.
- **Tags:** layout, alignment, consistency

---

## Positive Notes
- **Zero Token & Preview Exposure:** The implementation completely eliminates any local token preview or bypass button (`magicPreviewUrl` removed entirely). The confirmation state provides a clean, secure message instructing users to inspect their inbox.
- **Clear Error Translation Architecture:** API errors (such as 503 `smtp_not_configured`) map cleanly through `getLocalizedErrorMessage` to user-friendly localized error alerts in both English and Russian.
- **Visual Warmth & Distraction-Free Shell:** The authentication card features refined typography, subtle shadows, and centered composition, keeping the onboarding and login experience serene and accessible.
- **Seamless Locale & Theme Switching:** Top-right toggles allow instant switching between EN/RU and light/dark modes with full text and icon adaptation and zero layout jitter.
