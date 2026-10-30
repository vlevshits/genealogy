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
import { formatPersonName, PersonNameFormatMode } from '../../lib/formatters';
import { Locale } from '../../types/common';
import { cn } from '../../lib/utils';
import { CARD_HEIGHT, CARD_WIDTH } from '../../lib/tree-layouts';

export interface PersonNodeCardProps {
  node: TreePersonNodeDto;
  x?: number;
  y?: number;
  lod?: 0 | 1 | 2;
  isFocal?: boolean;
  isSelected?: boolean;
  isEdgeEndpointHighlighted?: boolean;
  isPulsing?: boolean;
  isHomePerson?: boolean;
  accessTier?: string;
  canEdit?: boolean;
  hasParentEdge?: boolean;
  hasChildEdge?: boolean;
  onSelect?: (node: TreePersonNodeDto) => void;
  onFocusNode?: (node: TreePersonNodeDto) => void;
  onQuickAdd?: (node: TreePersonNodeDto, relation: QuickAddRelation) => void;
  onEdit?: (node: TreePersonNodeDto) => void;
  className?: string;
}

const DRAG_THRESHOLD_PX = 6;

export function formatConciseSurnameInitials(
  node: TreePersonNodeDto,
  accessTier: string = 'admin'
): { displayName: string; isMaskedLiving: boolean } {
  const isPublicGuest = accessTier === 'public_anonymous';
  const rawGiven = (node.primary_name?.given_name || '').trim();
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
        : (node.primary_name?.surname || 'L').trim();
    const initial = sourceWord.charAt(0).toUpperCase() || 'L';
    return { displayName: `${initial}.`, isMaskedLiving: true };
  }

  const surname = (
    node.primary_name?.surname ||
    node.birth_surname ||
    ''
  ).trim();
  const given = (node.primary_name?.given_name || '').trim();
  const patronymic = (node.primary_name?.patronymic || '').trim();

  // If no surname, fall back to given name or gid
  if (!surname) {
    return {
      displayName: given || node.gid,
      isMaskedLiving: false,
    };
  }

  const initials: string[] = [];
  if (given) {
    const parts = given.split(/\s+/);
    for (const p of parts) {
      if (p.length > 0) {
        initials.push(`${p.charAt(0).toUpperCase()}.`);
      }
    }
  }
  if (patronymic) {
    const parts = patronymic.split(/\s+/);
    for (const p of parts) {
      if (p.length > 0) {
        initials.push(`${p.charAt(0).toUpperCase()}.`);
      }
    }
  }

  const initialsStr = initials.join(' ');
  const conciseName = initialsStr ? `${surname} ${initialsStr}` : surname;

  return {
    displayName: conciseName,
    isMaskedLiving: false,
  };
}

export function formatTreeCardName(
  node: TreePersonNodeDto,
  accessTier: string = 'admin',
  locale: Locale = 'en',
  format: PersonNameFormatMode = 'card'
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

  const fullName = formatPersonName(
    {
      given_name: node.primary_name.given_name,
      patronymic: node.primary_name.patronymic,
      surname: node.primary_name.surname,
      birth_surname: node.birth_surname,
    },
    {
      format,
      locale,
      showMaiden: false,
    }
  );

  return {
    displayName: fullName || node.gid,
    isMaskedLiving: false,
  };
}

