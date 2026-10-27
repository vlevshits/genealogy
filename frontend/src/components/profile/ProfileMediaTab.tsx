import React, { useState } from 'react';
import {
  Camera,
  FileText,
  Image as ImageIcon,
  Lock,
  Tag,
  Upload,
  Users,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  Dialog,
  EmptyState,
  FileDropzone,
} from '../ui';
import { MediaAssetSummaryDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface ProfileMediaTabProps {
  media: MediaAssetSummaryDto[];
  canEdit: boolean;
  isMaskedLiving?: boolean;
  onSelectPersonId?: (personId: string) => void;
}

export const ProfileMediaTab: React.FC<ProfileMediaTabProps> = ({
  media,
  canEdit,
  isMaskedLiving = false,
  onSelectPersonId,
}) => {
  const { t } = useI18n();
  const [selectedAsset, setSelectedAsset] =
    useState<MediaAssetSummaryDto | null>(null);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);

  if (isMaskedLiving) {
    return (
      <Card className="p-8">
        <EmptyState
          icon={<Lock className="h-8 w-8 text-warning" />}
          title={t('profile.media.maskedTitle')}
          description={t('profile.media.maskedDesc')}
        />
      </Card>
    );
  }

  const isDocumentAsset = (m: MediaAssetSummaryDto): boolean => {
    const lowerTitle = (m.title || '').toLowerCase();
    return (
      m.mime_type === 'application/pdf' ||
      lowerTitle.includes('certificate') ||
      lowerTitle.includes('scan') ||
      lowerTitle.includes('document') ||
      lowerTitle.includes('метрич') ||
      lowerTitle.includes('свидетельство') ||
      lowerTitle.includes('архив')
    );
  };

  const photos = media.filter((m) => !isDocumentAsset(m));
  const documents = media.filter((m) => isDocumentAsset(m));

  const renderMediaGrid = (items: MediaAssetSummaryDto[]) => (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      {items.map((item) => (
        <div
          key={item.id}
          onClick={() => setSelectedAsset(item)}
          className="group flex flex-col justify-between overflow-hidden rounded-2xl border border-border bg-surface hover:border-brand/60 hover:shadow-sm transition-all cursor-pointer"
        >
          <div className="relative flex h-40 items-center justify-center bg-surface-muted/60 p-4">
            {isDocumentAsset(item) ? (
              <FileText className="h-12 w-12 text-brand/70 group-hover:scale-105 transition-transform" />
            ) : (
              <ImageIcon className="h-12 w-12 text-brand/70 group-hover:scale-105 transition-transform" />
            )}

            <div className="absolute top-2.5 right-2.5 flex items-center gap-1.5">
              {item.is_private && (
                <Badge
                  variant="warning"
                  size="sm"
                  icon={<Lock className="h-2.5 w-2.5" />}
                >
                  {t('domain.privateBadge')}
                </Badge>
              )}
              <Badge variant="neutral" size="sm">
                {item.gid}
              </Badge>
            </div>
          </div>

          <div className="p-3.5 space-y-2">
            <div className="font-sans text-sm font-semibold text-primary line-clamp-2">
              {item.title}
            </div>

            {item.face_tags && item.face_tags.length > 0 && (
              <div className="flex flex-wrap items-center gap-1 text-xs text-secondary">
                <Users className="h-3.5 w-3.5 text-tertiary shrink-0" />
                <span>
                  {t('profile.media.taggedCount', {
                    count: item.face_tags.length,
                  })}
                </span>
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  );

  return (
    <div className="space-y-6">
      {/* Section 1: Portraits & Family Photographs */}
      <Card className="p-5 sm:p-6 space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <Camera className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('profile.media.photosTitle')}
            </h2>
            <Badge variant="neutral" size="sm">
              {photos.length}
            </Badge>
          </div>

          {canEdit && (
            <div className="flex flex-wrap items-center gap-2">
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<Tag className="h-3.5 w-3.5" />}
                onClick={() => setUploadDialogOpen(true)}
              >
                {t('profile.media.tagExistingBtn')}
              </Button>
              <Button
                size="sm"
                variant="primary"
                leftIcon={<Upload className="h-3.5 w-3.5" />}
                onClick={() => setUploadDialogOpen(true)}
              >
                {t('profile.media.addPhotoBtn')}
              </Button>
            </div>
          )}
        </div>

        {photos.length === 0 ? (
          <EmptyState
            icon={<Camera className="h-7 w-7 text-tertiary" />}
            title={t('profile.media.emptyPhotosTitle')}
            description={t('profile.media.emptyPhotosDesc')}
          />
        ) : (
          renderMediaGrid(photos)
        )}
      </Card>

      {/* Section 2: Documents & Archival Scans */}
      <Card className="p-5 sm:p-6 space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <FileText className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('profile.media.documentsTitle')}
            </h2>
            <Badge variant="neutral" size="sm">
              {documents.length}
            </Badge>
          </div>

          {canEdit && (
            <Button
              size="sm"
              variant="secondary"
              leftIcon={<Upload className="h-3.5 w-3.5" />}
              onClick={() => setUploadDialogOpen(true)}
            >
              {t('profile.media.uploadScanBtn')}
            </Button>
          )}
        </div>

        {documents.length === 0 ? (
          <EmptyState
            icon={<FileText className="h-7 w-7 text-tertiary" />}
            title={t('profile.media.emptyDocumentsTitle')}
            description={t('profile.media.emptyDocumentsDesc')}
          />
        ) : (
          renderMediaGrid(documents)
        )}
      </Card>

      {/* Lightbox Media Preview Dialog */}
      <Dialog
        open={Boolean(selectedAsset)}
        onOpenChange={(open) => {
          if (!open) setSelectedAsset(null);
        }}
        size="xl"
        title={selectedAsset?.title ?? ''}
        description={
          selectedAsset
            ? `${selectedAsset.gid} · ${selectedAsset.mime_type}`
            : undefined
        }
      >
        {selectedAsset && (
          <div className="space-y-4">
            <div className="relative flex min-h-60 items-center justify-center rounded-2xl border border-border bg-surface-muted/60 p-6">
              <ImageIcon className="h-16 w-16 text-brand/70" />
            </div>

            {selectedAsset.face_tags && selectedAsset.face_tags.length > 0 && (
              <div className="space-y-2">
                <div className="text-xs font-semibold uppercase tracking-wider text-secondary">
                  {t('profile.media.taggedRelativesHeading')}
                </div>
                <div className="flex flex-wrap gap-2">
                  {selectedAsset.face_tags.map((ft) => (
                    <Button
                      key={ft.id}
                      size="sm"
                      variant="secondary"
                      onClick={() => {
                        setSelectedAsset(null);
                        onSelectPersonId?.(ft.person_id);
                      }}
                    >
                      {ft.person_name}
                    </Button>
                  ))}
                </div>
              </div>
            )}

            <div className="flex justify-end">
              <Button
                variant="secondary"
                onClick={() => setSelectedAsset(null)}
              >
                {t('common.close')}
              </Button>
            </div>
          </div>
        )}
      </Dialog>

      {/* Upload Photo / Scan Modal */}
      <Dialog
        open={uploadDialogOpen}
        onOpenChange={setUploadDialogOpen}
        size="md"
        title={t('profile.media.uploadModalTitle')}
        description={t('profile.media.uploadModalDesc')}
      >
        <div className="space-y-4">
          <FileDropzone
            accept=".jpg,.png,.webp,.pdf"
            multiple
            onUpload={() => setUploadDialogOpen(false)}
          />
          <div className="flex justify-end">
            <Button
              variant="secondary"
              onClick={() => setUploadDialogOpen(false)}
            >
              {t('common.cancel')}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};
