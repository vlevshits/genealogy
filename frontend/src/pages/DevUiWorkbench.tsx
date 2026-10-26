import React, { useState, useMemo } from 'react';
import {
  BookOpen,
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
  Users,
} from 'lucide-react';
import {
  Accordion,
  ActivityFeed,
  Alert,
  Avatar,
  AvatarGroup,
  Badge,
  Banner,
  BottomNav,
  Breadcrumbs,
  BulkActionBar,
  Button,
  ButtonGroup,
  Card,
  CardBody,
  CardHeader,
  CheckboxGroup,
  Collapsible,
  Combobox,
  ComboboxOption,
  ComparisonFieldRow,
  ComparisonTable,
  ConfirmDialog,
  CopyButton,
  DataTable,
  DataTableColumn,
  DescriptionList,
  Dialog,
  DialogSize,
  Divider,
  Drawer,
  EmailInput,
  EmptyState,
  FilterBar,
  FilterChip,
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
  Stat,
  Stepper,
  Switch,
  Tabs,
  TextInput,
  Timeline,
  TreeView,
  useToast,
  YearInput,
} from '../components/ui';
import {
  GenDateInput,
  MarkdownEditor,
} from '../components/form';
import {
  EventForm,
  EventRow,
  KinshipLabel,
  LivingBadge,
  PersonCard,
  PersonListItem,
  PersonName,
  PersonPicker,
  PlacePicker,
  PlaceText,
  PrivacyBadge,
  SexSelector,
} from '../components/domain';
import {
  DirectoryPersonRow,
  generateLargePeopleDataset,
  SAMPLE_EVENTS,
  SAMPLE_PEOPLE,
  SAMPLE_PLACES,
} from '../lib/mock-data';
import { parseGenDate } from '../lib/gendate-parser';
import { GenDate } from '../types/gendate';
import { Locale, Sex, ThemeMode } from '../types/common';
import { useI18n } from '../i18n';
import { useTheme } from '../lib/theme';

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
  const [sexValue, setSexValue] = useState<Sex>('male');
  const [pickerSimMode, setPickerSimMode] = useState<'normal' | 'loading' | 'error' | 'empty'>('normal');
  const [selectedPersonId, setSelectedPersonId] = useState<string | null>('ind-1');
  const [selectedPlaceId, setSelectedPlaceId] = useState<string | null>('p-1');
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

  // Overlays state
  const [dialogOpenSize, setDialogOpenSize] = useState<DialogSize | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [drawerRightOpen, setDrawerRightOpen] = useState(false);
  const [drawerBottomOpen, setDrawerBottomOpen] = useState(false);

  // Stepper state
  const [wizardStep, setWizardStep] = useState(1);

  // 50,000-row virtualized DataTable state
  const rows50k = useMemo(() => generateLargePeopleDataset(50000), []);
  const [selectedTableIds, setSelectedTableIds] = useState<string[]>(['person-1', 'person-2']);
  const [mobileTablePreview, setMobileTablePreview] = useState(false);

  // ComparisonTable state
  const [diffChoices, setDiffChoices] = useState<Record<string, 'left' | 'right'>>({
    fullName: 'left',
    birthDate: 'right',
    birthPlace: 'left',
    occupation: 'right',
  });

  const comparisonRows: ComparisonFieldRow[] = [
    {
      fieldKey: 'fullName',
      label: 'ФИО / Full Name',
      leftValue: 'Левшиц Александр Петрович',
      rightValue: 'Левшиц Александр Петрович',
    },
    {
      fieldKey: 'birthDate',
      label: 'Дата рождения / Birth Date',
      leftValue: 'ок. 1912 г.',
      rightValue: '14.07.1912 (ст. ст.)',
    },
    {
      fieldKey: 'birthPlace',
      label: 'Место рождения / Birth Place',
      leftValue: 'Витебск, Витебская губерния',
      rightValue: 'Витебск',
    },
    {
      fieldKey: 'occupation',
      label: 'Профессия / Occupation',
      leftValue: 'Инженер',
      rightValue: 'Инженер-мостостроитель, ЛИИЖТ',
    },
  ];

  const tableColumns: DataTableColumn<DirectoryPersonRow>[] = [
    {
      id: 'gid',
      header: 'ID',
      width: '90px',
      sortable: true,
      sortValue: (r) => r.gid,
      accessor: (r) => <span className="font-mono text-xs text-tertiary">{r.gid}</span>,
    },
    {
      id: 'name',
      header: locale === 'ru' ? 'Персона' : 'Person',
      sortable: true,
      sortValue: (r) => `${r.surname} ${r.given_name}`,
      accessor: (r) => (
        <div className="flex items-center gap-2">
          <Avatar name={`${r.given_name} ${r.surname}`} sex={r.sex} size="xs" />
          <span className="font-medium text-primary truncate">
            {locale === 'ru'
              ? `${r.surname} ${r.given_name} ${r.patronymic}`
              : `${r.given_name} ${r.patronymic} ${r.surname}`}
          </span>
        </div>
      ),
    },
    {
      id: 'lifespan',
      header: locale === 'ru' ? 'Годы жизни' : 'Life Span',
      width: '150px',
      sortable: true,
      sortValue: (r) => r.birth_year,
      accessor: (r) => (
        <span className="font-mono text-xs text-secondary">
          {r.birth_year} – {r.is_living ? t('domain.living') : r.death_year ?? '?'}
        </span>
      ),
    },
    {
      id: 'birth_place',
      header: locale === 'ru' ? 'Место рождения' : 'Birthplace',
      sortable: true,
      sortValue: (r) => r.birth_place,
      accessor: (r) => <PlaceText place={r.birth_place} />,
    },
  ];

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
              action={<Button size="sm" variant="primary">+ Add Place</Button>}
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

          {/* 6. Overlays: Dialogs, ConfirmDialog, Drawers, Toasts */}
          <section id="wb-overlays" className="rounded-2xl border border-border bg-surface p-6 space-y-5">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navOverlays')}
            </h2>

            <div className="flex flex-wrap items-center gap-2.5">
              <Button size="sm" variant="secondary" onClick={() => setDialogOpenSize('sm')}>
                {t('workbench.openModalSm')}
              </Button>
              <Button size="sm" variant="secondary" onClick={() => setDialogOpenSize('md')}>
                {t('workbench.openModalMd')}
              </Button>
              <Button size="sm" variant="secondary" onClick={() => setDialogOpenSize('xl')}>
                {t('workbench.openModalXl')}
              </Button>
              <Button size="sm" variant="danger" onClick={() => setConfirmOpen(true)}>
                {t('workbench.openConfirmDanger')}
              </Button>
              <Button size="sm" variant="secondary" onClick={() => setDrawerRightOpen(true)}>
                {t('workbench.openDrawerRight')}
              </Button>
              <Button size="sm" variant="secondary" onClick={() => setDrawerBottomOpen(true)}>
                {t('workbench.openDrawerBottom')}
              </Button>
            </div>

            <div className="flex flex-wrap items-center gap-2.5 pt-2">
              <Button
                size="sm"
                variant="primary"
                onClick={() =>
                  addToast({
                    type: 'success',
                    title: 'Person profile saved',
                    description: 'I0001 · Aleksandr Petrovich Levshits',
                  })
                }
              >
                {t('workbench.triggerSuccessToast')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  addToast({
                    type: 'error',
                    title: 'Failed to upload scan',
                    description: 'Network timeout while uploading metric book page',
                    onRetry: () =>
                      addToast({ type: 'success', title: 'Retry succeeded!' }),
                  })
                }
              >
                {t('workbench.triggerErrorToast')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  addToast({
                    type: 'undo',
                    title: 'Deleted life event (E0003)',
                    durationSec: 6,
                    onUndo: () =>
                      addToast({ type: 'info', title: 'Event E0003 restored' }),
                  })
                }
              >
                {t('workbench.triggerUndoToast')}
              </Button>
            </div>

            {/* Dialog demo */}
            <Dialog
              open={dialogOpenSize !== null}
              onOpenChange={(o) => !o && setDialogOpenSize(null)}
              size={dialogOpenSize ?? 'md'}
              title={`Modal Dialog (${dialogOpenSize ?? 'md'})`}
              description="Focus is trapped inside; pressing Esc returns focus to the trigger button."
              footer={
                <>
                  <Button variant="secondary" onClick={() => setDialogOpenSize(null)}>
                    {t('common.cancel')}
                  </Button>
                  <Button variant="primary" onClick={() => setDialogOpenSize(null)}>
                    {t('common.confirm')}
                  </Button>
                </>
              }
            >
              <p className="text-secondary">
                On desktop screens this modal renders at standard archival width ({dialogOpenSize}).
                On mobile viewports (&lt; 768px), medium and larger dialogs automatically expand to full-screen.
              </p>
            </Dialog>

            {/* Destructive ConfirmDialog with impact list & TypedConfirmInput */}
            <ConfirmDialog
              open={confirmOpen}
              onOpenChange={setConfirmOpen}
              title="Delete Person Record (I0003)?"
              description="This action removes the person from families and logs a restorable snapshot in Change History."
              impactList={[
                'Deletes 6 personal life events',
                'Unlinks 2 family unions and 4 archival photographs',
              ]}
              requirePhrase="DELETE"
              onConfirm={() =>
                addToast({
                  type: 'undo',
                  title: 'Deleted I0003 (Pyotr Shcheglov)',
                  onUndo: () => {},
                })
              }
            />

            {/* Right Drawer (480px) */}
            <Drawer
              open={drawerRightOpen}
              onOpenChange={setDrawerRightOpen}
              side="right"
              title="Quick Inspector: I0001"
              description="Slide-over 480px panel for inspecting person details without leaving the tree canvas."
              footer={
                <Button variant="primary" onClick={() => setDrawerRightOpen(false)}>
                  {t('common.close')}
                </Button>
              }
            >
              <div className="space-y-4">
                <PersonListItem person={SAMPLE_PEOPLE[0]} />
                <DescriptionList
                  items={[
                    { label: 'GID', value: 'I0001' },
                    { label: 'Birth', value: '14 Jul 1912 (O.S.) · Vitebsk' },
                    { label: 'Death', value: '3 Nov 1989 · Leningrad' },
                  ]}
                />
              </div>
            </Drawer>

            {/* Bottom Sheet Drawer */}
            <Drawer
              open={drawerBottomOpen}
              onOpenChange={setDrawerBottomOpen}
              side="bottom"
              title="Mobile Filter Sheet"
              description="Bottom sheet presentation used on touch devices."
            >
              <p className="text-secondary">
                Filters and pickers smoothly adapt to bottom sheets on mobile screens.
              </p>
            </Drawer>
          </section>

          {/* 7. Navigation, Tabs, Stepper, Collapsible, Accordion, TreeView */}
          <section id="wb-navigation" className="rounded-2xl border border-border bg-surface p-6 space-y-6">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navNavigation')}
            </h2>

            <div className="space-y-2">
              <div className="text-xs font-medium text-secondary">
                Breadcrumbs (middle items collapse into ⋯ menu)
              </div>
              <Breadcrumbs
                maxVisible={4}
                items={[
                  { id: '1', label: 'Russian Empire' },
                  { id: '2', label: 'Vitebsk Governorate' },
                  { id: '3', label: 'Vitebsk Uyezd' },
                  { id: '4', label: 'Vitebsk Parish #2' },
                  { id: '5', label: 'Levshits Household (I0001)' },
                ]}
              />
            </div>

            <Stepper
              currentStepIndex={wizardStep}
              onStepClick={setWizardStep}
              steps={[
                { id: 'upload', title: 'Upload GEDCOM', description: 'Select 5.5.1 or 7.0 file' },
                { id: 'reconcile', title: 'Reconcile Matches', description: 'Review 12 duplicates' },
                { id: 'commit', title: 'Commit Snapshot', description: 'Apply changes safely' },
              ]}
            />

            <Tabs
              ariaLabel="Profile preview tabs"
              items={[
                {
                  value: 'timeline',
                  label: 'Timeline & Events',
                  count: 4,
                  content: (
                    <div className="text-xs text-secondary">
                      Chronological life events tab content.
                    </div>
                  ),
                },
                {
                  value: 'media',
                  label: 'Archival Media',
                  count: 12,
                  content: (
                    <div className="text-xs text-secondary">
                      Tagged photographs and scans.
                    </div>
                  ),
                },
                {
                  value: 'notes',
                  label: 'Research Notes',
                  count: 3,
                  content: (
                    <div className="text-xs text-secondary">
                      Markdown transcriptions and remarks.
                    </div>
                  ),
                },
              ]}
            />

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-3">
                <Collapsible title="Advanced PDF Tiling Options" defaultOpen>
                  <p className="text-xs text-secondary">
                    Configure A4 cut markers, 10mm glue overlap, and Russian sheet headers.
                  </p>
                </Collapsible>
                <Accordion
                  items={[
                    {
                      id: 'faq-1',
                      title: 'How does the Living Person Privacy Rule work?',
                      content:
                        'Anyone without a death event born within the last 110 years (or with unknown dates) is automatically protected.',
                    },
                    {
                      id: 'faq-2',
                      title: 'Can I undo a GEDCOM import?',
                      content:
                        'Yes. Every import creates an automatic pre-import tree snapshot for one-click rollback.',
                    },
                  ]}
                />
              </div>

              <div className="space-y-1.5">
                <div className="text-xs font-medium text-secondary">
                  TreeView (Hierarchical Places with Keyboard Arrows)
                </div>
                <TreeView
                  nodes={[
                    {
                      id: 'ru-emp',
                      label: 'Российская империя',
                      badge: 87,
                      children: [
                        {
                          id: 'spb-gub',
                          label: 'Санкт-Петербургская губерния',
                          badge: 42,
                          children: [{ id: 'spb-city', label: 'Санкт-Петербург', badge: 42 }],
                        },
                        {
                          id: 'tver-gub',
                          label: 'Тверская губерния',
                          badge: 19,
                          children: [{ id: 'tver-city', label: 'Тверь', badge: 19 }],
                        },
                      ],
                    },
                  ]}
                />
              </div>
            </div>

            <div className="space-y-1.5 pt-2">
              <div className="text-xs font-medium text-secondary">
                BottomNav (Mobile 5-Tab Navigation Bar + Safe-Area Inset + More Sheet)
              </div>
              <BottomNav
                inlinePreview
                currentPath="/tree"
                themeMode={theme}
                onThemeChange={setTheme}
              />
            </div>
          </section>

          {/* 8. Virtualized DataTable (50,000 Rows) */}
          <section id="wb-datatable" className="rounded-2xl border border-border bg-surface p-6 space-y-4">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <h2 className="font-serif text-lg font-bold text-primary">
                  {t('workbench.navDataTable')}
                </h2>
                <p className="text-xs text-secondary">
                  Smooth virtualized scrolling across 50,000 rows with sorting, shift-click range selection, and mobile card fallback.
                </p>
              </div>
              <Switch
                checked={mobileTablePreview}
                onCheckedChange={setMobileTablePreview}
                label={t('workbench.mobilePreviewToggle')}
              />
            </div>

            <FilterBar
              hasActiveFilters={multiSelectValues.length > 0}
              onClearAll={() => setMultiSelectValues([])}
            >
              {multiSelectValues.map((s) => (
                <FilterChip
                  key={s}
                  label="Surname"
                  value={s}
                  onRemove={() =>
                    setMultiSelectValues((prev) => prev.filter((x) => x !== s))
                  }
                />
              ))}
            </FilterBar>

            <DataTable
              data={rows50k}
              columns={tableColumns}
              getRowId={(r) => r.id}
              selectable
              selectedIds={selectedTableIds}
              onSelectionChange={setSelectedTableIds}
              forceMobilePreview={mobileTablePreview}
              viewportHeight={420}
              mobileRow={(row) => (
                <Card className="p-3">
                  <div className="flex items-center justify-between">
                    <div className="font-semibold text-sm text-primary">
                      {row.surname} {row.given_name} {row.patronymic}
                    </div>
                    <Badge variant="neutral" size="sm">
                      {row.gid}
                    </Badge>
                  </div>
                  <div className="mt-1 text-xs text-secondary">
                    {row.birth_year} – {row.death_year ?? t('domain.living')} · {row.birth_place}
                  </div>
                </Card>
              )}
            />

            <BulkActionBar
              selectedCount={selectedTableIds.length}
              onClearSelection={() => setSelectedTableIds([])}
              actions={
                <Button size="sm" variant="secondary">
                  Export Selected
                </Button>
              }
            />
          </section>

          {/* 9. ComparisonTable & Data Display */}
          <section id="wb-comparison" className="rounded-2xl border border-border bg-surface p-6 space-y-6">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navComparison')}
            </h2>

            <ComparisonTable
              rows={comparisonRows}
              choices={diffChoices}
              onChoiceChange={(fieldKey, side) =>
                setDiffChoices((prev) => ({ ...prev, [fieldKey]: side }))
              }
            />

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <Stat
                label="Individuals in Archive"
                value={1428}
                hint="Across 9 generations (1765–2026)"
                icon={<Users className="h-4 w-4" />}
              />
              <Stat
                label="Historical Places"
                value={184}
                hint="92% geocoded with coordinates"
                icon={<MapPin className="h-4 w-4" />}
              />
              <Stat
                label="Verified Sources & Scans"
                value={612}
                hint="Metric books, censuses, diplomas"
                icon={<BookOpen className="h-4 w-4" />}
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                  Timeline
                </div>
                <Timeline
                  items={[
                    {
                      id: 'tl-1',
                      title: '14 Jul 1912 (O.S.) · Birth in Vitebsk',
                      meta: 'Parish register #34',
                      body: 'Born to Pyotr Yakovlevich Levshits.',
                      tone: 'brand',
                    },
                    {
                      id: 'tl-2',
                      title: 'Jun 1935 · Graduated LIIZhT',
                      meta: 'Leningrad',
                      body: 'Bridge engineering faculty.',
                      tone: 'accent',
                    },
                  ]}
                />
              </div>

              <div className="space-y-4">
                <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                  ActivityFeed & AvatarGroup
                </div>
                <AvatarGroup
                  items={SAMPLE_PEOPLE.map((p) => ({
                    id: p.id,
                    name: `${p.given_name ?? ''} ${p.surname ?? ''}`,
                    sex: p.sex,
                  }))}
                  max={4}
                />
                <ActivityFeed
                  entries={[
                    {
                      id: 'af-1',
                      dayGroup: 'Today',
                      actorName: 'Vera Levshits',
                      summary: 'attached archival scan to Birth of Aleksandr Levshits (I0001)',
                      timestamp: '14:22',
                    },
                    {
                      id: 'af-2',
                      dayGroup: 'Yesterday',
                      actorName: 'Lev Shcheglov',
                      summary: 'merged duplicate place records for Vitebsk',
                      timestamp: '19:05',
                    },
                  ]}
                />
              </div>
            </div>
          </section>

          {/* 10. Genealogy Domain Composites */}
          <section id="wb-domain" className="rounded-2xl border border-border bg-surface p-6 space-y-6">
            <h2 className="font-serif text-lg font-bold text-primary">
              {t('workbench.navDomain')}
            </h2>

            {/* PersonName, GenDateText, LifeSpan, KinshipLabel, PrivacyBadge */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Card>
                <CardHeader>
                  <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                    PersonName, KinshipLabel & Badges
                  </div>
                </CardHeader>
                <CardBody className="space-y-3">
                  <div className="flex flex-wrap items-center gap-2">
                    <PersonName person={SAMPLE_PEOPLE[1]} variant="full" />
                    <LivingBadge />
                    <PrivacyBadge />
                  </div>
                  <div className="text-xs text-secondary">
                    Redacted mask: <PersonName person={SAMPLE_PEOPLE[4]} variant="full" />
                  </div>
                  <div className="flex flex-wrap gap-2 pt-1">
                    <KinshipLabel relationship="first_cousin" sex="female" />
                    <KinshipLabel relationship="great_grandparent" sex="male" />
                    <KinshipLabel relationship="parent_in_law_husband" sex="female" />
                  </div>
                </CardBody>
              </Card>

              <Card>
                <CardHeader>
                  <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                    SexSelector & Pickers (Cross-Script Search)
                  </div>
                </CardHeader>
                <CardBody className="space-y-3">
                  <SexSelector value={sexValue} onChange={setSexValue} />
                  <PersonPicker
                    value={selectedPersonId}
                    onChange={(id) => setSelectedPersonId(id)}
                    onCreatePerson={(q) =>
                      addToast({ type: 'info', title: `Quick-create "${q}"` })
                    }
                  />
                  <PlacePicker
                    value={selectedPlaceId}
                    onChange={(id) => setSelectedPlaceId(id)}
                    onCreatePlace={(q) =>
                      addToast({ type: 'info', title: `Quick-create place "${q}"` })
                    }
                  />
                </CardBody>
              </Card>
            </div>

            {/* Tree Canvas PersonCards */}
            <div className="space-y-2">
              <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                PersonCard (Tree Canvas Nodes with Quick-Add `+` Hover Actions)
              </div>
              <div className="flex flex-wrap items-center gap-4 rounded-2xl border border-border bg-canvas p-6">
                <PersonCard
                  person={SAMPLE_PEOPLE[0]}
                  selected
                  onQuickAdd={(p, dir) =>
                    addToast({
                      type: 'info',
                      title: `Quick-add ${dir} for ${p.given_name} (${p.gid})`,
                    })
                  }
                />
                <PersonCard
                  person={SAMPLE_PEOPLE[1]}
                  onQuickAdd={(p, dir) =>
                    addToast({
                      type: 'info',
                      title: `Quick-add ${dir} for ${p.given_name} (${p.gid})`,
                    })
                  }
                />
                <PersonCard person={SAMPLE_PEOPLE[4]} />
              </div>
            </div>

            {/* EventRows */}
            <div className="space-y-3">
              <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                EventRow (Life Milestones with Source Citations & Attached Scans)
              </div>
              {SAMPLE_EVENTS.map((ev) => (
                <EventRow
                  key={ev.id}
                  event={ev}
                  onEdit={(e) =>
                    addToast({ type: 'info', title: `Editing event ${e.gid}` })
                  }
                />
              ))}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
};

export default DevUiWorkbench;