export const PersonNodeCard: React.FC<PersonNodeCardProps> = ({
  node,
  x,
  y,
  lod = 0,
  isFocal = false,
  isSelected = false,
  isEdgeEndpointHighlighted = false,
  isPulsing = false,
  isHomePerson = false,
  accessTier = 'admin',
  canEdit = false,
  hasParentEdge = false,
  hasChildEdge = false,
  onSelect,
  onFocusNode,
  onQuickAdd,
  className,
}) => {
  const { t, locale } = useI18n();
  const longPressTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pointerStartRef = useRef<{ x: number; y: number } | null>(null);
  const wasDraggedRef = useRef<boolean>(false);

  const { displayName, isMaskedLiving } = formatTreeCardName(
    node,
    accessTier,
    locale,
    'card'
  );

  const stripeColorClass =
    node.sex === 'M'
      ? 'bg-male'
      : node.sex === 'F'
      ? 'bg-female'
      : 'bg-neutral';

  const endpointSuffix = isEdgeEndpointHighlighted
    ? ` - ${t('tree.connection.endpoint_badge')}`
    : '';

  const clearLongPress = () => {
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
  };

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    pointerStartRef.current = { x: e.clientX, y: e.clientY };
    wasDraggedRef.current = false;
    clearLongPress();
    longPressTimerRef.current = setTimeout(() => {
      if (wasDraggedRef.current) return;
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

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!pointerStartRef.current) return;
    const dist = Math.hypot(
      e.clientX - pointerStartRef.current.x,
      e.clientY - pointerStartRef.current.y
    );
    if (dist > DRAG_THRESHOLD_PX) {
      wasDraggedRef.current = true;
      clearLongPress();
    }
  };

  const handlePointerEnd = () => {
    clearLongPress();
    pointerStartRef.current = null;
  };

  // LOD 2: Micro-pill (28px high compact pill vertically centered in 120px height slot)
  if (lod === 2) {
    const slotStyle: React.CSSProperties | undefined =
      x !== undefined && y !== undefined
        ? {
            position: 'absolute',
            left: `${x}px`,
            top: `${y}px`,
            width: `${CARD_WIDTH}px`,
            height: `${CARD_HEIGHT}px`,
          }
        : {
            width: `${CARD_WIDTH}px`,
            height: `${CARD_HEIGHT}px`,
          };

    const { displayName: conciseName, isMaskedLiving: isLOD2Masked } =
      formatConciseSurnameInitials(node, accessTier);

    return (
      <div
        style={slotStyle}
        className="pointer-events-none relative flex h-[120px] items-center justify-center"
      >
        {hasParentEdge && (
          <div
            aria-hidden="true"
            data-testid="lod2-parent-stem"
            className={cn(
              'pointer-events-none absolute top-0 left-1/2 -translate-x-1/2 w-0.5 h-[46px]',
              node.is_direct_lineage ? 'bg-brand/80' : 'bg-secondary/70'
            )}
          />
        )}
        {hasChildEdge && (
          <div
            aria-hidden="true"
            data-testid="lod2-child-stem"
            className={cn(
              'pointer-events-none absolute bottom-0 left-1/2 -translate-x-1/2 w-0.5 h-[46px]',
              node.is_direct_lineage ? 'bg-brand/80' : 'bg-secondary/70'
            )}
          />
        )}
        <div
          role="treeitem"
          tabIndex={0}
          aria-selected={isSelected || isFocal}
          aria-label={`${displayName} (${node.gid})${endpointSuffix}`}
          title={isEdgeEndpointHighlighted ? t('tree.connection.endpoint_badge') : undefined}
          data-node-id={node.id}
          data-gid={node.gid}
          data-lod="2"
          data-connection-endpoint={isEdgeEndpointHighlighted ? 'true' : undefined}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerEnd}
          onPointerCancel={handlePointerEnd}
          onPointerLeave={handlePointerEnd}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              onSelect?.(node);
            }
          }}
          onClick={(e) => {
            e.stopPropagation();
            if (wasDraggedRef.current) {
              wasDraggedRef.current = false;
              return;
            }
            onSelect?.(node);
          }}
          className={cn(
            'group pointer-events-auto relative flex h-7 w-full items-center justify-between gap-1.5 overflow-hidden rounded-full border bg-surface/95 px-2.5 py-0.5 text-left shadow-xs transition-all cursor-pointer select-none focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-brand touch-target-expand antialiased',
            isPulsing && 'animate-pulse',
            isFocal || isSelected
              ? 'border-brand ring-2 ring-brand/30 shadow-md'
              : isEdgeEndpointHighlighted
              ? 'border-brand ring-2 ring-brand shadow-elevation-2'
              : isHomePerson
              ? 'border-brand/70 shadow-xs'
              : 'border-border-card hover:border-brand/60 hover:shadow-xs',
            className
          )}
        >
          <div className="flex items-center gap-1.5 min-w-0">
            <span
              aria-hidden="true"
              className={cn('h-2.5 w-2.5 rounded-full shrink-0', stripeColorClass)}
            />
            <span className="truncate text-xs font-medium text-primary">
              {conciseName}
            </span>
          </div>
          <div className="flex items-center gap-1 shrink-0">
            {isHomePerson && (
              <Home
                data-testid="lod2-home-icon"
                className="h-3 w-3 shrink-0 text-brand"
                aria-label={t('tree.homePersonShort')}
              />
            )}
            {node.is_private && !isLOD2Masked && (
              <Lock className="h-2.5 w-2.5 text-warning shrink-0" />
            )}
            {!node.is_living && (
              <span
                aria-label={t('person.quickAdd.livingNo')}
                title={t('person.quickAdd.livingNo')}
                className="text-[10px] text-tertiary"
              >
                †
              </span>
            )}
          </div>
        </div>
      </div>
    );
  }

  const extractYear = (dateStr?: string | null) => {
    if (!dateStr) return null;
    const match = dateStr.match(/\b\d{4}\b/);
    return match ? match[0] : null;
  };

  const birthYear = extractYear(node.birth?.date);
  const deathYear = extractYear(node.death?.date);

  const positionedStyle: React.CSSProperties | undefined =
    x !== undefined && y !== undefined
      ? {
          position: 'absolute',
          left: `${x}px`,
          top: `${y}px`,
          width: `${CARD_WIDTH}px`,
          height: `${CARD_HEIGHT}px`,
          maxHeight: `${CARD_HEIGHT}px`,
        }
      : {
          width: `${CARD_WIDTH}px`,
          height: `${CARD_HEIGHT}px`,
          maxHeight: `${CARD_HEIGHT}px`,
        };

  return (
    <div
      role="treeitem"
      tabIndex={0}
      aria-selected={isSelected || isFocal}
      aria-label={`${displayName} (${node.gid})${endpointSuffix}`}
      title={isEdgeEndpointHighlighted ? t('tree.connection.endpoint_badge') : undefined}
      data-node-id={node.id}
      data-gid={node.gid}
      data-lod={String(lod)}
      data-connection-endpoint={isEdgeEndpointHighlighted ? 'true' : undefined}
      style={positionedStyle}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerEnd}
      onPointerCancel={handlePointerEnd}
      onPointerLeave={handlePointerEnd}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onSelect?.(node);
        }
      }}
      onClick={(e) => {
        e.stopPropagation();
        if (wasDraggedRef.current) {
          wasDraggedRef.current = false;
          return;
        }
        onSelect?.(node);
      }}
      className={cn(
        'group relative flex flex-col justify-between overflow-hidden rounded-xl border bg-surface p-2 text-left shadow-xs transition-all cursor-pointer select-none focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-brand antialiased',
        isPulsing && 'animate-pulse',
        isFocal || isSelected
          ? 'border-brand ring-2 ring-brand/30 shadow-md'
          : isEdgeEndpointHighlighted
          ? 'border-brand ring-2 ring-brand shadow-elevation-2'
          : isHomePerson
          ? 'border-brand/70 shadow-xs'
          : 'border-border-card hover:border-brand/60 hover:shadow-sm',
        className
      )}
    >
      {/* Visual / screen reader endpoint indicator */}
      {isEdgeEndpointHighlighted && (
        <span className="sr-only">{t('tree.connection.endpoint_badge')}</span>
      )}

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
        ) : lod === 1 ? (
          /* LOD 1: Simplified vital years only */
          (birthYear || deathYear || !node.is_living) && (
            <div className="flex items-center gap-1 text-[11px] text-secondary truncate">
              <Calendar className="h-3 w-3 text-tertiary shrink-0" />
              <span className="truncate">
                {birthYear ? `* ${birthYear}` : ''}
                {birthYear && (deathYear || !node.is_living) ? '  ' : ''}
                {deathYear ? `† ${deathYear}` : !node.is_living ? '†' : ''}
              </span>
            </div>
          )
        ) : (
          /* LOD 0: Complete vital dates and locations */
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

        {/* LOD 0 Action Buttons: Compact Tree Quick Actions with size="xs" (h-6 py-0.5); suppressed in LOD 1 */}
        {lod === 0 && (
          <div
            className="flex items-center justify-end gap-1.5 border-t border-border/60 pt-0.5 opacity-90 group-hover:opacity-100 transition-opacity"
            onPointerDown={(e) => e.stopPropagation()}
            onClick={(e) => e.stopPropagation()}
          >
            <Button
              size="xs"
              variant="ghost"
              className="h-6 px-2 py-0.5 text-[11px] text-secondary hover:text-primary"
              leftIcon={<Crosshair className="h-3 w-3" />}
              onClick={() => onFocusNode?.(node)}
            >
              {t('tree.card.focusBtn')}
            </Button>

            {canEdit && !isMaskedLiving && (
              <Button
                size="xs"
                variant="ghost"
                className="h-6 px-2 py-0.5 text-[11px] text-brand hover:text-brand"
                leftIcon={<UserPlus className="h-3 w-3" />}
                onClick={() => onQuickAdd?.(node, 'child')}
              >
                {t('tree.card.addBtn')}
              </Button>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
