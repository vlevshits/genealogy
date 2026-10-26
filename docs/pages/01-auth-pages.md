# Page Specification: Authentication & Onboarding Views

## 1. Overview & Objective
The authentication views (`/login`, `/register`, `/setup`, `/magic-link/verify`) provide a clean, distraction-free environment centered entirely on security and effortless sign-in. The layout is stripped of full app navigation chrome, featuring an elegant centered card, subtle warm background, and top-right language/theme switches.

> [!IMPORTANT] Default Private Landing Page:
> Under the default instance configuration (`ENABLE_PUBLIC_ACCESS=false`), the Sign-In view (`/login`) is the **exclusive entry point** for unauthenticated visitors. Any request to root (`/`), the tree canvas (`/tree`), or internal resources is intercepted by the router/middleware and immediately redirected to `/login`. Only when public access is explicitly enabled does `/tree` accept unauthenticated traffic.

---

## 2. Layout Wireframe

```
+------------------------------------------------------------------------------------+
|                                                               [RU / EN]  [Light/🌙] |
|                                                                                    |
|                           +-------------------------------+                        |
|                           |      [Tree Brand Logo]        |                        |
|                           |   Levshits Family Archive     |                        |
|                           |                               |                        |
|                           | [Primary Passkey CTA Button]  |                        |
|                           |  🔐 Sign in with Touch ID /   |                        |
|                           |         Windows Hello         |                        |
|                           |                               |                        |
|                           | ------- or sign in with ------|                        |
|                           |                               |                        |
|                           | [ Email Address Input       ] |                        |
|                           | [ Send Magic Link Button    ] |                        |
|                           |                               |                        |
|                           | 🔑 Use Emergency Recovery Code|                        |
|                           +-------------------------------+                        |
|                                                                                    |
|                           © 2026 [Family Tree Title] • Private Family Archive      |
+------------------------------------------------------------------------------------+
```

---

## 3. Page Variants & Structures

### 3.1 Sign-In View (`/login`)
- **Default Entry Point:** Serves as the landing page for unauthenticated visitors when the archive is private.
- **Hero Element:** Large, prominent **"Sign in with Passkey"** button (`bg-brand text-white py-3 rounded-lg font-medium shadow-sm hover:opacity-95`).
- **Conditional UI / Autofill:** If the browser supports WebAuthn autofill, a gentle prompt allows instant biometric login without typing.
- **Divider:** Subtle rule with *"or sign in with email"*.
- **Magic Link Section:**
  - Email input field with placeholder (`name@example.com`).
  - Button: *"Send Magic Link"*.
- **Footer Links:**
  - *"Lost your device? Use Emergency Recovery Code"*.

### 3.2 First-Time Instance Setup View (`/setup`)
- **Trigger:** Only accessible on an uninitialized instance where 0 users exist.
- **Form Fields:**
  - Instance Tree Title (e.g., *"Levshits Family Archive"*).
  - Admin First Name.
  - Admin Last Name.
  - Admin Email Address.
- **Action Button:** *"Create Admin Passkey"*.
- **Step 2 (Recovery Codes Display):**
  - Displays a clean grid of 10 single-use emergency recovery codes.
  - *"Download as Text File"* or *"Copy to Clipboard"* action.
  - Required checkbox: *"I have saved these recovery codes in a secure place"* before unlocking the dashboard.

### 3.3 Invitation Registration View (`/register?invite_token=...`)
- **Banner:** *"You have been invited by [Inviter Name] to join the [Family Tree Name] as an [Editor/Viewer]"*.
- **Pre-linked Relative:** Shows *"You will be linked to [Person Name] in the family tree"*.
- **Form Fields:**
  - First Name
  - Last Name
- **Primary CTA:** *"Create Passkey & Join Tree"*.

---

## 4. Definition of Done (DoD)

- [ ] **Default Private Landing:** In the default private configuration (`ENABLE_PUBLIC_ACCESS=false`), unauthenticated visitors to `/` or `/tree` land on `/login`; no tree topology or living identities are leaked.
- [ ] **Distraction-Free Design:** Auth pages hide the global topbar and center the card with responsive padding on mobile screens.
- [ ] **One-Touch Passkey Login:** Clicking "Sign in with Passkey" initiates the biometric dialog immediately.
- [ ] **Magic Link State:** Requesting a magic link displays an instant confirmation view: *"Check your inbox. We sent a sign-in link to user@example.com"*.
- [ ] **Full Russian/English Parity:** All prompts, errors, and recovery instructions display in the chosen language.
- [ ] **Anti-Marketing Copy:** All auth cards, onboarding prompts, and button labels use simple, reassuring genealogical and archival copy with zero marketing buzzwords or tech stack mentions.
