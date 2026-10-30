# Implementation Report: Milestone 2.4 — Public Archive Navigation & Safe Return URL Redirection

**Date:** 2026-10-28  
**Status:** Completed, Verified & Approved  
**Scope:** Milestone 2.4 of [`docs/roadmap.md`](../roadmap.md) — `TASK-AUTH-PUBLIC-NAVIGATION`: Egress from `/login` back to Public Family Tree, Clickable Header Brand Navigation, Contextual Egress Button, and Safe Open-Redirect Defense ([`01-auth-pages.md`](../pages/01-auth-pages.md), [`01-authentication-passkeys.md`](../features/01-authentication-passkeys.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md)).

---

## 1. Summary of Completed Deliverables

### 1.1 Safe Navigation & Open-Redirect Defense Rail (`frontend/src/lib/navigation-utils.ts`)
- **Strict Relative Path Sanitization (`sanitizeReturnUrl`):**
  - Requires leading single `/` and rejects protocol-relative paths (`//evil.com`, `///evil.com`).
  - Rejects Windows backslash paths (`\evil.com`, `/tree\evil`).
  - Rejects external URI schemes (`https:`, `http:`, `javascript:`, `data:`, `vbscript:`).
  - Rejects control characters (`\r`, `\n`, null bytes).
  - Eliminates auth redirect loops by resetting targets matching `/login`, `/register`, `/setup`, `/magic-link/verify` to `/`.
- **Public Guest Route Classifier (`isPublicGuestRoute`):**
  - Classifies permitted public routes: `/`, `/tree*`, `/persons/:id*`, `/dev/ui`.
  - Classifies restricted auxiliary routes: `/people`, `/media`, `/places`, `/settings`, `/history`, `/quality`, `/export`.
- **Guest Bounce-Loop Prevention (`resolveGuestEgressTarget`):**
  - If `returnUrl` points to a permitted public route (`/tree` or `/persons/:id`), navigates there.
  - If `returnUrl` is empty or points to a restricted route (e.g. `/settings`), safely falls back to `/` rather than repeatedly bouncing unauthenticated visitors back to `/login`.
- **URL Constructor (`buildLoginUrl`):**
  - Generates `/login?returnUrl=...` carrying both `currentPath` and query parameters (`search`), omitting `returnUrl` on root `/` or auth pages.

### 1.2 Interactive Auth Header (`frontend/src/components/ui/auth-layout.tsx`)
- **Semantic Brand Navigation:**
  - Rendered brand tree logo and archive title as semantic `<a href="/" onClick={handleNavigateHome} ...>` with high-contrast focus ring (`focus-visible:ring-focus`).
  - Expanded interactive touch target to $\ge 40\text{px}$ (`min-h-10 py-1`) per `UX-048`.
  - Modifier key detection (`metaKey`, `ctrlKey`, `shiftKey`, middle-click) preserves native browser new-tab functionality while standard left-clicks route via SPA navigation.
- **Private Instance Containment:**
  - When `enable_public_access === false`, client router guard intercepts unauthenticated clicks to `/` and securely retains the visitor on `/login` with zero tree topology or living identities exposed.

### 1.3 Contextual Egress Button on `LoginCard.tsx`
- **Dynamic Action Button:**
  - When `enable_public_access === true`:
    - If `returnUrl` points to `/tree` or `/persons/:id`: renders secondary button *"Back to Family Tree"* / *"Вернуться к семейному древу"* with `<ArrowLeft />` icon.
    - If `returnUrl` is empty, root, or restricted: renders secondary button *"Browse Public Archive"* / *"Перейти к публичному архиву"* with `<Compass />` icon.
  - When `enable_public_access === false`:
    - Button is strictly omitted from the DOM.

### 1.4 Post-Authentication Redirection (`AuthPages.tsx`)
- Across all authentication methods (Passkey, password fallback, recovery code, magic link verification, invitation registration), successfully authenticating forwards to `sanitizeReturnUrl(rawReturnUrl, '/')`.
- Updated "Sign In" action buttons in `AppShell.tsx` and `BottomNav.tsx` to pass `buildLoginUrl(currentPath, search)`.
- Localized landmark navigation `<nav aria-label={t('nav.mainNav')}>` and `<nav aria-label={t('nav.bottomNav')}>` per `UX-049`.

---

## 2. Verification & Automated Test Evidence

1. **Unit & Security Tests:**
   - `frontend/tests/lib/navigation-utils.test.ts` (17 tests passing): Verified open-redirect neutralization across protocol-relative, scheme injection, control char, backslash, and auth loop vectors.
2. **Component & Integration Tests:**
   - `frontend/tests/components/auth-navigation.test.tsx` (23 tests passing):
     - Brand header navigation, focus outline, modifier key pass-through.
     - Public guest egress button contextual switching in EN and RU.
     - Omission of egress button in private mode (`enable_public_access: false`).
     - Restricted route fallback to `/`.
     - Post-auth redirection forwarding.
   - `frontend/tests/components/auth-and-modular-ui.test.tsx` (11 tests passing).
   - `frontend/tests/i18n/parity.test.ts` (10 tests passing, 100% key parity, zero emojis or icon glyphs).
3. **Full Test Suites & Build:**
   - Frontend: 18 test files, 168 tests passed, 0 failures.
   - Frontend Build: `tsc --noEmit && vite build` succeeded in 1.15s with 0 errors.
   - Backend: 50 integration tests passed across 13 test suites.

---

## 3. Review Verdicts

- **Architect Review (Mode 2):** **APPROVE WITH COMMENTS**  
  All security boundaries, open-redirect defenses, and bounce-loop prevention approved; implemented minor nits (modifier keys check, query param preservation).
- **QA Analyst Verification:** **PASS**  
  All acceptance criteria verified against test fixtures and boundary vectors with 0 defects.
- **UX Review:** **OK** ([`docs/ux-reviews/2026-10-28-auth-public-navigation-ux-review.md`](../ux-reviews/2026-10-28-auth-public-navigation-ux-review.md))  
  Addressed tasks `UX-048` ($\ge 40\text{px}$ touch target on brand link) and `UX-049` (localized `aria-label` on navigation landmarks).

---

## 4. Technical Debt Recorded
- Zero technical debt added.

