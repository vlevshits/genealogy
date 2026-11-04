import React, { useState } from 'react';
import {
  Calendar,
  Download,
  GitMerge,
  Layers,
  MapPin,
  Minus,
  MoreHorizontal,
  Plus,
  Sparkles,
  Trash2,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Banner,
  Button,
  ButtonGroup,
  CheckboxGroup,
  Combobox,
  ComboboxOption,
  CopyButton,
  Divider,
  EmailInput,
  EmptyState,
  IconButton,
  Kbd,
  MultiSelect,
  PageHeader,
  PasswordInput,
  ProgressBar,
  RadioCardGroup,
  RangeSlider,
  SearchInput,
  SegmentedControl,
  Select,
  SideNav,
  Skeleton,
  Slider,
  Spinner,
  TextInput,
  useToast,
  YearInput,
} from '../components/ui';
import {
  GenDateInput,
  MarkdownEditor,
} from '../components/form';
import {
  EventForm,
} from '../components/domain';
import {
  SAMPLE_PLACES,
} from '../lib/mock-data';
import { parseGenDate } from '../lib/gendate-parser';
import { GenDate } from '../types/gendate';
import { Locale, ThemeMode } from '../types/common';
import { useI18n } from '../i18n';
import { useTheme } from '../lib/theme';
import { WorkbenchAdvancedSections } from './WorkbenchAdvancedSections';

const GENDATE_PRESETS: string[] = [
  '14 Jul 1923',
  '14.07.1923',
  '14 июля 1923 г.',
  'about 1890',
  'ок. 1890',
  'before 1917',
  'до мая 1860',
  'after 1945',
  'после 1945 г.',
  'between 1914 and 1918',
  'между 1914 и 1918 гг.',
  'calculated 1905',
  'вычислено 1905 г.',
  '14.07.1905 (ст. ст.)',
  'в детстве во время войны',
];

