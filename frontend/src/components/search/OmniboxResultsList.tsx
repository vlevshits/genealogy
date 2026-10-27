import React from 'react';
import {
  BookOpen,
  Clock,
  FileText,
  GitBranch,
  History,
  Image as ImageIcon,
  MapPin,
  Settings,
  Trash2,
  Upload,
  User,
  UserPlus,
  Users,
  Zap,
} from 'lucide-react';
import {
  SearchActionDto,
  SearchMediaResultDto,
  SearchPersonResultDto,
  SearchPlaceResultDto,
} from '../../api/client';
import { Avatar, Badge, Button, Kbd } from '../ui';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface RecentPersonEntry {
  id: string;
  gid: string;
  display_name: string;
  sex?: string;
  birth_year?: number | null;
  death_year?: number | null;
  birth_place?: string | null;
  avatar_url?: string | null;
  is_living?: boolean;
}

export type OmniboxSelectableItem =
  | {
      kind: 'person';
      key: string;
      person: SearchPersonResultDto | RecentPersonEntry;
    }
  | {
      kind: 'place';
      key: string;
      place: SearchPlaceResultDto;
    }
  | {
      kind: 'media';
      key: string;
      media: SearchMediaResultDto;
    }
  | {
      kind: 'action';
      key: string;
      action: SearchActionDto;
    }
  | {
      kind: 'recent-search';
      key: string;
      queryText: string;
    };

export interface OmniboxResultsListProps {
  query: string;
  loading?: boolean;
  people: SearchPersonResultDto[];
  places: SearchPlaceResultDto[];
  media: SearchMediaResultDto[];
  actions: SearchActionDto[];
  recentPeople: RecentPersonEntry[];
  recentSearches: string[];
  selectableItems: OmniboxSelectableItem[];
  activeIndex: number;
  onActiveIndexChange: (index: number) => void;
  onSelectItem: (item: OmniboxSelectableItem, focusInTree?: boolean) => void;
  onClearRecent?: () => void;
}

function renderActionIcon(iconName: string): React.ReactNode {
  switch (iconName) {
    case 'git-branch':
      return <GitBranch className="h-4 w-4 text-brand" />;
    case 'users':
      return <Users className="h-4 w-4 text-brand" />;
    case 'book-open':
      return <BookOpen className="h-4 w-4 text-brand" />;
    case 'map-pin':
      return <MapPin className="h-4 w-4 text-accent" />;
    case 'user-plus':
      return <UserPlus className="h-4 w-4 text-brand" />;
    case 'upload':
      return <Upload className="h-4 w-4 text-secondary" />;
    case 'settings':
      return <Settings className="h-4 w-4 text-secondary" />;
    default:
      return <Zap className="h-4 w-4 text-secondary" />;
  }
}

