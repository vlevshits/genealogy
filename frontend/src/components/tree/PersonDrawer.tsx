import React from 'react';
import {
  Calendar,
  Crosshair,
  ExternalLink,
  Home,
  Lock,
  MapPin,
  PenTool,
  Plus,
  Trash2,
  UserPlus,
} from 'lucide-react';
import {
  Avatar,
  Badge,
  Button,
  Divider,
  Drawer,
} from '../ui';
import { QuickAddRelation, TreePersonNodeDto } from '../../api/client';
import { useI18n } from '../../i18n';
import { formatTreeCardName } from './PersonNodeCard';

export interface PersonImmediateFamily {
  parents: TreePersonNodeDto[];
  spouses: TreePersonNodeDto[];
  children: TreePersonNodeDto[];
}

export interface PersonDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  person: TreePersonNodeDto | null;
  family?: PersonImmediateFamily;
  accessTier?: string;
  canEdit?: boolean;
  isHomePerson?: boolean;
  onMakeFocal?: (person: TreePersonNodeDto) => void;
  onSetHomePerson?: (person: TreePersonNodeDto) => void;
  onSelectRelative?: (person: TreePersonNodeDto) => void;
  onQuickAdd?: (person: TreePersonNodeDto, relation: QuickAddRelation) => void;
  onEditPerson?: (person: TreePersonNodeDto) => void;
  onAddEvent?: (person: TreePersonNodeDto) => void;
  onDeletePerson?: (person: TreePersonNodeDto) => void;
  onOpenProfile?: (person: TreePersonNodeDto) => void;
}

