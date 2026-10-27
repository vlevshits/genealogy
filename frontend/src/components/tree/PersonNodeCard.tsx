import React, { useRef } from 'react';
import {
  Calendar,
  Crosshair,
  Home,
  Lock,
  MapPin,
  UserPlus,
} from 'lucide-react';
import { Avatar, Badge, Button } from '../ui';
import { QuickAddRelation, TreePersonNodeDto } from '../../api/client';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';
import { CARD_HEIGHT, CARD_WIDTH } from '../../lib/tree-layouts';

export interface PersonNodeCardProps {
  node: TreePersonNodeDto;
  x?: number;
  y?: number;
  isFocal?: boolean;
  isSelected?: boolean;
  isHomePerson?: boolean;
  accessTier?: string;
  canEdit?: boolean;
  onSelect?: (node: TreePersonNodeDto) => void;
  onFocusNode?: (node: TreePersonNodeDto) => void;
  onQuickAdd?: (node: TreePersonNodeDto, relation: QuickAddRelation) => void;
  onEdit?: (node: TreePersonNodeDto) => void;
  className?: string;
}

export function formatTreeCardName(
  node: TreePersonNodeDto,
  accessTier: string = 'admin'
): { displayName: string; isMaskedLiving: boolean } {
  const isPublicGuest = accessTier === 'public_anonymous';
  const rawGiven = (node.primary_name.given_name || '').trim();
  const isMaskedLiving =
    node.is_living &&
    (isPublicGuest ||
      rawGiven === 'Living' ||
      /^[A-Za-zА-Яа-яЁё]\.$/.test(rawGiven));

  if (isMaskedLiving) {
    if (/^[A-Za-zА-Яа-яЁё]\.$/.test(rawGiven)) {
      return { displayName: rawGiven, isMaskedLiving: true };
    }
    const sourceWord =
      rawGiven && rawGiven !== 'Living'
        ? rawGiven
        : (node.primary_name.surname || 'L').trim();
    const initial = sourceWord.charAt(0).toUpperCase() || 'L';
    return { displayName: `${initial}.`, isMaskedLiving: true };
  }

  const fullName = [
    node.primary_name.given_name,
    node.primary_name.patronymic,
    node.primary_name.surname,
  ]
    .filter(Boolean)
    .join(' ');

  return {
    displayName: fullName || node.gid,
    isMaskedLiving: false,
  };
}

