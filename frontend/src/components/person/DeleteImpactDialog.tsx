import React, { useEffect, useState } from 'react';
import { AlertTriangle, Trash2 } from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Dialog,
  useToast,
} from '../ui';
import {
  DeleteImpactDto,
  deletePerson,
  fetchPersonDeleteImpact,
  TreePersonNodeDto,
} from '../../api/client';
import { useUndoToast } from '../../hooks/useUndoToast';
import { useI18n } from '../../i18n';

export interface DeleteImpactDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  person: TreePersonNodeDto | null;
  initialImpact?: DeleteImpactDto | null;
  onPersonDeleted?: (deletedPersonId: string) => void;
}

export const DeleteImpactDialog: React.FC<DeleteImpactDialogProps> = ({
  open,
  onOpenChange,
  person,
  initialImpact,
  onPersonDeleted,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();
  const { showUndoToast } = useUndoToast();

  const fallbackImpact: DeleteImpactDto | null = person
    ? {
        person_id: person.id,
        gid: person.gid,
        display_name: [
          person.primary_name.given_name,
          person.primary_name.patronymic,
          person.primary_name.surname,
        ]
          .filter(Boolean)
          .join(' '),
        events_count: (person.birth ? 1 : 0) + (person.death ? 1 : 0),
        families_unlinked_count: 1,
        empty_families_deleted_count: 0,
        notes_count: person.notes ? 1 : 0,
        media_links_count: person.avatar_url ? 1 : 0,
      }
    : null;

  const [impact, setImpact] = useState<DeleteImpactDto | null>(
    initialImpact ?? fallbackImpact
  );
  const [deleting, setDeleting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (initialImpact) {
      setImpact(initialImpact);
      return;
    }
    if (!person || !open) return;
    setImpact(fallbackImpact);
    let cancelled = false;
    fetchPersonDeleteImpact(person.id)
      .then((res) => {
        if (!cancelled) setImpact(res);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [person, open, initialImpact]);

  if (!person || !impact) return null;

  const handleConfirmDelete = async () => {
    if (deleting) return;
    setDeleting(true);
    setErrorMsg(null);
    try {
      const res = await deletePerson(person.id);
      if (res?.tx_id) {
        showUndoToast({
          txId: res.tx_id,
          title: t('history.undo_toast_label'),
          description: t('person.deleteImpact.deletedToast'),
        });
      } else {
        addToast({
          type: 'success',
          title: t('person.deleteImpact.deletedToast'),
          description: `${impact.display_name} (${impact.gid})`,
        });
      }
      onPersonDeleted?.(person.id);
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(
        err instanceof Error ? err.message : t('common.errorLoading')
      );
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={t('person.deleteImpact.title')}
      description={t('person.deleteImpact.subtitle', {
        name: impact.display_name,
        gid: impact.gid,
      })}
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between rounded-xl border border-border bg-surface-muted/50 px-3.5 py-2.5 text-xs">
          <span className="font-semibold text-primary">
            {impact.display_name}
          </span>
          <Badge variant="neutral" size="sm">
            {impact.gid}
          </Badge>
        </div>

        <Alert
          variant="warning"
          icon={
            <AlertTriangle className="h-4 w-4 text-warning shrink-0 mt-0.5" />
          }
          title={t('person.deleteImpact.warningTitle')}
        >
          <ul className="list-disc pl-4 space-y-1 text-xs text-secondary">
            <li>
              {t('person.deleteImpact.eventsImpact', {
                count: impact.events_count,
              })}
            </li>
            <li>
              {t('person.deleteImpact.familiesImpact', {
                count: impact.families_unlinked_count,
                empty: impact.empty_families_deleted_count,
              })}
            </li>
            <li>
              {t('person.deleteImpact.mediaImpact', {
                count: impact.media_links_count,
              })}
            </li>
            <li>
              {t('person.deleteImpact.notesImpact', {
                count: impact.notes_count,
              })}
            </li>
          </ul>
        </Alert>

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={deleting}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="danger"
            loading={deleting}
            leftIcon={<Trash2 className="h-4 w-4" />}
            onClick={handleConfirmDelete}
          >
            {t('person.deleteImpact.confirmBtn')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
};
