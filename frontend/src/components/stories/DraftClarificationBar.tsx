import React, { useState } from 'react';
import { MessageSquarePlus, Send } from 'lucide-react';
import { Button, Card, TextInput } from '../ui';
import { useI18n } from '../../i18n';

export interface DraftClarificationBarProps {
  refining?: boolean;
  onSubmitClarification: (text: string) => void;
}

export const DraftClarificationBar: React.FC<DraftClarificationBarProps> = ({
  refining = false,
  onSubmitClarification,
}) => {
  const { t } = useI18n();
  const [text, setText] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!text.trim() || refining) return;
    onSubmitClarification(text.trim());
    setText('');
  };

  return (
    <Card
      data-testid="draft-clarification-bar"
      className="p-4 sm:p-5 space-y-2.5"
    >
      <div className="flex items-center gap-2">
        <MessageSquarePlus className="h-4 w-4 text-brand" />
        <h3 className="text-sm font-semibold text-primary">
          {t('stories.clarification.title')}
        </h3>
      </div>
      <p className="text-xs text-secondary">
        {t('stories.clarification.subtitle')}
      </p>

      <form
        onSubmit={handleSubmit}
        className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 pt-1"
      >
        <div className="flex-1 min-w-0">
          <TextInput
            data-testid="draft-clarification-input"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t('stories.clarification.placeholder')}
            aria-label={t('stories.clarification.title')}
          />
        </div>
        <Button
          type="submit"
          variant="primary"
          size="md"
          loading={refining}
          disabled={!text.trim()}
          data-testid="draft-clarification-submit-btn"
          leftIcon={<Send className="h-4 w-4" />}
          className="shrink-0"
        >
          {t('stories.clarification.sendBtn')}
        </Button>
      </form>
    </Card>
  );
};