export const PersonDrawer: React.FC<PersonDrawerProps> = ({
  open,
  onOpenChange,
  person,
  family,
  accessTier = 'admin',
  canEdit = false,
  isHomePerson = false,
  onMakeFocal,
  onSetHomePerson,
  onSelectRelative,
  onQuickAdd,
  onEditPerson,
  onAddEvent,
  onDeletePerson,
  onOpenProfile,
}) => {
  const { t, locale } = useI18n();

  if (!person) return null;

  const { displayName, isMaskedLiving } = formatTreeCardName(
    person,
    accessTier,
    locale,
    'card'
  );
  const isAuthenticated = accessTier !== 'public_anonymous';

  const renderRelativeButton = (rel: TreePersonNodeDto) => {
    const formatted = formatTreeCardName(rel, accessTier, locale, 'card');
    return (
      <Button
        key={rel.id}
        size="sm"
        variant="secondary"
        onClick={() => onSelectRelative?.(rel)}
        className="justify-start text-xs"
      >
        <span className="truncate">{formatted.displayName}</span>
        <span className="font-mono text-[10px] text-tertiary ml-1">
          ({rel.gid})
        </span>
      </Button>
    );
  };

  return (
    <Drawer
      open={open}
      onOpenChange={onOpenChange}
      side="right"
      modal={false}
      title={displayName}
      description={person.gid}
      className="sm:w-[440px]"
      footer={
        <div className="flex w-full items-center justify-between gap-2">
          <Button
            size="sm"
            variant="secondary"
            leftIcon={<Crosshair className="h-3.5 w-3.5" />}
            onClick={() => onMakeFocal?.(person)}
          >
            {t('tree.drawer.makeFocalBtn')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            rightIcon={<ExternalLink className="h-3.5 w-3.5" />}
            onClick={() => onOpenProfile?.(person)}
          >
            {t('tree.drawer.viewFullProfileBtn')}
          </Button>
        </div>
      }
    >
      <div className="space-y-5">
        {/* Identity Header */}
        <div className="flex items-start gap-3.5">
          <Avatar
            name={isMaskedLiving ? '?' : displayName}
            src={isMaskedLiving ? undefined : person.avatar_url ?? undefined}
            size="lg"
          />
          <div className="min-w-0 flex-1 space-y-1.5">
            <div className="font-sans text-base font-semibold text-primary">
              {displayName}
            </div>
            <div className="flex flex-wrap items-center gap-1.5">
              {isHomePerson && (
                <Badge
                  variant="brand"
                  size="sm"
                  icon={<Home className="h-3 w-3" />}
                >
                  {t('tree.homePersonBadge')}
                </Badge>
              )}
              {!person.is_living && (
                <span
                  aria-label={t('person.quickAdd.livingNo')}
                  title={t('person.quickAdd.livingNo')}
                  className="inline-flex h-5 w-5 items-center justify-center rounded-md bg-surface-muted font-serif text-xs font-semibold text-secondary"
                >
                  †
                </span>
              )}
              <Badge variant="neutral" size="sm">
                {person.sex === 'M'
                  ? t('person.sex.male')
                  : person.sex === 'F'
                  ? t('person.sex.female')
                  : t('person.sex.unknown')}
              </Badge>
              {person.is_private && !isMaskedLiving && (
                <Badge
                  variant="warning"
                  size="sm"
                  icon={<Lock className="h-3 w-3" />}
                >
                  {t('rbac.privateRecordBadge')}
                </Badge>
              )}
            </div>
          </div>
        </div>

        <Divider />

        {/* Core Facts */}
        <div className="space-y-2">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-tertiary">
            {t('tree.drawer.vitalFactsTitle')}
          </h3>

          {isMaskedLiving ? (
            <div className="flex items-center gap-2 rounded-xl border border-border bg-surface-muted/60 p-3 text-xs text-secondary">
              <Lock className="h-4 w-4 text-tertiary shrink-0" />
              <span>{t('tree.drawer.privateDetailsHidden')}</span>
            </div>
          ) : (
            <div className="rounded-xl border border-border bg-canvas p-3 space-y-2 text-xs text-secondary">
              {person.birth ? (
                <div className="flex items-center gap-2">
                  <Calendar className="h-3.5 w-3.5 text-tertiary shrink-0" />
                  <span>
                    <strong className="text-primary">
                      {t('domain.events.birth')}:
                    </strong>{' '}
                    {person.birth.date}
                    {person.birth.place ? ` · ${person.birth.place}` : ''}
                  </span>
                </div>
              ) : (
                <div className="text-tertiary">
                  {t('tree.drawer.noBirthRecorded')}
                </div>
              )}

              {person.death && (
                <div className="flex items-center gap-2">
                  <MapPin className="h-3.5 w-3.5 text-tertiary shrink-0" />
                  <span>
                    <strong className="text-primary">
                      {t('domain.events.death')}:
                    </strong>{' '}
                    {person.death.date}
                    {person.death.place ? ` · ${person.death.place}` : ''}
                  </span>
                </div>
              )}

              {person.notes && (
                <p className="border-t border-border pt-2 text-secondary leading-relaxed">
                  {person.notes}
                </p>
              )}
            </div>
          )}
        </div>

        {/* Immediate Family Links */}
        {family && (
          <div className="space-y-3">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-tertiary">
              {t('tree.drawer.immediateFamilyTitle')}
            </h3>

            <div className="space-y-2">
              <div className="text-xs font-medium text-secondary">
                {t('tree.drawer.parentsLabel')}
              </div>
              {family.parents.length > 0 ? (
                <div className="flex flex-wrap gap-1.5">
                  {family.parents.map(renderRelativeButton)}
                </div>
              ) : (
                <div className="text-xs text-tertiary">
                  {t('tree.drawer.noRelativesListed')}
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-xs font-medium text-secondary">
                {t('tree.drawer.spousesLabel')}
              </div>
              {family.spouses.length > 0 ? (
                <div className="flex flex-wrap gap-1.5">
                  {family.spouses.map(renderRelativeButton)}
                </div>
              ) : (
                <div className="text-xs text-tertiary">
                  {t('tree.drawer.noRelativesListed')}
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-xs font-medium text-secondary">
                {t('tree.drawer.childrenLabel')}
              </div>
              {family.children.length > 0 ? (
                <div className="flex flex-wrap gap-1.5">
                  {family.children.map(renderRelativeButton)}
                </div>
              ) : (
                <div className="text-xs text-tertiary">
                  {t('tree.drawer.noRelativesListed')}
                </div>
              )}
            </div>
          </div>
        )}

        {/* Home Person & Authoring Actions */}
        {canEdit && isAuthenticated && !isMaskedLiving && (
          <>
            <Divider />
            <div className="space-y-2.5">
              {!isHomePerson && (
                <Button
                  size="sm"
                  variant="secondary"
                  leftIcon={<Home className="h-3.5 w-3.5" />}
                  onClick={() => onSetHomePerson?.(person)}
                  className="w-full justify-center"
                >
                  {t('tree.setAsHomePerson')}
                </Button>
              )}

              <div className="space-y-2">
                <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                  {t('tree.drawer.quickActionsTitle')}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3 w-3" />}
                    onClick={() => onQuickAdd?.(person, 'father')}
                  >
                    {t('person.actions.addFather')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3 w-3" />}
                    onClick={() => onQuickAdd?.(person, 'mother')}
                  >
                    {t('person.actions.addMother')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3 w-3" />}
                    onClick={() => onQuickAdd?.(person, 'spouse')}
                  >
                    {t('person.actions.addSpouse')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3 w-3" />}
                    onClick={() => onQuickAdd?.(person, 'child')}
                  >
                    {t('person.actions.addChild')}
                  </Button>
                </div>

                <div className="flex flex-wrap gap-1.5 pt-1">
                  <Button
                    size="sm"
                    variant="ghost"
                    leftIcon={<PenTool className="h-3 w-3" />}
                    onClick={() => onEditPerson?.(person)}
                  >
                    {t('person.actions.editProfile')}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    leftIcon={<Plus className="h-3 w-3" />}
                    onClick={() => onAddEvent?.(person)}
                  >
                    {t('person.actions.addEvent')}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    leftIcon={<Trash2 className="h-3 w-3 text-danger" />}
                    onClick={() => onDeletePerson?.(person)}
                  >
                    {t('person.actions.deletePerson')}
                  </Button>
                </div>
              </div>
            </div>
          </>
        )}
      </div>
    </Drawer>
  );
};
