import React, { useState, useEffect } from 'react';
import {
  Archive,
  CheckCircle2,
  FolderGit2,
  Settings,
  Shield,
  User,
  Users,
} from 'lucide-react';
import {
  AccessDeniedCard,
  Badge,
  Button,
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  PageHeader,
  RoleBadge,
  SideNav,
} from '../components/ui';
import {
  BackupsSettingsPanel,
  FamilyMembersInvitesPanel,
  GeneralSettingsPanel,
  ProfileSettingsPanel,
  SecurityPasskeysPanel,
} from '../components/settings';
import {
  AdminSystemResponse,
  AdminUserSummaryDto,
  BackupListResponse,
  fetchAdminSettings,
  fetchAdminSystem,
  fetchBackups,
  InstanceSettingsDto,
  InvitationDto,
  PasskeyCredentialDto,
  SessionDto,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { useI18n } from '../i18n';

export type SettingsTabId =
  | 'general'
  | 'profile'
  | 'users'
  | 'security'
  | 'gedcom'
  | 'backups';

export interface SettingsPageProps {
  currentPath?: string;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
  onInstanceTitleChange?: (nextTitle: string) => void;
  initialSettings?: InstanceSettingsDto;
  initialSystem?: AdminSystemResponse;
  initialBackups?: BackupListResponse;
  initialPasskeys?: PasskeyCredentialDto[];
  initialSessions?: SessionDto[];
  initialUsers?: AdminUserSummaryDto[];
  initialInvites?: InvitationDto[];
}

const DEFAULT_SETTINGS: InstanceSettingsDto = {
  tree_title: 'Levshits Family Archive',
  default_locale: 'en',
  enable_public_access: false,
  living_person_age_threshold: 110,
  settings_locked_from_env: false,
};

const DEFAULT_SYSTEM: AdminSystemResponse = {
  app_version: '0.1.0',
  database: {
    status: 'connected',
    size_bytes: 12_582_912,
    tables_count: 14,
  },
  storage: {
    media_size_bytes: 48_234_496,
    media_files_count: 34,
    free_disk_bytes: 21_474_836_480,
    total_disk_bytes: 64_424_509_440,
  },
  backups: {
    last_backup_at: null,
    last_backup_status: 'none',
    total_backups: 0,
  },
  jobs: {
    queue_length: 0,
    active_workers: 2,
  },
};

const DEFAULT_BACKUPS: BackupListResponse = {
  items: [],
  retention_days: 30,
  cron_schedule: '0 3 * * *',
  backup_enabled: true,
};

function resolveTabFromPath(path: string): SettingsTabId {
  if (path.startsWith('/settings/backups')) return 'backups';
  if (path.startsWith('/settings/profile')) return 'profile';
  if (path.startsWith('/settings/users')) return 'users';
  if (path.startsWith('/settings/security')) return 'security';
  if (path.startsWith('/settings/gedcom')) return 'gedcom';
  return 'general';
}

function pathForTab(tab: SettingsTabId): string {
  if (tab === 'general') return '/settings';
  return `/settings/${tab}`;
}

export const SettingsPage: React.FC<SettingsPageProps> = ({
  currentPath = '/settings',
  userRole,
  onNavigate,
  onInstanceTitleChange,
  initialSettings,
  initialSystem,
  initialBackups,
  initialPasskeys,
  initialSessions,
  initialUsers,
  initialInvites,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const rawRole = userRole ?? user?.role ?? 'admin';
  const effectiveRole: 'admin' | 'editor' | 'viewer' =
    rawRole === 'editor' || rawRole === 'viewer' ? rawRole : 'admin';
  const isAdmin = effectiveRole === 'admin';
  const activeTab = resolveTabFromPath(currentPath);
  const isAdminOnlyTab =
    activeTab === 'general' ||
    activeTab === 'users' ||
    activeTab === 'gedcom' ||
    activeTab === 'backups';

  const [settings, setSettings] = useState<InstanceSettingsDto>(
    initialSettings ?? DEFAULT_SETTINGS
  );
  const [system, setSystem] = useState<AdminSystemResponse>(
    initialSystem ?? DEFAULT_SYSTEM
  );
  const [backupList, setBackupList] = useState<BackupListResponse>(
    initialBackups ?? DEFAULT_BACKUPS
  );

  useEffect(() => {
    if (!isAdmin) {
      return;
    }
    if (initialSettings && initialSystem && initialBackups) {
      return;
    }
    let cancelled = false;

    fetchAdminSettings()
      .then((data) => {
        if (!cancelled) setSettings(data);
      })
      .catch(() => {});

    fetchAdminSystem()
      .then((data) => {
        if (!cancelled) setSystem(data);
      })
      .catch(() => {});

    fetchBackups()
      .then((data) => {
        if (!cancelled) setBackupList(data);
      })
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, [isAdmin, initialSettings, initialSystem, initialBackups]);

  const handleTabChange = (nextValue: string) => {
    const nextTab = nextValue as SettingsTabId;
    const targetPath = pathForTab(nextTab);
    if (onNavigate) {
      onNavigate(targetPath);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', targetPath);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  const navItems = [
    {
      value: 'general',
      label: t('settings.tabs.general'),
      icon: <Settings className="h-4 w-4" />,
    },
    {
      value: 'profile',
      label: t('settings.tabs.profile'),
      icon: <User className="h-4 w-4" />,
    },
    {
      value: 'users',
      label: t('settings.tabs.users'),
      icon: <Users className="h-4 w-4" />,
    },
    {
      value: 'security',
      label: t('settings.tabs.security'),
      icon: <Shield className="h-4 w-4" />,
    },
    {
      value: 'gedcom',
      label: t('settings.tabs.gedcom'),
      icon: <FolderGit2 className="h-4 w-4" />,
    },
    {
      value: 'backups',
      label: t('settings.tabs.backups'),
      icon: <Archive className="h-4 w-4" />,
      badge: backupList.items.length > 0 ? backupList.items.length : undefined,
    },
  ];

  return (
    <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <PageHeader
        title={t('settings.pageTitle')}
        subtitle={t('settings.pageSubtitle')}
        secondaryActions={<RoleBadge role={effectiveRole} showIcon />}
      />

      {/* Standard Two-Column Admin Layout (07-admin-settings-page.md §2, §3.1) */}
      <div className="flex flex-col md:flex-row items-start gap-6 lg:gap-8">
        <aside className="w-full md:w-64 shrink-0">
          <Card className="p-2">
            <SideNav
              ariaLabel={t('settings.navAriaLabel')}
              items={navItems}
              value={activeTab}
              onChange={handleTabChange}
            />
          </Card>
        </aside>

        <div className="flex-1 min-w-0 w-full space-y-6">
          {!isAdmin && isAdminOnlyTab ? (
            <AccessDeniedCard
              currentRole={effectiveRole}
              onOpenProfile={() => handleTabChange('profile')}
              onBackToTree={
                onNavigate ? () => onNavigate('/tree') : undefined
              }
            />
          ) : (
            <>
              {activeTab === 'general' && (
                <GeneralSettingsPanel
                  settings={settings}
                  system={system}
                  onSettingsUpdated={setSettings}
                  onInstanceTitleChange={onInstanceTitleChange}
                />
              )}

              {activeTab === 'profile' && <ProfileSettingsPanel />}

              {activeTab === 'users' && (
                <FamilyMembersInvitesPanel
                  initialUsers={initialUsers}
                  initialInvites={initialInvites}
                />
              )}

              {activeTab === 'security' && (
                <SecurityPasskeysPanel
                  initialPasskeys={initialPasskeys}
                  initialSessions={initialSessions}
                />
              )}

              {activeTab === 'backups' && (
                <BackupsSettingsPanel
                  backupList={backupList}
                  onBackupCreated={(archive) => {
                    setBackupList((prev) => ({
                      ...prev,
                      items: [
                        archive,
                        ...prev.items.filter(
                          (a) => a.filename !== archive.filename
                        ),
                      ],
                    }));
                    setSystem((prev) => ({
                      ...prev,
                      backups: {
                        last_backup_at: archive.created_at,
                        last_backup_status: 'ok',
                        total_backups: prev.backups.total_backups + 1,
                      },
                    }));
                  }}
                />
              )}

              {activeTab === 'gedcom' && (
                <Card>
                  <CardHeader className="flex items-center justify-between">
                    <h2 className="text-base font-semibold text-primary">
                      {t('settings.tabs.gedcom')}
                    </h2>
                    <Badge
                      variant="brand"
                      icon={<CheckCircle2 className="h-3.5 w-3.5" />}
                    >
                      {t('settings.upcoming.badge')}
                    </Badge>
                  </CardHeader>
                  <CardBody>
                    <p className="text-sm text-secondary leading-relaxed">
                      {t('settings.upcoming.gedcomDesc')}
                    </p>
                  </CardBody>
                  <CardFooter>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => handleTabChange('general')}
                    >
                      {t('settings.tabs.general')}
                    </Button>
                  </CardFooter>
                </Card>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
