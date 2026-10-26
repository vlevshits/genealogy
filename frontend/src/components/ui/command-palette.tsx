import React, { useState, useEffect, useRef } from 'react';
import { Search, MapPin, User, Zap } from 'lucide-react';
import { cn, matchesCrossScript } from '../../lib/utils';
import { Kbd } from './kbd';
import { useI18n } from '../../i18n';

export interface CommandPaletteItem {
  id: string;
  group: 'people' | 'places' | 'actions';
  title: string;
  subtitle?: string;
  shortcut?: string;
  icon?: React.ReactNode;
  onSelect: () => void;
}

export interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  items: CommandPaletteItem[];
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  open,
  onOpenChange,
  items,
}) => {
  const { t } = useI18n();
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    const onGlobalKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        onOpenChange(!open);
      } else if (e.key === 'Escape' && open) {
        onOpenChange(false);
      }
    };
    document.addEventListener('keydown', onGlobalKey);
    return () => document.removeEventListener('keydown', onGlobalKey);
  }, [open, onOpenChange]);

  useEffect(() => {
    if (open) {
      setQuery('');
      setActiveIndex(0);
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [open]);

  if (!open) return null;

  const filtered = items.filter((item) =>
    matchesCrossScript(`${item.title} ${item.subtitle ?? ''}`, query)
  );

  const groups: Array<{ key: 'people' | 'places' | 'actions'; label: string }> = [
    { key: 'people', label: t('omnibox.groupPeople') },
    { key: 'places', label: t('omnibox.groupPlaces') },
    { key: 'actions', label: t('omnibox.groupActions') },
  ];

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIndex((i) => (i + 1) % Math.max(1, filtered.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIndex((i) => (i - 1 + Math.max(1, filtered.length)) % Math.max(1, filtered.length));
    } else if (e.key === 'Enter' && filtered[activeIndex]) {
      e.preventDefault();
      filtered[activeIndex].onSelect();
      onOpenChange(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-16 sm:pt-24 px-4">
      <div
        aria-hidden="true"
        onClick={() => onOpenChange(false)}
        className="fixed inset-0 bg-primary/40 backdrop-blur-xs"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t('omnibox.title')}
        onKeyDown={handleKeyDown}
        className="relative z-10 w-full max-w-xl overflow-hidden rounded-2xl border border-border bg-surface shadow-2xl"
      >
        <div className="flex items-center border-b border-border px-4 py-3">
          <Search className="h-4 w-4 text-tertiary mr-3 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setActiveIndex(0);
            }}
            placeholder={t('omnibox.placeholder')}
            aria-label={t('omnibox.placeholder')}
            className="w-full bg-transparent text-sm text-primary placeholder:text-tertiary focus:outline-none"
          />
          <Kbd>Esc</Kbd>
        </div>

        <div className="max-h-80 overflow-y-auto p-2">
          {filtered.length === 0 ? (
            <div className="py-8 text-center text-xs text-secondary">{t('omnibox.empty')}</div>
          ) : (
            groups.map((grp) => {
              const grpItems = filtered.filter((i) => i.group === grp.key);
              if (grpItems.length === 0) return null;
              return (
                <div key={grp.key} className="mb-2 last:mb-0">
                  <div className="px-3 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
                    {grp.label}
                  </div>
                  {grpItems.map((item) => {
                    const globalIdx = filtered.indexOf(item);
                    const isActive = globalIdx === activeIndex;
                    const defaultIcon =
                      item.group === 'people' ? (
                        <User className="h-4 w-4 text-brand" />
                      ) : item.group === 'places' ? (
                        <MapPin className="h-4 w-4 text-accent" />
                      ) : (
                        <Zap className="h-4 w-4 text-secondary" />
                      );
                    return (
                      <button
                        key={item.id}
                        type="button"
                        onMouseEnter={() => setActiveIndex(globalIdx)}
                        onClick={() => {
                          item.onSelect();
                          onOpenChange(false);
                        }}
                        className={cn(
                          'flex w-full items-center justify-between gap-3 rounded-xl px-3 py-2 text-left text-xs sm:text-sm transition-colors cursor-pointer',
                          isActive ? 'bg-surface-muted text-primary' : 'text-primary'
                        )}
                      >
                        <div className="flex items-center gap-3 min-w-0">
                          {item.icon ?? defaultIcon}
                          <div className="min-w-0">
                            <div className="font-medium truncate">{item.title}</div>
                            {item.subtitle && (
                              <div className="text-[11px] text-secondary truncate">
                                {item.subtitle}
                              </div>
                            )}
                          </div>
                        </div>
                        {item.shortcut && <Kbd>{item.shortcut}</Kbd>}
                      </button>
                    );
                  })}
                </div>
              );
            })
          )}
        </div>

        <div className="flex items-center justify-between border-t border-border bg-surface-muted/50 px-4 py-2 text-[11px] text-tertiary">
          <span>{t('omnibox.hintNavigate')}</span>
          <span>{t('omnibox.hintSelect')}</span>
          <span>{t('omnibox.hintClose')}</span>
        </div>
      </div>
    </div>
  );
};
