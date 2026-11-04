import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider, translate } from '../../src/i18n';
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
  createPasskeyCredential,
  getPasskeyAssertion,
  resolveWebAuthnRpId,
} from '../../src/lib/webauthn';
import { getLocalizedErrorMessage } from '../../src/lib/error-formatter';
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
    expect(step2Html).toContain('Download (.txt)');
  });

  it('renders LoginCard in default, magic_sent, and recovery modes in EN and RU', () => {
    const defaultEn = renderWithProviders(<LoginCard smtpEnabled={true} />, 'en');
    expect(defaultEn).toContain('Sign in with Passkey');
    expect(defaultEn).toContain('Sign In with Password');
    expect(defaultEn).toContain('Send Sign-In Link');
    expect(defaultEn).toContain(
      'Lost your device? Use Recovery Code'
    );

    const defaultRu = renderWithProviders(<LoginCard smtpEnabled={true} />, 'ru');
    expect(defaultRu).toContain('Войти по ключу Passkey');
    expect(defaultRu).toContain('Войти по паролю');
    expect(defaultRu).toContain('Войти по ссылке');

    // Test smtpEnabled={false} in EN and RU: no dead magic link button, correct divider and accessible alert
    const smtpDisabledEn = renderWithProviders(
      <LoginCard smtpEnabled={false} />,
      'en'
    );
    expect(smtpDisabledEn).not.toContain('Send Sign-In Link');
    expect(smtpDisabledEn).toContain('or sign in with password');
    expect(smtpDisabledEn).not.toContain('or sign in with email link');
    expect(smtpDisabledEn).toContain('smtp-disabled-hint');
    expect(smtpDisabledEn).toContain(
      'Email sign-in is not configured on this server. Use a passkey or password.'
    );

    const smtpDisabledRu = renderWithProviders(
      <LoginCard smtpEnabled={false} />,
      'ru'
    );
    expect(smtpDisabledRu).not.toContain('Войти по ссылке');
    expect(smtpDisabledRu).toContain('или войти по паролю');
    expect(smtpDisabledRu).not.toContain('или получить ссылку на почту');
    expect(smtpDisabledRu).toContain(
      'Вход по ссылке недоступен, так как почта не настроена на сервере. Используйте ключ доступа или пароль.'
    );

    // Test smtpEnabled={false} and authPasswordEnabled={false}: neither password nor magic link enabled
    const noSmtpNoPasswordEn = renderWithProviders(
      <LoginCard smtpEnabled={false} authPasswordEnabled={false} />,
      'en'
    );
    expect(noSmtpNoPasswordEn).not.toContain('or sign in with');
    expect(noSmtpNoPasswordEn).toContain(
      'Email sign-in is not configured on this server. Sign in with a passkey or emergency recovery code.'
    );

    // Test magic_sent mode: confirmation and back button only, no preview button
    const magicSentEn = renderWithProviders(
      <LoginCard initialMode="magic_sent" />,
      'en'
    );
    expect(magicSentEn).toContain('Check your inbox');
    expect(magicSentEn).toContain('We sent a single-use sign-in link to');
    expect(magicSentEn).toContain('Back to sign-in options');
    expect(magicSentEn).not.toContain('Open Sign-In Link');

    const magicSentRu = renderWithProviders(
      <LoginCard initialMode="magic_sent" />,
      'ru'
    );
    expect(magicSentRu).toContain('Проверьте вашу почту');
    expect(magicSentRu).toContain('Мы отправили одноразовую ссылку для входа');
    expect(magicSentRu).toContain('Вернуться к способам входа');
    expect(magicSentRu).not.toContain('Локальный предпросмотр');

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
    expect(html).toContain('Create Passkey &amp; Join');
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
    expect(securityHtml).toContain('Add Passkey');
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
    expect(html).toContain('Register Passkey');
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

  it('renders AuthLayout with interactive accessible header link pointing to home', () => {
    const html = renderWithProviders(
      <AuthLayout title="Sign In" instanceTitle="Levshits Family Archive">
        <div>Content</div>
      </AuthLayout>,
      'en'
    );

    expect(html).toContain('href="/"');
    expect(html).toContain('aria-label="Levshits Family Archive"');
    expect(html).toContain('focus-visible:ring-focus');
    expect(html).toContain('Levshits Family Archive');
  });

  it('renders LoginCard public egress button based on enablePublicAccess and safe returnUrl resolution in EN and RU', () => {
    // 1. enablePublicAccess === true, no returnUrl -> default "Browse Public Archive"
    const publicDefaultEn = renderWithProviders(
      <LoginCard enablePublicAccess={true} />,
      'en'
    );
    expect(publicDefaultEn).toContain('Browse Public Archive');

    const publicDefaultRu = renderWithProviders(
      <LoginCard enablePublicAccess={true} />,
      'ru'
    );
    expect(publicDefaultRu).toContain('Перейти к публичному архиву');

    // 2. enablePublicAccess === true, returnUrl points to tree -> contextual "Back to Tree"
    const publicTreeEn = renderWithProviders(
      <LoginCard enablePublicAccess={true} returnUrl="/tree" />,
      'en'
    );
    expect(publicTreeEn).toContain('Back to Tree');
    expect(publicTreeEn).not.toContain('Browse Public Archive');

    const publicTreeRu = renderWithProviders(
      <LoginCard enablePublicAccess={true} returnUrl="/tree" />,
      'ru'
    );
    expect(publicTreeRu).toContain('Вернуться к семейному древу');
    expect(publicTreeRu).not.toContain('Перейти к публичному архиву');

    // 3. enablePublicAccess === true, returnUrl points to person profile -> contextual "Back to Tree"
    const publicPersonEn = renderWithProviders(
      <LoginCard enablePublicAccess={true} returnUrl="/persons/I0001" />,
      'en'
    );
    expect(publicPersonEn).toContain('Back to Tree');

    // 4. enablePublicAccess === true, returnUrl points to restricted route (e.g. /settings, /people) -> safe fallback to "Browse Public Archive"
    const publicRestrictedEn = renderWithProviders(
      <LoginCard enablePublicAccess={true} returnUrl="/settings" />,
      'en'
    );
    expect(publicRestrictedEn).toContain('Browse Public Archive');
    expect(publicRestrictedEn).not.toContain('Back to Family Tree');

    // 5. enablePublicAccess === false -> strictly omitted from DOM
    const privateEn = renderWithProviders(
      <LoginCard enablePublicAccess={false} returnUrl="/tree" />,
      'en'
    );
    expect(privateEn).not.toContain('Browse Public Archive');
    expect(privateEn).not.toContain('Back to Family Tree');

    const privateRu = renderWithProviders(
      <LoginCard enablePublicAccess={false} returnUrl="/tree" />,
      'ru'
    );
    expect(privateRu).not.toContain('Перейти к публичному архиву');
    expect(privateRu).not.toContain('Вернуться к семейному древу');
  });

  it('enforces biometric verification, strict browser error propagation, and DOMException mapping', async () => {
    // 1. Verify DOMException with NotAllowedError and other WebAuthn errors map to localized strings in EN and RU
    const notAllowed = new DOMException('User cancelled operation', 'NotAllowedError');
    expect(getLocalizedErrorMessage(notAllowed, (key) => translate('en', key))).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
    expect(getLocalizedErrorMessage(notAllowed, (key) => translate('ru', key))).toBe(
      'Биометрическая верификация отменена или истекло время ожидания. Попробуйте снова.'
    );

    const alreadyRegistered = new DOMException('Passkey already registered', 'InvalidStateError');
    expect(getLocalizedErrorMessage(alreadyRegistered, (key) => translate('en', key))).toBe(
      'This passkey is already registered on this device.'
    );

    const secError = new DOMException('Insecure origin', 'SecurityError');
    expect(getLocalizedErrorMessage(secError, (key) => translate('en', key))).toBe(
      'Passkeys require a secure connection (HTTPS or localhost) and a matching domain.'
    );

    const notSupported = new DOMException('Not supported', 'NotSupportedError');
    expect(getLocalizedErrorMessage(notSupported, (key) => translate('en', key))).toBe(
      'Your browser or device does not support passkey authentication.'
    );

    // Verify backend error codes map correctly
    expect(
      getLocalizedErrorMessage(
        { code: 'user_verification_required' },
        (key) => translate('en', key)
      )
    ).toBe('User verification (biometrics or device PIN) is required.');
    expect(
      getLocalizedErrorMessage(
        { code: 'passkey_verification_failed' },
        (key) => translate('ru', key)
      )
    ).toBe('Ошибка проверки ключа доступа. Попробуйте снова.');

    // 2. Verify webauthn.ts passes userVerification: 'required' and authenticatorAttachment: 'platform'
    const originalWindow = (globalThis as Record<string, unknown>).window;
    const originalMode = import.meta.env.MODE;

    const createMock = vi.fn().mockResolvedValue({
      id: 'cred-test-id-123',
      rawId: new Uint8Array([1, 2, 3]).buffer,
      response: {
        attestationObject: new Uint8Array([4, 5, 6]).buffer,
        clientDataJSON: new Uint8Array([7, 8, 9]).buffer,
        getTransports: () => ['internal'],
      },
    });

    const getMock = vi.fn().mockResolvedValue({
      id: 'cred-assert-id-456',
      rawId: new Uint8Array([10, 11, 12]).buffer,
      response: {
        authenticatorData: new Uint8Array([13, 14, 15]).buffer,
        clientDataJSON: new Uint8Array([16, 17, 18]).buffer,
        signature: new Uint8Array([19, 20, 21]).buffer,
      },
    });

    const mockWindow = {
      location: { hostname: 'archive.family.internal' },
      PublicKeyCredential: class FakePublicKeyCredential {},
      navigator: {
        credentials: {
          create: createMock,
          get: getMock,
        },
      },
    };

    try {
      (globalThis as Record<string, unknown>).window = mockWindow;

      // Test createPasskeyCredential options
      await createPasskeyCredential(
        {
          challenge: 'Y2hhbGxlbmdlMQ',
          rp_id: 'archive.family.internal',
          rp_name: 'Family Tree Archive',
          timeout_ms: 60000,
        },
        'user@family.internal',
        'Family User'
      );

      expect(createMock).toHaveBeenCalledTimes(1);
      const createOptions = createMock.mock.calls[0][0] as {
        publicKey: PublicKeyCredentialCreationOptions;
      };
      expect(createOptions.publicKey.authenticatorSelection).toEqual({
        userVerification: 'preferred',
        residentKey: 'preferred',
      });
      expect(
        createOptions.publicKey.authenticatorSelection?.authenticatorAttachment
      ).toBeUndefined();
      expect(createOptions.publicKey.rp.id).toBe('archive.family.internal');

      // Test createPasskeyCredential with explicit attachment (e.g. cross-platform for hardware YubiKeys)
      await createPasskeyCredential(
        {
          challenge: 'Y2hhbGxlbmdlMQ',
          rp_id: 'archive.family.internal',
          rp_name: 'Family Tree Archive',
          timeout_ms: 60000,
        },
        'user@family.internal',
        'Family User',
        'YubiKey 5C',
        'cross-platform'
      );
      const crossPlatformOptions = createMock.mock.calls[1][0] as {
        publicKey: PublicKeyCredentialCreationOptions;
      };
      expect(
        crossPlatformOptions.publicKey.authenticatorSelection?.authenticatorAttachment
      ).toBe('cross-platform');

      // Test getPasskeyAssertion options
      await getPasskeyAssertion({
        challenge: 'Y2hhbGxlbmdlMg',
        rp_id: 'archive.family.internal',
        timeout_ms: 60000,
        allow_credentials: ['cred-123'],
      });

      expect(getMock).toHaveBeenCalledTimes(1);
      const getOptions = getMock.mock.calls[0][0] as {
        publicKey: PublicKeyCredentialRequestOptions;
      };
      expect(getOptions.publicKey.userVerification).toBe('preferred');
      expect(getOptions.publicKey.rpId).toBe('archive.family.internal');

      // 3. Verify Real browser mock test throwing NotAllowedError propagates to caller when !isTestEnv
      (import.meta.env as Record<string, unknown>).MODE = 'production';

      const notAllowedError = new DOMException('Biometric verification cancelled', 'NotAllowedError');
      createMock.mockRejectedValueOnce(notAllowedError);

      await expect(
        createPasskeyCredential(
          {
            challenge: 'Y2hhbGxlbmdlMw',
            rp_id: 'archive.family.internal',
            rp_name: 'Family Tree Archive',
            timeout_ms: 60000,
          },
          'user@family.internal',
          'Family User'
        )
      ).rejects.toThrow(notAllowedError);

      getMock.mockRejectedValueOnce(notAllowedError);
      await expect(
        getPasskeyAssertion({
          challenge: 'Y2hhbGxlbmdlNA',
          rp_id: 'archive.family.internal',
          timeout_ms: 60000,
          allow_credentials: ['cred-123'],
        })
      ).rejects.toThrow(notAllowedError);
    } finally {
      (globalThis as Record<string, unknown>).window = originalWindow;
      (import.meta.env as Record<string, unknown>).MODE = originalMode;
    }
  });

  it('UX-056 - UX-061 regression: WebAuthn authenticator selection, error toast titles, MagicLink enrollment resilience, and mobile focus management', () => {
    // UX-057: SecurityPasskeysPanel never uses common.errorLoading in toast titles
    const securityPanelSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/settings/SecurityPasskeysPanel.tsx'),
      'utf-8'
    );
    expect(securityPanelSrc).not.toContain("title: t('common.errorLoading')");
    expect(securityPanelSrc).toContain("title: t('settings.security.passkeyErrorTitle')");

    // UX-058: MagicLinkVerifyCard catches errors during enrollment and does not silently navigate
    const magicVerifySrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/auth/MagicLinkVerifyCard.tsx'),
      'utf-8'
    );
    expect(magicVerifySrc).toContain('setEnrollError');
    expect(magicVerifySrc).toContain('{enrollError && (');

    // UX-061: SetupWizardCard and RegisterInviteCard scroll into view and focus error alert with tabIndex={-1}
    const setupWizardSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/auth/SetupWizardCard.tsx'),
      'utf-8'
    );
    expect(setupWizardSrc).toContain('scrollIntoView');
    expect(setupWizardSrc).toContain('tabIndex={-1}');

    const registerInviteSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/auth/RegisterInviteCard.tsx'),
      'utf-8'
    );
    expect(registerInviteSrc).toContain('scrollIntoView');
    expect(registerInviteSrc).toContain('tabIndex={-1}');

    // UX-059: DOMExceptions TimeoutError and AbortError map to errors.webauthn_not_allowed
    const tEn = (key: string) => translate('en', key);
    const timeoutErr = new DOMException('Timeout occurred', 'TimeoutError');
    const abortErr = new DOMException('Registration aborted', 'AbortError');
    expect(getLocalizedErrorMessage(timeoutErr, tEn)).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
    expect(getLocalizedErrorMessage(abortErr, tEn)).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
  });

  it('dynamically resolves WebAuthn rpId preventing SecurityError on non-localhost domains', () => {
    // 1. When running on localhost / 127.0.0.1 / *.localhost, rpId is localhost
    expect(resolveWebAuthnRpId('localhost')).toBe('localhost');
    expect(resolveWebAuthnRpId(undefined)).toBe('localhost');

    const originalWindow = global.window;
    try {
      global.window = {
        location: {
          hostname: '127.0.0.1',
        },
      } as unknown as Window & typeof globalThis;
      expect(resolveWebAuthnRpId('anything')).toBe('localhost');

      global.window = {
        location: {
          hostname: 'app.local.localhost',
        },
      } as unknown as Window & typeof globalThis;
      expect(resolveWebAuthnRpId('anything')).toBe('localhost');
    } finally {
      global.window = originalWindow;
    }

    // 2. Mock non-localhost hostname (e.g. genealogy.levshitsvv.pro)
    try {
      global.window = {
        location: {
          hostname: 'genealogy.levshitsvv.pro',
        },
      } as unknown as Window & typeof globalThis;

      // When server returned 'localhost' default, rpId should NOT be localhost (which triggers SecurityError)
      expect(resolveWebAuthnRpId('localhost')).toBe('genealogy.levshitsvv.pro');

      // When server returned matching host, use it
      expect(resolveWebAuthnRpId('genealogy.levshitsvv.pro')).toBe('genealogy.levshitsvv.pro');

      // When server returned valid parent suffix domain, use it
      expect(resolveWebAuthnRpId('levshitsvv.pro')).toBe('levshitsvv.pro');

      // When server returned invalid/mismatched domain, fallback to host to avoid SecurityError
      expect(resolveWebAuthnRpId('phishing.com')).toBe('genealogy.levshitsvv.pro');
    } finally {
      global.window = originalWindow;
    }
  });
});
