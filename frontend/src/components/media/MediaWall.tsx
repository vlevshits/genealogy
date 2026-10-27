import React from 'react';
import { MediaAssetSummaryDto } from '../../api/client';
import { Button, EmptyState } from '../ui';
import { useI18n } from '../../i18n';
import { MediaCard } from './MediaCard';

export interface MediaWallProps {
  items: MediaAssetSummaryDto[];
  onSelectAsset: (asset: MediaAssetSummaryDto) => void;
  nextCursor?: string | null;
  loadingMore?: boolean;
  onLoadMore?: () => void;
}

export const MediaWall: React.FC<MediaWallProps> = ({
  items,
  onSelectAsset,
  nextCursor = null,
  loadingMore = false,
  onLoadMore,
}) => {
  const { t } = useI18n();

  if (items.length === 0) {
    return (
      <div className="rounded-2xl border border-border bg-surface p-10">
        <EmptyState
          title={t('mediaArchive.emptyTitle')}
          description={t('mediaArchive.emptyDescription')}
        />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Fluid Responsive Masonry Grid */}
      <div
        className="columns-1 sm:columns-2 lg:columns-3 gap-4"
        role="list"
        aria-label={t('mediaArchive.wallAriaLabel')}
      >
        {items.map((asset) => (
          <div key={asset.id} role="listitem">
            <MediaCard asset={asset} onSelect={onSelectAsset} />
          </div>
        ))}
      </div>

      {nextCursor && onLoadMore && (
        <div className="flex justify-center pt-2">
          <Button
            variant="secondary"
            size="md"
            loading={loadingMore}
            onClick={onLoadMore}
          >
            {t('mediaArchive.loadMoreBtn')}
          </Button>
        </div>
      )}
    </div>
  );
};
