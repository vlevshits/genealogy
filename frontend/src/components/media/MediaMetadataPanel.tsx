import React, { useEffect, useState } from 'react';
import {
  Calendar,
  ExternalLink,
  Lock,
  MapPin,
  Plus,
  Save,
  Star,
  Trash2,
  Users,
} from 'lucide-react';
import {
  FaceTagDto,
  getMediaCropUrl,
  MediaAssetSummaryDto,
  UpdateMediaInput,
} from '../../api/client';
import {
  Avatar,
  Badge,
  Button,
  Switch,
  Textarea,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';
import {
  DEFAULT_FACE_CANDIDATES,
  FaceTagCandidatePerson,
} from './PersonFacePicker';

export interface MediaMetadataPanelProps {
  asset: MediaAssetSummaryDto;
  canEdit: boolean;
  candidates?: FaceTagCandidatePerson[];
  highlightedTagId?: string | null;
  onHighlightTag?: (tagId: string | null) => void;
  onSaveMetadata: (patch: UpdateMediaInput) => Promise<void> | void;
  onDeleteAsset?: () => Promise<void> | void;
  onTogglePrimaryAvatar?: (
    tag: FaceTagDto,
    nextIsPrimary: boolean
  ) => Promise<void> | void;
  onDeleteFaceTag?: (tagId: string) => Promise<void> | void;
  onStartTaggingAnotherFace?: () => void;
  onOpenPerson?: (personId: string) => void;
}

export const MediaMetadataPanel: React.FC<MediaMetadataPanelProps> = ({
  asset,
  canEdit,
  candidates = DEFAULT_FACE_CANDIDATES,
  highlightedTagId = null,
  onHighlightTag,
  onSaveMetadata,
  onDeleteAsset,
  onTogglePrimaryAvatar = () => {},
  onDeleteFaceTag = () => {},
  onStartTaggingAnotherFace = () => {},
  onOpenPerson = () => {},
}) => {
  const { t } = useI18n();

  const [title, setTitle] = useState<string>(asset.title);
  const [dateTaken, setDateTaken] = useState<string>(asset.date_taken ?? '');
  const [placePath, setPlacePath] = useState<string>(
    asset.place_full_name ?? ''
  );
  const [description, setDescription] = useState<string>(
    asset.description ?? ''
  );
  const [isPrivate, setIsPrivate] = useState<boolean>(asset.is_private);
  const [saving, setSaving] = useState<boolean>(false);

  useEffect(() => {
    setTitle(asset.title);
    setDateTaken(asset.date_taken ?? '');
    setPlacePath(asset.place_full_name ?? '');
    setDescription(asset.description ?? '');
    setIsPrivate(asset.is_private);
  }, [asset]);

  const candidateById = new Map<string, FaceTagCandidatePerson>(
    candidates.map((c) => [c.id, c])
  );

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

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canEdit || saving) return;
    setSaving(true);
    try {
      await onSaveMetadata({
        title: title.trim() || asset.title,
        date_taken: dateTaken.trim() || null,
        place_path: placePath.trim() || null,
        description: description.trim() || null,
        is_private: isPrivate,
        version: asset.version,
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex flex-col h-full rounded-2xl border border-border bg-surface p-4 sm:p-5 space-y-5 overflow-y-auto">
      {/* Section 1: Metadata (Editable for Editor/Admin, Read-Only for Viewer) */}
      <div className="space-y-3">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Badge variant="neutral">{asset.gid}</Badge>
            {asset.is_private && (
              <Badge variant="warning" icon={<Lock className="h-3 w-3" />}>
                {t('profile.privateBadge')}
              </Badge>
            )}
          </div>
          {asset.mime_type && (
            <span className="font-mono text-[11px] text-tertiary">
              {asset.mime_type}
            </span>
          )}
        </div>

        {canEdit ? (
          <form onSubmit={handleSave} className="space-y-3">
            <FormField name="media_title" label={t('mediaArchive.fieldTitle')}>
              <TextInput
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                aria-label={t('mediaArchive.fieldTitle')}
              />
            </FormField>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <FormField
                name="media_date"
                label={t('mediaArchive.fieldDateTaken')}
              >
                <TextInput
                  value={dateTaken}
                  onChange={(e) => setDateTaken(e.target.value)}
                  placeholder={t('mediaArchive.fieldDatePlaceholder')}
                  aria-label={t('mediaArchive.fieldDateTaken')}
                />
              </FormField>

              <FormField
                name="media_place"
                label={t('mediaArchive.fieldPlace')}
              >
                <TextInput
                  value={placePath}
                  onChange={(e) => setPlacePath(e.target.value)}
                  placeholder={t('mediaArchive.fieldPlacePlaceholder')}
                  aria-label={t('mediaArchive.fieldPlace')}
                />
              </FormField>
            </div>

            <FormField
              name="media_description"
              label={t('mediaArchive.fieldDescription')}
            >
              <Textarea
                rows={2}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder={t('mediaArchive.fieldDescriptionPlaceholder')}
                aria-label={t('mediaArchive.fieldDescription')}
              />
            </FormField>

            <Switch
              checked={isPrivate}
              onCheckedChange={setIsPrivate}
              label={t('mediaArchive.privateToggleLabel')}
            />

            <div className="flex items-center justify-between gap-2 pt-2">
              {onDeleteAsset ? (
                <Button
                  type="button"
                  variant="danger"
                  size="sm"
                  onClick={() => {
                    void onDeleteAsset();
                  }}
                  leftIcon={<Trash2 className="h-3.5 w-3.5" />}
                >
                  {t('mediaArchive.deleteMediaBtn')}
                </Button>
              ) : (
                <div />
              )}

              <Button
                type="submit"
                variant="primary"
                size="sm"
                loading={saving}
                leftIcon={<Save className="h-3.5 w-3.5" />}
              >
                {t('mediaArchive.saveMetadataBtn')}
              </Button>
            </div>
          </form>
        ) : (
          <div className="space-y-2">
            <h2 className="font-serif text-xl font-semibold text-primary">
              {asset.title}
            </h2>
            {asset.date_taken && (
              <p className="inline-flex items-center gap-1.5 text-xs text-secondary mr-3">
                <Calendar className="h-3.5 w-3.5 text-tertiary" />
                <span>{asset.date_taken}</span>
              </p>
            )}
            {asset.place_full_name && (
              <p className="inline-flex items-center gap-1.5 text-xs text-secondary">
                <MapPin className="h-3.5 w-3.5 text-tertiary" />
                <span>{asset.place_full_name}</span>
              </p>
            )}
            {asset.description && (
              <p className="text-sm text-secondary pt-1">{asset.description}</p>
            )}
          </div>
        )}
      </div>

      {/* Section 2: Tagged People List & Primary Avatar Controls */}
      <div className="border-t border-border pt-4 space-y-3">
        <div className="flex items-center justify-between gap-2">
          <h3 className="inline-flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-secondary">
            <Users className="h-3.5 w-3.5 text-brand" />
            <span>
              {t('mediaArchive.taggedPeopleHeading', {
                count: asset.face_tags.length,
              })}
            </span>
          </h3>

          {canEdit && asset.mime_type.startsWith('image/') && (
            <Button
              variant="secondary"
              size="sm"
              onClick={onStartTaggingAnotherFace}
              leftIcon={<Plus className="h-3.5 w-3.5" />}
              className="h-7 min-h-0 px-2.5 text-xs"
            >
              {t('mediaArchive.tagAnotherFaceBtn')}
            </Button>
          )}
        </div>

        {asset.face_tags.length === 0 ? (
          <p className="text-xs text-secondary py-2">
            {t('mediaArchive.noFacesTaggedYet')}
          </p>
        ) : (
          <div className="space-y-2">
            {asset.face_tags.map((tag) => {
              const isHighlighted = highlightedTagId === tag.id;
              const vitals = formatPersonVitals(tag);
              const cand = candidateById.get(tag.person_id);
              const cropPreviewUrl = tag.is_primary_avatar
                ? getMediaCropUrl(asset.id, {
                    x: tag.x,
                    y: tag.y,
                    width: tag.width,
                    height: tag.height,
                  })
                : cand?.avatarUrl ?? undefined;

              return (
                <div
                  key={tag.id}
                  onMouseEnter={() => onHighlightTag?.(tag.id)}
                  onMouseLeave={() => onHighlightTag?.(null)}
                  className={cn(
                    'rounded-xl border p-2.5 transition-colors space-y-2',
                    isHighlighted
                      ? 'border-brand bg-brand/8'
                      : 'border-border bg-surface-muted/40'
                  )}
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2.5 min-w-0">
                      <Avatar
                        name={tag.person_name}
                        src={cropPreviewUrl}
                        size="sm"
                      />
                      <div className="min-w-0">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => onOpenPerson(tag.person_id)}
                          rightIcon={<ExternalLink className="h-3 w-3" />}
                          className="h-auto min-h-0 p-0 text-left text-sm font-semibold text-primary hover:text-brand"
                        >
                          {tag.person_name}
                        </Button>
                        {vitals && (
                          <div className="text-xs text-secondary truncate">
                            {vitals}
                          </div>
                        )}
                      </div>
                    </div>

                    {tag.is_primary_avatar && (
                      <Badge
                        variant="brand"
                        icon={<Star className="h-3 w-3" />}
                      >
                        {t('mediaArchive.primaryAvatarBadge')}
                      </Badge>
                    )}
                  </div>

                  {canEdit && (
                    <div className="flex items-center justify-between gap-2 pt-1.5 border-t border-border/60">
                      <Button
                        variant={tag.is_primary_avatar ? 'secondary' : 'ghost'}
                        size="sm"
                        onClick={() => {
                          void onTogglePrimaryAvatar(
                            tag,
                            !tag.is_primary_avatar
                          );
                        }}
                        leftIcon={<Star className="h-3 w-3" />}
                        className="h-7 min-h-0 px-2 text-xs"
                      >
                        {tag.is_primary_avatar
                          ? t('mediaArchive.clearPrimaryAvatarBtn')
                          : t('mediaArchive.setAsPrimaryAvatarBtn')}
                      </Button>

                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          void onDeleteFaceTag(tag.id);
                        }}
                        leftIcon={<Trash2 className="h-3 w-3 text-danger" />}
                        aria-label={t('mediaArchive.removeFaceTagAria')}
                        className="h-7 min-h-0 px-2 text-xs text-danger hover:bg-danger/10"
                      >
                        {t('mediaArchive.removeFaceTagBtn')}
                      </Button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
