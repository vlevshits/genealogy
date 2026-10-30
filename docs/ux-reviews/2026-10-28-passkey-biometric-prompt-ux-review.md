# UX Review — Phase 2.3.1 Passkey Biometric Prompt & Verification Hardening — 2026-10-28

## Verdict: OK (ALL FINDINGS RESOLVED)

Screens reviewed:
- `/login` (Passkey button, biometric prompt trigger, error state rendering for cancelled/unsupported/failed biometric prompts)
- `/setup` (Admin passkey registration during first-run setup)
- `/invite/:token` (`/register`) (New user invite acceptance with passkey registration)
- `/settings` (Security & Passkeys panel, passkey registration, rename, revoke)
- `/magic-link/verify` (Post-verification passkey enrollment prompt)

Viewports verified:
- Desktop: 1440×900
- Mobile: 390×844
- Themes: Light & Dark
- Languages: English (`en`) & Russian (`ru`)

Recordings & Visual Evidence:
- Desktop Light (`en`): `.playwright-mcp/page-2026-10-28T18-06-52-329Z.png`
- Desktop Light (`ru`): `.playwright-mcp/page-2026-10-28T18-07-09-121Z.png`
- Desktop Dark (`ru`): `.playwright-mcp/page-2026-10-28T18-07-35-516Z.png`
- Mobile Dark (`ru` 390px): `.playwright-mcp/page-2026-10-28T18-07-45-642Z.png`
- Mobile Light (`en` 390px): `.playwright-mcp/page-2026-10-28T18-08-03-766Z.png`
- Setup Wizard Desktop Light (`en` 1440px): `.playwright-mcp/page-2026-10-28T18-08-27-070Z.png`

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
| **UX-029** | Tree canvas pointer capture swallows left clicks on person cards; keyboard activation missing | **fixed** |
| **UX-030** | Mobile `PersonBottomSheet` missing authoring actions (Add Mother, Add Spouse, Add Event, Delete) | open |
| **UX-031** | Face tags on photograph canvas keyboard inaccessible; missing focus rings, tooltip disclosure, and delete | open |
| **UX-032** | Non-admin authorized users (Editors & Viewers) locked out of Profile and Passkeys in mobile "More" drawer | open |
| **UX-033** | Invitation creation modal lacks 'Admin' role selection option | open |
| **UX-034** | Authenticated mobile header button congestion and title squeeze | open |
| **UX-035** | Destructive "Delete Person" button in `PersonDrawer` lacks visual danger treatment | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |
| **UX-037** | Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px) | open |
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | **fixed** |
| **UX-039** | Low non-text contrast (< 3:1) for collateral connector edges in dark mode | **fixed** |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile | **fixed** |
| **UX-041** | Missing tooltip / explanatory subtitle for "Wide tree" / "Compact tree" toggle | **fixed** |
| **UX-042** | Lack of card-level visual hierarchy between direct lineage and collateral branches | open |
| **UX-043** | Mobile toolbar vertical height reduction and icon assistance for Wide/Compact toggle | suggestion (open) |
| **UX-044** | Misleading divider label and dead action button when SMTP is disabled | **fixed** |
| **UX-045** | Inaccessible and skipped SMTP disabled hint for screen reader and keyboard users | **fixed** |
| **UX-046** | Missing keyboard focus management and long email text wrapping in `magic_sent` confirmation view | **fixed** |
| **UX-047** | Multi-line Emergency Recovery button icon alignment on mobile viewports | **fixed** |
| **UX-048** | Disconnected / floating connector lines in LOD 2 zoom mode (< 0.25x) | **fixed** |
| **UX-049** | Semantic exclusion and scaling illegibility of component headers in ARIA tree (WCAG 1.3.1) | **fixed** |
| **UX-050** | Shelf grid start X coordinate clamped to 0, misaligning isolated records | **fixed** |
| **UX-051** | LOD 2 micro-pill renders full name instead of concise surname/initials | **fixed** |
| **UX-052** | Missing Home person anchor badge in LOD 2 micro-pill | **fixed** |
| **UX-053** | Singular English pluralization for All Nodes count badge (`1 individuals`) | **fixed** |
| **UX-054** | SegmentedControl horizontal overflow on mobile viewports (< 480px) lacks scroll affordance | suggestion (open) |
| **UX-055** | Cross-generational multi-tier spouse connectors in all-nodes mode should route via outer clearance corridors | suggestion (open) |
| **UX-056** | Hardware Security Key (YubiKey) Registration Blocked by Enforced `authenticatorAttachment: 'platform'` | **fixed** |
| **UX-057** | Misleading Error Toast Title "Failed to load options" / "Ошибка загрузки данных" on Passkey Enrollment / Management Failures | **fixed** |
| **UX-058** | Silent Failure and Premature Navigation on Passkey Enrollment in `MagicLinkVerifyCard` | **fixed** |
| **UX-059** | Missing WebAuthn `TimeoutError` and `AbortError` Mapping in `getLocalizedErrorMessage` | **fixed** |
| **UX-060** | Jargon-Heavy Copy for Passkey Replay Attack Error (`errors.passkey_replay_detected`) | **fixed** |
| **UX-061** | Off-Screen Error Placement on Long Mobile Viewports in `SetupWizardCard` and `RegisterInviteCard` | **fixed** |

