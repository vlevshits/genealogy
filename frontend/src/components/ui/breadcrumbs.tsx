import React from 'react';
import { ChevronRight, MoreHorizontal } from 'lucide-react';
import { cn } from '../../lib/utils';
import { DropdownMenu } from './dropdown-menu';
import { useI18n } from '../../i18n';

export interface BreadcrumbItem {
  id: string;
  label: string;
  href?: string;
  onClick?: () => void;
}

export interface BreadcrumbsProps extends React.HTMLAttributes<HTMLElement> {
  items: BreadcrumbItem[];
  maxVisible?: number;
  ref?: React.Ref<HTMLElement>;
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({
  items,
  maxVisible = 4,
  className,
  ref,
  ...props
}) => {
  const { t } = useI18n();
  if (items.length === 0) return null;

  const shouldCollapse = items.length > maxVisible;
  const firstItem = items[0];
  const hiddenMiddle = shouldCollapse ? items.slice(1, items.length - 2) : [];
  const trailingItems = shouldCollapse ? items.slice(items.length - 2) : items.slice(1);

  return (
    <nav
      ref={ref}
      aria-label={t('nav.breadcrumbsAria')}
      className={cn('flex items-center flex-wrap gap-1.5 text-xs sm:text-sm', className)}
      {...props}
    >
      <ol className="flex items-center flex-wrap gap-1.5">
        <li className="inline-flex items-center">
          {items.length === 1 ? (
            <span aria-current="page" className="font-semibold text-primary">
              {firstItem.label}
            </span>
          ) : (
            <a
              href={firstItem.href || '#'}
              onClick={(e) => {
                if (firstItem.onClick) {
                  e.preventDefault();
                  firstItem.onClick();
                }
              }}
              className="text-secondary hover:text-primary transition-colors"
            >
              {firstItem.label}
            </a>
          )}
        </li>

        {shouldCollapse && hiddenMiddle.length > 0 && (
          <li className="inline-flex items-center gap-1.5">
            <ChevronRight className="h-3.5 w-3.5 text-tertiary" />
            <DropdownMenu
              ariaLabel={t('nav.moreBreadcrumbs')}
              trigger={
                <button
                  type="button"
                  aria-label={t('nav.moreBreadcrumbs')}
                  className="inline-flex h-6 w-6 items-center justify-center rounded-md border border-border bg-surface text-secondary hover:bg-surface-muted"
                >
                  <MoreHorizontal className="h-3.5 w-3.5" />
                </button>
              }
              items={hiddenMiddle.map((m) => ({
                id: m.id,
                label: m.label,
                onSelect: m.onClick,
              }))}
            />
          </li>
        )}

        {trailingItems.map((item, idx) => {
          const isLast = idx === trailingItems.length - 1;
          return (
            <li key={item.id} className="inline-flex items-center gap-1.5">
              <ChevronRight className="h-3.5 w-3.5 text-tertiary shrink-0" />
              {isLast ? (
                <span aria-current="page" className="font-semibold text-primary truncate max-w-60">
                  {item.label}
                </span>
              ) : (
                <a
                  href={item.href || '#'}
                  onClick={(e) => {
                    if (item.onClick) {
                      e.preventDefault();
                      item.onClick();
                    }
                  }}
                  className="text-secondary hover:text-primary transition-colors truncate max-w-48"
                >
                  {item.label}
                </a>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
};
