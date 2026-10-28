import React from 'react';
import { ArrowRight, Camera, Image as ImageIcon } from 'lucide-react';
import { Badge, Button, Card, EmptyState } from '../ui';
import { getMediaFileUrl, MediaAssetSummaryDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface LatestPhotosWidgetProps {
  photos: MediaAssetSummaryDto[];
  onPreviewPhoto?: (asset: MediaAssetSummaryDto) => void;
  onOpenMediaArchive?: () => void;
}

export const LatestPhotosWidget: React.FC<LatestPhotosWidgetProps> = ({
  photos,
  onPreviewPhoto,
  onOpenMediaArchive,
}) => {
  const { t } = useI18n();

  return (
    <Card
      data-testid="dashboard-latest-photos-widget"
      className="p-5 sm:p-6 space-y-4 flex flex-col justify-between"
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Camera className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('dashboard.latestPhotos.title')}
            </h2>
            <Badge variant="neutral" size="sm">
              {photos.length}
            </Badge>
          </div>

          {onOpenMediaArchive && (
            <Button
              size="sm"
              variant="ghost"
              rightIcon={<ArrowRight className="h-3.5 w-3.5" />}
              onClick={onOpenMediaArchive}
            >
              {t('dashboard.latestPhotos.openGalleryBtn')}
            </Button>
          )}
        </div>

        {photos.length === 0 ? (
          <EmptyState
            icon={<ImageIcon className="h-6 w-6 text-tertiary" />}
            title={t('dashboard.latestPhotos.emptyTitle')}
            description={t('dashboard.latestPhotos.emptyDesc')}
          />
        ) : (
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
            {photos.slice(0, 6).map((asset) => {
              const thumbUrl = asset.file_url || getMediaFileUrl(asset.id);
              const tagCount = asset.face_tags?.length ?? 0;
              return (
                <div
                  key={asset.id}
                  role="button"
                  tabIndex={0}
                  data-testid={`dashboard-photo-${asset.id}`}
                  onClick={() => onPreviewPhoto?.(asset)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      onPreviewPhoto?.(asset);
                    }
                  }}
                  className="group relative overflow-hidden rounded-2xl border border-border bg-surface-muted aspect-4/3 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                >
                  <img
                    src={thumbUrl}
                    alt={asset.title || t('nav.media')}
                    loading="lazy"
                    className="h-full w-full object-cover transition-transform duration-200 group-hover:scale-105"
                  />
                  <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-primary/80 via-primary/40 to-transparent p-2.5 text-surface">
                    <div className="text-xs font-medium truncate">
                      {asset.title}
                    </div>
                    {tagCount > 0 && (
                      <div className="text-[10px] opacity-90">
                        {t('dashboard.latestPhotos.taggedCount', {
                          count: tagCount,
                        })}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </Card>
  );
};
