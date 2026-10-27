# Implementation Report: Phase 1.13 — Progressive Web App (PWA) & Mobile/Tablet Experience

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 13 of [`docs/roadmap.md`](../roadmap.md) — First-class Progressive Web App (PWA) with Web App Manifest, maskable and Apple touch icons, privacy-preserving static shell Service Worker (strict zero-cache guarantee on `/api/*` and media), mobile `BottomNav`, swipeable `BottomSheet`, safe-area insets, and multi-touch gestures on the SVG tree canvas ([`18-progressive-web-app-mobile.md`](../features/18-progressive-web-app-mobile.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Web App Manifest & App Shell Integration (`frontend/public/`, `frontend/index.html`)
- **Web App Manifest ([`frontend/public/manifest.webmanifest`](../../frontend/public/manifest.webmanifest)):**
  - Configured with `display: "standalone"`, `display_override: ["window-controls-overlay", "standalone", "minimal-ui"]`, `start_url: "/?source=pwa"`, `theme_color: "#F8F9FA"`, `background_color: "#F8F9FA"`.
  - Icon assets generated in `frontend/public/icons/`: 192x192, 512x512, maskable 192x192, maskable 512x512, and scalable SVG.
  - App shortcuts configured for quick launch: Family Tree (`/tree?source=pwa_shortcut`) and People Directory (`/people?source=pwa_shortcut`).
- **Apple & Mobile Head Meta Tags ([`frontend/index.html`](../../frontend/index.html)):**
  - `viewport-fit=cover` in viewport meta tag.
  - `apple-mobile-web-app-capable: yes`, `apple-mobile-web-app-status-bar-style: default`.
  - Apple touch icon `<link rel="apple-touch-icon" href="/icons/icon-192.png">`.
  - Dynamic `<meta name="theme-color">` synchronization in `theme.tsx` between `#F8F9FA` (light) and `#0E1116` (dark).
- **Nginx Caching Rules ([`frontend/nginx.conf`](../../frontend/nginx.conf)):**
  - `/sw.js` and `/manifest.webmanifest` served with `Cache-Control: no-cache, no-store, must-revalidate`.
  - Correct MIME type mapping for `application/manifest+json`.

### 1.2 Privacy-Preserving Static Shell Service Worker ([`frontend/public/sw.js`](../../frontend/public/sw.js))
- **MANDATORY PRIVACY GUARANTEE:**
  - The Service Worker strictly intercepts and inspects every outgoing fetch.
  - If `url.pathname.startsWith('/api/')` or `url.pathname.startsWith('/internal-media/')`, it **bypasses `CacheStorage` unconditionally** and fetches directly from the network (`event.respondWith(fetch(request))`).
  - Genealogical records, living person details, and session cookies are NEVER stored in browser CacheStorage.
- **Static Shell Caching (`genealogy-shell-v1`):**
  - Caches only hashed JS, CSS, static SVG, WOFF2 fonts, and the base `index.html` shell for fast, reliable application boot.
  - Graceful cleanup of outdated cache versions on activation.

### 1.3 Mobile Navigation, Ergonomics & Gestures
- **Tree Canvas Touch Gestures ([`frontend/src/components/tree/TreeCanvas.tsx`](../../frontend/src/components/tree/TreeCanvas.tsx)):**
  - Hardware-accelerated centroid-anchored two-finger pinch-to-zoom (`0.25x` to `3.0x`) and two-finger panning.
  - `touch-action: none` applied to canvas container to prevent browser scroll/pull-to-refresh conflicts.
  - Unified PointerEvent tracking prevents competing event race conditions.
  - 6px movement threshold on cards cancels long-press selection during canvas panning.
- **Mobile Toolbar & BottomNav Clearance ([`frontend/src/components/tree/TreeToolbar.tsx`](../../frontend/src/components/tree/TreeToolbar.tsx)):**
  - Top controls stack cleanly on narrow viewports (`< 640px`) without collision.
  - Zoom controls and mini-map toggle sit at `bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))]`, clearing the mobile `BottomNav`.
  - Viewport height uses `100dvh` to prevent mobile address bar jump.
  - All interactive buttons meet or exceed the 44×44px touch target standard.
- **PWA Install Prompt & Offline Banner ([`frontend/src/components/pwa/`](../../frontend/src/components/pwa/)):**
  - `<PwaInstallPrompt />`: Non-intrusive, dismissible banner listening to `beforeinstallprompt` with a 90-day dismissal cooldown stored in `localStorage` (`pwa_install_dismissed_at`), plus a 2-step iOS Safari guide.
  - `<PwaOfflineBanner />`: Quiet banner notifying the user when offline with safe-area padding (`pt-[max(0.5rem,env(safe-area-inset-top,0px))]`).

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Architect Review** | `architect-reviewer` (Mode 2) | **APPROVE** | Validated strict zero-cache API guarantee, standalone manifest, and mobile touch ergonomics. |
| **QA Verification** | `qa-analyst` (DoD + Playwright) | **PASS** | Live browser tests verified `/manifest.webmanifest` (200 OK), `/sw.js` (200 OK), offline banner triggers, and touchAction "none". |
| **UX Review** | `ux-reviewer` | **OK** | Verified mobile viewports (390px, 768px, desktop); resolved `UX-001`, `UX-002`, `UX-006`, `UX-007`, `UX-008`. |
| **PWA & SW Tests** | `npm test -- tests/pwa/pwa-manifest-sw.test.tsx` | **PASS (5/5)** | Validated manifest structure and SW network-only bypass logic. |
| **Full Frontend Tests** | `npm test -- --run` | **PASS (98/98)** | 14 test files passed with zero failures. |
| **Production Build** | `npm run build` | **PASS** | Clean build in 1.09s (`tsc --noEmit && vite build`). |
