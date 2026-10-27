import React, { useEffect, useState } from 'react';
import { WifiOff } from 'lucide-react';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PwaOfflineBannerProps {
  forceOffline?: boolean;
  className?: string;
}

export const PwaOfflineBanner: React.FC<PwaOfflineBannerProps> = ({
  forceOffline,
  className,
}) => {
  const { t } = useI18n();
  const [isOffline, setIsOffline] = useState<boolean>(() => {
    if (forceOffline !== undefined) return forceOffline;
    if (typeof navigator !== 'undefined' && typeof navigator.onLine === 'boolean') {
      return !navigator.onLine;
    }
    return false;
  });

  useEffect(() => {
    if (forceOffline !== undefined) {
      setIsOffline(forceOffline);
      return;
    }
    if (typeof window === 'undefined') return;

    const handleOnline = () => setIsOffline(false);
    const handleOffline = () => setIsOffline(true);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [forceOffline]);

  if (!isOffline) return null;

  return (
    <div
      role="status"
      aria-live="polite"
      data-testid="pwa-offline-banner"
      className={cn(
        'w-full border-b border-warning/30 bg-warning/10 px-4 py-2 pt-[max(0.5rem,env(safe-area-inset-top,0px))] text-xs text-primary',
        className
      )}
    >
      <div className="mx-auto flex max-w-7xl items-center justify-center gap-2 text-center">
        <WifiOff className="h-3.5 w-3.5 shrink-0 text-warning" />
        <span>{t('pwa.offline_banner')}</span>
      </div>
    </div>
  );
};