---

## Tasks for frontend-developer

### UX-056 [major] Hardware Security Key (YubiKey) Registration Blocked by Enforced `authenticatorAttachment: 'platform'`
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** `/setup`, `/register`, `/settings` (Security & Passkeys panel)
- **Problem:**
  In `createPasskeyCredential` (`frontend/src/lib/webauthn.ts` line 136):
  ```ts
  authenticatorSelection: {
    authenticatorAttachment: attachment || 'platform',
    userVerification: 'required',
    residentKey: 'required',
    requireResidentKey: true,
  },
  ```
  Neither `SetupWizardCard`, `RegisterInviteCard`, nor `SecurityPasskeysPanel` pass an `attachment` parameter. Consequently, `authenticatorAttachment` defaulted unconditionally to `'platform'`.
  This instructed the browser client to accept **only** built-in platform authenticators (e.g. Touch ID, Face ID, Windows Hello).
  Cross-platform hardware security keys (e.g., YubiKey USB-A/C, Nitrokey, external FIDO2 dongles) were rejected or excluded from the prompt.
- **Evidence:** `frontend/src/lib/webauthn.ts:136`, `frontend/src/components/auth/SetupWizardCard.tsx:61`, `frontend/src/components/auth/RegisterInviteCard.tsx:133`, `frontend/src/components/settings/SecurityPasskeysPanel.tsx:121`.
- **Resolution:**
  Updated `frontend/src/lib/webauthn.ts` to only include `authenticatorAttachment` conditionally when `attachment` is explicitly supplied:
  `...(attachment ? { authenticatorAttachment: attachment } : {})`.
  When `attachment` is omitted, the browser allows cross-platform authenticators and external USB security keys per W3C WebAuthn specification.
- **Verification:** Verified in `frontend/src/lib/webauthn.ts` and automated regression suite in `frontend/tests/components/auth-and-modular-ui.test.tsx`.
- **Tags:** a11y | consistency | hierarchy

---

### UX-057 [major] Misleading Error Toast Title "Failed to load options" / "Ошибка загрузки данных" on Passkey Enrollment / Management Failures
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** `/settings?tab=security` (Desktop & Mobile, Light & Dark, EN & RU)
- **Problem:**
  In `SecurityPasskeysPanel.tsx`, error handlers used `title: t('common.errorLoading')`, which mapped to *"Failed to load options"* / *"Ошибка загрузки данных"*. When passkey operations failed or timed out, users saw misleading toast titles indicating data loading problems.
- **Evidence:** `frontend/src/components/settings/SecurityPasskeysPanel.tsx:138`, `frontend/src/i18n/en.json:103`, `frontend/src/i18n/ru.json:103`.
- **Resolution:**
  1. Added `settings.security.passkeyErrorTitle` ("Passkey Error" / "Ошибка ключа доступа") in `en.json` and `ru.json`.
  2. Updated `SecurityPasskeysPanel.tsx` error toasts to use `settings.security.passkeyErrorTitle` for passkey mutations and `common.error` ("Error" / "Ошибка") for other actions. `common.errorLoading` is completely eliminated from the panel.
- **Verification:** Verified zero occurrences of `common.errorLoading` in `SecurityPasskeysPanel.tsx` and validated via automated regression tests in `auth-and-modular-ui.test.tsx`.
- **Tags:** copy | consistency | states

---

### UX-058 [major] Silent Failure and Premature Navigation on Passkey Enrollment in `MagicLinkVerifyCard`
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** `/magic-link/verify`
- **Problem:**
  When a user clicked *"Register Passkey on This Device"*, if biometric verification failed or was cancelled, the empty `catch` block swallowed the error and unconditionally called `onComplete?.()`. The user was prematurely navigated into the app with no indication that passkey enrollment failed.
- **Evidence:** `frontend/src/components/auth/MagicLinkVerifyCard.tsx:94-96`.
- **Resolution:**
  Updated `handleEnrollPasskey` to catch the error and populate an `enrollError` state using `getLocalizedErrorMessage(err, t)`. Rendered an inline `<Alert variant="danger" role="alert">{enrollError}</Alert>` in `MagicLinkVerifyCard.tsx`. The user remains on the card and can either retry registration or explicitly click *"Skip for now"*.
