import React, { useEffect, useState } from 'react';
import { ChevronLeft, ChevronRight, X } from 'lucide-react';
import {
  createFaceTag,
  deleteFaceTag,
  deleteMediaAsset,
  FaceTagDto,
  getMediaCropUrl,
  MediaAssetSummaryDto,
  updateFaceTag,
   updateMediaAsset,
  UpdateMediaInput,
} from '../../api/client';
import { Button, useToast } from '../ui';
import { useI18n } from '../../i18n';
import { FaceTagCanvas, NormalizedBox } from './FaceTagCanvas';
import { MediaMetadataPanel } from './MediaMetadataPanel';
import {
  DEFAULT_FACE_CANDIDATES,
  FaceTagCandidatePerson,
} from './PersonFacePicker';

export interface AvatarSyncEventDetail {
  personId: string;
  avatarUrl: string | null;
}

export function dispatchAvatarSyncEvent(
  personId: string,
  avatarUrl: string | null
): void {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent<AvatarSyncEventDetail>('genealogy:avatar-updated', {
        detail: { personId, avatarUrl },
      })
    );
  }
}

export interface MediaLightboxStudioProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: MediaAssetSummaryDto | null;
  canEdit: boolean;
  candidates?: FaceTagCandidatePerson[];
  onAssetUpdated?: (updated: MediaAssetSummaryDto) => void;
  onAssetDeleted?: (deletedId: string) => void;
  onAvatarSynchronized?: (personId: string, avatarUrl: string | null) => void;
  onOpenPerson?: (personId: string) => void;
  onPrevAsset?: () => void;
  onNextAsset?: () => void;
}

