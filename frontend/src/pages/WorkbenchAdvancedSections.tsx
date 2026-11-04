import React, { useMemo, useState } from 'react';
import { BookOpen, MapPin, Users } from 'lucide-react';
import {
  Accordion,
  ActivityFeed,
  Avatar,
  AvatarGroup,
  Badge,
  BottomNav,
  Breadcrumbs,
  BulkActionBar,
  Button,
  Card,
  CardBody,
  CardHeader,
  Collapsible,
  ComparisonFieldRow,
  ComparisonTable,
  ConfirmDialog,
  DataTable,
  DataTableColumn,
  DescriptionList,
  Dialog,
  DialogSize,
  Drawer,
  FilterBar,
  FilterChip,
  Stat,
  Stepper,
  Switch,
  Tabs,
  Timeline,
  TreeView,
  useToast,
} from '../components/ui';
import {
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
} from '../lib/mock-data';
import { Sex } from '../types/common';
import { useI18n } from '../i18n';
import { useTheme } from '../lib/theme';

export const WorkbenchAdvancedSections: React.FC = () => {
  const { t, locale } = useI18n();
  const { theme, setTheme } = useTheme();
  const { addToast } = useToast();

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
  const [multiSelectValues, setMultiSelectValues] = useState<string[]>(['Левшиц', 'Кузнецов']);

  // Domain composite states
  const [sexValue, setSexValue] = useState<Sex>('male');
  const [selectedPersonId, setSelectedPersonId] = useState<string | null>('ind-1');
  const [selectedPlaceId, setSelectedPlaceId] = useState<string | null>('p-1');

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

  return (
    <>
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
    </>
  );
};
