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
  AccessDeniedCard,
  RoleBadge,
} from '../../src/components/ui';
import {
  FamilyMembersInvitesPanel,
  RoleChangeDialog,
  UserDeactivateDialog,
} from '../../src/components/settings';
import {
  PublicGuestBanner,
  TreePrivacyPreviewCard,
  buildDefaultTreePreview,
} from '../../src/components/tree';
import { SettingsPage } from '../../src/pages/SettingsPage';

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

const SAMPLE_ADMIN_USER = {
  id: 'u-admin-1',
  email: 'vera@levshits.local',
  first_name: 'Vera',
  last_name: 'Levshits',
  display_name: 'Vera Levshits',
  role: 'admin' as const,
  person_id: 'I0001',
  is_active: true,
  passkeys_count: 2,
  has_password: true,
  created_at: '2026-10-26T00:00:00Z',
  last_login_at: '2026-10-26T12:00:00Z',
};

const SAMPLE_VIEWER_USER = {
  id: 'u-viewer-1',
  email: 'pavel@levshits.local',
  first_name: 'Pavel',
  last_name: 'Levshits',
  display_name: 'Pavel Levshits',
  role: 'viewer' as const,
  person_id: 'I0008',
  is_active: false,
  deactivated_at: '2026-10-26T15:00:00Z',
  passkeys_count: 1,
  has_password: false,
  created_at: '2026-10-26T00:00:00Z',
  last_login_at: null,
};

describe('Phase 1.3 RBAC Matrix, User Governance & Living-Person Privacy UI', () => {
  it('contains zero raw <button>, <input>, or <select> tags across all new RBAC & Tree Privacy components', () => {
    const filesToCheck = [
      path.resolve(__dirname, '../../src/components/ui/access-denied-card.tsx'),
      path.resolve(__dirname, '../../src/components/ui/role-badge.tsx'),
      path.resolve(
        __dirname,
        '../../src/components/settings/RoleChangeDialog.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/UserDeactivateDialog.tsx'
      ),
      path.resolve(__dirname, '../../src/components/settings/UserTableRow.tsx'),
      path.resolve(
        __dirname,
        '../../src/components/tree/PublicGuestBanner.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/tree/TreePrivacyPreviewCard.tsx'
      ),
    ];

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

  it('enforces SettingsPage RBAC boundary: Viewer and Editor receive AccessDeniedCard on admin tabs but can open /settings/profile', () => {
    for (const role of ['viewer', 'editor'] as const) {
      for (const restrictedPath of [
        '/settings',
        '/settings/users',
        '/settings/backups',
        '/settings/gedcom',
      ]) {
        const deniedHtml = renderWithProviders(
          <SettingsPage currentPath={restrictedPath} userRole={role} />,
          'en'
        );
        expect(deniedHtml).toContain('Administrator Access Required');
        expect(deniedHtml).toContain('Open My Profile');
      }

      const profileHtml = renderWithProviders(
        <SettingsPage currentPath="/settings/profile" userRole={role} />,
        'en'
      );
      expect(profileHtml).not.toContain('Administrator Access Required');
      expect(profileHtml).toContain('Save Profile');
    }

    const deniedRu = renderWithProviders(
      <SettingsPage currentPath="/settings/users" userRole="viewer" />,
      'ru'
    );
    expect(deniedRu).toContain('Требуются права администратора');
    expect(deniedRu).toContain('Перейти в мой профиль');
  });

  it('renders RoleBadge, UserTableRow, RoleChangeDialog (with Sole Admin warning), and UserDeactivateDialog', () => {
    const panelHtml = renderWithProviders(
      <FamilyMembersInvitesPanel
        initialUsers={[SAMPLE_ADMIN_USER, SAMPLE_VIEWER_USER]}
        initialInvites={[]}
      />,
      'en'
    );

    expect(panelHtml).toContain('Vera Levshits');
    expect(panelHtml).toContain('Administrator');
    expect(panelHtml).toContain('Node I0001');
    expect(panelHtml).toContain('Change role');
    expect(panelHtml).toContain('Deactivate');
    expect(panelHtml).toContain('Pavel Levshits');
    expect(panelHtml).toContain('Deactivated');
    expect(panelHtml).toContain('Reactivate');

    // Sole Admin warning inside RoleChangeDialog when activeAdminCount === 1
    const roleModalHtml = renderWithProviders(
      <RoleChangeDialog
        open
        onOpenChange={() => {}}
        user={SAMPLE_ADMIN_USER}
        activeAdminCount={1}
      />,
      'en'
    );
    expect(roleModalHtml).toContain('Sole Active Administrator');
    expect(roleModalHtml).toContain(
      'This account is the only active administrator.'
    );

    const deactivateModalHtml = renderWithProviders(
      <UserDeactivateDialog
        open
        onOpenChange={() => {}}
        user={SAMPLE_ADMIN_USER}
        activeAdminCount={1}
      />,
      'en'
    );
    expect(deactivateModalHtml).toContain(
      'The sole active administrator cannot be deactivated.'
    );

    const badgeHtml = renderWithProviders(
      <div>
        <RoleBadge role="admin" showIcon />
        <RoleBadge role="editor" showIcon />
        <RoleBadge role="viewer" isActive={false} />
        <AccessDeniedCard
          onOpenProfile={() => {}}
          onBackToTree={() => {}}
        />
      </div>,
      'en'
    );
    expect(badgeHtml).toContain('Administrator');
    expect(badgeHtml).toContain('Editor');
    expect(badgeHtml).toContain('Viewer');
    expect(badgeHtml).toContain('Deactivated');
  });

  it('renders PublicGuestBanner and TreePrivacyPreviewCard with masked Living records for public_anonymous vs full unredacted Living records for viewer and private records for editor/admin in EN and RU', () => {
    const guestEn = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('public_anonymous')}
        onSignIn={() => {}}
      />,
      'en'
    );
    expect(guestEn).toContain('Public Guest View');
    expect(guestEn).toContain('Living Family Members Are Protected');
    expect(guestEn).toContain('Living Levshits');
    expect(guestEn).toContain('Vital dates &amp; locations protected');
    expect(guestEn).not.toContain('Elena Alexandrovna Levshits');
    expect(guestEn).not.toContain('1994-06-14');
    expect(guestEn).not.toContain('Nikolai Petrovich Morozov');

    const guestRu = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('public_anonymous')}
        onSignIn={() => {}}
      />,
      'ru'
    );
    expect(guestRu).toContain('Публичный гостевой просмотр');
    expect(guestRu).toContain('Живущий Levshits');
    expect(guestRu).toContain('Даты и места защищены');

    // Authenticated Viewer sees full living details, read-only badge, and zero private records
    const viewerEn = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('viewer')}
      />,
      'en'
    );
    expect(viewerEn).toContain('Elena Alexandrovna Levshits');
    expect(viewerEn).toContain('1994-06-14');
    expect(viewerEn).toContain('Saint Petersburg, Russia');
    expect(viewerEn).toContain('Viewer · Read-Only');
    expect(viewerEn).not.toContain('Nikolai Petrovich Morozov');

    // Authenticated Editor/Admin sees full living details AND private records
    const editorEn = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('editor')}
      />,
      'en'
    );
    expect(editorEn).toContain('Elena Alexandrovna Levshits');
    expect(editorEn).toContain('Nikolai Petrovich Morozov');
    expect(editorEn).toContain('Private Record');

    const bannerStandalone = renderWithProviders(
      <PublicGuestBanner onSignIn={() => {}} />,
      'en'
    );
    expect(bannerStandalone).toContain('Sign In to Family Archive');
  });
});
