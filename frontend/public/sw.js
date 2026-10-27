/**
 * Levshits Family Archive — Privacy-Preserving Service Worker
 * Cache Name: genealogy-shell-v1
 *
 * PRIVACY GUARANTEE:
 * - NEVER caches any /api/* JSON response
 * - NEVER caches any /internal-media/* or /media/file/* asset
 * - Only caches static application shell assets (JS, CSS, SVG, fonts, icons, index.html)
 */

const CACHE_NAME = 'genealogy-shell-v1';

const PRECACHE_URLS = [
  '/',
  '/index.html',
  '/manifest.webmanifest',
  '/favicon.svg',
  '/icons/icon.svg',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key !== CACHE_NAME)
            .map((oldKey) => caches.delete(oldKey))
        )
      )
      .then(() => self.clients.claim())
  );
});

/**
 * Returns true if the request URL targets a private API or protected media endpoint
 * that must NEVER be stored in CacheStorage.
 */
function isPrivateOrApiRequest(url) {
  return (
    url.pathname.startsWith('/api/') ||
    url.pathname === '/api' ||
    url.pathname.startsWith('/internal-media/') ||
    url.pathname === '/internal-media'
  );
}

/**
 * Returns true if the request URL is an immutable/static asset suitable for Cache-First.
 */
function isStaticShellAsset(url) {
  return (
    url.pathname.startsWith('/assets/') ||
    url.pathname.startsWith('/icons/') ||
    url.pathname.startsWith('/fonts/') ||
    url.pathname === '/favicon.svg' ||
    url.pathname === '/manifest.webmanifest'
  );
}

self.addEventListener('fetch', (event) => {
  const { request } = event;

  if (request.method !== 'GET') {
    return;
  }

  const url = new URL(request.url);

  // Only handle same-origin requests
  if (url.origin !== self.location.origin) {
    return;
  }

  // Strict Privacy Boundary: NEVER cache /api/* or /internal-media/*
  if (isPrivateOrApiRequest(url)) {
    event.respondWith(fetch(request));
    return;
  }

  // Cache-First for static assets (/assets/*, /icons/*, /fonts/*, /favicon.svg)
  if (isStaticShellAsset(url)) {
    event.respondWith(
      caches.match(request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }
        return fetch(request).then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const responseClone = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put(request, responseClone);
            });
          }
          return networkResponse;
        });
      })
    );
    return;
  }

  // Network-First with SPA fallback to /index.html for HTML navigation requests
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const responseClone = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put('/index.html', responseClone);
            });
          }
          return networkResponse;
        })
        .catch(() =>
          caches.match('/index.html').then((cachedShell) => {
            if (cachedShell) {
              return cachedShell;
            }
            return new Response('Offline', {
              status: 503,
              headers: { 'Content-Type': 'text/plain; charset=utf-8' },
            });
          })
        )
    );
  }
});
