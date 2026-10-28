import React, { useEffect, useState } from 'react';
import { MapPin, Network, Save } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  SegmentedControl,
  Select,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  createPlace,
  PlaceDto,
  PlaceType,
  updatePlace,
} from '../../api/client';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';

export interface PlaceFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialPlace?: PlaceDto | null;
  availableParents?: PlaceDto[];
  onPlaceSaved?: (saved: PlaceDto) => void;
}

export const PlaceFormDialog: React.FC<PlaceFormDialogProps> = ({
  open,
  onOpenChange,
  initialPlace,
  availableParents = [],
  onPlaceSaved,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [mode, setMode] = useState<'chain' | 'single'>(
    initialPlace ? 'single' : 'chain'
  );
  const [chainPath, setChainPath] = useState<string>('');
  const [name, setName] = useState<string>(initialPlace?.name ?? '');
  const [placeType, setPlaceType] = useState<PlaceType>(
    initialPlace?.place_type ?? 'city'
  );
  const [parentId, setParentId] = useState<string>(
    initialPlace?.parent_id ?? ''
  );
  const [latStr, setLatStr] = useState<string>(
    initialPlace?.latitude !== null && initialPlace?.latitude !== undefined
      ? String(initialPlace.latitude)
      : ''
  );
  const [lngStr, setLngStr] = useState<string>(
    initialPlace?.longitude !== null && initialPlace?.longitude !== undefined
      ? String(initialPlace.longitude)
      : ''
  );
  const [altNamesRaw, setAltNamesRaw] = useState<string>(
    initialPlace?.alt_names?.map((a) => a.name).join(', ') ?? ''
  );
  const [saving, setSaving] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (initialPlace) {
      setMode('single');
      setName(initialPlace.name);
      setPlaceType(initialPlace.place_type);
      setParentId(initialPlace.parent_id ?? '');
      setLatStr(
        initialPlace.latitude !== null && initialPlace.latitude !== undefined
          ? String(initialPlace.latitude)
          : ''
      );
      setLngStr(
        initialPlace.longitude !== null && initialPlace.longitude !== undefined
          ? String(initialPlace.longitude)
          : ''
      );
      setAltNamesRaw(
        initialPlace.alt_names?.map((a) => a.name).join(', ') ?? ''
      );
    } else {
      setMode('chain');
      setChainPath('');
      setName('');
      setPlaceType('city');
      setParentId('');
      setLatStr('');
      setLngStr('');
      setAltNamesRaw('');
    }
    setErrorMsg(null);
  }, [initialPlace, open]);

  const placeTypeOptions = [
    { value: 'country', label: t('places.types.country') },
    { value: 'region', label: t('places.types.region') },
    { value: 'district', label: t('places.types.district') },
    { value: 'city', label: t('places.types.city') },
    { value: 'village', label: t('places.types.village') },
    { value: 'parish', label: t('places.types.parish') },
    { value: 'cemetery', label: t('places.types.cemetery') },
    { value: 'address', label: t('places.types.address') },
    { value: 'other', label: t('places.types.other') },
  ];

  const parentOptions = [
    { value: '', label: t('places.form.noParentOption') },
    ...availableParents
      .filter((p) => p.id !== initialPlace?.id)
      .map((p) => ({
        value: p.id,
        label: p.full_name,
      })),
  ];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    setErrorMsg(null);
    try {
      const parsedAltNames = altNamesRaw
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
        .map((n) => ({ name: n }));

      const lat = latStr.trim() ? Number(latStr.trim()) : null;
      const lng = lngStr.trim() ? Number(lngStr.trim()) : null;

      const saved = initialPlace
        ? await updatePlace(initialPlace.id, {
            name: name.trim(),
            place_type: placeType,
            parent_id: parentId || null,
            latitude: Number.isFinite(lat) ? lat : null,
            longitude: Number.isFinite(lng) ? lng : null,
            alt_names: parsedAltNames,
            version: initialPlace.version ?? 1,
          })
        : mode === 'chain'
        ? await createPlace({
            path: chainPath.trim(),
            latitude: Number.isFinite(lat) ? lat : null,
            longitude: Number.isFinite(lng) ? lng : null,
            alt_names: parsedAltNames,
          })
        : await createPlace({
            name: name.trim(),
            place_type: placeType,
            parent_id: parentId || null,
            latitude: Number.isFinite(lat) ? lat : null,
            longitude: Number.isFinite(lng) ? lng : null,
            alt_names: parsedAltNames,
          });

      addToast({
        type: 'success',
        title: t('places.form.savedToast'),
        description: saved.full_name,
      });
      onPlaceSaved?.(saved);
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(getLocalizedErrorMessage(err, t));
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
        initialPlace ? t('places.form.editTitle') : t('places.form.addTitle')
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {!initialPlace && (
          <SegmentedControl
            ariaLabel={t('places.form.addTitle')}
            value={mode}
            onChange={(val) => setMode(val as 'chain' | 'single')}
            options={[
              {
                value: 'chain',
                label: t('places.form.modeChain'),
                icon: <Network className="h-3.5 w-3.5" />,
              },
              {
                value: 'single',
                label: t('places.form.modeSingle'),
                icon: <MapPin className="h-3.5 w-3.5" />,
              },
            ]}
          />
        )}

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        {!initialPlace && mode === 'chain' ? (
          <FormField
            name="chain_path"
            label={t('places.form.chainPathLabel')}
            hint={t('places.form.chainPathHint')}
            required
          >
            <TextInput
              value={chainPath}
              onChange={(e) => setChainPath(e.target.value)}
              placeholder={t('places.form.chainPathPlaceholder')}
              required
            />
          </FormField>
        ) : (
          <div className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <FormField
                name="place_name"
                label={t('places.form.nameLabel')}
                required
              >
                <TextInput
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                />
              </FormField>

              <FormField
                name="place_type"
                label={t('places.form.typeLabel')}
              >
                <Select
                  ariaLabel={t('places.form.typeLabel')}
                  value={placeType}
                  onChange={(val) => setPlaceType(val as PlaceType)}
                  options={placeTypeOptions}
                />
              </FormField>
            </div>

            <FormField
              name="place_parent"
              label={t('places.form.parentLabel')}
            >
              <Select
                ariaLabel={t('places.form.parentLabel')}
                value={parentId}
                onChange={(val) => setParentId(val)}
                options={parentOptions}
              />
            </FormField>
          </div>
        )}

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <FormField
            name="place_lat"
            label={t('places.form.latLabel')}
            optionalText
          >
            <TextInput
              value={latStr}
              onChange={(e) => setLatStr(e.target.value)}
              placeholder="56.8587"
            />
          </FormField>

          <FormField
            name="place_lng"
            label={t('places.form.lngLabel')}
            optionalText
          >
            <TextInput
              value={lngStr}
              onChange={(e) => setLngStr(e.target.value)}
              placeholder="35.9176"
            />
          </FormField>
        </div>

        <FormField
          name="place_alt_names"
          label={t('places.form.altNamesLabel')}
          optionalText
        >
          <TextInput
            value={altNamesRaw}
            onChange={(e) => setAltNamesRaw(e.target.value)}
            placeholder={t('places.form.altNamesPlaceholder')}
          />
        </FormField>

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
            disabled={
              saving ||
              (!initialPlace && mode === 'chain'
                ? !chainPath.trim()
                : !name.trim())
            }
            leftIcon={<Save className="h-4 w-4" />}
          >
            {t('places.form.saveBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
