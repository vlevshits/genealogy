import React, { useEffect, useState } from 'react';
import { Eye, Save } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  MarkdownView,
  Switch,
  Textarea,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  createNote,
  NoteDto,
  updateNote,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface NoteEditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entityType?: 'individual' | 'family' | 'event' | 'place' | 'media';
  entityId: string;
  initialNote?: NoteDto | null;
  onNoteSaved?: (saved: NoteDto) => void;
}

export const NoteEditDialog: React.FC<NoteEditDialogProps> = ({
  open,
  onOpenChange,
  entityType = 'individual',
  entityId,
  initialNote = null,
  onNoteSaved,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [body, setBody] = useState<string>(initialNote?.body ?? '');
  const [isPrivate, setIsPrivate] = useState<boolean>(
    Boolean(initialNote?.is_private)
  );
  const [previewMode, setPreviewMode] = useState<boolean>(false);
  const [saving, setSaving] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    setBody(initialNote?.body ?? '');
    setIsPrivate(Boolean(initialNote?.is_private));
    setPreviewMode(false);
    setErrorMsg(null);
  }, [initialNote, open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!body.trim() || saving) return;
    setSaving(true);
    setErrorMsg(null);
    try {
      const saved = initialNote
        ? await updateNote(initialNote.id, {
            body: body.trim(),
            is_private: isPrivate,
            version: initialNote.version ?? 1,
          })
        : await createNote({
            entity_type: entityType,
            entity_id: entityId,
            body: body.trim(),
            is_private: isPrivate,
          });

      addToast({
        type: 'success',
        title: t('profile.notes.savedToast'),
      });
      onNoteSaved?.(saved);
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(
        err instanceof Error ? err.message : t('common.errorLoading')
      );
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={
        initialNote
          ? t('profile.notes.editDialogTitle')
          : t('profile.notes.addDialogTitle')
      }
      description={t('profile.notes.dialogSubtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-secondary">
            {t('profile.biography.markdownHint')}
          </span>
          <Button
            size="sm"
            variant="ghost"
            leftIcon={<Eye className="h-3.5 w-3.5" />}
            onClick={() => setPreviewMode((p) => !p)}
          >
            {previewMode
              ? t('profile.biography.editModeBtn')
              : t('profile.biography.previewBtn')}
          </Button>
        </div>

        {previewMode ? (
          <div className="min-h-36 rounded-xl border border-border bg-surface-muted/40 p-4">
            <MarkdownView
              content={body.trim() || t('profile.notes.emptyPreview')}
            />
          </div>
        ) : (
          <FormField
            name="note_body"
            label={t('profile.notes.bodyLabel')}
            required
          >
            <Textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              rows={6}
              placeholder={t('profile.notes.bodyPlaceholder')}
              required
            />
          </FormField>
        )}

        <Switch
          checked={isPrivate}
          onCheckedChange={setIsPrivate}
          label={t('profile.notes.privateToggleLabel')}
        />

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={saving}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={saving || !body.trim()}
            leftIcon={<Save className="h-4 w-4" />}
          >
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