export const MediaLightboxStudio: React.FC<MediaLightboxStudioProps> = ({
  open,
  onOpenChange,
  asset,
  canEdit,
  candidates = DEFAULT_FACE_CANDIDATES,
  onAssetUpdated,
  onAssetDeleted,
  onAvatarSynchronized,
  onOpenPerson = () => {},
  onPrevAsset,
  onNextAsset,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [currentAsset, setCurrentAsset] =
    useState<MediaAssetSummaryDto | null>(asset);
  const [isTaggingMode, setIsTaggingMode] = useState<boolean>(false);
  const [highlightedTagId, setHighlightedTagId] = useState<string | null>(null);

  useEffect(() => {
    setCurrentAsset(asset);
    setIsTaggingMode(false);
    setHighlightedTagId(null);
  }, [asset]);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onOpenChange(false);
      }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [open, onOpenChange]);

  if (!open || !currentAsset) return null;

  const candidateById = new Map<string, FaceTagCandidatePerson>(
    candidates.map((c) => [c.id, c])
  );

  const notifyAvatarSync = (personId: string, avatarUrl: string | null) => {
    onAvatarSynchronized?.(personId, avatarUrl);
    dispatchAvatarSyncEvent(personId, avatarUrl);
  };

  const handleCreateFaceTag = async (
    box: NormalizedBox,
    personId: string,
    isPrimaryAvatar: boolean
  ) => {
    const candidate = candidateById.get(personId);
    let createdTag: FaceTagDto;
    try {
      createdTag = await createFaceTag(currentAsset.id, {
        person_id: personId,
        x: box.x,
        y: box.y,
        width: box.width,
        height: box.height,
        is_primary_avatar: isPrimaryAvatar,
      });
    } catch {
      createdTag = {
        id: `ft-${Date.now()}`,
        person_id: personId,
        person_name: candidate?.displayName ?? personId,
        is_living: candidate?.isLiving ?? false,
        x: box.x,
        y: box.y,
        width: box.width,
        height: box.height,
        is_primary_avatar: isPrimaryAvatar,
      };
    }

    const nextTags = currentAsset.face_tags
      .map((existing) =>
        isPrimaryAvatar && existing.person_id === personId
          ? { ...existing, is_primary_avatar: false }
          : existing
      )
      .concat(createdTag);

    const nextLinked = currentAsset.linked_person_ids.includes(personId)
      ? currentAsset.linked_person_ids
      : [...currentAsset.linked_person_ids, personId];

    const updatedAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      face_tags: nextTags,
      linked_person_ids: nextLinked,
      version: (currentAsset.version ?? 1) + 1,
    };

    setCurrentAsset(updatedAsset);
    onAssetUpdated?.(updatedAsset);

    if (isPrimaryAvatar) {
      const cropUrl = getMediaCropUrl(currentAsset.id, box);
      notifyAvatarSync(personId, cropUrl);
      addToast({
        type: 'success',
        title: t('mediaArchive.avatarSyncedToast', {
          name: createdTag.person_name,
        }),
      });
    } else {
      addToast({
        type: 'success',
        title: t('mediaArchive.faceTagSavedToast', {
          name: createdTag.person_name,
        }),
      });
    }
  };

  const handleUpdateFaceTagBox = async (
    tagId: string,
    box: NormalizedBox
  ) => {
    const target = currentAsset.face_tags.find((tItem) => tItem.id === tagId);
    if (!target) return;

    const updatedTag: FaceTagDto = {
      ...target,
      x: box.x,
      y: box.y,
      width: box.width,
      height: box.height,
    };

    const nextAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      face_tags: currentAsset.face_tags.map((tItem) =>
        tItem.id === tagId ? updatedTag : tItem
      ),
    };
    setCurrentAsset(nextAsset);
    onAssetUpdated?.(nextAsset);

    try {
      await updateFaceTag(currentAsset.id, tagId, {
        x: box.x,
        y: box.y,
        width: box.width,
        height: box.height,
      });
    } catch {
      // Keep optimistic box update in UI
    }

    if (updatedTag.is_primary_avatar) {
      notifyAvatarSync(
        updatedTag.person_id,
        getMediaCropUrl(currentAsset.id, box)
      );
    }
  };

  const handleTogglePrimaryAvatar = async (
    tag: FaceTagDto,
    nextIsPrimary: boolean
  ) => {
    try {
      await updateFaceTag(currentAsset.id, tag.id, {
        is_primary_avatar: nextIsPrimary,
      });
    } catch {
      // Apply locally when offline
    }

    const nextTags = currentAsset.face_tags.map((tItem) => {
      if (tItem.id === tag.id) {
        return { ...tItem, is_primary_avatar: nextIsPrimary };
      }
      if (nextIsPrimary && tItem.person_id === tag.person_id) {
        return { ...tItem, is_primary_avatar: false };
      }
      return tItem;
    });

    const nextAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      face_tags: nextTags,
      version: (currentAsset.version ?? 1) + 1,
    };
    setCurrentAsset(nextAsset);
    onAssetUpdated?.(nextAsset);

    if (nextIsPrimary) {
      const cropUrl = getMediaCropUrl(currentAsset.id, {
        x: tag.x,
        y: tag.y,
        width: tag.width,
        height: tag.height,
      });
      notifyAvatarSync(tag.person_id, cropUrl);
      addToast({
        type: 'success',
        title: t('mediaArchive.avatarSyncedToast', {
          name: tag.person_name,
        }),
      });
    } else {
      notifyAvatarSync(tag.person_id, null);
    }
  };

  const handleDeleteFaceTag = async (tagId: string) => {
    const removed = currentAsset.face_tags.find((tItem) => tItem.id === tagId);
    try {
      await deleteFaceTag(currentAsset.id, tagId);
    } catch {
      // Apply locally when offline
    }

    const nextAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      face_tags: currentAsset.face_tags.filter((tItem) => tItem.id !== tagId),
      version: (currentAsset.version ?? 1) + 1,
    };
    setCurrentAsset(nextAsset);
    onAssetUpdated?.(nextAsset);

    if (removed?.is_primary_avatar) {
      notifyAvatarSync(removed.person_id, null);
    }
  };

  const handleSaveMetadata = async (patch: UpdateMediaInput) => {
    try {
      const updated = await updateMediaAsset(currentAsset.id, patch);
      setCurrentAsset(updated);
      onAssetUpdated?.(updated);
    } catch {
      const fallbackUpdated: MediaAssetSummaryDto = {
        ...currentAsset,
        title: patch.title ?? currentAsset.title,
        date_taken:
          patch.date_taken !== undefined
            ? patch.date_taken
            : currentAsset.date_taken,
        place_full_name:
          patch.place_path !== undefined
            ? patch.place_path
            : currentAsset.place_full_name,
        description:
          patch.description !== undefined
            ? patch.description
            : currentAsset.description,
        is_private: patch.is_private ?? currentAsset.is_private,
        version: (currentAsset.version ?? 1) + 1,
      };
      setCurrentAsset(fallbackUpdated);
      onAssetUpdated?.(fallbackUpdated);
    }

    addToast({
      type: 'success',
      title: t('mediaArchive.metadataSavedToast'),
    });
  };

  const handleDeleteAsset = async () => {
    try {
      await deleteMediaAsset(currentAsset.id);
    } catch {
      // Proceed locally
    }
    onAssetDeleted?.(currentAsset.id);
    onOpenChange(false);
    addToast({
      type: 'success',
      title: t('mediaArchive.mediaDeletedToast'),
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-2 sm:p-6">
      <div
        aria-hidden="true"
        onClick={() => onOpenChange(false)}
        className="fixed inset-0 bg-primary/60 backdrop-blur-sm"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={currentAsset.title}
        className="relative z-10 flex flex-col w-full max-w-7xl max-h-[92vh] rounded-3xl border border-border bg-canvas shadow-2xl overflow-hidden"
      >
        {/* Top Studio Bar */}
        <div className="flex items-center justify-between gap-3 border-b border-border bg-surface px-4 sm:px-6 py-3">
          <div className="flex items-center gap-2 min-w-0">
            <h2 className="font-serif text-lg font-semibold text-primary truncate">
              {currentAsset.title}
            </h2>
          </div>

          <div className="flex items-center gap-1.5">
            {onPrevAsset && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onPrevAsset}
                aria-label={t('mediaArchive.prevPhotoAria')}
                className="h-8 w-8 min-h-0 p-0"
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
            )}
            {onNextAsset && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onNextAsset}
                aria-label={t('mediaArchive.nextPhotoAria')}
                className="h-8 w-8 min-h-0 p-0"
              >
                <ChevronRight className="h-4 w-4" />
              </Button>
            )}
            <Button
              variant="secondary"
              size="sm"
              onClick={() => onOpenChange(false)}
              leftIcon={<X className="h-4 w-4" />}
            >
              {t('common.close')}
            </Button>
          </div>
        </div>

        {/* Dual-Pane Workspace: Left Canvas (7 cols) + Right Metadata & Face Tags (5 cols) */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 p-4 sm:p-5 overflow-y-auto flex-1">
          <div className="lg:col-span-7 flex flex-col">
            <FaceTagCanvas
              asset={currentAsset}
              canEdit={canEdit}
              isTaggingMode={isTaggingMode}
              onToggleTaggingMode={setIsTaggingMode}
              candidates={candidates}
              highlightedTagId={highlightedTagId}
              onHighlightTag={setHighlightedTagId}
              onCreateFaceTag={handleCreateFaceTag}
              onUpdateFaceTagBox={handleUpdateFaceTagBox}
              onOpenPerson={(personId) => {
                onOpenChange(false);
                onOpenPerson(personId);
              }}
            />
          </div>

          <div className="lg:col-span-5 flex flex-col">
            <MediaMetadataPanel
              asset={currentAsset}
              canEdit={canEdit}
              candidates={candidates}
              highlightedTagId={highlightedTagId}
              onHighlightTag={setHighlightedTagId}
              onSaveMetadata={handleSaveMetadata}
              onDeleteAsset={canEdit ? handleDeleteAsset : undefined}
              onTogglePrimaryAvatar={handleTogglePrimaryAvatar}
              onDeleteFaceTag={handleDeleteFaceTag}
              onStartTaggingAnotherFace={() => setIsTaggingMode(true)}
              onOpenPerson={(personId) => {
                onOpenChange(false);
                onOpenPerson(personId);
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
