# Page Specification: Authentication & Onboarding Views

## 1. Overview & Objective
The authentication views (`/login`, `/register`, `/setup`, `/magic-link/verify`) provide a clean, distraction-free environment centered entirely on security and effortless sign-in. Hosted within `AuthLayout`, these views strip out full application navigation chrome while retaining essential utilities:
- An interactive top header featuring a clickable family archive brand logo and tree title navigating to `/`, alongside language (`RU / EN`) and theme (`Light / Dark`) toggles.
- An elegant centered authentication card with responsive padding on mobile screens.
- Subtle warm archival background and quiet footer copyright.

> [!IMPORTANT] Access Posture & Default Private Gatekeeper:
> - **Default Private Instance (`ENABLE_PUBLIC_ACCESS=false`):** The Sign-In view (`/login`) is the **exclusive entry point** for unauthenticated visitors. Any attempt to access root (`/`), the tree canvas (`/tree`), or internal resources is intercepted by the router/middleware gatekeeper and redirected to `/login?returnUrl=<target>`. Clicking the brand logo in the `AuthLayout` header invokes navigation to `/`, where the private gatekeeper immediately retains the user on `/login` without leaking tree topology or living identities.
> - **Public Tree Mode Enabled (`ENABLE_PUBLIC_ACCESS=true`):** Unauthenticated visitors have access to the public home page (`/`), tree canvas (`/tree`), and individual profiles (`/persons/:id`) in masked read-only guest mode. When visiting `/login` (via bookmark, direct navigation, or clicking "Sign In" in the public header), visitors can seamlessly return to the public archive by clicking the header brand logo or activating the prominent "Browse Public Archive / Back to Family Tree" action on `LoginCard`.

---

## 2. Layout Wireframe

```
+------------------------------------------------------------------------------------+
| [[Icon] Levshits Family Archive]                              [RU / EN]  [Light/🌙] |
| (Clickable brand header: navigates to '/')                                         |
|                                                                                    |
|                           +-------------------------------+                        |
|                           |   Sign In to Family Archive   |                        |
|                           |  Private family history and   |                        |
|                           |     genealogical records      |                        |
|                           |                               |                        |
|                           | [Primary Passkey CTA Button]  |                        |
|                           |  Sign in with Passkey         |                        |
|                           |                               |                        |
|                           | ------- or sign in with ------|                        |
|                           |                               |                        |
|                           | [ Email Address Input       ] |                        |
|                           | [ Password Input            ] |                        |
|                           | [ Sign In with Password     ] |                        |
|                           | [ Send Magic Link Button    ] |                        |
|                           |                               |                        |
|                           | - (when enable_public_access) |                        |
|                           | [ Browse Public Archive     ] |                        |
|                           |                               |                        |
|                           |  Use Emergency Recovery Code  |                        |
|                           +-------------------------------+                        |
|                                                                                    |
|                           © 2026 [Family Tree Title] • Private Family Archive      |
+------------------------------------------------------------------------------------+
```

---

## 3. Page Variants & Structures

### 3.1 Sign-In View (`/login`)

#### 3.1.1 Header Brand Navigation (`AuthLayout`)
- **Brand Identity Trigger:** The top header renders the brand tree logo and the resolved instance title (e.g. *"Levshits Family Archive"*).
- **Interactive Element:** Rendered as an accessible interactive control (button or link) with clear focus ring (`focus-visible:ring-2 focus-visible:ring-focus`) and hover opacity styling.
- **Routing Behavior:**
  - Invokes `onNavigate('/')`.
  - **When `enable_public_access === true`:** Immediately transitions the unauthenticated visitor to the public home dashboard (`/`) in public guest mode.
  - **When `enable_public_access === false`:** Navigating to `/` triggers the client router guard; because the instance is private and the visitor is unauthenticated, the app securely retains them on `/login`. No private tree records, living person details, or statistics are leaked or briefly flashed.

#### 3.1.2 Hero Passkey & Password/Magic-Link Sections
- **Default Entry Point:** Serves as the landing page for unauthenticated visitors when the archive is private.
- **Hero Element:** Large, prominent **"Sign in with Passkey"** button (`bg-brand text-white py-3 rounded-lg font-medium shadow-sm hover:opacity-95`).
- **Conditional UI / Autofill:** If the browser supports WebAuthn autofill, a gentle prompt allows instant biometric login without typing.
- **Divider:** Subtle rule with *"or sign in with password or email link"* (adapting based on enabled options).
- **Password Fallback Section (when `auth_password_enabled` is true):**
  - Password input field and *"Sign In with Password"* button.
