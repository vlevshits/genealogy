import React, { useEffect, useState } from 'react';
import { Download, PlusSquare, Share, Smartphone } from 'lucide-react';
import { Button } from '../ui';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

const DISMISS_STORAGE_KEY = 'pwa_install_dismissed_at';
const DISMISS_TTL_MS = 90 * 24 * 60 * 60 * 1000; // 90 days

interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>;
}

export interface PwaInstallPromptProps {
  instanceTitle?: string;
  forceVisible?: boolean;
  forceIosMode?: boolean;
  onDismiss?: () => void;
  className?: string;
}

function isStandaloneDisplayMode(): boolean {
  if (typeof window === 'undefined') return false;
  const standaloneMedia =
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(display-mode: standalone)').matches;
  const iosStandalone = Boolean(
    (window.navigator as Navigator & { standalone?: boolean }).standalone
  );
  return standaloneMedia || iosStandalone;
}

function isRecentlyDismissed(): boolean {
  if (typeof window === 'undefined') return false;
  try {
    const raw = window.localStorage.getItem(DISMISS_STORAGE_KEY);
    if (!raw) return false;
    const timestamp = Number(raw);
    if (Number.isNaN(timestamp)) return false;
    return Date.now() - timestamp < DISMISS_TTL_MS;
  } catch {
    return false;
  }
}

function detectIosSafari(): boolean {
  if (typeof navigator === 'undefined') return false;
  const ua = navigator.userAgent || '';
  const isIOS = /iPad|iPhone|iPod/.test(ua);
  const isWebkit = /WebKit/.test(ua);
  const isCriOS = /CriOS|FxiOS|EdgiOS/.test(ua);
  return isIOS && isWebkit && !isCriOS && !isStandaloneDisplayMode();
}

export const PwaInstallPrompt: React.FC<PwaInstallPromptProps> = ({
  instanceTitle,
  forceVisible,
  forceIosMode,
  onDismiss,
  className,
}) => {
  const { t } = useI18n();
  const [deferredPrompt, setDeferredPrompt] =
    useState<BeforeInstallPromptEvent | null>(null);
  const [dismissed, setDismissed] = useState<boolean>(() =>
    forceVisible ? false : isRecentlyDismissed()
  );
  const [isIos, setIsIos] = useState<boolean>(() =>
    forceIosMode !== undefined ? forceIosMode : detectIosSafari()
  );
  const [showIosGuide, setShowIosGuide] = useState<boolean>(
    Boolean(forceIosMode)
  );

  useEffect(() => {
    if (forceIosMode !== undefined) {
      setIsIos(forceIosMode);
      setShowIosGuide(forceIosMode);
    }
  }, [forceIosMode]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    if (!forceVisible && (isStandaloneDisplayMode() || isRecentlyDismissed())) {
      return;
    }

    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault();
      setDeferredPrompt(e as BeforeInstallPromptEvent);
    };

    window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt);
    return () => {
      window.removeEventListener(
        'beforeinstallprompt',
        handleBeforeInstallPrompt
      );
    };
  }, [forceVisible]);

  const handleDismiss = () => {
    setDismissed(true);
    setDeferredPrompt(null);
    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(DISMISS_STORAGE_KEY, String(Date.now()));
      } catch {
        // Ignore storage quota errors
      }
    }
    onDismiss?.();
  };

  const handleInstallClick = async () => {
    if (isIos && !deferredPrompt) {
      setShowIosGuide((prev) => !prev);
      return;
    }
    if (!deferredPrompt) return;
    try {
      await deferredPrompt.prompt();
      const choice = await deferredPrompt.userChoice;
      if (choice.outcome === 'dismissed') {
        handleDismiss();
      } else {
        setDeferredPrompt(null);
      }
    } catch {
      handleDismiss();
    }
  };

  const shouldRender =
    !dismissed &&
    (Boolean(forceVisible) ||
      Boolean(deferredPrompt) ||
      (isIos && !isStandaloneDisplayMode()));

  if (!shouldRender) return null;

  const resolvedInstanceName = instanceTitle || t('app.instanceName');

  return (
    <aside
      aria-label={t('pwa.install_title')}
      data-testid="pwa-install-prompt"
      className={cn(
        'mx-auto w-full max-w-2xl px-4 pt-3',
        className
      )}
    >
      <div className="flex flex-col gap-3 rounded-2xl border border-border bg-surface p-3.5 shadow-sm sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-brand/15 text-brand">
            <Smartphone className="h-5 w-5" />
          </div>
          <div className="min-w-0 space-y-0.5">
            <h2 className="text-sm font-semibold text-primary">
              {t('pwa.install_title')}
            </h2>
            <p className="text-xs text-secondary">
              {t('pwa.install_prompt', { instanceName: resolvedInstanceName })}
            </p>
          </div>
        </div>

        <div className="flex items-center justify-end gap-2 shrink-0">
          <Button
            size="sm"
            variant="ghost"
            onClick={handleDismiss}
          >
            {t('pwa.dismiss_action')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            leftIcon={<Download className="h-3.5 w-3.5" />}
            onClick={() => void handleInstallClick()}
          >
            {t('pwa.install_action')}
          </Button>
        </div>
      </div>

      {showIosGuide && (
        <div
          data-testid="pwa-ios-guide"
          className="mt-2 rounded-xl border border-border bg-surface-muted/80 p-3 text-xs text-secondary space-y-1.5"
        >
          <div className="flex items-center gap-2">
            <Share className="h-3.5 w-3.5 shrink-0 text-brand" />
            <span>{t('pwa.ios_share_step')}</span>
          </div>
          <div className="flex items-center gap-2">
            <PlusSquare className="h-3.5 w-3.5 shrink-0 text-brand" />
            <span>{t('pwa.ios_add_step')}</span>
          </div>
        </div>
      )}
    </aside>
  );
};