export const DevUiWorkbench: React.FC = () => {
  const { t, locale, setLocale } = useI18n();
  const { theme, setTheme } = useTheme();
  const { addToast } = useToast();

  const [activeSection, setActiveSection] = useState<string>('actions');

  // Interactive harness states
  const [segmentedMode, setSegmentedMode] = useState<'hourglass' | 'pedigree' | 'fan'>('hourglass');
  const [pickerSimMode, setPickerSimMode] = useState<'normal' | 'loading' | 'error' | 'empty'>('normal');
  const [multiSelectValues, setMultiSelectValues] = useState<string[]>(['Левшиц', 'Кузнецов']);
  const [sliderValue, setSliderValue] = useState<number>(12);
  const [yearRange, setYearRange] = useState<[number, number]>([1860, 1950]);
  const [yearSingle, setYearSingle] = useState<number | null>(1912);
  const [markdownBio, setMarkdownBio] = useState<string>(
    '## Архивная биография\nИнженер-мостостроитель **Александр Петрович Левшиц** (1912–1989).\n- Окончил *ЛИИЖТ* в 1935 году\n- Награждён [Орденом Красной Звезды](https://pamyat-naroda.ru)'
  );

  // GenDateInput harness state
  const [genDateValue, setGenDateValue] = useState<GenDate>(() =>
    parseGenDate('14.07.1905 (ст. ст.)', 'ru').value
  );

  // EventForm server error / warning simulation
  const [simServerErrors, setSimServerErrors] = useState<Record<string, string> | undefined>(
    undefined
  );
  const [simSoftWarning, setSimSoftWarning] = useState<string | null | undefined>(undefined);

  const simulatedAsyncLoader = async (
    query: string,
    _signal: AbortSignal
  ): Promise<ComboboxOption[]> => {
    await new Promise((r) => setTimeout(r, 180));
    if (pickerSimMode === 'loading') {
      await new Promise((r) => setTimeout(r, 10000));
    }
    if (pickerSimMode === 'error') {
      throw new Error(t('common.errorLoading'));
    }
    if (pickerSimMode === 'empty') {
      return [];
    }
    return SAMPLE_PLACES.filter((p) =>
      p.name.toLowerCase().includes(query.toLowerCase())
    ).map((p) => ({
      value: p.id,
      label: p.name,
      description: p.hierarchy_names?.join(', '),
    }));
  };

  return (
    <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8 space-y-8">
      {/* Workbench Header + Instant Theme & Locale Switchers */}
      <PageHeader
        title={t('workbench.title')}
        subtitle={t('workbench.subtitle')}
        secondaryActions={
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <span className="text-xs font-medium text-secondary">
                {t('workbench.localeLabel')}:
              </span>
              <SegmentedControl<Locale>
                size="sm"
                ariaLabel={t('workbench.localeLabel')}
                value={locale}
                onChange={setLocale}
                options={[
                  { value: 'en', label: 'EN' },
                  { value: 'ru', label: 'RU' },
                ]}
              />
            </div>

            <div className="flex items-center gap-2">
              <span className="text-xs font-medium text-secondary">
                {t('workbench.themeLabel')}:
              </span>
              <SegmentedControl<ThemeMode>
                size="sm"
                ariaLabel={t('workbench.themeLabel')}
                value={theme}
                onChange={setTheme}
                options={[
                  { value: 'light', label: t('common.lightMode') },
                  { value: 'dark', label: t('common.darkMode') },
                  { value: 'system', label: t('common.systemMode') },
                ]}
              />
            </div>
          </div>
        }
      />

      {/* Workbench Layout: Left SideNav + Main Showcase Column */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        <div className="lg:col-span-3 lg:sticky lg:top-20">
          <SideNav
            value={activeSection}
            onChange={(val) => {
              setActiveSection(val);
              const el = document.getElementById(`wb-${val}`);
              el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }}
            items={[
              { value: 'actions', label: t('workbench.navActions'), badge: 5 },
              { value: 'feedback', label: t('workbench.navFeedback'), badge: 9 },
              { value: 'inputs', label: t('workbench.navInputs'), badge: 12 },
              { value: 'gendate', label: t('workbench.navGenDate'), badge: '15 cases' },
              { value: 'form', label: t('workbench.navForm'), badge: 6 },
              { value: 'overlays', label: t('workbench.navOverlays'), badge: 8 },
              { value: 'navigation', label: t('workbench.navNavigation'), badge: 10 },
              { value: 'datatable', label: t('workbench.navDataTable'), badge: '50k' },
              { value: 'comparison', label: t('workbench.navComparison'), badge: 10 },
              { value: 'domain', label: t('workbench.navDomain'), badge: 13 },
            ]}
          />
        </div>

        <div className="lg:col-span-9 space-y-10">
          {/* 1. Actions & Buttons */}
          <section id="wb-actions" className="rounded-2xl border border-border bg-surface p-6 space-y-5">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navActions')}
            </h2>

            <div className="space-y-3">
              <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                Button Variants & Sizes
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <Button variant="primary" leftIcon={<Plus className="h-4 w-4" />}>
                  Primary CTA
                </Button>
                <Button variant="secondary" leftIcon={<Download className="h-4 w-4" />}>
                  Secondary
                </Button>
                <Button variant="ghost">Ghost</Button>
                <Button variant="danger" leftIcon={<Trash2 className="h-4 w-4" />}>
                  Danger
                </Button>
                <Button variant="link">Link Action</Button>
                <Button variant="primary" loading>
                  {t('common.saving')}
                </Button>
                <Button variant="secondary" disabled>
                  Disabled
                </Button>
              </div>

              <div className="flex flex-wrap items-center gap-3 pt-2">
                <Button size="sm" variant="secondary">
                  Size sm (32px)
                </Button>
                <Button size="md" variant="secondary">
                  Size md (40px)
                </Button>
                <Button size="lg" variant="secondary">
                  Size lg (48px)
                </Button>
              </div>
            </div>

            <Divider />

            <div className="flex flex-wrap items-center gap-6">
              <div className="space-y-1.5">
                <div className="text-xs font-medium text-secondary">
                  IconButton & ButtonGroup
                </div>
                <div className="flex items-center gap-3">
                  <ButtonGroup>
                    <IconButton size="sm" label="Zoom in">
                      <Plus className="h-4 w-4" />
                    </IconButton>
                    <span className="px-2.5 font-mono text-xs text-secondary">100%</span>
                    <IconButton size="sm" label="Zoom out">
                      <Minus className="h-4 w-4" />
                    </IconButton>
                  </ButtonGroup>
                  <IconButton variant="secondary" label="More options">
                    <MoreHorizontal className="h-4 w-4" />
                  </IconButton>
                  <IconButton variant="floating" label="Center canvas">
                    <Sparkles className="h-4 w-4" />
                  </IconButton>
                </div>
              </div>

              <div className="space-y-1.5">
                <div className="text-xs font-medium text-secondary">SegmentedControl</div>
                <SegmentedControl
                  ariaLabel="Tree view mode"
                  value={segmentedMode}
                  onChange={setSegmentedMode}
                  options={[
                    { value: 'hourglass', label: 'Hourglass' },
                    { value: 'pedigree', label: 'Pedigree' },
                    { value: 'fan', label: 'Fan Chart' },
                  ]}
                />
              </div>

              <div className="space-y-1.5">
                <div className="text-xs font-medium text-secondary">CopyButton</div>
                <CopyButton value="I0001 — Levshits Family Archive" />
              </div>
            </div>
          </section>

          {/* 2. Badges, Status & Feedback */}
          <section id="wb-feedback" className="rounded-2xl border border-border bg-surface p-6 space-y-5">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navFeedback')}
            </h2>

            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="neutral">Neutral</Badge>
              <Badge variant="brand">Editor Role</Badge>
              <Badge variant="accent">Archival Milestone</Badge>
              <Badge variant="success">Verified Source</Badge>
              <Badge variant="warning">Probable Match 84%</Badge>
              <Badge variant="danger">Merge Conflict</Badge>
              <Badge variant="outline">GEDCOM 7.0</Badge>
              <Kbd>⌘K</Kbd>
              <Kbd>Shift + ↵</Kbd>
              <Spinner size="sm" />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <ProgressBar
                value={74}
                label="GEDCOM 7.0 Import"
                stepText="Indexing 1,420 individuals…"
                variant="brand"
              />
              <div className="space-y-2">
                <div className="text-xs font-medium text-secondary">Skeleton Placeholders</div>
                <div className="flex items-center gap-3">
                  <Skeleton variant="circle" className="h-10 w-10" />
                  <div className="flex-1 space-y-2">
                    <Skeleton variant="text" className="w-2/3" />
                    <Skeleton variant="text" className="w-1/2" />
                  </div>
                </div>
              </div>
            </div>

            <div className="space-y-3">
              <Alert variant="info" title="Passkey Security Available">
                Register a hardware or device passkey for one-tap passwordless sign-in.
              </Alert>
              <Alert variant="warning" title="Soft Chronology Warning">
                {t('domain.eventForm.softWarningChronology')}
              </Alert>
            </div>

            <Banner variant="info" dismissible>
              Public guest mode preview — living people are automatically redacted.
            </Banner>

            <EmptyState
              icon={<MapPin className="h-6 w-6" />}
              title="No places recorded in this region yet"
              description="Add historical parishes, cities, or guberniyas to visualize family migration paths."
              action={
                <Button
                  size="sm"
                  variant="primary"
                  leftIcon={<Plus className="h-4 w-4" />}
                >
                  Add Place
                </Button>
              }
            />
          </section>

          {/* 3. Inputs, Pickers & Sliders */}
          <section id="wb-inputs" className="rounded-2xl border border-border bg-surface p-6 space-y-6">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navInputs')}
            </h2>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">TextInput (with counter)</label>
                <TextInput
                  defaultValue="Александр Петрович Левшиц"
                  maxLength={60}
                  showCount
                  aria-label="Full Name Input"
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">SearchInput (debounced)</label>
                <SearchInput placeholder="Search surnames (Esc to clear)…" />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">PasswordInput (strength hint)</label>
                <PasswordInput
                  defaultValue="archival-passkey-2026"
                  showStrengthHint
                  aria-label="Password Input"
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">EmailInput & YearInput</label>
                <div className="flex items-center gap-2">
                  <EmailInput defaultValue="archive@levshits.family" aria-label="Email Input" />
                  <YearInput value={yearSingle} onChange={setYearSingle} />
                </div>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">
                  Select (Desktop dropdown / Mobile &lt;768px bottom sheet)
                </label>
                <Select
                  ariaLabel="Relationship Type"
                  defaultValue="married"
                  options={[
                    { value: 'married', label: 'Married (Брак)' },
                    { value: 'civil_union', label: 'Civil Union (Гражданский союз)' },
                    { value: 'partners', label: 'Partners (Партнёры)' },
                    { value: 'unknown', label: 'Unknown (Неизвестно)' },
                  ]}
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-secondary">
                  MultiSelect (Removable chips, Backspace removes last)
                </label>
                <MultiSelect
                  ariaLabel="Filter surnames"
                  value={multiSelectValues}
                  onChange={setMultiSelectValues}
                  options={[
                    { value: 'Левшиц', label: 'Левшиц (Levshits)' },
                    { value: 'Кузнецов', label: 'Кузнецов (Kuznetsov)' },
                    { value: 'Морозова', label: 'Морозова (Morozova)' },
                    { value: 'Щеглов', label: 'Щеглов (Shcheglov)' },
                  ]}
                />
              </div>
            </div>

            {/* Async Combobox / Picker State Simulator */}
            <div className="rounded-xl border border-border bg-surface-muted/40 p-4 space-y-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <span className="text-xs font-semibold text-primary">
                  Async Combobox Simulator (150ms debounce + AbortController)
                </span>
                <SegmentedControl<'normal' | 'loading' | 'error' | 'empty'>
                  size="sm"
                  ariaLabel="Async simulation mode"
                  value={pickerSimMode}
                  onChange={setPickerSimMode}
                  options={[
                    { value: 'normal', label: t('workbench.simulateNormal') },
                    { value: 'loading', label: t('workbench.simulateLoading') },
                    { value: 'error', label: t('workbench.simulateError') },
                    { value: 'empty', label: t('workbench.simulateEmpty') },
                  ]}
                />
              </div>
              <Combobox
                ariaLabel="Async place search demo"
                loadOptions={simulatedAsyncLoader}
                onCreate={(q) =>
                  addToast({ type: 'info', title: `Created place "${q}"` })
                }
                placeholder="Type to test async loading / abort / retry…"
              />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
              <Slider
                label="PDF Tile Overlap (mm)"
                min={5}
                max={20}
                unit=" mm"
                value={sliderValue}
                onChange={setSliderValue}
              />
              <RangeSlider
                label="Birth Year Filter Range"
                min={1750}
                max={2026}
                value={yearRange}
                onChange={setYearRange}
              />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 pt-2">
              <CheckboxGroup
                ariaLabel="Export options"
                defaultValue={['photos', 'sources']}
                options={[
                  {
                    value: 'photos',
                    label: 'Include High-Resolution Portraits',
                    description: 'Embed cropped avatars inside GEDZIP archive',
                  },
                  {
                    value: 'sources',
                    label: 'Include Archival Source Citations',
                    description: 'Export SOUR tags for every life event',
                  },
                ]}
              />

              <RadioCardGroup
                ariaLabel="GEDCOM Import Mode"
                defaultValue="clean"
                options={[
                  {
                    value: 'clean',
                    label: 'Clean Slate Import',
                    description: 'Replaces current tree with automatic pre-import snapshot.',
                    icon: <Layers className="h-4 w-4" />,
                  },
                  {
                    value: 'merge',
                    label: 'Append & Reconcile',
                    description: 'Matches existing people and opens side-by-side diff.',
                    icon: <GitMerge className="h-4 w-4" />,
                  },
                ]}
              />
            </div>
          </section>

          {/* 4. GenDateInput & Bilingual Parser Harness */}
          <section id="wb-gendate" className="rounded-2xl border border-border bg-surface p-6 space-y-5">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="font-serif text-lg font-bold text-primary">
                  {t('workbench.navGenDate')}
                </h2>
                <p className="text-xs text-secondary">
                  {t('workbench.presetsTitle')}
                </p>
              </div>
              <Calendar className="h-5 w-5 text-brand" />
            </div>

            <GenDateInput
              value={genDateValue}
              onChange={(res) => setGenDateValue(res.value)}
            />

            <div className="flex flex-wrap gap-1.5">
              {GENDATE_PRESETS.map((preset) => (
                <button
                  key={preset}
                  type="button"
                  onClick={() => {
                    const parsed = parseGenDate(preset, locale);
                    setGenDateValue(parsed.value);
                  }}
                  className="rounded-lg border border-border bg-surface-muted px-2.5 py-1 font-mono text-xs text-primary hover:border-brand hover:text-brand transition-colors cursor-pointer"
                >
                  {preset}
                </button>
              ))}
            </div>

            <div className="rounded-xl border border-border bg-canvas p-3 font-mono text-xs text-secondary overflow-x-auto">
              {JSON.stringify(
                {
                  value: genDateValue,
                  sortKey: parseGenDate(genDateValue.text, locale).sortKey,
                },
                null,
                2
              )}
            </div>
          </section>

          {/* 5. Form System, MarkdownEditor & EventForm */}
          <section id="wb-form" className="rounded-2xl border border-border bg-surface p-6 space-y-6">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h2 className="font-serif text-lg font-bold text-primary">
                {t('workbench.navForm')}
              </h2>
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() =>
                    setSimServerErrors({
                      description: 'Server 422: Description conflicts with locked archival record',
                    })
                  }
                >
                  {t('workbench.triggerServerError')}
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() =>
                    setSimSoftWarning(t('domain.eventForm.softWarningChronology'))
                  }
                >
                  {t('workbench.triggerSoftWarning')}
                </Button>
              </div>
            </div>

            <EventForm
              externalServerErrors={simServerErrors}
              externalSoftWarning={simSoftWarning}
              onSubmit={(vals) => {
                setSimServerErrors(undefined);
                addToast({
                  type: 'success',
                  title: 'Event saved',
                  description: `${vals.event_type} · ${vals.description}`,
                });
              }}
            />

            <Divider />

            <div className="space-y-2">
              <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                MarkdownEditor (Write / Preview + Sanitized MarkdownView)
              </div>
              <MarkdownEditor value={markdownBio} onChange={setMarkdownBio} />
            </div>
          </section>

          <WorkbenchAdvancedSections />
        </div>
      </div>
    </div>
  );
};

export default DevUiWorkbench;
