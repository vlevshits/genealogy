import React, { useState } from 'react';
import { Upload } from 'lucide-react';
import {
  ApiError,
  createMediaAsset,
  MediaAssetSummaryDto,
} from '../../api/client';
import {
  Alert,
  Button,
  Dialog,
  FileDropzone,
  ProgressBar,
  Switch,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import { useI18n } from '../../i18n';

export interface MediaUploadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultLinkedPersonId?: string;
  onUploaded?: (created: MediaAssetSummaryDto[]) => void;
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve) => {
    if (typeof FileReader === 'undefined') {
      resolve('');
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      resolve(typeof reader.result === 'string' ? reader.result : '');
    };
    reader.onerror = () => resolve('');
    reader.readAsDataURL(file);
  });
}

export const MediaUploadDialog: React.FC<MediaUploadDialogProps> = ({
  open,
  onOpenChange,
  defaultLinkedPersonId,
  onUploaded,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [defaultTitle, setDefaultTitle] = useState<string>('');
  const [dateTaken, setDateTaken] = useState<string>('');
  const [placePath, setPlacePath] = useState<string>('');
  const [isPrivate, setIsPrivate] = useState<boolean>(false);
  const [uploading, setUploading] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleReset = () => {
    setSelectedFiles([]);
    setDefaultTitle('');
    setDateTaken('');
    setPlacePath('');
    setIsPrivate(false);
    setUploading(false);
    setProgress(0);
    setErrorMsg(null);
  };

  const handleUploadBatch = async (filesToUpload: File[]) => {
    const batch = filesToUpload.length > 0 ? filesToUpload : selectedFiles;
    if (batch.length === 0 || uploading) return;

    setUploading(true);
    setProgress(10);
    setErrorMsg(null);

    const createdAssets: MediaAssetSummaryDto[] = [];
    try {
      for (let i = 0; i < batch.length; i += 1) {
        const file = batch[i];
        const dataUrl = await readFileAsDataUrl(file);
        const titleForAsset =
          batch.length === 1 && defaultTitle.trim()
            ? defaultTitle.trim()
            : file.name.replace(/\.[^.]+$/, '');

        const created = await createMediaAsset({
          file_name: file.name,
          mime_type: file.type || 'image/webp',
          data: dataUrl || undefined,
          title: titleForAsset,
          date_taken: dateTaken.trim() || null,
          place_path: placePath.trim() || null,
          is_private: isPrivate,
          linked_person_ids: defaultLinkedPersonId
            ? [defaultLinkedPersonId]
            : undefined,
        });
        createdAssets.push(created);
        setProgress(Math.round(((i + 1) / batch.length) * 100));
      }

      addToast({
        type: 'success',
        title: t('mediaArchive.uploadSuccessToast', {
          count: createdAssets.length,
        }),
      });
      onUploaded?.(createdAssets);
      handleReset();
      onOpenChange(false);
    } catch (err) {
      if (err instanceof ApiError) {
        setErrorMsg(err.message);
      } else {
        setErrorMsg(
          err instanceof Error ? err.message : t('common.errorLoading')
        );
      }
    } finally {
      setUploading(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) handleReset();
        onOpenChange(nextOpen);
      }}
      size="xl"
      title={t('mediaArchive.uploadDialogTitle')}
      description={t('mediaArchive.uploadDialogSubtitle')}
    >
      <div className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <FileDropzone
          accept=".jpg,.jpeg,.png,.webp,.gif,.tif,.tiff,.pdf,.mp3,.wav"
          maxSizeMb={25}
          maxFiles={50}
          multiple
          onUpload={(files) => {
            setSelectedFiles(files);
          }}
        />

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <FormField
            name="default_title"
            label={t('mediaArchive.fieldTitle')}
          >
            <TextInput
              value={defaultTitle}
              onChange={(e) => setDefaultTitle(e.target.value)}
              placeholder={t('mediaArchive.fieldTitlePlaceholder')}
            />
          </FormField>

          <FormField
            name="date_taken"
            label={t('mediaArchive.fieldDateTaken')}
          >
            <TextInput
              value={dateTaken}
              onChange={(e) => setDateTaken(e.target.value)}
              placeholder={t('mediaArchive.fieldDatePlaceholder')}
            />
          </FormField>

          <FormField
            name="place_path"
            label={t('mediaArchive.fieldPlace')}
          >
            <TextInput
              value={placePath}
              onChange={(e) => setPlacePath(e.target.value)}
              placeholder={t('mediaArchive.fieldPlacePlaceholder')}
            />
          </FormField>
        </div>

        <Switch
          checked={isPrivate}
          onCheckedChange={setIsPrivate}
          label={t('mediaArchive.privateToggleLabel')}
        />

        {uploading && (
          <ProgressBar
            value={progress}
            label={t('mediaArchive.uploadingProgress')}
          />
        )}

        <div className="flex items-center justify-end gap-2 pt-2 border-t border-border">
          <Button
            variant="secondary"
            onClick={() => {
              handleReset();
              onOpenChange(false);
            }}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            disabled={selectedFiles.length === 0 || uploading}
            loading={uploading}
            onClick={() => {
              void handleUploadBatch(selectedFiles);
            }}
            leftIcon={<Upload className="h-4 w-4" />}
          >
            {t('mediaArchive.submitUploadBtn', {
              count: selectedFiles.length,
            })}
          </Button>
        </div>
      </div>
    </Dialog>
  );
};
