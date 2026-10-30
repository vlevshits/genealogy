import React from 'react';
import { Crosshair, ExternalLink, GitCommitVertical, X } from 'lucide-react';
import { Avatar, Badge, Button, IconButton } from '../ui';
import { FamilyDto, TreePersonNodeDto } from '../../api/client';
import { PositionedEdge } from '../../lib/tree-layouts';
import { useI18n } from '../../i18n';
import { formatTreeCardName } from './PersonNodeCard';
import { cn } from '../../lib/utils';

export interface ConnectionNavPopoverProps {
  edge: PositionedEdge;
  sourcePerson?: TreePersonNodeDto | null;
  targetPerson?: TreePersonNodeDto | null;
  family?: FamilyDto | null;
  marriageDate?: string | null;
  marriagePlace?: string | null;
  accessTier?: string;
  isMobile?: boolean;
  pan: { x: number; y: number };
  zoom: number;
  containerRect?: { width: number; height: number; left: number; top: number } | null;
  onJumpToPerson: (personId: string) => void;
  onFindKinship: (personAId: string, personBId: string) => void;
  onClose: () => void;
  onViewPerson?: (personId: string) => void;
  className?: string;
}

export const ConnectionNavPopover: React.FC<ConnectionNavPopoverProps> = ({
  edge,
  sourcePerson,
  targetPerson,
  family,
  marriageDate,
  marriagePlace,
  accessTier = 'admin',
  isMobile = false,
  pan,
  zoom,
  containerRect,
  onJumpToPerson,
  onFindKinship,
  onClose,
  onViewPerson,
  className,
}) => {
  const { t, locale } = useI18n();

  const isSpouse = edge.kind === 'spouse';
  const relationshipBadgeLabel = isSpouse
    ? t('tree.connection.marriage')
    : t('tree.connection.parent_child');

  const marriageEvent = family?.events?.find(
    (e) =>
      e.event_type?.toUpperCase() === 'MARR' ||
      e.event_type?.toLowerCase() === 'marriage'
  );

  const rawMarriageDate =
    marriageDate ??
    family?.marriage_date ??
    marriageEvent?.date_text ??
    (marriageEvent?.date
      ? typeof marriageEvent.date === 'string'
        ? marriageEvent.date
        : marriageEvent.date.text ||
          (marriageEvent.date.start?.y ? String(marriageEvent.date.start.y) : null)
      : null);

  const effectiveMarriageDate: string | null = rawMarriageDate ?? null;

  const effectiveMarriagePlace: string | null =
    marriagePlace ??
    family?.marriage_place ??
    marriageEvent?.place_full_name ??
    null;

  // Role labels
  const getSourceRoleLabel = () => {
    if (isSpouse) return t('tree.connection.spouse');
    if (!sourcePerson) return t('tree.connection.parent');
    if (sourcePerson.sex === 'M') return t('tree.connection.father');
    if (sourcePerson.sex === 'F') return t('tree.connection.mother');
    return t('tree.connection.parent');
  };

  const getTargetRoleLabel = () => {
    if (isSpouse) return t('tree.connection.spouse');
    if (!targetPerson) return t('tree.connection.child');
    if (targetPerson.sex === 'M') return t('tree.connection.son');
    if (targetPerson.sex === 'F') return t('tree.connection.daughter');
    return t('tree.connection.child');
  };

  const sourceNameInfo = sourcePerson
    ? formatTreeCardName(sourcePerson, accessTier, locale, 'card')
    : { displayName: edge.sourceId, isMaskedLiving: false };

  const targetNameInfo = targetPerson
    ? formatTreeCardName(targetPerson, accessTier, locale, 'card')
    : { displayName: edge.targetId, isMaskedLiving: false };

  // Calculate screen positioning
  const hasMarriageDetails = Boolean(
    isSpouse && (effectiveMarriageDate || effectiveMarriagePlace)
  );
  const popoverWidth = 300;
  const popoverHeight = hasMarriageDetails ? 275 : 220;
  const margin = 16;

  const rawScreenX = pan.x + edge.midX * zoom;
  const rawScreenY = pan.y + edge.midY * zoom;

  const contWidth = containerRect?.width || (typeof window !== 'undefined' ? window.innerWidth : 800);
  const contHeight = containerRect?.height || (typeof window !== 'undefined' ? window.innerHeight : 600);

  const maxX = contWidth - popoverWidth - margin;
  const maxY = contHeight - popoverHeight - margin;

  const clampedX = Math.max(margin, Math.min(maxX, rawScreenX - popoverWidth / 2));
  // Position above midpoint by default, or below if too close to top
  const initialTop = rawScreenY - popoverHeight - 16;
  const clampedY = initialTop < margin ? Math.min(maxY, rawScreenY + 16) : Math.min(maxY, initialTop);

  return (
    <div
      role="dialog"
      aria-label={relationshipBadgeLabel}
      data-testid="connection-nav-popover"
      style={
        isMobile
          ? undefined
          : {
              position: 'absolute',
              left: `${Math.round(clampedX)}px`,
              top: `${Math.round(clampedY)}px`,
              width: `${popoverWidth}px`,
            }
      }
      className={cn(
        'z-30 rounded-2xl border border-border-card bg-surface/95 p-3.5 shadow-elevation-3 backdrop-blur-md antialiased transition-all',
        isMobile
          ? 'fixed top-16 inset-x-3 w-auto max-w-sm mx-auto'
          : 'w-[300px]',
        className
      )}
      onClick={(e) => e.stopPropagation()}
    >
      {/* Header: Relationship Badge + Close Button */}
      <div className="flex items-center justify-between gap-2 mb-3">
        <Badge
          variant={isSpouse ? 'warning' : 'brand'}
          size="sm"
          className="font-medium"
        >
          {relationshipBadgeLabel}
        </Badge>
        <IconButton
          variant="ghost"
          size="sm"
          onClick={onClose}
          label={t('tree.connection.close')}
          className="h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-primary touch-target-expand"
        >
          <X className="h-4 w-4" />
        </IconButton>
      </div>

      {/* Marriage Event Details */}
      {hasMarriageDetails && (
        <div
          data-testid="connection-marriage-details"
          className="mb-3 px-3 py-2 rounded-xl bg-amber-600/10 dark:bg-amber-400/10 border border-amber-600/20 dark:border-amber-400/20 text-xs text-secondary space-y-1"
        >
          {effectiveMarriageDate && (
            <div className="flex items-center justify-between gap-2">
              <span className="text-tertiary">{t('tree.connection.marriageDate')}:</span>
              <span className="font-semibold text-primary">{effectiveMarriageDate}</span>
            </div>
          )}
          {effectiveMarriagePlace && (
            <div className="flex items-center justify-between gap-2">
              <span className="text-tertiary">{t('tree.connection.marriagePlace')}:</span>
              <span
                className="font-semibold text-primary truncate max-w-[160px] text-right"
                title={effectiveMarriagePlace}
              >
                {effectiveMarriagePlace}
              </span>
            </div>
          )}
        </div>
      )}

      {/* Relatives Participant Chips */}
      <div className="space-y-2 mb-3">
        {/* Person A (Source) */}
        <div className="flex items-center justify-between gap-2 p-2 rounded-xl bg-canvas border border-border-card">
          <div className="flex items-center gap-2 min-w-0">
            <Avatar
              name={sourceNameInfo.isMaskedLiving ? '?' : sourceNameInfo.displayName}
              src={sourceNameInfo.isMaskedLiving ? undefined : sourcePerson?.avatar_url ?? undefined}
              size="sm"
            />
            <div className="min-w-0">
              <div className="text-xs font-semibold text-primary truncate">
                {sourceNameInfo.displayName}
              </div>
              <div className="text-[10px] text-tertiary">
                {getSourceRoleLabel()}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-1 shrink-0">
            <IconButton
              variant="ghost"
              size="sm"
              onClick={() => onJumpToPerson(edge.sourceId)}
              label={t('tree.connection.jump_to', { name: sourceNameInfo.displayName })}
              className="h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-brand touch-target-expand"
            >
              <Crosshair className="h-3.5 w-3.5" />
            </IconButton>
            {onViewPerson && sourcePerson && (
              <IconButton
                variant="ghost"
                size="sm"
                onClick={() => onViewPerson(sourcePerson.id)}
                label={t('tree.connection.view_person')}
                className="h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-brand touch-target-expand"
              >
                <ExternalLink className="h-3.5 w-3.5" />
              </IconButton>
            )}
          </div>
        </div>

        {/* Person B (Target) */}
        <div className="flex items-center justify-between gap-2 p-2 rounded-xl bg-canvas border border-border-card">
          <div className="flex items-center gap-2 min-w-0">
            <Avatar
              name={targetNameInfo.isMaskedLiving ? '?' : targetNameInfo.displayName}
              src={targetNameInfo.isMaskedLiving ? undefined : targetPerson?.avatar_url ?? undefined}
              size="sm"
            />
            <div className="min-w-0">
              <div className="text-xs font-semibold text-primary truncate">
                {targetNameInfo.displayName}
              </div>
              <div className="text-[10px] text-tertiary">
                {getTargetRoleLabel()}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-1 shrink-0">
            <IconButton
              variant="ghost"
              size="sm"
              onClick={() => onJumpToPerson(edge.targetId)}
              label={t('tree.connection.jump_to', { name: targetNameInfo.displayName })}
              className="h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-brand touch-target-expand"
            >
              <Crosshair className="h-3.5 w-3.5" />
            </IconButton>
            {onViewPerson && targetPerson && (
              <IconButton
                variant="ghost"
                size="sm"
                onClick={() => onViewPerson(targetPerson.id)}
                label={t('tree.connection.view_person')}
                className="h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-brand touch-target-expand"
              >
                <ExternalLink className="h-3.5 w-3.5" />
              </IconButton>
            )}
          </div>
        </div>
      </div>

      {/* Kinship Finder Trigger Button */}
      <Button
        variant="secondary"
        size="sm"
        leftIcon={<GitCommitVertical className="h-3.5 w-3.5" />}
        onClick={() => onFindKinship(edge.sourceId, edge.targetId)}
        className="w-full text-xs"
      >
        {t('tree.connection.find_kinship')}
      </Button>
    </div>
  );
};
