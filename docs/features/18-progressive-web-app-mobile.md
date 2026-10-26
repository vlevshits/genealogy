# Feature Specification: Progressive Web App (PWA) & Mobile/Tablet Experience

## 1. Overview & Objective
Enable family members to install the genealogy web application directly onto mobile phones, tablets, and desktop computers as a **Progressive Web App (PWA)** with a standalone, native-app feel. Simultaneously, ensure every view across the application is meticulously tailored for mobile (`< 768px`) and tablet (`768px – 1024px`) touch devices, with ergonomic touch targets, thumb-accessible navigation, safe-area insets, and fluid multi-touch gesture support on the family tree canvas.

### Core Objectives:
1. **PWA Installability & Standalone Display:** First-class Web App Manifest, maskable and Apple touch icons, theme-color synchronization, and standalone window launch with zero browser address-bar clutter.
2. **Privacy-Preserving Service Worker:** Caches static application shell assets (HTML, CSS, JS, fonts, SVGs) for instant launch, while strictly bypassing CacheStorage for authenticated genealogical records, living-person data, and session cookies.
3. **Subtle, Quiet Install Prompt:** Conforms strictly to the anti-marketing principle in [00 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards)—no intrusive popups or SaaS banners; install guidance is discrete, calm, and helpful.
4. **Mobile & Tablet Touch Ergonomics:**
   - 44×44px minimum interactive touch targets (48×48px on primary actions).
   - Bottom navigation (`BottomNav`) for primary thumb reach on mobile screens.
   - Smooth swipeable bottom sheets (`BottomSheet`) for quick person inspection, filters, and forms.
   - Hardware-accelerated multi-touch gestures on the SVG tree canvas (pinch-to-zoom, two-finger pan, tap-to-select).
   - Full support for notched displays, rounded corners, and home-indicator bars via CSS `env(safe-area-inset-*)`.

---

## 2. Data Model
Reference: [10-core-data-model.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md).

### 2.1 Storage & Caching Boundaries
No backend database schema additions are required. Storage on the client is strictly partitioned:

| Storage Mechanism | Allowed Content | Strictly Forbidden Content |
| :--- | :--- | :--- |
| **Service Worker `CacheStorage`** (`genealogy-shell-v{N}`) | Pre-hashed static assets: JS/CSS bundles, WOFF2 fonts, static SVGs, web app icons, offline fallback shell. | **All API responses (`/api/v1/*`)**, person names, photos, life events, session cookies, auth tokens. |
| **`localStorage`** | UI preferences (`genealogy_theme`, `genealogy_locale`), install prompt dismissal timestamp (`pwa_install_dismissed_at`). | Sensitive family data, passwords, recovery codes. |
| **`IndexedDB`** | Currently unused in Phase 1 MVP (reserved for Phase 2/3 local map tile caching). | Personal biographical facts, private documents. |

> [!CAUTION] Privacy & Security Guard:
> Family privacy is paramount. Service Worker caches must never hold sensitive living-person information or personal health/military facts. If an unauthenticated user or shared household device opens the PWA while offline, the app displays the static offline shell and login state—never stale cached family records.

---

## 3. User Experience & Flows

### 3.1 Web App Manifest (`manifest.webmanifest`)
The manifest is served from `/manifest.webmanifest` (Content-Type: `application/manifest+json`) and dynamically populated:

```json
{
  "id": "/?source=pwa",
  "start_url": "/?source=pwa",
  "scope": "/",
  "display": "standalone",
  "display_override": ["window-controls-overlay", "standalone", "minimal-ui"],
  "orientation": "any",
  "name": "Levshits Family Archive",
  "short_name": "Levshits Tree",
  "description": "Private family tree and historical archive",
  "lang": "en",
  "dir": "ltr",
  "theme_color": "#F8F9FA",
  "background_color": "#F8F9FA",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any"
    },
    {
      "src": "/icons/icon-maskable-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "maskable"
    },
    {
      "src": "/icons/icon-maskable-512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "maskable"
    },
    {
      "src": "/icons/icon.svg",
      "sizes": "any",
      "type": "image/svg+xml",
      "purpose": "any"
    }
  ],
  "categories": ["education", "lifestyle", "productivity"],
  "shortcuts": [
    {
      "name": "Family Tree",
      "short_name": "Tree",
      "url": "/tree?source=pwa_shortcut",
      "icons": [{ "src": "/icons/shortcut-tree.png", "sizes": "96x96" }]
    },
    {
      "name": "People Directory",
      "short_name": "People",
      "url": "/people?source=pwa_shortcut",
      "icons": [{ "src": "/icons/shortcut-people.png", "sizes": "96x96" }]
    }
  ]
}
```

