import React from 'react';
import { MapPin, Sparkles } from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import { useI18n } from '../../i18n';

export interface PlacesQualityBannerProps {
  issueCount: number;
  highConfidenceCount?: number;
  onReviewFixes: () => void;
}

export const PlacesQualityBanner: React.FC<PlacesQualityBannerProps> = ({
  issueCount,
  highConfidenceCount = 0,
  onReviewFixes,
}) => {
  const { t } = useI18n();

  if (issueCount <= 0) return null;

  return (
    <Card
      data-testid="places-quality-banner"
      className="p-3.5 sm:p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-brand/30 bg-surface-subtle"
    >
      <div className="flex items-start sm:items-center gap-3 min-w-0">
        <div className="h-9 w-9 rounded-lg bg-brand/10 flex items-center justify-center text-brand shrink-0">
          <MapPin className="h-4.5 w-4.5" />
        </div>
        <div className="space-y-0.5 min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-semibold text-primary">
              {t('places.qualityBannerTitle', { count: issueCount })}
            </span>
            {highConfidenceCount > 0 && (
              <Badge variant="success" size="sm">
                {t('places.qualityBannerHighBadge', {
                  count: highConfidenceCount,
                })}
              </Badge>
            )}
          </div>
          <p className="text-xs text-secondary">
            {t('places.qualityBannerDesc')}
          </p>
        </div>
      </div>

      <Button
        size="sm"
        variant="primary"
        data-testid="places-quality-banner-btn"
        leftIcon={<Sparkles className="h-3.5 w-3.5" />}
        onClick={onReviewFixes}
        className="shrink-0"
      >
        {t('places.qualityBannerActionBtn')}
      </Button>
    </Card>
  );
};
