import React, { useState } from 'react';
import { Upload, UserPlus } from 'lucide-react';
import { BrandLogoIcon, Button } from '../ui';
import { IndividualDto, UserDto } from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import { StartWithSelfDialog } from './StartWithSelfDialog';

export interface EmptyTreeOnboardingCardProps {
  canOnboard?: boolean;
  userRole?: 'admin' | 'editor' | 'viewer' | string;
  initialDialogOpen?: boolean;
  onPersonCreated?: (individual: IndividualDto, user: UserDto) => void;
  onImportGedcom?: () => void;
}

export const EmptyTreeOnboardingCard: React.FC<EmptyTreeOnboardingCardProps> = ({
  canOnboard,
  userRole,
  initialDialogOpen = false,
  onPersonCreated,
  onImportGedcom,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const [dialogOpen, setDialogOpen] = useState<boolean>(initialDialogOpen);

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isEditorOrAdmin =
    canOnboard !== undefined
      ? canOnboard
      : effectiveRole === 'admin' || effectiveRole === 'editor';

  const handleImportClick = () => {
    if (onImportGedcom) {
      onImportGedcom();
      return;
    }
    if (typeof window !== 'undefined') {
      window.history.pushState({}, '', '/settings/gedcom');
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  return (
    <>
      <section
        aria-label={t('onboarding.emptyTitle')}
        className="mx-auto max-w-2xl rounded-3xl border border-border bg-surface p-8 sm:p-12 shadow-sm text-center space-y-6"
      >
        <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-brand/10 text-brand">
          <BrandLogoIcon className="h-7 w-7" />
        </div>

        {isEditorOrAdmin ? (
          <>
            <div className="space-y-2">
              <h2 className="font-sans text-xl sm:text-2xl font-semibold tracking-tight text-primary">
                {t('onboarding.emptyTitle')}
              </h2>
              <p className="mx-auto max-w-lg text-sm sm:text-base text-secondary leading-relaxed">
                {t('onboarding.emptyDescription')}
              </p>
            </div>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-3 pt-1">
              <Button
                variant="primary"
                size="lg"
                leftIcon={<UserPlus className="h-4 w-4" />}
                onClick={() => setDialogOpen(true)}
              >
                {t('onboarding.startWithSelfBtn')}
              </Button>

              <Button
                variant="secondary"
                size="lg"
                leftIcon={<Upload className="h-4 w-4" />}
                onClick={handleImportClick}
              >
                {t('onboarding.importGedcomBtn')}
              </Button>
            </div>

            <p className="mx-auto max-w-md border-t border-border pt-4 text-xs text-tertiary leading-relaxed">
              {t('onboarding.gedcomHint')}
            </p>
          </>
        ) : (
          <div className="space-y-2 py-2">
            <h2 className="font-sans text-xl sm:text-2xl font-semibold tracking-tight text-primary">
              {t('onboarding.emptyViewerTitle')}
            </h2>
            <p className="mx-auto max-w-md text-sm sm:text-base text-secondary leading-relaxed">
              {t('onboarding.viewerWaitingNotice')}
            </p>
          </div>
        )}
      </section>

      {isEditorOrAdmin && (
        <StartWithSelfDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          onCompleted={(individual, updatedUser) => {
            onPersonCreated?.(individual, updatedUser);
          }}
        />
      )}
    </>
  );
};