- **Dynamic Theme Colors:**
  - Light Mode: `theme_color: "#F8F9FA"`, `background_color: "#F8F9FA"`.
  - Dark Mode: `theme_color: "#0E1116"`, `background_color: "#0E1116"`.
  - Updated dynamically in DOM via `<meta name="theme-color" content="...">` whenever the user toggles light/dark mode.
- **Dynamic Names:** `name` uses `instance_settings.tree_title` (e.g. *"Levshits Family Archive"* / *"Семейный архив Левшиц"*); `short_name` is truncated to 12 characters.

### 3.2 Apple iOS / iPadOS Safari Integration
Safari does not support the standard Web App Manifest install prompt banner. Full parity requires specific `<head>` meta tags:
- `<meta name="apple-mobile-web-app-capable" content="yes">`
- `<meta name="apple-mobile-web-app-status-bar-style" content="default">`
- `<meta name="apple-mobile-web-app-title" content="Family Tree">`
- `<link rel="apple-touch-icon" sizes="180x180" href="/icons/apple-touch-icon-180.png">`
- `<link rel="apple-touch-icon" sizes="167x167" href="/icons/apple-touch-icon-167.png">` (iPad Pro)
- `<link rel="apple-touch-icon" sizes="152x152" href="/icons/apple-touch-icon-152.png">` (iPad)
- Viewport configuration: `<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover, maximum-scale=5.0">`.