- **Verification:** Verified error handling state and alert presence in `frontend/src/components/auth/MagicLinkVerifyCard.tsx` and automated regression test in `auth-and-modular-ui.test.tsx`.
- **Tags:** states | feedback | a11y

---

### UX-059 [minor] Missing WebAuthn `TimeoutError` and `AbortError` Mapping in `getLocalizedErrorMessage`
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** `/login`, `/setup`, `/register`, `/settings`
- **Problem:**
  In `error-formatter.ts`, only `NotAllowedError` was mapped to `errors.webauthn_not_allowed`. Browsers emitting `TimeoutError` or `AbortError` fell through to the generic unhelpful *"An unexpected error occurred."* message.
- **Evidence:** `frontend/src/lib/error-formatter.ts:50-67`.
- **Resolution:**
  Updated `error-formatter.ts` to match `domErr.name === 'NotAllowedError' || domErr.name === 'TimeoutError' || domErr.name === 'AbortError'`, returning the user-friendly localized message *"Biometric verification was cancelled or timed out. Please try again."* / *"Биометрическая верификация отменена или истекло время ожидания. Попробуйте снова."*
- **Verification:** Verified with unit test assertions for `DOMException('TimeoutError')` and `DOMException('AbortError')` in `frontend/tests/components/auth-and-modular-ui.test.tsx`.
- **Tags:** states | copy | consistency

---

### UX-060 [minor] Jargon-Heavy Copy for Passkey Replay Attack Error (`errors.passkey_replay_detected`)
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** All authentication screens, EN & RU
- **Problem:**
  `errors.passkey_replay_detected` used technical cryptographic terminology (*"Passkey replay detected"* / *"Обнаружен повтор подписи ключа доступа"*), alarming non-technical family members.
- **Evidence:** `frontend/src/i18n/en.json:1732`, `frontend/src/i18n/ru.json:1732`.
- **Resolution:**
  Rephrased in `en.json` and `ru.json` to:
  - EN: *"Passkey security verification failed. Please try signing in again."*
  - RU: *"Проверка безопасности ключа доступа не удалась. Попробуйте войти снова."*
- **Verification:** Verified 100% key parity and i18n convention compliance via `tests/i18n/parity.test.ts`.
- **Tags:** copy

---

### UX-061 [minor] Off-Screen Error Placement on Long Mobile Viewports in `SetupWizardCard` and `RegisterInviteCard`
- **Status:** **fixed**
- **Screen / viewport / theme / lang:** `/setup` and `/register`, mobile viewports (≤ 390px)
- **Problem:**
  The submit button is positioned below numerous form inputs. On mobile viewports (390×844), an error displayed at the top of the form was scrolled out of the visible screen area, leaving the user with no visible feedback when submission failed.
- **Evidence:** `frontend/src/components/auth/SetupWizardCard.tsx:104`, `frontend/src/components/auth/RegisterInviteCard.tsx:182`.
- **Resolution:**
  Added `errorRef` to both `SetupWizardCard` and `RegisterInviteCard` on the error alert with `tabIndex={-1}` and `className="focus:outline-none"`. A `useEffect` hook triggers `scrollIntoView({ behavior: 'smooth', block: 'nearest' })` and `.focus()` whenever an error is encountered.
- **Verification:** Verified ref implementation and tabIndex focus handling in both components and automated regression tests in `auth-and-modular-ui.test.tsx`.
- **Tags:** a11y | layout | space

---

## Positive Notes
- **Clear & Prominent Passkey Action:** The *"Sign in with Passkey"* CTA is positioned at the top of the login card with high visual prominence (`variant="primary"`, `size="lg"`, with `Fingerprint` icon), accompanied by concise explanatory subtitle text.
- **Design System & Semantic Tokens:** Error states use the design system `<Alert variant="danger">` with `role="alert"`, maintaining accessible contrast (≥ 4.5:1 text contrast) in both light and dark modes.
- **Zero Raw HTML Controls:** All reviewed views use canonical design-system primitives (`Button`, `TextInput`, `EmailInput`, `PasswordInput`, `Divider`), strictly complying with workspace UI rules.
- **Strict Localization Hygiene:** Zero emojis or decorative icon glyphs in `en.json` and `ru.json`, passing the automated parity test suite with 100% key parity.
- **Touch Target Accessibility:** All buttons, including mobile toggles and primary action triggers, meet or exceed the 40×40px WCAG 2.2 AA target size.
