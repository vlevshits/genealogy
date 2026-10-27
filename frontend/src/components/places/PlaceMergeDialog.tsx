import React, { useEffect, useState } from 'react';
import { AlertTriangle, GitMerge } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  Select,
  useToast,
} from '../ui';
import { FormField } from '../form';
import { mergePlace, PlaceDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface PlaceMergeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sourcePlace: PlaceDto | null;
  availableTargets: PlaceDto[];
  onMerged?: (targetPlace: PlaceDto, mergedSourceId: string) => void;
}

export const PlaceMergeDialog: React.FC<PlaceMergeDialogProps> = ({
  open,
  onOpenChange,
  sourcePlace,
  availableTargets,
  onMerged,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const candidateTargets = availableTargets.filter(
    (p) => p.id !== sourcePlace?.id
  );
  const [targetId, setTargetId] = useState<string>(
    candidateTargets[0]?.id ?? ''
  );
  const [merging, setMerging] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    const firstCandidate = availableTargets.find(
      (p) => p.id !== sourcePlace?.id
    );
    setTargetId(firstCandidate?.id ?? '');
    setErrorMsg(null);
  }, [sourcePlace, availableTargets, open]);

  if (!sourcePlace) return null;

  const handleConfirmMerge = async (e: React.FormEvent) => {
    e.preventDefault();
    if (merging || !targetId) return;
    setMerging(true);
    setErrorMsg(null);
    try {
      const res = await mergePlace(targetId, sourcePlace.id);
      const selectedTargetPlace = availableTargets.find(
        (p) => p.id === targetId
      );
      const resolvedTargetPlace: PlaceDto | undefined =
        res && 'target_place' in res
          ? res.target_place
          : (res as PlaceDto | undefined) ?? selectedTargetPlace;
      const targetName =
        resolvedTargetPlace?.full_name ??
        selectedTargetPlace?.full_name ??
        'Target Place';
      addToast({
        type: 'success',
        title: t('places.merge.mergedToast'),
        description: targetName,
      });
      if (resolvedTargetPlace) {
        onMerged?.(resolvedTargetPlace, sourcePlace.id);
      }
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(
        err instanceof Error ? err.message : t('common.errorLoading')
      );
    } finally {
      setMerging(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={t('places.merge.title')}
      description={t('places.merge.subtitle', {
        source: sourcePlace.full_name,
      })}
    >
      <form onSubmit={handleConfirmMerge} className="space-y-4">
        <Alert
          variant="warning"
          icon={
            <AlertTriangle className="h-4 w-4 text-warning shrink-0 mt-0.5" />
          }
        >
          {t('places.merge.warningDesc', { source: sourcePlace.name })}
        </Alert>

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <FormField
          name="merge_target_place"
          label={t('places.merge.targetSelectLabel')}
          required
        >
          <Select
            ariaLabel={t('places.merge.targetSelectLabel')}
            value={targetId}
            onChange={(val) => setTargetId(val)}
            options={candidateTargets.map((p) => ({
              value: p.id,
              label: `${p.full_name} (${p.gid})`,
            }))}
          />
        </FormField>

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={merging}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={merging}
            disabled={merging || !targetId}
            leftIcon={<GitMerge className="h-4 w-4" />}
          >
            {t('places.merge.confirmBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
