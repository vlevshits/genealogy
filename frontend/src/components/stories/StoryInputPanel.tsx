import React, { useState } from 'react';
import {
  Cpu,
  FileAudio,
  FileText,
  Mic,
  RotateCcw,
  Sparkles,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  FileDropzone,
  SegmentedControl,
  Select,
  Textarea,
} from '../ui';
import { useI18n } from '../../i18n';
import { AudioRecorderWidget } from './AudioRecorderWidget';
import { StoryExtractionEngine } from '../../api/gemini-story-extractor';

export type StoryInputMethod = 'text' | 'audio_record' | 'audio_upload';

export interface StoryInputPanelProps {
  narrativeText?: string;
  onNarrativeTextChange: (text: string) => void;
  language: string;
  onLanguageChange: (lang: string) => void;
  extracting?: boolean;
  transcribing?: boolean;
  onExtractSubtree: () => void;
  onTranscribeBlob: (blob: Blob | File, speechTranscript?: string) => void;
  engine?: StoryExtractionEngine;
  onEngineChange?: (engine: StoryExtractionEngine) => void;
}

export const StoryInputPanel: React.FC<StoryInputPanelProps> = ({
  narrativeText = '',
  onNarrativeTextChange,
  language,
  onLanguageChange,
  extracting = false,
  transcribing = false,
  onExtractSubtree,
  onTranscribeBlob,
  engine = 'gemini',
  onEngineChange,
}) => {
  const { t } = useI18n();
  const safeText = narrativeText ?? '';
  const [inputMode, setInputMode] = useState<StoryInputMethod>('text');

  const handleLoadSample = (templateKey: 'default' | 'parents' | 'marriage') => {
    if (templateKey === 'parents') {
      onNarrativeTextChange(t('stories.input.sampleParentsText'));
    } else if (templateKey === 'marriage') {
      onNarrativeTextChange(t('stories.input.sampleMarriageText'));
    } else {
      onNarrativeTextChange(t('stories.input.sampleDefaultText'));
    }
  };

  return (
    <Card
      data-testid="story-input-panel"
      className="p-4 sm:p-5 space-y-4"
    >
      {/* Top Bar: Input Method Tabs + Sample Templates */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="max-w-full overflow-x-auto">
          <SegmentedControl<StoryInputMethod>
            size="sm"
            ariaLabel={t('stories.input.methodAria')}
            value={inputMode}
            onChange={setInputMode}
            options={[
              {
                value: 'text',
                label: t('stories.input.tabText'),
                icon: <FileText className="h-3.5 w-3.5" />,
              },
              {
                value: 'audio_record',
                label: t('stories.input.tabRecord'),
                icon: <Mic className="h-3.5 w-3.5" />,
              },
              {
                value: 'audio_upload',
                label: t('stories.input.tabUpload'),
                icon: <FileAudio className="h-3.5 w-3.5" />,
              },
            ]}
          />
        </div>

        {inputMode === 'text' && (
          <div className="flex flex-wrap items-center gap-1.5">
            <Button
              size="sm"
              variant="ghost"
              data-testid="load-sample-story-btn"
              onClick={() => handleLoadSample('default')}
            >
              {t('stories.input.loadSampleBtn')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => handleLoadSample('parents')}
            >
              {t('stories.input.templateParentsBtn')}
            </Button>
            {safeText.length > 0 && (
              <Button
                size="sm"
                variant="ghost"
                leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
                onClick={() => onNarrativeTextChange('')}
              >
                {t('common.clear')}
              </Button>
            )}
          </div>
        )}
      </div>

      {/* Mode 1: Type Narrative */}
      {inputMode === 'text' && (
        <div className="space-y-2">
          <Textarea
            data-testid="story-narrative-textarea"
            value={safeText}
            onChange={(e) => onNarrativeTextChange(e.target.value)}
            placeholder={t('stories.input.placeholder')}
            aria-label={t('stories.input.textareaAria')}
            rows={5}
            className="min-h-[150px] max-h-[380px]"
          />
          <div className="flex items-center justify-between text-xs text-tertiary">
            <span>
              {t('stories.input.charCount', {
                count: safeText.length,
              })}
            </span>
            <span>{t('stories.input.privacyHint')}</span>
          </div>
        </div>
      )}

      {/* Mode 2: In-Browser Voice Recording */}
      {inputMode === 'audio_record' && (
        <AudioRecorderWidget
          transcribing={transcribing}
          language={language}
          onTranscribe={(blob, speechTranscript) => {
            onTranscribeBlob(blob, speechTranscript);
            setInputMode('text');
          }}
        />
      )}

      {/* Mode 3: Upload Audio File */}
      {inputMode === 'audio_upload' && (
        <div
          data-testid="story-audio-upload-zone"
          className="space-y-3"
        >
          <p className="text-xs text-secondary">
            {t('stories.input.uploadFormatsHint')}
          </p>
          <FileDropzone
            accept=".webm,.mp3,.wav,.m4a"
            maxSizeMb={25}
            maxFiles={1}
            multiple={false}
            onUpload={(files) => {
              if (files[0]) {
                onTranscribeBlob(files[0]);
                setInputMode('text');
              }
            }}
          />
        </div>
      )}

      {/* Extraction Engine & Model Selection Toolbar */}
      <div
        data-testid="story-engine-selector-bar"
        className="flex flex-wrap items-center justify-between gap-3 p-2.5 rounded-xl bg-surface-muted border border-border"
      >
        <div className="flex flex-wrap items-center gap-2.5">
          <span className="text-xs font-medium text-secondary whitespace-nowrap">
            {t('stories.engine.label')}:
          </span>
          <SegmentedControl<StoryExtractionEngine>
            size="sm"
            ariaLabel={t('stories.engine.label')}
            value={engine}
            onChange={(val) => onEngineChange?.(val)}
            options={[
              {
                value: 'gemini',
                label: t('stories.engine.gemini'),
                icon: <Sparkles className="h-3.5 w-3.5 text-brand" />,
              },
              {
                value: 'builtin',
                label: t('stories.engine.builtin'),
                icon: <Cpu className="h-3.5 w-3.5" />,
              },
            ]}
          />
          {engine === 'gemini' && (
            <Badge variant="brand" size="sm" className="font-mono text-[11px]">
              {t('stories.engine.modelBadge')}
            </Badge>
          )}
        </div>
      </div>

      {/* Bottom Action Bar: Language Selector + Extract Button */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2 border-t border-border">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-secondary whitespace-nowrap">
            {t('stories.input.languageLabel')}:
          </span>
          <div className="w-48">
            <Select
              size="sm"
              value={language}
              onChange={onLanguageChange}
              ariaLabel={t('stories.input.languageLabel')}
              options={[
                { value: 'auto', label: t('stories.input.langAuto') },
                { value: 'ru', label: t('stories.input.langRu') },
                { value: 'en', label: t('stories.input.langEn') },
              ]}
            />
          </div>
        </div>

        <Button
          variant="primary"
          size="md"
          loading={extracting}
          disabled={!narrativeText.trim()}
          data-testid="extract-draft-subtree-btn"
          leftIcon={<Sparkles className="h-4 w-4" />}
          onClick={onExtractSubtree}
        >
          {t('stories.input.extractBtn')}
        </Button>
      </div>
    </Card>
  );
};
