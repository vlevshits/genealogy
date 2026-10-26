import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import {
  ToastProvider,
  ConfirmProvider,
  ThemeToggleButton,
  LanguageToggleButton,
  AuthLayout,
} from '../../src/components/ui';
import {
  LoginCard,
  MagicLinkVerifyCard,
  RecoveryCodesCard,
  RegisterInviteCard,
  SetupWizardCard,
} from '../../src/components/auth';
import { clearMagicLinkVerificationCache } from '../../src/components/auth/MagicLinkVerifyCard';
import {
  base64UrlToUint8Array,
  bufferToBase64Url,
} from '../../src/lib/webauthn';
import {
  FamilyMembersInvitesPanel,
  SecurityPasskeysPanel,
} from '../../src/components/settings';

function renderWithProviders(
  node: React.ReactNode,
  locale: 'en' | 'ru' = 'en'
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>
            <AuthProvider>{node}</AuthProvider>
          </ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

const TEN_SAMPLE_CODES = [
  'ABCD-EFGH-JKLM',
  'NPQR-STUV-WXYZ',
  '2345-6789-ABCD',
  'EFGH-JKLM-NPQR',
  'STUV-WXYZ-2345',
  '6789-ABCD-EFGH',
  'JKLM-NPQR-STUV',
  'WXYZ-2345-6789',
  'ABCD-2345-EFGH',
  'JKLM-6789-NPQR',
];

describe('Phase 1.2 Modular UI, Reusable Toggle Buttons & Authentication Views', () => {
  it('contains zero raw <button>, <input>, or <select> tags in app-shell.tsx, auth-layout.tsx, components/auth/*, and components/settings/*', () => {
    const filesToCheck = [
      path.resolve(__dirname, '../../src/components/ui/app-shell.tsx'),
      path.resolve(__dirname, '../../src/components/ui/auth-layout.tsx'),
      path.resolve(__dirname, '../../src/components/ui/theme-toggle-button.tsx'),
      path.resolve(__dirname, '../../src/components/ui/language-toggle-button.tsx'),
    ];

    for (const dir of [
      path.resolve(__dirname, '../../src/components/auth'),
      path.resolve(__dirname, '../../src/components/settings'),
    ]) {
      for (const entry of fs.readdirSync(dir)) {
        if (entry.endsWith('.tsx')) {
          filesToCheck.push(path.join(dir, entry));
        }
      }
    }

    for (const file of filesToCheck) {
      const src = fs.readFileSync(file, 'utf-8');
      expect(src, `${file} should not contain raw <button>`).not.toMatch(
        /<button[\s>]/
      );
      expect(src, `${file} should not contain raw <input>`).not.toMatch(
        /<input[\s>]/
      );
      expect(src, `${file} should not contain raw <select>`).not.toMatch(
        /<select[\s>]/
      );
    }
  });

  it('renders ThemeToggleButton, LanguageToggleButton, and AuthLayout in EN and RU', () => {
    const htmlEn = renderWithProviders(
      <AuthLayout title="Sign In" subtitle="Private Family Archive">
        <div className="flex gap-2">
          <LanguageToggleButton />
          <ThemeToggleButton />
        </div>
      </AuthLayout>,
      'en'
    );

    expect(htmlEn).toContain('Sign In');
    expect(htmlEn).toContain('EN');
    expect(htmlEn).toContain('Select color theme');

    const htmlRu = renderWithProviders(
      <AuthLayout title="Вход в архив">
        <LanguageToggleButton />
      </AuthLayout>,
      'ru'
    );

    expect(htmlRu).toContain('Вход в архив');
    expect(htmlRu).toContain('RU');
  });

  it('renders SetupWizardCard (Step 1 & Step 2 RecoveryCodesCard with 10 codes and required confirmation checkbox)', () => {
    const step1Html = renderWithProviders(
      <SetupWizardCard defaultTreeTitle="Levshits Family Archive" />,
      'en'
    );
    expect(step1Html).toContain('Family Archive Title');
    expect(step1Html).toContain('Administrator First Name');
    expect(step1Html).toContain('Create Admin Passkey');

    const step2Html = renderWithProviders(
      <RecoveryCodesCard
        codes={TEN_SAMPLE_CODES}
        requireConfirmation
        onContinue={() => {}}
      />,
      'en'
    );
    for (const code of TEN_SAMPLE_CODES) {
      expect(step2Html).toContain(code);
    }
    expect(step2Html).toContain(
      'I have saved these recovery codes in a secure place'
    );
    expect(step2Html).toContain('Download as Text File');
  });

  it('renders LoginCard in default, magic_sent, and recovery modes in EN and RU', () => {
    const defaultEn = renderWithProviders(<LoginCard />, 'en');
    expect(defaultEn).toContain('Sign in with Passkey');
    expect(defaultEn).toContain('Sign In with Password');
    expect(defaultEn).toContain('Send Magic Link');
    expect(defaultEn).toContain(
      'Lost your device? Use Emergency Recovery Code'
    );

    const defaultRu = renderWithProviders(<LoginCard />, 'ru');
    expect(defaultRu).toContain('Войти по ключу Passkey');
    expect(defaultRu).toContain('Войти по паролю');
    expect(defaultRu).toContain('Отправить ссылку для входа');

    const magicSentEn = renderWithProviders(
      <LoginCard initialMode="magic_sent" />,
      'en'
    );
    expect(magicSentEn).toContain('Check your inbox');

    const recoveryEn = renderWithProviders(
      <LoginCard initialMode="recovery" />,
      'en'
    );
    expect(recoveryEn).toContain('Emergency Recovery Code Sign-In');
    expect(recoveryEn).toContain('ABCD-EFGH-JKLM');
  });

  it('renders RegisterInviteCard with invitation banner and linked person notice', () => {
    const html = renderWithProviders(
      <RegisterInviteCard
        inviteToken="sample-token"
        initialInvite={{
          email: 'elena@levshits.local',
          invitee_name: 'Elena Kuznetsova',
          inviter_name: 'Vera Levshits',
          role: 'editor',
          person_id: 'I0002',
          tree_title: 'Levshits Family Archive',
          expires_at: '2026-11-02T00:00:00Z',
          auth_password_enabled: true,
        }}
      />,
      'en'
    );

    expect(html).toContain(
      'You have been invited by Vera Levshits to join Levshits Family Archive as editor.'
    );
    expect(html).toContain('I0002');
    expect(html).toContain('Create Passkey &amp; Join Tree');
  });

  it('renders SecurityPasskeysPanel and FamilyMembersInvitesPanel with passkeys, sessions, and family members', () => {
    const securityHtml = renderWithProviders(
      <SecurityPasskeysPanel
        initialPasskeys={[
          {
            id: 'pk-1',
            credential_id: 'cred-1',
            device_name: 'MacBook Pro Touch ID',
            transports: ['internal'],
            aaguid: null,
            sign_count: 4,
            created_at: '2026-10-26T10:00:00Z',
            last_used_at: '2026-10-26T14:00:00Z',
          },
        ]}
        initialSessions={[
          {
            id: 's-1',
            user_agent: 'Safari on macOS',
            ip_address: '192.168.1.15',
            is_current: true,
            created_at: '2026-10-26T10:00:00Z',
            last_seen_at: '2026-10-26T14:00:00Z',
            expires_at: '2026-11-25T14:00:00Z',
          },
        ]}
        initialRemainingCodes={9}
      />,
      'en'
    );

    expect(securityHtml).toContain('MacBook Pro Touch ID');
    expect(securityHtml).toContain('Register Another Passkey');
    expect(securityHtml).toContain('10 codes generated · 9 unused remaining');
    expect(securityHtml).toContain('Safari on macOS');
    expect(securityHtml).toContain('Current Device');

    const usersHtml = renderWithProviders(
      <FamilyMembersInvitesPanel
        initialUsers={[
          {
            id: 'u-1',
            email: 'vera@levshits.local',
            first_name: 'Vera',
            last_name: 'Levshits',
            display_name: 'Vera Levshits',
            role: 'admin',
            person_id: 'I0001',
            is_active: true,
            passkeys_count: 2,
            has_password: true,
            created_at: '2026-10-26T00:00:00Z',
            last_login_at: '2026-10-26T12:00:00Z',
          },
        ]}
        initialInvites={[
          {
            id: 'inv-1',
            role: 'editor',
            person_id: 'I0002',
            email: 'elena@levshits.local',
            invitee_name: 'Elena Kuznetsova',
            created_by: 'u-1',
            created_at: '2026-10-26T10:00:00Z',
            expires_at: '2026-11-02T10:00:00Z',
            used_at: null,
            revoked_at: null,
          },
        ]}
      />,
      'en'
    );

    expect(usersHtml).toContain('Vera Levshits');
    expect(usersHtml).toContain('2 passkey(s)');
    expect(usersHtml).toContain('Create sign-in link');
    expect(usersHtml).toContain('Elena Kuznetsova');
    expect(usersHtml).toContain('Invite Family Member');
  });

  it('DEF-001 & DEF-002 regression: base64UrlToUint8Array decodes base64url challenge bytes without double-encoding', () => {
    const sampleChallenge = 'a3A3L2k1WW9NTEhPeldHUXU4d1F6Zz09';
    const rawBytes = base64UrlToUint8Array(sampleChallenge);
    // Round-tripping back to base64url must match the exact server challenge string
    // (previously stringToUint8Array UTF-8 encoded the string, causing double-encoding)
    expect(bufferToBase64Url(rawBytes.buffer)).toBe(sampleChallenge);

    // Also test URL-safe characters (- and _) without padding from raw binary bytes
    const rawBinary = new Uint8Array([251, 239, 255, 254, 62, 63, 0, 128, 255]);
    const urlSafeChallenge = bufferToBase64Url(rawBinary.buffer);
    expect(urlSafeChallenge).toContain('-');
    expect(urlSafeChallenge).toContain('_');
    const decodedUrlSafe = base64UrlToUint8Array(urlSafeChallenge);
    expect(bufferToBase64Url(decodedUrlSafe.buffer)).toBe(urlSafeChallenge);
  });

  it('DEF-003 regression: MagicLinkVerifyCard renders post-verification passkey enrollment prompt and exposes deduplication cache reset', () => {
    clearMagicLinkVerificationCache();
    const html = renderWithProviders(
      <MagicLinkVerifyCard
        token="sample-magic-token"
        initialVerifiedUser={{
          id: 'u-2',
          email: 'elena@levshits.local',
          first_name: 'Elena',
          last_name: 'Kuznetsova',
          display_name: 'Elena Kuznetsova',
          role: 'editor',
          person_id: 'I0002',
          is_active: true,
          passkeys_count: 0,
          has_password: false,
          created_at: '2026-10-26T00:00:00Z',
          last_login_at: '2026-10-26T12:00:00Z',
        }}
      />,
      'en'
    );
    expect(html).toContain('You are signed in');
    expect(html).toContain('Register Passkey on This Device');
    expect(html).toContain('Continue to Archive');
  });

  it('uses BrandLogoIcon (family tree icon, never GitBranch/GitFork) and crisp screen sans-serif title typography across AppShell, AuthLayout, BottomNav, and App', () => {
    const filesUsingBrandLogo = [
      path.resolve(__dirname, '../../src/components/ui/app-shell.tsx'),
      path.resolve(__dirname, '../../src/components/ui/auth-layout.tsx'),
      path.resolve(__dirname, '../../src/components/ui/bottom-nav.tsx'),
      path.resolve(__dirname, '../../src/App.tsx'),
    ];

    for (const file of filesUsingBrandLogo) {
      const src = fs.readFileSync(file, 'utf-8');
      expect(src, `${file} should use BrandLogoIcon`).toContain('BrandLogoIcon');
      expect(src, `${file} should not use GitBranch`).not.toContain('GitBranch');
      expect(src, `${file} should not use GitFork`).not.toContain('GitFork');
    }

    const indexCss = fs.readFileSync(
      path.resolve(__dirname, '../../src/index.css'),
      'utf-8'
    );
    expect(indexCss).toContain("--font-serif: 'Plus Jakarta Sans'");
    expect(indexCss).not.toContain("--font-serif: 'Lora'");
  });
});
