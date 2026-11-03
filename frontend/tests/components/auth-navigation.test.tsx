import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { ToastProvider, ConfirmProvider, AuthLayout } from '../../src/components/ui';
import { AppShell } from '../../src/components/ui/app-shell';
import { UserDto } from '../../src/api/client';

const mockAdminUser: UserDto = {
  id: 'u-admin-1',
  email: 'admin@levshits.family',
  display_name: 'Vladimir Levshits',
  role: 'admin',
  is_active: true,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};
import { LoginCard } from '../../src/components/auth';
import { AuthPages } from '../../src/pages/AuthPages';
import {
  sanitizeReturnUrl,
  isPublicGuestRoute,
  resolveGuestEgressTarget,
  buildLoginUrl,
} from '../../src/lib/navigation-utils';

function renderWithProviders(
  node: React.ReactNode,
  locale: 'en' | 'ru' = 'en',
  user: UserDto | null = null
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>
            <AuthProvider initialUser={user}>{node}</AuthProvider>
          </ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

describe('Milestone 2.4 — TASK-AUTH-PUBLIC-NAVIGATION Test Suite', () => {
  describe('1. Header Brand Navigation (AuthLayout)', () => {
    it('renders semantic <a href="/"> with focus ring, accessible label, and BrandLogoIcon', () => {
      const html = renderWithProviders(
        <AuthLayout title="Sign In" instanceTitle="Kuznetsov Archive">
          <div>Login Body</div>
        </AuthLayout>,
        'en'
      );

      expect(html).toContain('<a href="/"');
      expect(html).toContain('aria-label="Kuznetsov Archive"');
      expect(html).toContain('focus-visible:ring-focus');
      expect(html).toContain('focus-visible:ring-2');
      expect(html).toContain('Kuznetsov Archive');
    });

    it('renders localized default aria-label and title when instanceTitle is omitted', () => {
      const htmlEn = renderWithProviders(
        <AuthLayout title="Sign In">
          <div>Login Body</div>
        </AuthLayout>,
        'en'
      );
      expect(htmlEn).toContain('Family Archive');

      const htmlRu = renderWithProviders(
        <AuthLayout title="Вход в систему">
          <div>Login Body</div>
        </AuthLayout>,
        'ru'
      );
      expect(htmlRu).toContain('Семейный архив');
    });

    it('renders AppShell brand anchor and title with whitespace-nowrap and shrink-0 to prevent vertical wrapping', () => {
      const html = renderWithProviders(
        <AppShell instanceTitle="Family Tree Archive">
          <div>Page Content</div>
        </AppShell>,
        'en',
        mockAdminUser
      );

      expect(html).toContain('whitespace-nowrap');
      expect(html).toContain('truncate');
      expect(html).toContain('Family Tree Archive');
      expect(html).toContain('hidden xl:inline');
      expect(html).toContain('hidden lg:inline');
    });
  });

  describe('2. Public Guest Egress on LoginCard', () => {
    it('renders "Browse Public Archive" when enablePublicAccess=true and returnUrl is omitted or empty', () => {
      const htmlEn = renderWithProviders(
        <LoginCard enablePublicAccess={true} />,
        'en'
      );
      expect(htmlEn).toContain('Browse Public Archive');

      const htmlRu = renderWithProviders(
        <LoginCard enablePublicAccess={true} />,
        'ru'
      );
      expect(htmlRu).toContain('Перейти к публичному архиву');
    });

    it('renders "Back to Family Tree" when enablePublicAccess=true and returnUrl is /tree or /persons/:id', () => {
      const htmlTree = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/tree?focus=I001" />,
        'en'
      );
      expect(htmlTree).toContain('Back to Family Tree');
      expect(htmlTree).not.toContain('Browse Public Archive');

      const htmlPerson = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/persons/I0042" />,
        'en'
      );
      expect(htmlPerson).toContain('Back to Family Tree');
      expect(htmlPerson).not.toContain('Browse Public Archive');

      const htmlPersonRu = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/persons/I0042" />,
        'ru'
      );
      expect(htmlPersonRu).toContain('Вернуться к семейному древу');
      expect(htmlPersonRu).not.toContain('Перейти к публичному архиву');
    });

    it('falls back safely to "Browse Public Archive" when returnUrl is a restricted route (preventing loop)', () => {
      const htmlSettings = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/settings" />,
        'en'
      );
      expect(htmlSettings).toContain('Browse Public Archive');
      expect(htmlSettings).not.toContain('Back to Family Tree');

      const htmlPeople = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/people" />,
        'en'
      );
      expect(htmlPeople).toContain('Browse Public Archive');
      expect(htmlPeople).not.toContain('Back to Family Tree');

      const htmlMedia = renderWithProviders(
        <LoginCard enablePublicAccess={true} returnUrl="/media" />,
        'en'
      );
      expect(htmlMedia).toContain('Browse Public Archive');
      expect(htmlMedia).not.toContain('Back to Family Tree');
    });

    it('strictly omits egress button from DOM when enablePublicAccess=false', () => {
      const htmlEn = renderWithProviders(
        <LoginCard enablePublicAccess={false} returnUrl="/tree" />,
        'en'
      );
      expect(htmlEn).not.toContain('Browse Public Archive');
      expect(htmlEn).not.toContain('Back to Family Tree');

      const htmlRu = renderWithProviders(
        <LoginCard enablePublicAccess={false} returnUrl="/tree" />,
        'ru'
      );
      expect(htmlRu).not.toContain('Перейти к публичному архиву');
      expect(htmlRu).not.toContain('Вернуться к семейному древу');
    });
  });

  describe('3. AuthPages Rendering and Route Integration', () => {
    it('renders LoginCard inside AuthLayout when currentPath is /login', () => {
      const html = renderWithProviders(
        <AuthPages
          currentPath="/login"
          search="?returnUrl=%2Ftree"
          instanceTitle="Family Heritage"
          onNavigate={() => {}}
        />,
        'en'
      );

      expect(html).toContain('Family Heritage');
      expect(html).toContain('href="/"');
      expect(html).toContain('Sign in with Passkey');
    });

    it('renders RegisterInviteCard when currentPath is /register', () => {
      const html = renderWithProviders(
        <AuthPages
          currentPath="/register"
          search="?invite_token=sample-token&returnUrl=%2Ftree"
          instanceTitle="Family Heritage"
          inviteToken="sample-token"
          onNavigate={() => {}}
        />,
        'en'
      );

      expect(html).toContain('Family Heritage');
      expect(html).toContain('href="/"');
    });

    it('renders MagicLinkVerifyCard when currentPath is /magic-link/verify', () => {
      const html = renderWithProviders(
        <AuthPages
          currentPath="/magic-link/verify"
          search="?token=sample-token&returnUrl=%2Ftree"
          instanceTitle="Family Heritage"
          onNavigate={() => {}}
        />,
        'en'
      );

      expect(html).toContain('Family Heritage');
      expect(html).toContain('href="/"');
    });
  });

  describe('4. Navigation Utils & Redirection Defenses (Traceable DoD)', () => {
    describe('sanitizeReturnUrl', () => {
      it('preserves valid relative paths with query parameters and hash anchors', () => {
        expect(sanitizeReturnUrl('/tree')).toBe('/tree');
        expect(sanitizeReturnUrl('/tree?focus=I001')).toBe('/tree?focus=I001');
        expect(sanitizeReturnUrl('/tree?focus=I001#card')).toBe(
          '/tree?focus=I001#card'
        );
        expect(sanitizeReturnUrl('/persons/I0001')).toBe('/persons/I0001');
        expect(sanitizeReturnUrl('/settings/security')).toBe('/settings/security');
      });

      it('neutralizes protocol-relative open-redirect payloads (//evil.com)', () => {
        expect(sanitizeReturnUrl('//evil.com')).toBe('/');
        expect(sanitizeReturnUrl('//evil.com/phish')).toBe('/');
        expect(sanitizeReturnUrl('///evil.com')).toBe('/');
        expect(sanitizeReturnUrl('////evil.com')).toBe('/');
      });

      it('neutralizes absolute URLs with external schemes', () => {
        expect(sanitizeReturnUrl('https://attacker.org')).toBe('/');
        expect(sanitizeReturnUrl('http://attacker.org/steal')).toBe('/');
        expect(sanitizeReturnUrl('ftp://attacker.org')).toBe('/');
      });

      it('neutralizes pseudo-schemes (javascript:, data:, vbscript:)', () => {
        expect(sanitizeReturnUrl('javascript:alert(1)')).toBe('/');
        expect(sanitizeReturnUrl('/javascript:alert(1)')).toBe('/');
        expect(sanitizeReturnUrl('data:text/html,<script>alert(1)</script>')).toBe(
          '/'
        );
        expect(sanitizeReturnUrl('/data:text/html,evil')).toBe('/');
        expect(sanitizeReturnUrl('vbscript:msgbox(1)')).toBe('/');
        expect(sanitizeReturnUrl('/vbscript:msgbox(1)')).toBe('/');
      });

      it('neutralizes backslashes to prevent URL parsing confusion (\\evil.com)', () => {
        expect(sanitizeReturnUrl('\\evil.com')).toBe('/');
        expect(sanitizeReturnUrl('/\\evil.com')).toBe('/');
        expect(sanitizeReturnUrl('/tree\\evil')).toBe('/');
        expect(sanitizeReturnUrl('\\\\evil.com')).toBe('/');
      });

      it('neutralizes control characters and CRLF injection payloads', () => {
        expect(sanitizeReturnUrl('/tree\nSet-Cookie:bad')).toBe('/');
        expect(sanitizeReturnUrl('/tree\r\nLocation:evil')).toBe('/');
        expect(sanitizeReturnUrl('/tree\0payload')).toBe('/');
        expect(sanitizeReturnUrl('/tree\x1b[31m')).toBe('/');
      });

      it('neutralizes auth loop targets (/login, /register, /setup, /magic-link)', () => {
        expect(sanitizeReturnUrl('/login')).toBe('/');
        expect(sanitizeReturnUrl('/login?returnUrl=/tree')).toBe('/');
        expect(sanitizeReturnUrl('/register')).toBe('/');
        expect(sanitizeReturnUrl('/register/invite')).toBe('/');
        expect(sanitizeReturnUrl('/setup')).toBe('/');
        expect(sanitizeReturnUrl('/setup/admin')).toBe('/');
        expect(sanitizeReturnUrl('/magic-link/verify?token=tok')).toBe('/');
      });

      it('respects custom fallback when specified', () => {
        expect(sanitizeReturnUrl('//evil.com', '/fallback')).toBe('/fallback');
        expect(sanitizeReturnUrl(null, '/fallback')).toBe('/fallback');
        expect(sanitizeReturnUrl('', '/fallback')).toBe('/fallback');
      });
    });

    describe('isPublicGuestRoute', () => {
      it('allows /, /tree, and /persons/:id as public guest surface', () => {
        expect(isPublicGuestRoute('/')).toBe(true);
        expect(isPublicGuestRoute('/tree')).toBe(true);
        expect(isPublicGuestRoute('/tree?focus=I001')).toBe(true);
        expect(isPublicGuestRoute('/tree/sub')).toBe(true);
        expect(isPublicGuestRoute('/persons/I0001')).toBe(true);
        expect(isPublicGuestRoute('/persons/abc-123')).toBe(true);
      });

      it('blocks authenticated-only auxiliary routes from guest access', () => {
        expect(isPublicGuestRoute('/people')).toBe(false);
        expect(isPublicGuestRoute('/media')).toBe(false);
        expect(isPublicGuestRoute('/places')).toBe(false);
        expect(isPublicGuestRoute('/settings')).toBe(false);
        expect(isPublicGuestRoute('/history')).toBe(false);
        expect(isPublicGuestRoute('/quality')).toBe(false);
        expect(isPublicGuestRoute('/export')).toBe(false);
        expect(isPublicGuestRoute('/persons/')).toBe(false);
        expect(isPublicGuestRoute(null)).toBe(false);
      });
    });

    describe('resolveGuestEgressTarget', () => {
      it('returns permitted public returnUrl directly', () => {
        expect(resolveGuestEgressTarget('/tree')).toBe('/tree');
        expect(resolveGuestEgressTarget('/tree?focus=I002')).toBe('/tree?focus=I002');
        expect(resolveGuestEgressTarget('/persons/I0002')).toBe('/persons/I0002');
      });

      it('falls back to "/" for restricted or invalid returnUrl', () => {
        expect(resolveGuestEgressTarget('/settings')).toBe('/');
        expect(resolveGuestEgressTarget('/people')).toBe('/');
        expect(resolveGuestEgressTarget('/media')).toBe('/');
        expect(resolveGuestEgressTarget('//evil.com')).toBe('/');
        expect(resolveGuestEgressTarget('/login')).toBe('/');
        expect(resolveGuestEgressTarget('')).toBe('/');
        expect(resolveGuestEgressTarget(null)).toBe('/');
      });
    });

    describe('buildLoginUrl', () => {
      it('attaches sanitized returnUrl parameter for non-root, non-auth paths', () => {
        expect(buildLoginUrl('/tree')).toBe('/login?returnUrl=%2Ftree');
        expect(buildLoginUrl('/persons/I001')).toBe('/login?returnUrl=%2Fpersons%2FI001');
        expect(buildLoginUrl('/settings', '?tab=security')).toBe(
          '/login?returnUrl=%2Fsettings%3Ftab%3Dsecurity'
        );
      });

      it('omits returnUrl when on root "/" or auth views', () => {
        expect(buildLoginUrl('/')).toBe('/login');
        expect(buildLoginUrl('/login')).toBe('/login');
        expect(buildLoginUrl('/register')).toBe('/login');
        expect(buildLoginUrl('/setup')).toBe('/login');
      });
    });
  });
});