export const OmniboxResultsList: React.FC<OmniboxResultsListProps> = ({
  query,
  loading = false,
  people,
  places,
  media,
  actions,
  recentPeople,
  recentSearches,
  selectableItems,
  activeIndex,
  onActiveIndexChange,
  onSelectItem,
  onClearRecent,
}) => {
  const { t } = useI18n();
  const isQueryEmpty = query.trim().length === 0;

  const findSelectableIndex = (key: string): number =>
    selectableItems.findIndex((item) => item.key === key);

  const formatPersonLifespan = (
    p: SearchPersonResultDto | RecentPersonEntry
  ): string => {
    const parts: string[] = [];
    if (p.birth_year) {
      parts.push(`${t('domain.bornPrefix')} ${p.birth_year}`);
    }
    if (!p.is_living && p.death_year) {
      parts.push(`† ${p.death_year}`);
    }
    if (p.birth_place) {
      parts.push(p.birth_place);
    }
    return parts.join(' · ');
  };

  if (!isQueryEmpty && selectableItems.length === 0 && !loading) {
    return (
      <div className="py-10 px-4 text-center space-y-1">
        <p className="text-sm font-medium text-primary">{t('omnibox.empty')}</p>
        <p className="text-xs text-secondary">{t('omnibox.emptyHint')}</p>
      </div>
    );
  }

  return (
    <div className="max-h-96 overflow-y-auto p-2 space-y-3" role="listbox" aria-label={t('omnibox.title')}>
      {/* Empty query state: Recently Viewed People & Recent Searches */}
      {isQueryEmpty && recentPeople.length > 0 && (
        <div className="space-y-1">
          <div className="flex items-center justify-between px-3 py-1">
            <span className="inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
              <Clock className="h-3 w-3" />
              <span>{t('omnibox.recentPeople')}</span>
            </span>
            {onClearRecent && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onClearRecent}
                leftIcon={<Trash2 className="h-3 w-3" />}
                className="h-6 min-h-0 px-2 py-0.5 text-[11px] text-tertiary hover:text-primary"
              >
                {t('omnibox.clearRecent')}
              </Button>
            )}
          </div>
          {recentPeople.map((person) => {
            const key = `recent-person-${person.id}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            const lifespan = formatPersonLifespan(person);
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
                className={cn(
                  'flex items-center justify-between gap-2 rounded-xl px-2.5 py-1.5 transition-colors',
                  isActive ? 'bg-surface-muted' : 'hover:bg-surface-muted/60'
                )}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    onSelectItem({ kind: 'person', key, person }, false)
                  }
                  className="flex-1 justify-start gap-3 px-1 py-1 text-left font-normal min-w-0"
                >
                  <Avatar
                    name={person.display_name}
                    src={person.avatar_url ?? undefined}
                    size="sm"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block text-sm font-medium text-primary truncate">
                      {person.display_name}
                    </span>
                    {lifespan && (
                      <span className="block text-xs text-secondary truncate">
                        {lifespan}
                      </span>
                    )}
                  </span>
                </Button>
                <div className="flex items-center gap-1.5 shrink-0">
                  <Badge variant="neutral">{person.gid}</Badge>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() =>
                      onSelectItem({ kind: 'person', key, person }, true)
                    }
                    leftIcon={<GitBranch className="h-3 w-3" />}
                    className="h-7 min-h-0 px-2 text-xs text-secondary hover:text-primary"
                  >
                    {t('omnibox.focusInTree')}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {isQueryEmpty && recentSearches.length > 0 && (
        <div className="space-y-1">
          <div className="flex items-center justify-between px-3 py-1">
            <span className="inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
              <History className="h-3 w-3" />
              <span>{t('omnibox.recentSearches')}</span>
            </span>
            {onClearRecent && recentPeople.length === 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onClearRecent}
                leftIcon={<Trash2 className="h-3 w-3" />}
                className="h-6 min-h-0 px-2 py-0.5 text-[11px] text-tertiary hover:text-primary"
              >
                {t('omnibox.clearRecent')}
              </Button>
            )}
          </div>
          {recentSearches.map((qText) => {
            const key = `recent-search-${qText}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    onSelectItem({ kind: 'recent-search', key, queryText: qText })
                  }
                  leftIcon={<History className="h-3.5 w-3.5 text-tertiary" />}
                  className={cn(
                    'w-full justify-start rounded-xl px-3 py-2 text-left text-sm font-normal',
                    isActive ? 'bg-surface-muted text-primary' : 'text-secondary'
                  )}
                >
                  {qText}
                </Button>
              </div>
            );
          })}
        </div>
      )}

      {/* Search Results: People */}
      {!isQueryEmpty && people.length > 0 && (
        <div className="space-y-1">
          <div className="px-3 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
            {t('omnibox.groupPeople')}
          </div>
          {people.map((person) => {
            const key = `person-${person.id}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            const lifespan = formatPersonLifespan(person);
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
                className={cn(
                  'flex items-center justify-between gap-2 rounded-xl px-2.5 py-1.5 transition-colors',
                  isActive ? 'bg-surface-muted' : 'hover:bg-surface-muted/60'
                )}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    onSelectItem({ kind: 'person', key, person }, false)
                  }
                  className="flex-1 justify-start gap-3 px-1 py-1 text-left font-normal min-w-0"
                >
                  {person.avatar_url ? (
                    <Avatar
                      name={person.display_name}
                      src={person.avatar_url}
                      size="sm"
                    />
                  ) : (
                    <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand/15 text-brand">
                      <User className="h-4 w-4" />
                    </span>
                  )}
                  <span className="min-w-0 flex-1">
                    <span className="block text-sm font-medium text-primary truncate">
                      {person.display_name}
                    </span>
                    {lifespan && (
                      <span className="block text-xs text-secondary truncate">
                        {lifespan}
                      </span>
                    )}
                  </span>
                </Button>
                <div className="flex items-center gap-1.5 shrink-0">
                  <Badge variant="neutral">{person.gid}</Badge>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() =>
                      onSelectItem({ kind: 'person', key, person }, true)
                    }
                    leftIcon={<GitBranch className="h-3 w-3" />}
                    rightIcon={<Kbd className="hidden sm:inline-flex">⇧↵</Kbd>}
                    className="h-7 min-h-0 px-2 text-xs text-secondary hover:text-primary"
                  >
                    <span className="hidden md:inline">
                      {t('omnibox.focusInTree')}
                    </span>
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Search Results: Places */}
      {!isQueryEmpty && places.length > 0 && (
        <div className="space-y-1">
          <div className="px-3 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
            {t('omnibox.groupPlaces')}
          </div>
          {places.map((place) => {
            const key = `place-${place.id}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onSelectItem({ kind: 'place', key, place })}
                  className={cn(
                    'w-full justify-between gap-3 rounded-xl px-3 py-2 text-left font-normal',
                    isActive ? 'bg-surface-muted text-primary' : 'text-primary'
                  )}
                >
                  <span className="flex items-center gap-3 min-w-0">
                    <MapPin className="h-4 w-4 text-accent shrink-0" />
                    <span className="min-w-0">
                      <span className="block text-sm font-medium truncate">
                        {place.name}
                      </span>
                      <span className="block text-xs text-secondary truncate">
                        {place.full_name}
                      </span>
                    </span>
                  </span>
                  <Badge variant="neutral">{place.gid}</Badge>
                </Button>
              </div>
            );
          })}
        </div>
      )}

      {/* Search Results: Media */}
      {!isQueryEmpty && media.length > 0 && (
        <div className="space-y-1">
          <div className="px-3 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
            {t('omnibox.groupMedia')}
          </div>
          {media.map((item) => {
            const key = `media-${item.id}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            const isPdf = item.mime_type.includes('pdf');
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    onSelectItem({ kind: 'media', key, media: item })
                  }
                  className={cn(
                    'w-full justify-between gap-3 rounded-xl px-3 py-2 text-left font-normal',
                    isActive ? 'bg-surface-muted text-primary' : 'text-primary'
                  )}
                >
                  <span className="flex items-center gap-3 min-w-0">
                    {isPdf ? (
                      <FileText className="h-4 w-4 text-secondary shrink-0" />
                    ) : (
                      <ImageIcon className="h-4 w-4 text-brand shrink-0" />
                    )}
                    <span className="text-sm font-medium truncate">
                      {item.title}
                    </span>
                  </span>
                  <Badge variant="neutral">{item.gid}</Badge>
                </Button>
              </div>
            );
          })}
        </div>
      )}

      {/* Quick Actions */}
      {actions.length > 0 && (
        <div className="space-y-1">
          <div className="px-3 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
            {t('omnibox.groupActions')}
          </div>
          {actions.map((action) => {
            const key = `action-${action.id}`;
            const idx = findSelectableIndex(key);
            const isActive = idx === activeIndex;
            return (
              <div
                key={key}
                role="option"
                aria-selected={isActive}
                onMouseEnter={() => {
                  if (idx >= 0) onActiveIndexChange(idx);
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onSelectItem({ kind: 'action', key, action })}
                  className={cn(
                    'w-full justify-between gap-3 rounded-xl px-3 py-2 text-left font-normal',
                    isActive ? 'bg-surface-muted text-primary' : 'text-primary'
                  )}
                >
                  <span className="flex items-center gap-3 min-w-0">
                    {renderActionIcon(action.icon)}
                    <span className="min-w-0">
                      <span className="block text-sm font-medium truncate">
                        {action.label}
                      </span>
                      {action.hint && (
                        <span className="block text-xs text-secondary truncate">
                          {action.hint}
                        </span>
                      )}
                    </span>
                  </span>
                  <Kbd>↵</Kbd>
                </Button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
