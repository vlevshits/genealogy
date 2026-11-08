import React, { useEffect, useState } from 'react';
import { Button, Dialog, Select, TextInput } from '../ui';
import { FormField } from '../form';
import {
  FixPlaceHierarchyRequest,
  PlaceDto,
  PlaceHierarchyIssueDto,
} from '../../api/client';
import { useI18n } from '../../i18n';
import { normalizePlaceType } from '../../lib/domain-enums';

export interface PlaceHierarchyCustomizeModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  issue: PlaceHierarchyIssueDto | null;
  availablePlaces: PlaceDto[];
  saving?: boolean;
  onSubmit: (payload: FixPlaceHierarchyRequest) => void;
}

const PLACE_TYPE_KEYS = [
  'country',
  'region',
  'district',
  'city',
  'village',
  'parish',
  'cemetery',
  'address',
  'other',
] as const;

export const PlaceHierarchyCustomizeModal: React.FC<
  PlaceHierarchyCustomizeModalProps
> = ({
  open,
  onOpenChange,
  issue,
  availablePlaces,
  saving = false,
  onSubmit,
}) => {
  const { t } = useI18n();
  const [cleanName, setCleanName] = useState('');
  const [placeType, setPlaceType] = useState('city');
  const [parentId, setParentId] = useState<string>('');

  useEffect(() => {
    if (issue) {
      setCleanName(issue.proposed_clean_name || issue.current_name);
      setPlaceType(
        normalizePlaceType(
          issue.proposed_place_type || issue.current_place_type || 'city'
        )
      );
      setParentId(issue.proposed_parent_id || issue.current_parent_id || '');
    }
  }, [issue]);

  if (!issue) return null;

  const parentOptions = [
    { value: '', label: t('quality.places.noParent') },
    ...availablePlaces
      .filter((p) => p.id !== issue.place_id)
      .map((p) => ({
        value: p.id,
        label: `${p.full_name || p.name} (${p.gid})`,
      })),
  ];

  const typeOptions = PLACE_TYPE_KEYS.map((key) => ({
    value: key,
    label: t(`places.types.${key}`),
  }));

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    if (!cleanName.trim()) return;
    onSubmit({
      place_id: issue.place_id,
      clean_name: cleanName.trim(),
      place_type: placeType,
      parent_id: parentId || null,
      create_missing_parents: true,
      proposed_chain: issue.proposed_chain,
    });
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('quality.places.customizeModalTitle')}
      description={t('quality.places.customizeModalDesc')}
      size="md"
      footer={
        <div className="flex items-center justify-end gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onOpenChange(false)}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            size="sm"
            loading={saving}
            disabled={!cleanName.trim()}
            data-testid="save-customized-place-fix-btn"
            onClick={handleSave}
          >
            {t('quality.places.applyFixBtn')}
          </Button>
        </div>
      }
    >
      <form onSubmit={handleSave} className="space-y-4">
        <div className="rounded-lg bg-surface-muted p-3 border border-border text-xs text-secondary">
          <span className="font-semibold text-primary">
            {t('quality.places.currentRecordLabel')}:
          </span>{' '}
          {issue.current_full_name || issue.current_name}
        </div>

        <FormField
          name="clean_name"
          label={t('quality.places.cleanLeafNameLabel')}
          required
        >
          <TextInput
            value={cleanName}
            onChange={(e) => setCleanName(e.target.value)}
            aria-label={t('quality.places.cleanLeafNameLabel')}
          />
        </FormField>

        <FormField
          name="place_type"
          label={t('places.formTypeLabel')}
        >
          <Select
            value={placeType}
            onChange={setPlaceType}
            options={typeOptions}
            ariaLabel={t('places.formTypeLabel')}
          />
        </FormField>

        <FormField
          name="parent_id"
          label={t('places.formParentLabel')}
        >
          <Select
            value={parentId}
            onChange={setParentId}
            options={parentOptions}
            ariaLabel={t('places.formParentLabel')}
          />
        </FormField>
      </form>
    </Dialog>
  );
};
