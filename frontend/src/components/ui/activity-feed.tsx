import React from 'react';
import { Avatar } from './avatar';
import { cn } from '../../lib/utils';

export interface ActivityFeedEntry {
  id: string;
  dayGroup: string;
  actorName: string;
  summary: React.ReactNode;
  timestamp: string;
  action?: React.ReactNode;
}

export interface ActivityFeedProps {
  entries: ActivityFeedEntry[];
  className?: string;
}

export const ActivityFeed: React.FC<ActivityFeedProps> = ({ entries, className }) => {
  const days = Array.from(new Set(entries.map((e) => e.dayGroup)));

  return (
    <div className={cn('space-y-6', className)}>
      {days.map((day) => {
        const groupEntries = entries.filter((e) => e.dayGroup === day);
        return (
          <div key={day} className="space-y-2.5">
            <h4 className="text-xs font-semibold uppercase tracking-wider text-tertiary">
              {day}
            </h4>
            <div className="divide-y divide-border rounded-xl border border-border bg-surface">
              {groupEntries.map((entry) => (
                <div
                  key={entry.id}
                  className="flex items-center justify-between gap-3 px-4 py-3 text-xs sm:text-sm"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <Avatar name={entry.actorName} size="sm" />
                    <div className="min-w-0">
                      <div className="text-primary">
                        <span className="font-semibold">{entry.actorName}</span>{' '}
                        <span className="text-secondary">{entry.summary}</span>
                      </div>
                      <div className="text-[11px] font-mono text-tertiary">{entry.timestamp}</div>
                    </div>
                  </div>
                  {entry.action && <div className="shrink-0">{entry.action}</div>}
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
};