export const PersonNodeCard: React.FC<PersonNodeCardProps> = ({
  node,
  x,
  y,
  isFocal = false,
  isSelected = false,
  isHomePerson = false,
  accessTier = 'admin',
  canEdit = false,
  onSelect,
  onFocusNode,
  onQuickAdd,
  className,
}) => {
  const { t } = useI18n();
  const longPressTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const { displayName, isMaskedLiving } = formatTreeCardName(node, accessTier);

  const stripeColorClass =
    node.sex === 'M'
      ? 'bg-male'
      : node.sex === 'F'
      ? 'bg-female'
      : 'bg-neutral';

  const handlePointerDown = () => {
    if (longPressTimerRef.current) clearTimeout(longPressTimerRef.current);
    longPressTimerRef.current = setTimeout(() => {
      if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
        try {
          navigator.vibrate(15);
        } catch {
          // Ignore vibration errors on unsupported browsers
        }
      }
      onSelect?.(node);
    }, 500);
  };

  const clearLongPress = () => {
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
  };

  const positionedStyle: React.CSSProperties | undefined =
    x !== undefined && y !== undefined
      ? {
          position: 'absolute',
          left: `${x}px`,
          top: `${y}px`,
          width: `${CARD_WIDTH}px`,
          minHeight: `${CARD_HEIGHT}px`,
        }
      : {
          width: `${CARD_WIDTH}px`,
          minHeight: `${CARD_HEIGHT}px`,
        };

  return (
    <div
      role="treeitem"
      aria-selected={isSelected || isFocal}
      aria-label={`${displayName} (${node.gid})`}
      data-node-id={node.id}
      data-gid={node.gid}
      style={positionedStyle}
      onPointerDown={(e) => {
        e.stopPropagation();
        handlePointerDown();
      }}
      onPointerUp={clearLongPress}
      onPointerCancel={clearLongPress}
      onPointerLeave={clearLongPress}
      onClick={(e) => {
        e.stopPropagation();
        onSelect?.(node);
      }}
      className={cn(
        'group relative flex flex-col justify-between overflow-hidden rounded-xl border bg-surface p-2.5 text-left shadow-xs transition-all cursor-pointer select-none',
        isFocal || isSelected
          ? 'border-brand ring-2 ring-brand/30 shadow-md'
          : 'border-border hover:border-brand/60 hover:shadow-sm',
        className
      )}
    >
      {/* Gender Lineage Left Accent Bar */}
      <span
        aria-hidden="true"
        className={cn('absolute inset-y-0 left-0 w-1.5', stripeColorClass)}
      />

      <div className="pl-2 space-y-1.5">
        <div className="flex items-start justify-between gap-1.5">
          <div className="flex items-center gap-2 min-w-0">
            <Avatar
              name={isMaskedLiving ? '?' : displayName}
              src={isMaskedLiving ? undefined : node.avatar_url ?? undefined}
              size="sm"
            />
            <div className="min-w-0">
              <div className="font-sans text-xs font-semibold text-primary truncate">
                {displayName}
              </div>
              <div className="font-mono text-[10px] text-tertiary">
                {node.gid}
              </div>
            </div>
          </div>

          <div className="flex flex-wrap items-center justify-end gap-1 shrink-0">
            {isHomePerson && (
              <Badge
                variant="brand"
                size="sm"
                icon={<Home className="h-2.5 w-2.5" />}
                title={t('tree.homePersonBadge')}
              >
                {t('tree.homePersonShort')}
              </Badge>
            )}
            {node.is_private && !isMaskedLiving && (
              <Badge
                variant="warning"
                size="sm"
                icon={<Lock className="h-2.5 w-2.5" />}
              >
                {t('domain.privateBadge')}
              </Badge>
            )}
            {!node.is_living && (
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

        {/* Dates & Locations or Public Masked Notice */}
        {isMaskedLiving ? (
          <div className="flex items-center gap-1 text-[11px] text-tertiary">
            <Lock className="h-3 w-3 shrink-0" />
            <span className="truncate">
              {t('tree.privacyPreview.birthHiddenLabel')}
            </span>
          </div>
        ) : (
          <div className="space-y-0.5 text-[11px] text-secondary">
            {node.birth && (
              <div className="flex items-center gap-1 truncate">
                <Calendar className="h-3 w-3 text-tertiary shrink-0" />
                <span className="truncate">
                  * {node.birth.date}
                  {node.birth.place ? ` · ${node.birth.place}` : ''}
                </span>
              </div>
            )}
            {node.death && (
              <div className="flex items-center gap-1 truncate">
                <MapPin className="h-3 w-3 text-tertiary shrink-0" />
                <span className="truncate">
                  † {node.death.date}
                  {node.death.place ? ` · ${node.death.place}` : ''}
                </span>
              </div>
            )}
          </div>
        )}

        {/* Compact Tree Quick Actions (Focus & Quick-Add) */}
        <div
          className="flex items-center justify-end gap-1.5 border-t border-border/60 pt-1 opacity-85 group-hover:opacity-100 transition-opacity"
          onClick={(e) => e.stopPropagation()}
        >
          <Button
            size="sm"
            variant="ghost"
            className="h-6 px-2 text-[11px] text-secondary hover:text-primary"
            leftIcon={<Crosshair className="h-3 w-3" />}
            onClick={() => onFocusNode?.(node)}
          >
            {t('tree.card.focusBtn')}
          </Button>

          {canEdit && !isMaskedLiving && (
            <Button
              size="sm"
              variant="ghost"
              className="h-6 px-2 text-[11px] text-brand hover:text-brand"
              leftIcon={<UserPlus className="h-3 w-3" />}
              onClick={() => onQuickAdd?.(node, 'child')}
            >
              {t('tree.card.addBtn')}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
};
