import React, { useRef, useState } from 'react';
import {
  Crosshair,
  Download,
  ExternalLink,
  FileText,
  Maximize2,
  Mic,
  Move,
  Plus,
  RotateCcw,
  Sparkles,
  Star,
  UserCheck,
  X,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { FaceTagDto, MediaAssetSummaryDto } from '../../api/client';
import { Avatar, Badge, Button, IconButton } from '../ui';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';
import {
  DEFAULT_FACE_CANDIDATES,
  FaceTagCandidatePerson,
  PersonFacePicker,
} from './PersonFacePicker';
import { NormalizedBox } from './faceDetection';

export type { NormalizedBox };

export interface FaceTagCanvasProps {
  asset: MediaAssetSummaryDto;
  canEdit: boolean;
  isTaggingMode?: boolean;
  onToggleTaggingMode?: (active: boolean) => void;
  candidates?: FaceTagCandidatePerson[];
  candidateBoxes?: NormalizedBox[];
  onDetectFaces?: (imgElement: HTMLImageElement | null) => Promise<void> | void;
  isDetectingFaces?: boolean;
  onDismissCandidateBox?: (index: number) => void;
  targetPersonId?: string | null;
  onCancelTargetPerson?: () => void;
  highlightedTagId?: string | null;
  onHighlightTag?: (tagId: string | null) => void;
  onCreateFaceTag?: (
    box: NormalizedBox,
    personId: string,
    isPrimaryAvatar: boolean
  ) => Promise<void> | void;
  onUpdateFaceTagBox?: (
    tagId: string,
    box: NormalizedBox
  ) => Promise<void> | void;
  onOpenPerson?: (personId: string) => void;
}

function clamp01(val: number): number {
  return Math.max(0, Math.min(1, val));
}

export const FaceTagCanvas: React.FC<FaceTagCanvasProps> = ({
  asset,
  canEdit,
  isTaggingMode = false,
  onToggleTaggingMode = () => {},
  candidates = DEFAULT_FACE_CANDIDATES,
  candidateBoxes = [],
  onDetectFaces,
  isDetectingFaces = false,
  onDismissCandidateBox,
  targetPersonId = null,
  onCancelTargetPerson,
  highlightedTagId = null,
  onHighlightTag,
  onCreateFaceTag = () => {},
  onUpdateFaceTagBox,
  onOpenPerson = () => {},
}) => {
  const { t } = useI18n();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);

  const [zoom, setZoom] = useState<number>(1);
  const [hoveredTagId, setHoveredTagId] = useState<string | null>(null);
  const [draftBox, setDraftBox] = useState<NormalizedBox | null>(null);
  const [drawStart, setDrawStart] = useState<{ x: number; y: number } | null>(
    null
  );
  const [activeDragTag, setActiveDragTag] = useState<{
    tagId: string;
    mode: 'move' | 'resize';
    startPointer: { x: number; y: number };
    initialBox: NormalizedBox;
  } | null>(null);
  const [submittingTag, setSubmittingTag] = useState<boolean>(false);

  const isImage = asset.mime_type.startsWith('image/');
  const isPdf =
    asset.mime_type === 'application/pdf' ||
    asset.mime_type.startsWith('text/');
  const isAudio = asset.mime_type.startsWith('audio/');

  const candidateById = new Map<string, FaceTagCandidatePerson>(
    candidates.map((c) => [c.id, c])
  );

  const targetPerson = targetPersonId
    ? candidateById.get(targetPersonId)
    : null;

  const getNormalizedPointer = (
    clientX: number,
    clientY: number
  ): { x: number; y: number } | null => {
    const el = containerRef.current;
    if (!el) return null;
    const rect = el.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    return {
      x: clamp01((clientX - rect.left) / rect.width),
      y: clamp01((clientY - rect.top) / rect.height),
    };
  };

  const handleAdoptCandidate = (candBox: NormalizedBox, index: number) => {
    onDismissCandidateBox?.(index);
    if (targetPersonId) {
      void onCreateFaceTag(candBox, targetPersonId, false);
      onCancelTargetPerson?.();
    } else {
      setDraftBox(candBox);
    }
  };

  const handlePointerDownCanvas = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!canEdit || !isTaggingMode || !isImage) return;
    if ((e.target as HTMLElement).closest('[data-face-tag-interactive="true"]')) {
      return;
    }
    const pt = getNormalizedPointer(e.clientX, e.clientY);
    if (!pt) return;
    setDrawStart(pt);
    setDraftBox({
      x: pt.x,
      y: pt.y,
      width: 0.02,
      height: 0.02,
    });
  };

  const handlePointerMoveCanvas = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!canEdit || !isTaggingMode) return;
    const pt = getNormalizedPointer(e.clientX, e.clientY);
    if (!pt) return;

    if (drawStart) {
      const minX = Math.min(drawStart.x, pt.x);
      const minY = Math.min(drawStart.y, pt.y);
      const w = Math.max(0.05, Math.abs(pt.x - drawStart.x));
      const h = Math.max(0.05, Math.abs(pt.y - drawStart.y));
      setDraftBox({
        x: clamp01(minX),
        y: clamp01(minY),
        width: Math.min(w, 1 - clamp01(minX)),
        height: Math.min(h, 1 - clamp01(minY)),
      });
      return;
    }

    if (activeDragTag && onUpdateFaceTagBox) {
      const dx = pt.x - activeDragTag.startPointer.x;
      const dy = pt.y - activeDragTag.startPointer.y;
      if (activeDragTag.mode === 'move') {
        const nextX = clamp01(
          Math.min(
            1 - activeDragTag.initialBox.width,
            activeDragTag.initialBox.x + dx
          )
        );
        const nextY = clamp01(
          Math.min(
            1 - activeDragTag.initialBox.height,
            activeDragTag.initialBox.y + dy
          )
        );
        void onUpdateFaceTagBox(activeDragTag.tagId, {
          x: nextX,
          y: nextY,
          width: activeDragTag.initialBox.width,
          height: activeDragTag.initialBox.height,
        });
      } else {
        const nextW = Math.max(
          0.05,
          Math.min(
            1 - activeDragTag.initialBox.x,
            activeDragTag.initialBox.width + dx
          )
        );
        const nextH = Math.max(
          0.05,
          Math.min(
            1 - activeDragTag.initialBox.y,
            activeDragTag.initialBox.height + dy
          )
        );
        void onUpdateFaceTagBox(activeDragTag.tagId, {
          x: activeDragTag.initialBox.x,
          y: activeDragTag.initialBox.y,
          width: nextW,
          height: nextH,
        });
      }
    }
  };

  const handlePointerUpCanvas = () => {
    if (drawStart) {
      setDrawStart(null);
      let finalBox = draftBox;
      if (finalBox && (finalBox.width < 0.06 || finalBox.height < 0.06)) {
        finalBox = {
          x: clamp01(Math.min(0.8, finalBox.x)),
          y: clamp01(Math.min(0.8, finalBox.y)),
          width: 0.18,
          height: 0.22,
        };
      }

      if (targetPersonId && finalBox) {
        setDraftBox(null);
        void onCreateFaceTag(finalBox, targetPersonId, false);
        onCancelTargetPerson?.();
      } else {
        setDraftBox(finalBox);
      }
    }
    if (activeDragTag) {
      setActiveDragTag(null);
    }
  };

  const formatPersonVitals = (tag: FaceTagDto): string => {
    const cand = candidateById.get(tag.person_id);
    if (!cand) return '';
    const parts: string[] = [];
    if (cand.birthYear) {
      parts.push(`${t('domain.bornPrefix')} ${cand.birthYear}`);
    }
    if (!cand.isLiving && cand.deathYear) {
      parts.push(`† ${cand.deathYear}`);
    }
    return parts.join(' · ');
  };

  return (
    <div className="flex flex-col h-full bg-surface-muted/60 rounded-2xl border border-border overflow-hidden">
      {/* Top Banner when Target Person is Armed */}
      {canEdit && isTaggingMode && targetPerson && (
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 bg-brand/15 border-b border-brand/40 px-4 py-2 text-xs text-primary font-medium">
          <div className="flex items-center gap-2 min-w-0">
            <UserCheck className="h-4 w-4 text-brand shrink-0" />
            <span className="break-words">
              {t('mediaArchive.targetPersonBanner', {
                name: targetPerson.displayName,
              })}
            </span>
          </div>
          <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
            {onDetectFaces && isImage && (
              <Button
                variant="secondary"
                size="sm"
                disabled={isDetectingFaces}
                loading={isDetectingFaces}
                onClick={() => void onDetectFaces(imgRef.current)}
                leftIcon={<Sparkles className="h-3.5 w-3.5 text-sky-500" />}
                className="h-7 min-h-0 px-2.5 text-xs text-sky-950 dark:text-sky-100 bg-sky-50 dark:bg-sky-950/40 border-sky-300 dark:border-sky-700 touch-target-expand"
              >
                {isDetectingFaces
                  ? t('mediaArchive.detectingFacesBtn')
                  : t('mediaArchive.autoDetectFacesBtn')}
              </Button>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={onCancelTargetPerson}
              leftIcon={<X className="h-3.5 w-3.5" />}
              className="h-7 sm:h-6 min-h-0 px-2 text-xs touch-target-expand"
            >
              {t('mediaArchive.cancelTargetPerson')}
            </Button>
          </div>
        </div>
      )}

      {/* Top Tagging Banner when standard Tag Faces mode is active without target person */}
      {canEdit && isTaggingMode && !targetPerson && (
        <div className="flex flex-wrap items-center justify-between gap-2 bg-brand/12 border-b border-brand/30 px-4 py-2 text-xs text-primary">
          <span className="inline-flex items-center gap-2 font-medium">
            <Crosshair className="h-4 w-4 text-brand" />
            <span>{t('mediaArchive.taggingModeBanner')}</span>
          </span>

          <div className="flex items-center gap-2">
            {onDetectFaces && isImage && (
              <Button
                variant="secondary"
                size="sm"
                disabled={isDetectingFaces}
                loading={isDetectingFaces}
                onClick={() => void onDetectFaces(imgRef.current)}
                leftIcon={<Sparkles className="h-3.5 w-3.5 text-sky-500" />}
                className="h-7 min-h-0 px-2.5 text-xs text-sky-950 dark:text-sky-100 bg-sky-50 dark:bg-sky-950/40 border-sky-300 dark:border-sky-700"
              >
                {isDetectingFaces
                  ? t('mediaArchive.detectingFacesBtn')
                  : t('mediaArchive.autoDetectFacesBtn')}
              </Button>
            )}

            {!draftBox && (
              <Button
                variant="secondary"
                size="sm"
                onClick={() =>
                  setDraftBox({
                    x: 0.36,
                    y: 0.28,
                    width: 0.22,
                    height: 0.26,
                  })
                }
                leftIcon={<Plus className="h-3.5 w-3.5" />}
                className="h-7 min-h-0 px-2.5 text-xs"
              >
                {t('mediaArchive.placeFaceBoxBtn')}
              </Button>
            )}

            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setDraftBox(null);
                onToggleTaggingMode(false);
              }}
              className="h-7 min-h-0 px-2.5 text-xs"
            >
              {t('mediaArchive.doneTaggingBtn')}
            </Button>
          </div>
        </div>
      )}

      {/* Main Viewport Stage */}
      <div className="relative flex-1 flex items-center justify-center overflow-auto p-4 min-h-80 sm:min-h-[460px]">
        {isImage ? (
          <div
            ref={containerRef}
            onPointerDown={handlePointerDownCanvas}
            onPointerMove={handlePointerMoveCanvas}
            onPointerUp={handlePointerUpCanvas}
            style={{
              transform: `scale(${zoom})`,
              transformOrigin: 'center center',
            }}
            className={cn(
              'relative inline-block max-w-full max-h-full select-none transition-transform duration-150',
              canEdit && isTaggingMode ? 'cursor-crosshair' : 'cursor-default'
            )}
          >
            <img
              ref={imgRef}
              src={asset.file_url}
              alt={asset.title}
              draggable={false}
              className="block max-h-[62vh] w-auto rounded-xl border border-border object-contain shadow-md"
            />

            {/* Candidate Suggestion Rectangles from Auto-Detect */}
            {canEdit &&
              isTaggingMode &&
              candidateBoxes.map((candBox, idx) => (
                <div
                  key={`cand-${idx}-${candBox.x}-${candBox.y}`}
                  data-face-tag-interactive="true"
                  role="button"
                  tabIndex={0}
                  aria-label={t('mediaArchive.candidateFaceHover')}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleAdoptCandidate(candBox, idx);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleAdoptCandidate(candBox, idx);
                    }
                  }}
                  style={{
                    left: `${Number((candBox.x * 100).toFixed(2))}%`,
                    top: `${Number((candBox.y * 100).toFixed(2))}%`,
                    width: `${Number((candBox.width * 100).toFixed(2))}%`,
                    height: `${Number((candBox.height * 100).toFixed(2))}%`,
                  }}
                  className="absolute rounded-lg border-2 border-dashed border-sky-600 dark:border-sky-400 bg-sky-500/10 hover:bg-sky-500/20 focus-visible:ring-2 focus-visible:ring-sky-500 focus-visible:outline-none transition-all cursor-pointer group z-20 flex flex-col justify-between p-1 shadow-md"
                >
                  <div className="flex items-center justify-between w-full pointer-events-auto">
                    <div className="flex items-center gap-1 self-start rounded-md bg-sky-600/90 text-[10px] font-semibold text-white px-1.5 py-0.5 shadow-xs">
                      <Sparkles className="h-2.5 w-2.5" />
                      <span>{t('mediaArchive.suggestedFaceBadge')}</span>
                    </div>
                    {onDismissCandidateBox && (
                      <IconButton
                        label={t('mediaArchive.dismissCandidateAria')}
                        size="sm"
                        variant="ghost"
                        onClick={(e) => {
                          e.stopPropagation();
                          onDismissCandidateBox(idx);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.stopPropagation();
                            e.preventDefault();
                            onDismissCandidateBox(idx);
                          }
                        }}
                        className="h-5 w-5 min-h-0 min-w-0 p-0 rounded-full bg-surface/90 text-secondary hover:text-danger hover:bg-surface shadow-xs touch-target-expand cursor-pointer"
                      >
                        <X className="h-3 w-3" />
                      </IconButton>
                    )}
                  </div>
                  <div className="opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 transition-opacity self-center rounded-md bg-surface/95 text-primary text-[11px] font-medium px-2 py-0.5 shadow-md border border-border pointer-events-none">
                    {t('mediaArchive.suggestedFaceTooltip')}
                  </div>
                </div>
              ))}

            {/* Existing Face Bounding Boxes */}
            {asset.face_tags.map((tag) => {
              const isHovered =
                hoveredTagId === tag.id || highlightedTagId === tag.id;
              const vitals = formatPersonVitals(tag);

              return (
                <div
                  key={tag.id}
                  data-face-tag-interactive="true"
                  onMouseEnter={() => {
                    setHoveredTagId(tag.id);
                    onHighlightTag?.(tag.id);
                  }}
                  onMouseLeave={() => {
                    setHoveredTagId(null);
                    onHighlightTag?.(null);
                  }}
                  style={{
                    left: `${tag.x * 100}%`,
                    top: `${tag.y * 100}%`,
                    width: `${tag.width * 100}%`,
                    height: `${tag.height * 100}%`,
                  }}
                  className={cn(
                    'absolute rounded-lg border-2 transition-all',
                    isHovered || isTaggingMode
                      ? 'border-brand bg-brand/15 shadow-lg'
                      : 'border-surface/80 bg-transparent hover:border-brand'
                  )}
                >
                  {/* Editor Drag Move & Resize Handles in Tagging Mode */}
                  {canEdit && isTaggingMode && (
                    <>
                      <span
                        title={t('mediaArchive.moveFaceBoxAria')}
                        onPointerDown={(e) => {
                          e.stopPropagation();
                          const pt = getNormalizedPointer(e.clientX, e.clientY);
                          if (!pt) return;
                          setActiveDragTag({
                            tagId: tag.id,
                            mode: 'move',
                            startPointer: pt,
                            initialBox: {
                              x: tag.x,
                              y: tag.y,
                              width: tag.width,
                              height: tag.height,
                            },
                          });
                        }}
                        className="absolute top-1 left-1 inline-flex h-5 w-5 items-center justify-center rounded-md bg-surface/90 text-primary shadow-xs cursor-move touch-target-expand before:absolute before:-inset-2 before:content-['']"
                      >
                        <Move className="h-3 w-3" />
                      </span>
                      <span
                        title={t('mediaArchive.resizeFaceBoxAria')}
                        onPointerDown={(e) => {
                          e.stopPropagation();
                          const pt = getNormalizedPointer(e.clientX, e.clientY);
                          if (!pt) return;
                          setActiveDragTag({
                            tagId: tag.id,
                            mode: 'resize',
                            startPointer: pt,
                            initialBox: {
                              x: tag.x,
                              y: tag.y,
                              width: tag.width,
                              height: tag.height,
                            },
                          });
                        }}
                        className="absolute bottom-0.5 right-0.5 inline-flex h-5 w-5 items-center justify-center rounded-md bg-brand text-surface shadow-xs cursor-nwse-resize touch-target-expand before:absolute before:-inset-2 before:content-['']"
                      >
                        <Maximize2 className="h-3 w-3" />
                      </span>
                    </>
                  )}

                  {/* Hover Tooltip Card with Name, Vital Dates (†) & Profile Link */}
                  <div
                    className={cn(
                      'absolute left-0 top-full mt-1.5 z-20 min-w-48 rounded-xl border border-border bg-surface/95 backdrop-blur-md p-2.5 shadow-xl transition-opacity',
                      isHovered
                        ? 'opacity-100 pointer-events-auto'
                        : 'opacity-0 pointer-events-none'
                    )}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <Avatar name={tag.person_name} size="xs" />
                        <div className="min-w-0">
                          <div className="text-xs font-semibold text-primary truncate">
                            {tag.person_name}
                          </div>
                          {vitals && (
                            <div className="text-[11px] text-secondary truncate">
                              {vitals}
                            </div>
                          )}
                        </div>
                      </div>
                      {tag.is_primary_avatar && (
                        <Badge
                          variant="brand"
                          icon={<Star className="h-2.5 w-2.5" />}
                        >
                          {t('mediaArchive.primaryAvatarShort')}
                        </Badge>
                      )}
                    </div>
                    <div className="mt-2 pt-1.5 border-t border-border flex justify-end">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onOpenPerson(tag.person_id)}
                        rightIcon={<ExternalLink className="h-3 w-3" />}
                        className="h-6 min-h-0 px-2 text-[11px] text-brand"
                      >
                        {t('mediaArchive.openProfileLink')}
                      </Button>
                    </div>
                  </div>
                </div>
              );
            })}

            {/* Newly Drawn Draft Face Bounding Box & Autocomplete Picker */}
            {canEdit && draftBox && (
              <div
                data-face-tag-interactive="true"
                style={{
                  left: `${draftBox.x * 100}%`,
                  top: `${draftBox.y * 100}%`,
                  width: `${draftBox.width * 100}%`,
                  height: `${draftBox.height * 100}%`,
                }}
                className="absolute rounded-lg border-2 border-dashed border-brand bg-brand/20 shadow-xl z-30"
              >
                {!drawStart && (
                  <div className="absolute left-0 top-full mt-2 z-40">
                    <PersonFacePicker
                      candidates={candidates}
                      submitting={submittingTag}
                      onCancel={() => setDraftBox(null)}
                      onConfirm={async (personId, isPrimaryAvatar) => {
                        setSubmittingTag(true);
                        try {
                          await onCreateFaceTag(
                            draftBox,
                            personId,
                            isPrimaryAvatar
                          );
                          setDraftBox(null);
                        } finally {
                          setSubmittingTag(false);
                        }
                      }}
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        ) : isPdf ? (
          <div className="flex flex-col items-center justify-center gap-3 p-10 text-center">
            <FileText className="h-14 w-14 text-accent" />
            <p className="text-sm font-medium text-primary">{asset.title}</p>
            <p className="text-xs text-secondary">
              {t('mediaArchive.documentPreviewDesc')}
            </p>
          </div>
        ) : isAudio ? (
          <div className="flex flex-col items-center justify-center gap-3 p-10 text-center">
            <Mic className="h-14 w-14 text-brand" />
            <p className="text-sm font-medium text-primary">{asset.title}</p>
            <p className="text-xs text-secondary">
              {t('mediaArchive.audioPreviewDesc')}
            </p>
          </div>
        ) : null}
      </div>

      {/* Bottom Canvas Controls Bar: Tag Faces Toggle, Zoom Controls, Download */}
      <div className="flex flex-wrap items-center justify-between gap-2 border-t border-border bg-surface px-4 py-2.5">
        <div className="flex items-center gap-2">
          {canEdit && isImage && (
            <Button
              variant={isTaggingMode ? 'primary' : 'secondary'}
              size="sm"
              onClick={() => {
                if (isTaggingMode) {
                  setDraftBox(null);
                  onCancelTargetPerson?.();
                }
                onToggleTaggingMode(!isTaggingMode);
              }}
              leftIcon={<Crosshair className="h-3.5 w-3.5" />}
              className="h-8 min-h-8 px-3 text-xs touch-target-expand"
            >
              {t('mediaArchive.tagFacesToolBtn')}
            </Button>
          )}
        </div>

        <div className="flex items-center gap-1.5">
          {isImage && (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={() =>
                  setZoom((z) => Math.max(0.5, Number((z - 0.25).toFixed(2))))
                }
                aria-label={t('mediaArchive.zoomOutAria')}
                className="h-8 w-8 min-h-0 p-0 touch-target-expand"
              >
                <ZoomOut className="h-4 w-4" />
              </Button>
              <span className="w-12 text-center font-mono text-xs text-secondary">
                {Math.round(zoom * 100)}%
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() =>
                  setZoom((z) => Math.min(2.5, Number((z + 0.25).toFixed(2))))
                }
                aria-label={t('mediaArchive.zoomInAria')}
                className="h-8 w-8 min-h-0 p-0 touch-target-expand"
              >
                <ZoomIn className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setZoom(1)}
                leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
                className="h-8 min-h-0 px-2 text-xs touch-target-expand"
              >
                {t('mediaArchive.zoomResetBtn')}
              </Button>
            </>
          )}

          <a
            href={asset.file_url}
            download={asset.original_file_name || asset.title}
            className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 text-xs font-medium text-primary hover:bg-surface-muted touch-target-expand"
          >
            <Download className="h-3.5 w-3.5" />
            <span>{t('mediaArchive.downloadBtn')}</span>
          </a>
        </div>
      </div>
    </div>
  );
};
