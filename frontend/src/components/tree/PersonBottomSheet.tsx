import React, { useState } from 'react';
import {
  ChevronDown,
  ChevronUp,
  Crosshair,
  ExternalLink,
  Home,
  Lock,
  PenTool,
  UserPlus,
} from 'lucide-react';
import {
  Avatar,
  Badge,
  Button,
  Drawer,
} from '../ui';
import { QuickAddRelation, TreePersonNodeDto } from '../../api/client';
import { useI18n } from '../../i18n';
import { formatTreeCardName } from './PersonNodeCard';
import { PersonImmediateFamily } from './PersonDrawer';

export interface PersonBottomSheetProps {
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
  onOpenProfile?: (person: TreePersonNodeDto) => void;
}

export const PersonBottomSheet: React.FC<PersonBottomSheetProps> = ({
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
  onOpenProfile,
}) => {
  const { t, locale } = useI18n();
  const [expanded, setExpanded] = useState<boolean>(false);

  if (!person) return null;

  const { displayName, isMaskedLiving } = formatTreeCardName(
    person,
    accessTier,
    locale,
    'card'
  );
  const isAuthenticated = accessTier !== 'public_anonymous';

  return (
    <Drawer
      open={open}
      onOpenChange={onOpenChange}
      side="bottom"
      modal={false}
      title={displayName}
      description={person.gid}
      className={expanded ? 'max-h-[75vh]' : 'max-h-[32vh]'}
    >
      <div className="space-y-3">
        {/* Peek Header Row */}
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2.5 min-w-0">
            <Avatar
              name={isMaskedLiving ? '?' : displayName}
              src={isMaskedLiving ? undefined : person.avatar_url ?? undefined}
              size="md"
            />
            <div className="min-w-0">
              {isMaskedLiving ? (
                <div className="flex items-center gap-1 text-xs text-secondary">
                  <Lock className="h-3 w-3 shrink-0" />
                  <span>{t('tree.drawer.privateDetailsHidden')}</span>
                </div>
              ) : (
                <div className="text-xs text-secondary truncate">
                  {person.birth ? `* ${person.birth.date}` : ''}
                  {person.death ? ` · † ${person.death.date}` : ''}
                </div>
              )}
              <div className="flex flex-wrap items-center gap-1 mt-1">
                {isHomePerson && (
                  <Badge
                    variant="brand"
                    size="sm"
                    icon={<Home className="h-2.5 w-2.5" />}
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
              </div>
            </div>
          </div>

          <div className="flex items-center gap-1.5 shrink-0">
            <Button
              size="sm"
              variant="secondary"
              leftIcon={
                expanded ? (
                  <ChevronDown className="h-3.5 w-3.5" />
                ) : (
                  <ChevronUp className="h-3.5 w-3.5" />
                )
              }
              onClick={() => setExpanded((prev) => !prev)}
            >
              {expanded
                ? t('tree.drawer.collapseSheet')
                : t('tree.drawer.expandSheet')}
            </Button>

            <Button
              size="sm"
              variant="primary"
              rightIcon={<ExternalLink className="h-3.5 w-3.5" />}
              onClick={() => onOpenProfile?.(person)}
            >
              {t('tree.card.profileBtn')}
            </Button>
          </div>
        </div>

        {/* Expanded Content (75% height) */}
        {expanded && (
          <div className="border-t border-border pt-3 space-y-3">
            {family && (
              <div className="space-y-2">
                <div className="text-xs font-semibold text-secondary">
                  {t('tree.drawer.immediateFamilyTitle')}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {[
                    ...family.parents,
                    ...family.spouses,
                    ...family.children,
                  ].map((rel) => {
                    const formatted = formatTreeCardName(
                      rel,
                      accessTier,
                      locale,
                      'card'
                    );
                    return (
                      <Button
                        key={rel.id}
                        size="sm"
                        variant="secondary"
                        onClick={() => onSelectRelative?.(rel)}
                      >
                        {formatted.displayName} ({rel.gid})
                      </Button>
                    );
                  })}
                </div>
              </div>
            )}

            <div className="flex flex-wrap items-center gap-1.5">
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<Crosshair className="h-3.5 w-3.5" />}
                onClick={() => onMakeFocal?.(person)}
              >
                {t('tree.drawer.makeFocalBtn')}
              </Button>

              {canEdit && isAuthenticated && !isMaskedLiving && !isHomePerson && (
                <Button
                  size="sm"
                  variant="secondary"
                  leftIcon={<Home className="h-3.5 w-3.5" />}
                  onClick={() => onSetHomePerson?.(person)}
                >
                  {t('tree.setAsHomePerson')}
                </Button>
              )}

              {canEdit && !isMaskedLiving && (
                <>
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                    onClick={() => onQuickAdd?.(person, 'father')}
                  >
                    {t('person.actions.addFather')}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                    onClick={() => onQuickAdd?.(person, 'child')}
                  >
                    {t('person.actions.addChild')}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    leftIcon={<PenTool className="h-3.5 w-3.5" />}
                    onClick={() => onEditPerson?.(person)}
                  >
                    {t('person.actions.editProfile')}
                  </Button>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </Drawer>
  );
};