- **Magic Link Section:**
  - Email input field with placeholder (`name@example.com`).
  - Button: *"Send Magic Link"*.
  - **Zero Token/Preview Exposure:** When a magic link is requested, the view transitions to a pure confirmation state:
    - Success alert: *"Check your inbox. We sent a single-use sign-in link to {email}. The link is valid for 15 minutes."*
    - Single secondary button: *"Back to sign-in options"*.
    - Under **no circumstances** is an "Open Sign-In Link (Local Preview)" button or token rendered in the UI or stored in client state.
  - **Non-SMTP Handling:** When SMTP is unconfigured (`smtp_enabled === false` or API returns `smtp_not_configured`), the UI renders a localized alert explaining email delivery is not configured, guiding users to passkeys, password fallback, or an admin-generated link.

#### 3.1.3 Public Archive Return Action (`LoginCard`)
- **Conditional Visibility:** Rendered **strictly when `enable_public_access === true`** (retrieved from `setupStatus.enable_public_access`).
- **Suppression in Private Mode:** When `enable_public_access === false`, this action is **strictly omitted** from the DOM; private instances present zero egress routes to unauthenticated users.
- **Visual Presentation:** A distinct secondary action button or outline card button (`variant="secondary"` or `variant="outline"`), styled calmly below the sign-in forms and above the emergency recovery link.
- **Action Labeling:**
  - Default: *"Browse Public Archive"* (EN) / *"Перейти к публичному архиву"* (RU).
  - Contextual (when returning from tree/profile): *"Back to Family Tree"* (EN) / *"Вернуться к семейному древу"* (RU).
  - Zero icon glyphs or emojis in translation strings per UI & i18n rules; optional left icon rendered exclusively via React Lucide component (e.g. `<Compass className="h-4 w-4" />` or `<ArrowLeft className="h-4 w-4" />`).
- **Destination Resolution:**
  - If a `returnUrl` query parameter is present:
    - Validate that `returnUrl` is a safe internal relative path.
    - Check whether `returnUrl` belongs to the permitted public guest surface (`/`, `/tree`, `/persons/:id`).
    - If permitted, navigate to `returnUrl`.
    - If `returnUrl` points to a restricted auxiliary module (`/people`, `/media`, `/places`, `/settings`, `/history`, `/quality`, `/people/merge`, `/export`), fallback safely to `/` (or `/tree`) rather than bouncing the unauthenticated visitor right back to `/login`.
  - If `returnUrl` is absent, navigate to `/`.

#### 3.1.4 Return URL Preservation & Open-Redirect Protection
- **Query Parameter:** `/login?returnUrl=<encoded_path>`.
- **Sources:**
  - Public guest clicking "Sign In" from the top navigation bar or guest announcement banner (passes current location, e.g. `/tree` or `/persons/c7a8b9d0-...`).
  - Unauthenticated visitor attempting to access a protected route on private or public instances (e.g. `/settings` or `/people`).
  - Direct deep links shared among authenticated family members.
- **Sanitization & Safety Rails:**
  - The application strictly validates `returnUrl` to prevent Open Redirect vulnerabilities.
  - Allowed: Must begin with a single `/` and must NOT begin with `//` (protocol-relative URLs) or contain URI schemes (`http:`, `https:`, `javascript:`, `data:`).
  - Control characters, newlines (`\r`, `\n`), and invalid characters cause the target to be discarded.
  - If validation fails or the parameter is empty, the redirection target defaults to `/`.
- **Post-Authentication Navigation:**
  - Upon successful authentication via any method on `/login`:
    - Passkey WebAuthn assertion completion (`verifyPasskeyLogin`).
    - Password fallback sign-in (`loginWithPassword`).
    - Emergency recovery code verification (`loginWithRecoveryCode`).
  - The client transitions directly to the validated `returnUrl` (e.g. `onNavigate(resolvedReturnUrl)`), preserving full visitor context.

#### 3.1.5 Footer Links
- *"Lost your device? Use Emergency Recovery Code"*.

---

