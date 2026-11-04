import React from 'react';
import {
  Calendar,
  FileText,
  Image as ImageIcon,
  Lock,
  MapPin,
  Mic,
  Users,
} from 'lucide-react';
import { MediaAssetSummaryDto } from '../../api/client';
import { Avatar, Badge, Button, Card } from '../ui';
import { useI18n } from '../../i18n';

export interface MediaCardProps {
  asset: MediaAssetSummaryDto;
  onSelect: (asset: MediaAssetSummaryDto) => void;
}

export const MediaCard: React.FC<MediaCardProps> = ({ asset, onSelect }) => {
  const { t } = useI18n();
  const [imgError, setImgError] = React.useState(false);

  React.useEffect(() => {
    setImgError(false);
  }, [asset.file_url]);

  const isImage = asset.mime_type.startsWith('image/');
  const isPdf =
    asset.mime_type === 'application/pdf' ||
    asset.mime_type.startsWith('text/');
  const isAudio = asset.mime_type.startsWith('audio/');

  const aspectRatioStyle =
    isImage && !imgError && asset.width && asset.height && asset.height > 0
      ? { aspectRatio: `${asset.width} / ${asset.height}` }
      : undefined;

  const taggedCount = asset.face_tags.length;

  return (
    <Card className="group break-inside-avoid mb-4 overflow-hidden border border-border bg-surface hover:border-brand/60 transition-all shadow-xs">
      {/* Thumbnail / Media Preview Area */}
      <div
        style={aspectRatioStyle}
        className="relative w-full overflow-hidden bg-surface-muted/70 min-h-44 flex items-center justify-center"
      >
        {isImage && !imgError ? (
          <img
            src={asset.file_url}
            alt={asset.title}
            loading="lazy"
            onError={() => setImgError(true)}
            className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.02]"
          />
        ) : isPdf ? (
          <div className="flex flex-col items-center justify-center gap-2 py-10 px-4 text-secondary">
            <FileText className="h-10 w-10 text-accent" />
            <span className="text-xs font-medium uppercase tracking-wider">
              PDF
            </span>
          </div>
        ) : isAudio ? (
          <div className="flex flex-col items-center justify-center gap-2 py-10 px-4 text-secondary">
            <Mic className="h-10 w-10 text-brand" />
            <span className="text-xs font-medium uppercase tracking-wider">
              Audio
            </span>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center gap-2 py-10 px-4 text-secondary">
            <ImageIcon className="h-10 w-10 text-tertiary" />
            <span className="text-xs font-medium text-tertiary text-center line-clamp-1 px-2">
              {asset.title}
            </span>
          </div>
        )}

        {/* Top Badges Overlay */}
        <div className="absolute top-2.5 left-2.5 right-2.5 flex items-center justify-between gap-2 pointer-events-none">
          <div className="flex flex-wrap items-center gap-1.5">
            {asset.date_taken && (
              <Badge
                variant="neutral"
                icon={<Calendar className="h-3 w-3" />}
                className="bg-surface/90 backdrop-blur-xs"
              >
                {asset.date_taken}
              </Badge>
            )}
            {asset.is_private && (
              <Badge
                variant="warning"
                icon={<Lock className="h-3 w-3" />}
                className="bg-surface/90 backdrop-blur-xs"
              >
                {t('profile.privateBadge')}
              </Badge>
            )}
          </div>
          <Badge variant="neutral" className="bg-surface/90 backdrop-blur-xs">
            {asset.gid}
          </Badge>
        </div>
      </div>

      {/* Card Footer: Title, Place & Tagged Faces Summary */}
      <div className="p-3.5 space-y-2.5">
        <div>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onSelect(asset)}
            className="h-auto min-h-0 p-0 text-left font-serif text-base font-semibold text-primary hover:text-brand line-clamp-2 justify-start w-full"
          >
            {asset.title}
          </Button>
          {asset.place_full_name && (
            <p className="mt-1 inline-flex items-center gap-1 text-xs text-secondary truncate max-w-full">
              <MapPin className="h-3 w-3 text-tertiary shrink-0" />
              <span className="truncate">{asset.place_full_name}</span>
            </p>
          )}
        </div>

        <div className="flex items-center justify-between gap-2 pt-2 border-t border-border">
          {taggedCount > 0 ? (
            <div className="flex items-center gap-2 min-w-0">
              <div className="flex -space-x-1.5 overflow-hidden shrink-0">
                {asset.face_tags.slice(0, 4).map((tag) => (
                  <Avatar
                    key={tag.id}
                    name={tag.person_name}
                    size="xs"
                    className="ring-2 ring-surface"
                  />
                ))}
              </div>
              <span className="text-xs text-secondary truncate">
                {t('mediaArchive.taggedPeopleCount', { count: taggedCount })}
              </span>
            </div>
          ) : (
            <span className="inline-flex items-center gap-1 text-xs text-tertiary">
              <Users className="h-3.5 w-3.5" />
              <span>{t('mediaArchive.untaggedBadge')}</span>
            </span>
          )}

          <Button
            variant="secondary"
            size="sm"
            onClick={() => onSelect(asset)}
            className="px-2.5 text-xs shrink-0"
          >
            {t('mediaArchive.openStudioBtn')}
          </Button>
        </div>
      </div>
    </Card>
  );
};
