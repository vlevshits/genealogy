# Progressive Web App & Mobile UX — Implementation Memory

**Spec:** [docs/features/18-progressive-web-app-mobile.md](../../features/18-progressive-web-app-mobile.md) · **Status:** implemented

## 1. Summary
Full Progressive Web App (PWA) capabilities, mobile-optimized layouts, and touch ergonomics for smartphones and tablets. Includes Web App Manifest, offline static asset caching via Service Worker, touch gesture handling (pinch-to-zoom and pan on the tree canvas), minimum 40px touch targets, and iOS home-screen launch support.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Manifest | `frontend/public/manifest.webmanifest` | App name, icons, standalone display mode, theme colors |
| Service Worker | `frontend/public/sw.js` | Cache-first for immutable static assets, network-first for API |
| Entry HTML | `frontend/index.html` | Meta viewport, apple-touch-icon, theme-color tags |
| Components | `frontend/src/components/pwa/` | Install prompt banner, offline notification bar |
| Touch Controls | `frontend/src/components/tree/` | Pinch-to-zoom multi-touch handlers in `TreeCanvas.tsx` |
| Tests | `frontend/tests/pwa/` | `pwa-manifest-sw.test.tsx` (manifest validation, cache headers) |

## 3. Key Flows
1. **PWA Installation Flow:**
   - User visits family tree on mobile Chrome or Safari.
   - PWA install criteria are met → UI presents custom "Add to Home Screen" prompt.
   - App launches standalone without browser navigation bars.
2. **Mobile Tree Navigation:**
   - Two-finger pinch triggers smooth canvas zoom centered on the midpoint between touch contact points.
   - One-finger drag pans the canvas viewport.

## 4. Invariants & Business Rules
- **No API Caching in Service Worker:** API responses (`/api/v1/*`) are never cached by the Service Worker to avoid serving stale living-person data or violating privacy policies.
- **Touch Target Floor:** All interactive buttons, tabs, and inputs must have an active target area of at least 40×40px.

## 5. Decisions
- **Network-First for Content:** Family data changes in real time; only static UI assets (JS, CSS, icons, fonts) use cache-first strategy.

## 6. Gotchas & Known Limitations
- iOS Safari requires explicit `apple-touch-icon` links and standalone viewport meta tags to avoid white flash on launch.

_Last curated: 2026-11-04 (memory bank bootstrap)_
