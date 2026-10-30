import { describe, it, expect } from 'vitest';
import {
  sanitizeReturnUrl,
  isPublicGuestRoute,
  resolveGuestEgressTarget,
  buildLoginUrl,
} from '../../src/lib/navigation-utils';

describe('navigation-utils', () => {
  describe('sanitizeReturnUrl', () => {
    it('accepts valid internal relative paths', () => {
      expect(sanitizeReturnUrl('/')).toBe('/');
      expect(sanitizeReturnUrl('/tree')).toBe('/tree');
      expect(sanitizeReturnUrl('/tree?focus=I0001')).toBe('/tree?focus=I0001');
      expect(sanitizeReturnUrl('/tree?focus=I0001#node-1')).toBe(
        '/tree?focus=I0001#node-1'
      );
      expect(sanitizeReturnUrl('/persons/I0001')).toBe('/persons/I0001');
      expect(sanitizeReturnUrl('/people')).toBe('/people');
      expect(sanitizeReturnUrl('/media?view=grid')).toBe('/media?view=grid');
    });

    it('falls back to default "/" or custom fallback for null, undefined, or empty', () => {
      expect(sanitizeReturnUrl(null)).toBe('/');
      expect(sanitizeReturnUrl(undefined)).toBe('/');
      expect(sanitizeReturnUrl('')).toBe('/');
      expect(sanitizeReturnUrl('   ')).toBe('/');
      expect(sanitizeReturnUrl('', '/custom')).toBe('/custom');
      expect(sanitizeReturnUrl(null, '/custom')).toBe('/custom');
    });

    it('rejects open-redirect attacks with protocol-relative URLs', () => {
      expect(sanitizeReturnUrl('//evil.com')).toBe('/');
      expect(sanitizeReturnUrl('//evil.com/tree')).toBe('/');
      expect(sanitizeReturnUrl('///evil.com')).toBe('/');
      expect(sanitizeReturnUrl('////evil.com/payload')).toBe('/');
    });

    it('rejects absolute URLs with explicit schemes (http, https, ftp)', () => {
      expect(sanitizeReturnUrl('https://evil.com')).toBe('/');
      expect(sanitizeReturnUrl('https://evil.com/tree')).toBe('/');
      expect(sanitizeReturnUrl('http://attacker.com')).toBe('/');
      expect(sanitizeReturnUrl('ftp://attacker.com/file')).toBe('/');
    });

    it('rejects javascript, data, and vbscript pseudo-schemes', () => {
      expect(sanitizeReturnUrl('javascript:alert(1)')).toBe('/');
      expect(sanitizeReturnUrl('/javascript:alert(1)')).toBe('/');
      expect(sanitizeReturnUrl('data:text/html,<script>alert(1)</script>')).toBe('/');
      expect(sanitizeReturnUrl('/data:text/html,evil')).toBe('/');
      expect(sanitizeReturnUrl('vbscript:msgbox(1)')).toBe('/');
      expect(sanitizeReturnUrl('/vbscript:msgbox(1)')).toBe('/');
    });

    it('rejects backslashes to prevent browser protocol/slash confusion', () => {
      expect(sanitizeReturnUrl('/\\evil.com')).toBe('/');
      expect(sanitizeReturnUrl('/\\/evil.com')).toBe('/');
      expect(sanitizeReturnUrl('\\\\evil.com')).toBe('/');
      expect(sanitizeReturnUrl('/tree\\evil')).toBe('/');
    });

    it('rejects control characters, newlines, and null bytes', () => {
      expect(sanitizeReturnUrl('/tree\nSet-Cookie:session=evil')).toBe('/');
      expect(sanitizeReturnUrl('/tree\r\nLocation:http://evil.com')).toBe('/');
      expect(sanitizeReturnUrl('/tree\0payload')).toBe('/');
      expect(sanitizeReturnUrl('/tree\x1b')).toBe('/');
    });

    it('prevents auth redirect loops by resetting auth view targets to fallback', () => {
      expect(sanitizeReturnUrl('/login')).toBe('/');
      expect(sanitizeReturnUrl('/login?returnUrl=/tree')).toBe('/');
      expect(sanitizeReturnUrl('/register')).toBe('/');
      expect(sanitizeReturnUrl('/register?invite_token=abc')).toBe('/');
      expect(sanitizeReturnUrl('/setup')).toBe('/');
      expect(sanitizeReturnUrl('/setup?step=2')).toBe('/');
      expect(sanitizeReturnUrl('/magic-link/verify?token=xyz')).toBe('/');
    });
  });

  describe('isPublicGuestRoute', () => {
    it('identifies public guest permitted routes', () => {
      expect(isPublicGuestRoute('/')).toBe(true);
      expect(isPublicGuestRoute('/tree')).toBe(true);
      expect(isPublicGuestRoute('/tree?focus=I001')).toBe(true);
      expect(isPublicGuestRoute('/tree/subview')).toBe(true);
      expect(isPublicGuestRoute('/persons/I0001')).toBe(true);
      expect(isPublicGuestRoute('/persons/c7a8b9d0-1234')).toBe(true);
      expect(isPublicGuestRoute('/persons/c7a8b9d0-1234?tab=facts')).toBe(true);
    });

    it('identifies restricted routes not permitted for guests', () => {
      expect(isPublicGuestRoute('/people')).toBe(false);
      expect(isPublicGuestRoute('/media')).toBe(false);
      expect(isPublicGuestRoute('/places')).toBe(false);
      expect(isPublicGuestRoute('/settings')).toBe(false);
      expect(isPublicGuestRoute('/settings/profile')).toBe(false);
      expect(isPublicGuestRoute('/history')).toBe(false);
      expect(isPublicGuestRoute('/quality')).toBe(false);
      expect(isPublicGuestRoute('/export')).toBe(false);
      expect(isPublicGuestRoute('/dev/ui')).toBe(false);
      expect(isPublicGuestRoute('/login')).toBe(false);
      expect(isPublicGuestRoute('/register')).toBe(false);
      expect(isPublicGuestRoute('/setup')).toBe(false);
      expect(isPublicGuestRoute('/magic-link/verify')).toBe(false);
      expect(isPublicGuestRoute('/persons/')).toBe(false);
      expect(isPublicGuestRoute(null)).toBe(false);
      expect(isPublicGuestRoute(undefined)).toBe(false);
      expect(isPublicGuestRoute('')).toBe(false);
      expect(isPublicGuestRoute('//evil.com')).toBe(false);
      expect(isPublicGuestRoute('https://evil.com/tree')).toBe(false);
    });
  });

  describe('resolveGuestEgressTarget', () => {
    it('returns public destination when returnUrl points to permitted guest route', () => {
      expect(resolveGuestEgressTarget('/tree')).toBe('/tree');
      expect(resolveGuestEgressTarget('/tree?focus=I001')).toBe('/tree?focus=I001');
      expect(resolveGuestEgressTarget('/persons/I0001')).toBe('/persons/I0001');
    });

    it('falls back safely to "/" when returnUrl points to restricted route (preventing bounce loops)', () => {
      expect(resolveGuestEgressTarget('/people')).toBe('/');
      expect(resolveGuestEgressTarget('/media')).toBe('/');
      expect(resolveGuestEgressTarget('/settings')).toBe('/');
      expect(resolveGuestEgressTarget('/history')).toBe('/');
      expect(resolveGuestEgressTarget('/quality')).toBe('/');
    });

    it('falls back safely to "/" when returnUrl is missing, malicious, or an auth route', () => {
      expect(resolveGuestEgressTarget(null)).toBe('/');
      expect(resolveGuestEgressTarget(undefined)).toBe('/');
      expect(resolveGuestEgressTarget('')).toBe('/');
      expect(resolveGuestEgressTarget('//evil.com')).toBe('/');
      expect(resolveGuestEgressTarget('https://evil.com')).toBe('/');
      expect(resolveGuestEgressTarget('/login')).toBe('/');
    });
  });

  describe('buildLoginUrl', () => {
    it('constructs /login URL with sanitized returnUrl for protected routes', () => {
      expect(buildLoginUrl('/tree')).toBe('/login?returnUrl=%2Ftree');
      expect(buildLoginUrl('/tree', '?focus=I001')).toBe(
        '/login?returnUrl=%2Ftree%3Ffocus%3DI001'
      );
      expect(buildLoginUrl('/persons/I0001')).toBe(
        '/login?returnUrl=%2Fpersons%2FI0001'
      );
      expect(buildLoginUrl('/people')).toBe('/login?returnUrl=%2Fpeople');
      expect(buildLoginUrl('/settings/security')).toBe(
        '/login?returnUrl=%2Fsettings%2Fsecurity'
      );
    });

    it('does not append returnUrl when on root "/" without search', () => {
      expect(buildLoginUrl('/')).toBe('/login');
    });

    it('does not append returnUrl when already on an auth route', () => {
      expect(buildLoginUrl('/login')).toBe('/login');
      expect(buildLoginUrl('/register')).toBe('/login');
      expect(buildLoginUrl('/setup')).toBe('/login');
      expect(buildLoginUrl('/magic-link/verify')).toBe('/login');
    });

    it('safely handles malicious or empty inputs', () => {
      expect(buildLoginUrl(null)).toBe('/login');
      expect(buildLoginUrl(undefined)).toBe('/login');
      expect(buildLoginUrl('')).toBe('/login');
      expect(buildLoginUrl('//evil.com')).toBe('/login');
      expect(buildLoginUrl('https://evil.com')).toBe('/login');
    });
  });
});
