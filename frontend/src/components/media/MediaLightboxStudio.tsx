import React, { useEffect, useState } from 'react';
import { ChevronLeft, ChevronRight, X } from 'lucide-react';
import {
  createFaceTag,
  createMediaLink,
  deleteFaceTag,
  deleteMediaAsset,
  deleteMediaLink,
  FaceTagDto,
  getMediaCropUrl,
  MediaAssetSummaryDto,
  MediaLinkDto,
  updateFaceTag,
  updateMediaAsset,
  UpdateMediaInput,
} from '../../api/client';
import { Button, useToast } from '../ui';
import { useI18n } from '../../i18n';
import { FaceTagCanvas, NormalizedBox } from './FaceTagCanvas';
import { MediaMetadataPanel } from './MediaMetadataPanel';
import { LinkPersonDialog } from './LinkPersonDialog';
import {
  detectFacesInImage,
  filterOverlappingCandidates,
} from './faceDetection';
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
  const [targetPersonId, setTargetPersonId] = useState<string | null>(null);
  const [candidateBoxes, setCandidateBoxes] = useState<NormalizedBox[]>([]);
  const [isDetectingFaces, setIsDetectingFaces] = useState<boolean>(false);
  const [isLinkPersonDialogOpen, setIsLinkPersonDialogOpen] =
    useState<boolean>(false);

  useEffect(() => {
    setCurrentAsset(asset);
    setIsTaggingMode(false);
    setHighlightedTagId(null);
    setTargetPersonId(null);
    setCandidateBoxes([]);
    setIsDetectingFaces(false);
    setIsLinkPersonDialogOpen(false);
  }, [asset]);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (isLinkPersonDialogOpen) {
          setIsLinkPersonDialogOpen(false);
        } else if (targetPersonId) {
          setTargetPersonId(null);
        } else {
          onOpenChange(false);
        }
      }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [open, onOpenChange, isLinkPersonDialogOpen, targetPersonId]);

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

    // Filter candidate boxes to remove ones overlapping newly created tag
    setCandidateBoxes((prev) =>
      filterOverlappingCandidates(prev, [createdTag], 0.3)
    );
    setTargetPersonId(null);

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

  const handleDetectFaces = async (imgElement: HTMLImageElement | null) => {
    if (!imgElement) return;
    setIsDetectingFaces(true);
    try {
      const rawCandidates = await detectFacesInImage(imgElement);
      const remaining = filterOverlappingCandidates(
        rawCandidates,
        currentAsset.face_tags,
        0.3
      );
      setCandidateBoxes(remaining);

      if (remaining.length > 0) {
        addToast({
          type: 'info',
          title: t('mediaArchive.autoDetectFoundCount', {
            count: remaining.length,
          }),
        });
      } else {
        addToast({
          type: 'info',
          title: t('mediaArchive.autoDetectNoneFound'),
        });
      }
    } catch {
      addToast({
        type: 'info',
        title: t('mediaArchive.autoDetectNoneFound'),
      });
    } finally {
      setIsDetectingFaces(false);
    }
  };

  const handleDismissCandidateBox = (index: number) => {
    setCandidateBoxes((prev) => prev.filter((_, i) => i !== index));
  };

  const handleLinkPerson = async (personId: string) => {
    let createdLink: MediaLinkDto;
    try {
      createdLink = await createMediaLink(currentAsset.id, {
        entity_type: 'individual',
        entity_id: personId,
        role: 'depicted',
      });
    } catch {
      createdLink = {
        id: `link-${Date.now()}`,
        media_id: currentAsset.id,
        entity_type: 'individual',
        entity_id: personId,
        role: 'depicted',
      };
    }

    const nextLinked = currentAsset.linked_person_ids.includes(personId)
      ? currentAsset.linked_person_ids
      : [...currentAsset.linked_person_ids, personId];

    const nextLinks = [...(currentAsset.links || []), createdLink];

    const updatedAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      linked_person_ids: nextLinked,
      links: nextLinks,
      version: (currentAsset.version ?? 1) + 1,
    };

    setCurrentAsset(updatedAsset);
    onAssetUpdated?.(updatedAsset);

    const person = candidateById.get(personId);
    addToast({
      type: 'success',
      title: t('mediaArchive.personLinkedToast', {
        name: person?.displayName || personId,
      }),
    });
  };

  const handleUnlinkPerson = async (personId: string) => {
    const link = currentAsset.links?.find(
      (l) => l.entity_type === 'individual' && l.entity_id === personId
    );
    const linkId = link ? link.id : personId;

    try {
      await deleteMediaLink(currentAsset.id, linkId);
    } catch {
      // Apply locally if offline
    }

    const stillTagged = currentAsset.face_tags.some(
      (tItem) => tItem.person_id === personId
    );

    const nextLinked = stillTagged
      ? currentAsset.linked_person_ids
      : currentAsset.linked_person_ids.filter((pid) => pid !== personId);

    const nextLinks = (currentAsset.links || []).filter(
      (l) => !(l.entity_type === 'individual' && l.entity_id === personId)
    );

    const updatedAsset: MediaAssetSummaryDto = {
      ...currentAsset,
      linked_person_ids: nextLinked,
      links: nextLinks,
      version: (currentAsset.version ?? 1) + 1,
    };

    setCurrentAsset(updatedAsset);
    onAssetUpdated?.(updatedAsset);

    if (targetPersonId === personId) {
      setTargetPersonId(null);
    }

    const person = candidateById.get(personId);
    addToast({
      type: 'info',
      title: t('mediaArchive.personUnlinkedToast', {
        name: person?.displayName || personId,
      }),
    });
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
    <>
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
                  className="h-8 w-8 min-h-0 p-0 touch-target-expand"
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
                  className="h-8 w-8 min-h-0 p-0 touch-target-expand"
                >
                  <ChevronRight className="h-4 w-4" />
                </Button>
              )}
              <Button
                variant="secondary"
                size="sm"
                onClick={() => onOpenChange(false)}
                leftIcon={<X className="h-4 w-4" />}
                className="h-8 min-h-8 px-2.5 text-xs touch-target-expand"
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
                candidateBoxes={candidateBoxes}
                onDetectFaces={handleDetectFaces}
                isDetectingFaces={isDetectingFaces}
                onDismissCandidateBox={handleDismissCandidateBox}
                targetPersonId={targetPersonId}
                onCancelTargetPerson={() => setTargetPersonId(null)}
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
                targetPersonId={targetPersonId}
                onSelectTargetPerson={setTargetPersonId}
                onSaveMetadata={handleSaveMetadata}
                onDeleteAsset={canEdit ? handleDeleteAsset : undefined}
                onTogglePrimaryAvatar={handleTogglePrimaryAvatar}
                onDeleteFaceTag={handleDeleteFaceTag}
                onStartTaggingAnotherFace={() => setIsTaggingMode(true)}
                onLinkPersonClick={() => setIsLinkPersonDialogOpen(true)}
                onUnlinkPerson={handleUnlinkPerson}
                onOpenPerson={(personId) => {
                  onOpenChange(false);
                  onOpenPerson(personId);
                }}
              />
            </div>
          </div>
        </div>
      </div>

      {canEdit && (
        <LinkPersonDialog
          open={isLinkPersonDialogOpen}
          onOpenChange={setIsLinkPersonDialogOpen}
          asset={currentAsset}
          candidates={candidates}
          onLinkPerson={handleLinkPerson}
        />
      )}
    </>
  );
};