### 3.3 Quiet, Non-Intrusive Install Flow
In adherence to [00 §3.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#32-forbidden-marketing--promotional-buzzwords), install prompts must remain humble, calm, and quiet.

```
+--------------------------------------------------------------+
| [Tree Icon]  Install to Home Screen                          |
| Access the family archive quickly from your home screen.     |
|                                     [ Not Now ]  [ Install ] |
+--------------------------------------------------------------+
```

1. **Detection:**
   - Standalone check: `window.matchMedia('(display-mode: standalone)').matches` or `navigator.standalone === true`. When true, all install prompts are completely suppressed.
2. **Android & Chromium (Desktop/Mobile):**
   - Catches `beforeinstallprompt` event and saves the prompt deferred event.
   - Triggers:
     - Passive: A discreet menu action under *Settings & Profile* (`"Install Application"` / `"Установить приложение"`).
     - Contextual: A quiet bottom toast/banner shown after the user has completed at least 2 distinct browsing sessions, dismissible for 90 days.
   - Clicking *"Install"* executes `prompt()` and records outcome.
3. **iOS Safari:**
   - Displays a clean, 2-step visual guidance sheet when the user selects *"Install Application"* from the menu:
     1. *"Tap the Share button (⎋ / [↑]) in the browser toolbar."*
     2. *"Scroll down and select 'Add to Home Screen' ( ⊞ )."*
   - Includes a visual screenshot/diagram of the Safari share sheet.

### 3.4 Service Worker Lifecycle & Offline Experience
- **Registration:** Registered on page load (`/sw.js`) with scope `/`.
- **Pre-Caching:** Static assets built by Vite (hashed bundle files, Lucide icons, fonts, logo images).
- **Runtime Strategy:**
  - `GET /assets/*`, `GET /icons/*`, `GET /fonts/*`: **Cache-First**, falling back to network.
  - `GET /api/v1/*`: **Network-Only**. Never cached by Service Worker.
  - Navigation requests (`GET /`, `GET /tree`, etc.): **Network-First**, falling back to cached `index.html` shell.
- **Offline Banner:** When `navigator.onLine === false`, a sticky, non-blocking top bar appears:
  `"You are currently offline. Changes cannot be saved until your connection is restored."` / `"Вы не в сети. Изменения нельзя сохранить до восстановления связи."`
  State-changing buttons (`Save`, `Add`, `Upload`) show a disabled tooltip while offline.

### 3.5 Mobile & Tablet Responsive Architecture

```
Breakpoints:
  Mobile:  < 768px       (Compact single-column, bottom navigation, bottom sheets)
  Tablet:  768px - 1024px (Condensed topbar or side rail, 2-column stacked or 60/40 split)
  Desktop: > 1024px      (Full universal shell, side drawers, expanded tables)
```

#### Mobile Navigation (`< 768px`)
- **Top Header:** Compact height `h-14` + `env(safe-area-inset-top)`. Contains:
  - Left: Tree brand icon + short title.
  - Right: Quick Omnibox search trigger (`🔍`), User Avatar menu.
- **Bottom Navigation Bar (`BottomNav`):** Fixed at screen bottom, height `h-16` + `env(safe-area-inset-bottom)`.
  - 5 thumb-friendly items: `[ 🏠 Home ] [ 🌳 Tree ] [ 👥 People ] [ 🖼 Media ] [ ⋯ More ]`.
  - `More` button opens a bottom sheet with role-filtered items: Places, Language switcher, Theme toggle for all authenticated users; Print/PDF, Recent Changes, Data Quality, Duplicate Person Merge, and Admin Settings for Administrators.

#### Tablet Navigation (`768px – 1024px`)
- Topbar or vertical icon navigation rail (`w-16`).
- Canvas views occupy 100% of remaining area.
- Multi-pane views (such as Person Profile Timeline + Map) arrange either in an ergonomic 60/40 dual column or top-bottom split with toggleable tabs.

### 3.6 Interactive Tree Canvas Touch Gestures
On mobile and tablet devices, the SVG canvas (`/tree`) must support natural multi-touch gestures:

| Gesture | Interaction | Technical Handling |
| :--- | :--- | :--- |
| **Pinch-to-Zoom** | Two-finger pinch inwards/outwards | Calculates distance between touches, applies smooth logarithmic scale clamped between `0.25x` and `3.0x`. Center point of zoom follows touch centroid. |
| **Two-Finger Pan** | Dragging two fingers across canvas | Translates SVG `viewBox` coordinates smoothly with momentum physics. |
| **Single-Finger Drag** | Dragging one finger on empty canvas | Translates SVG viewport (pan). |
| **Tap on Person Card** | Tap with finger (< 250ms, < 10px move) | Selects person and opens **Quick-View Bottom Sheet**. Prevents accidental link clicks while panning. |
| **Long Press on Card** | Press and hold (> 500ms) | Opens contextual radial/popup action menu (*"View in Tree"*, *"Add Relative"*, *"Make Focal Person"*). Haptic feedback triggered via `navigator.vibrate(15)` if supported. |

#### Canvas Overlays on Mobile:
- Controls (Zoom `+` / `-`, Recenter `⌂`, Generation depth) condense into a compact floating pill in the top-right corner to preserve 90%+ canvas area.
- Bottom-right radar mini-map is collapsed by default on mobile (`< 768px`) with an icon toggle.

### 3.7 Mobile Quick-View Bottom Sheet (`BottomSheet`)
On mobile screens, clicking a card opens a swipeable **Bottom Sheet** rather than the desktop right-side slide-over drawer:
- **Snap Points:**
  - `Peek (25% height)`: Shows avatar, full name, birth/death years, kinship badge, and *"View Profile"* CTA. Keeps the tree context visible above.
  - `Expanded (75% height)`: Shows vital facts, immediate family links, and quick actions (*"Add Relative"*, *"Edit"*).
  - `Dismissed (0%)`: Swiping down past threshold smoothly dismisses the sheet.
- **Backdrop:** Non-blocking on peek mode (allows panning tree behind sheet); modal frosted backdrop on expanded mode.

### 3.8 Safe-Area Inset Handling
All fixed elements (top header, bottom navigation bar, floating canvas toolbars, toast alerts, bottom sheets) must apply CSS environment variables:
```css
/* Top app bar */
padding-top: max(0.75rem, env(safe-area-inset-top));

/* Bottom navigation bar & fixed action buttons */
padding-bottom: max(0.75rem, env(safe-area-inset-bottom));
padding-left: max(1rem, env(safe-area-inset-left));
padding-right: max(1rem, env(safe-area-inset-right));
```

---

## 4. Permissions & Privacy
- **Installation:** Any visitor (Admin, Editor, Viewer, or Public Guest) can install the application to their home screen.
- **Privacy Parity:** In standalone PWA mode, all RBAC and living-person privacy filtering rules ([02-rbac-access-control.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md)) operate identically to desktop browser sessions.
- **Multi-User Isolation:** If multiple users share a mobile tablet, logging out clears the local session cookie and flushes in-memory React query cache.

---

## 5. API Endpoints

| Method | Endpoint | Description | Content-Type |
| :--- | :--- | :--- | :--- |
| `GET` | `/manifest.webmanifest` | Web App Manifest with localized strings, dynamic theme colors, and icons. | `application/manifest+json` |
| `GET` | `/sw.js` | Service Worker script. | `application/javascript` |
| `GET` | `/icons/icon-192.png` | Standard icon (192×192). | `image/png` |
| `GET` | `/icons/icon-512.png` | Standard icon (512×512). | `image/png` |
| `GET` | `/icons/icon-maskable-192.png` | Maskable icon with safe zone (192×192). | `image/png` |
| `GET` | `/icons/icon-maskable-512.png` | Maskable icon with safe zone (512×512). | `image/png` |
| `GET` | `/icons/apple-touch-icon-180.png` | iOS home screen icon (180×180). | `image/png` |

---

## 6. Localization (EN / RU)
All PWA and mobile interface elements must have complete English and Russian localization:

| Key / Element | English (en) | Russian (ru) |
| :--- | :--- | :--- |
| `pwa.install_title` | Install Family Tree | Установить приложение |
| `pwa.install_prompt` | Add the family archive to your home screen for quick access. | Добавьте семейный архив на экран «Домой» для быстрого доступа. |
| `pwa.install_action` | Install | Установить |
| `pwa.ios_share_step` | 1. Tap the Share button in Safari | 1. Нажмите кнопку «Поделиться» в Safari |
| `pwa.ios_add_step` | 2. Select "Add to Home Screen" | 2. Выберите «На экран „Домой“» |
| `pwa.offline_banner` | You are offline. Changes cannot be saved. | Вы не в сети. Изменения нельзя сохранить. |
| `nav.more` | More | Ещё |

---

## 7. Edge Cases & Non-Goals

### Non-Goals:
- **Full Offline Tree Editing:** In Phase 1 MVP, data mutations require live server validation for optimistic concurrency and living-person privacy checks. Full offline local-database sync (e.g. SQLite WASM / CRDTs) is deferred to future major releases.
- **Push Notifications:** Family anniversary push notifications are handled via email digests ([16 §4.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md#42-optional-email-digest-off-by-default)); mobile Web Push is deferred.

### Edge Cases:
- **iOS Standalone Link Traversal:** Ensuring internal application links do not open in external Mobile Safari tabs. Handled via React Router internal history routing.
- **Notch Landscape Rotation:** When a tablet or mobile device is rotated sideways, asymmetrical horizontal insets (`safe-area-inset-left` and `safe-area-inset-right`) must pad side drawers and floating canvas buttons to prevent obscuration by the dynamic island / camera notch.
- **Double-Tap Zoom Conflict:** Double-tapping cards must not cause the mobile browser viewport to zoom in involuntarily; handled via `touch-action: manipulation` or custom canvas gesture listener.

---

## 8. Definition of Done (DoD)

- [ ] **Lighthouse PWA Validation:** The application passes all automated Lighthouse PWA installability audits (valid manifest, service worker registered, HTTPS/localhost, maskable icons).
- [ ] **Standalone Launch:** When launched from the mobile home screen, the app displays in standalone mode with no browser URL bar or bottom navigation chrome.
- [ ] **Zero Data Caching in SW:** Service Worker CacheStorage contains exclusively static assets; verification confirms zero `/api/v1/*` responses or private family records are stored.
- [ ] **Quiet Install Flow:** Install prompt appears strictly via menu action or quiet dismissible banner; no intrusive modal dialogs or marketing slogans.
- [ ] **Safe-Area Insets:** All fixed headers, bottom bars, bottom sheets, and floating canvas controls render clear of notches, camera cutouts, and iOS home-indicator bars in portrait and landscape.
- [ ] **Touch Target Minimums:** All interactive controls (buttons, links, form inputs, list items) have a minimum touch hit area of 44×44px.
- [ ] **Mobile Bottom Navigation:** On screens `< 768px`, primary navigation renders at the bottom of the viewport with thumb-accessible tabs and a `More` bottom sheet.
- [ ] **Pinch-to-Zoom & Pan:** Tree canvas supports smooth multi-touch pinch-to-zoom and two-finger panning at 60fps on iOS and Android touch screens.
- [ ] **Mobile Card Bottom Sheet:** Tapping a person card on mobile opens a swipeable bottom sheet with peek (25%) and expanded (75%) snap points, keeping the tree view visible.
- [ ] **Tablet Responsiveness:** All pages display cleanly on tablet screens (`768px – 1024px`) without awkward horizontal scrolling or clipped modals.
- [ ] **Full EN/RU Parity:** Manifest strings, install instructions, offline banners, and mobile navigation labels provide 100% parity in English and Russian.