### 3.2 First-Time Instance Setup View (`/setup`)
- **Trigger:** Only accessible on an uninitialized instance where 0 users exist.
- **Form Fields:**
  - Instance Tree Title (e.g., *"Levshits Family Archive"*).
  - Admin First Name.
  - Admin Last Name.
  - Admin Email Address.
  - Fallback Password (optional, min 10 chars).
- **Action Button:** *"Create Admin Passkey"*.
- **Step 2 (Recovery Codes Display):**
  - Displays a clean grid of 10 single-use emergency recovery codes.
  - *"Download as Text File"* or *"Copy to Clipboard"* action.
  - Required checkbox: *"I have saved these recovery codes in a secure place"* before unlocking the dashboard.
- **Post-Setup Redirection:** Navigates to `/` (Home dashboard) upon completing setup.

---

### 3.3 Invitation Registration View (`/register?invite_token=...`)
- **Banner:** *"You have been invited by [Inviter Name] to join the [Family Tree Name] as an [Editor/Viewer]"*.
- **Pre-linked Relative:** Shows *"You will be linked to [Person Name] in the family tree"*.
- **Form Fields:**
  - First Name
  - Last Name
  - Fallback Password (optional).
- **Primary CTA:** *"Create Passkey & Join Tree"*.
- **Return URL Handling:** If a `returnUrl` is provided in the query string, registration success navigates to the validated `returnUrl`; otherwise defaults to `/`.

---

### 3.4 Magic Link Verification View (`/magic-link/verify?token=...`)
- **Validation:** Single-use token verification against backend `magic_links`.
- **Return URL Handling:** Preserves `returnUrl` query parameter across verification; upon successful session creation, forwards user to the sanitized `returnUrl` (defaulting to `/`).

---

## 4. Definition of Done (DoD)

- [ ] **Clickable Auth Header Brand:** In `AuthLayout`, the brand logo icon and instance title are interactive (button/link with focus indicator). Clicking them invokes navigation to `/`.
- [ ] **Public Mode Header Exit:** When `ENABLE_PUBLIC_ACCESS=true`, clicking the `AuthLayout` header brand navigates unauthenticated visitors to the public Home dashboard (`/`) in read-only guest mode.
- [ ] **Private Mode Gatekeeper Protection:** When `ENABLE_PUBLIC_ACCESS=false`, clicking the `AuthLayout` header brand retains the unauthenticated visitor safely on `/login`; no tree topology, ancestor names, or living identities are leaked.
- [ ] **Public Archive Return Action on LoginCard:** When `ENABLE_PUBLIC_ACCESS=true`, `LoginCard` displays a prominent "Browse Public Archive / Back to Family Tree" action button.
- [ ] **Private Mode Public Action Suppression:** When `ENABLE_PUBLIC_ACCESS=false`, the "Browse Public Archive" action is completely omitted from the DOM.
- [ ] **Safe Guest Return Resolution:** Clicking "Browse Public Archive" navigates to `returnUrl` if it is a permitted public route (`/`, `/tree`, `/persons/:id`); if `returnUrl` is absent or points to a restricted route (e.g. `/people`, `/settings`), it safely navigates to `/` without triggering a redirect loop.
- [ ] **Open-Redirect Protection on Return URL:** The `returnUrl` parameter is strictly validated to ensure it is an internal relative path starting with a single `/`. External schemes (`http:`, `https:`, `//`, `javascript:`) are rejected and reset to `/`.
- [ ] **Post-Authentication Return URL Preservation:** Successful sign-in via Passkey, password fallback, magic link verification, or emergency recovery code navigates to the validated `returnUrl` instead of always defaulting to `/`.
- [ ] **Distraction-Free Design:** Auth pages hide the global topbar navigation and center the card with responsive mobile padding.
- [ ] **One-Touch Passkey Login:** Clicking "Sign in with Passkey" initiates the biometric dialog immediately.
- [ ] **Magic Link State & Zero Token Exposure:** Requesting a magic link displays an instant confirmation view without exposing preview URLs or tokens in responses or UI.
- [ ] **Non-SMTP Guidance:** When SMTP is disabled (`smtp_not_configured`), an informative localized alert guides the user to passkeys, password, or asking an administrator for a sign-in link.
- [ ] **Full Russian/English Parity:** All prompts, button labels, and hints display cleanly in EN and RU.
- [ ] **Anti-Marketing & Icon Quality Standards:** All auth views use quiet, family-archival copywriting with zero tech-stack mentions and zero hardcoded icons or emojis in localization strings.
