/**
 * Navigation utilities for safe redirect handling, public guest route checks,
 * and guest egress resolution.
 */

/**
 * Sanitizes a candidate return URL to prevent Open Redirect attacks, protocol-relative
 * redirects, control-character injections, and auth bounce loops.
 *
 * @param url The raw return URL string from search parameters or user input
 * @param defaultFallback Fallback path if the URL is invalid or unsafe (defaults to '/')
 * @returns A safe, internal relative path
 */
export function sanitizeReturnUrl(
  url: string | null | undefined,
  defaultFallback: string = '/'
): string {
  if (!url || typeof url !== 'string') {
    return defaultFallback;
  }

  const trimmed = url.trim();

  // Must begin with a single '/' and must NOT begin with '//' or contain '\'
  if (!trimmed.startsWith('/') || trimmed.startsWith('//') || trimmed.includes('\\')) {
    return defaultFallback;
  }

  // Reject ASCII control characters (< 0x20 or 0x7F)
  // eslint-disable-next-line no-control-regex
  if (/[\x00-\x1F\x7F]/.test(trimmed)) {
    return defaultFallback;
  }

  // Reject explicit pseudo-schemes in path like /javascript: or /data:
  const lower = trimmed.toLowerCase();
  if (
    lower.startsWith('/javascript:') ||
    lower.startsWith('/data:') ||
    lower.startsWith('/vbscript:')
  ) {
    return defaultFallback;
  }

  try {
    const parsed = new URL(trimmed, 'http://localhost');
    // Ensure URL resolved to our mock host without protocol hopping
    if (parsed.origin !== 'http://localhost') {
      return defaultFallback;
    }

    const pathname = parsed.pathname;

    // Prevent auth redirect loops: do not redirect back into auth views
    if (
      pathname === '/login' ||
      pathname.startsWith('/login/') ||
      pathname === '/register' ||
      pathname.startsWith('/register/') ||
      pathname === '/setup' ||
      pathname.startsWith('/setup/') ||
      pathname === '/magic-link' ||
      pathname.startsWith('/magic-link/')
    ) {
      return defaultFallback;
    }

    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return defaultFallback;
  }
}

/**
 * Determines whether a given internal path is part of the permitted public guest surface
 * (/ , /tree*, /persons/:id) per 01-auth-pages.md §1 & 02-rbac-access-control.md §3.2.
 */
export function isPublicGuestRoute(url: string | null | undefined): boolean {
  if (!url || typeof url !== 'string') {
    return false;
  }

  const trimmed = url.trim();
  if (!trimmed.startsWith('/') || trimmed.startsWith('//') || trimmed.includes('\\')) {
    return false;
  }

  const pathname = trimmed.split(/[?#]/)[0];

  if (pathname === '/') {
    return true;
  }

  if (pathname === '/tree' || pathname.startsWith('/tree/')) {
    return true;
  }

  if (pathname.startsWith('/persons/') && pathname.length > '/persons/'.length) {
    return true;
  }

  return false;
}

/**
 * Resolves the destination for guest egress on LoginCard ("Browse Public Archive" / "Back to Tree").
 * If returnUrl is present and points to a permitted public route, returns returnUrl.
 * If returnUrl points to a restricted route (/settings, /people, etc.) or is empty,
 * safely falls back to '/' to avoid redirect loops.
 */
export function resolveGuestEgressTarget(returnUrl?: string | null): string {
  const sanitized = sanitizeReturnUrl(returnUrl, '/');
  if (sanitized === '/') {
    return '/';
  }

  if (isPublicGuestRoute(sanitized)) {
    return sanitized;
  }

  return '/';
}

/**
 * Constructs a /login URL carrying the current relative path as returnUrl,
 * omitting returnUrl if already on an auth route or on root '/'.
 */
export function buildLoginUrl(
  currentPath?: string | null,
  search?: string | null
): string {
  if (!currentPath || typeof currentPath !== 'string') {
    return '/login';
  }

  const trimmed = currentPath.trim();
  if (!trimmed.startsWith('/') || trimmed.startsWith('//') || trimmed.includes('\\')) {
    return '/login';
  }

  let full = trimmed;
  if (search && typeof search === 'string') {
    const sTrimmed = search.trim();
    if (sTrimmed && !full.includes('?')) {
      full = `${full}${sTrimmed.startsWith('?') ? sTrimmed : `?${sTrimmed}`}`;
    }
  }

  const pathname = full.split(/[?#]/)[0];

  // Do not attach returnUrl if current path is auth route or plain root
  if (
    pathname === '/login' ||
    pathname.startsWith('/login/') ||
    pathname === '/register' ||
    pathname.startsWith('/register/') ||
    pathname === '/setup' ||
    pathname.startsWith('/setup/') ||
    pathname === '/magic-link' ||
    pathname.startsWith('/magic-link/')
  ) {
    return '/login';
  }

  if (pathname === '/' && !full.includes('?') && !full.includes('#')) {
    return '/login';
  }

  const sanitized = sanitizeReturnUrl(full, '/');
  if (sanitized === '/') {
    return '/login';
  }

  return `/login?returnUrl=${encodeURIComponent(sanitized)}`;
}
