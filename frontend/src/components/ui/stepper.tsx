import React from 'react';
import { Check, AlertCircle } from 'lucide-react';
import { cn } from '../../lib/utils';

export type StepStatus = 'done' | 'current' | 'upcoming' | 'error';

export interface StepItem {
  id: string;
  title: string;
  description?: string;
  status?: StepStatus;
}

export interface StepperProps {
  steps: StepItem[];
  currentStepIndex: number;
  onStepClick?: (index: number) => void;
  className?: string;
}

export const Stepper: React.FC<StepperProps> = ({
  steps,
  currentStepIndex,
  onStepClick,
  className,
}) => {
  return (
    <ol aria-label="Progress steps" className={cn('flex flex-wrap items-center gap-4', className)}>
      {steps.map((step, idx) => {
        const status: StepStatus =
          step.status ??
          (idx < currentStepIndex ? 'done' : idx === currentStepIndex ? 'current' : 'upcoming');
        const canClick = Boolean(onStepClick && idx < currentStepIndex);

        return (
          <li key={step.id} className="flex items-center gap-3">
            <button
              type="button"
              disabled={!canClick}
              aria-current={status === 'current' ? 'step' : undefined}
              onClick={() => canClick && onStepClick?.(idx)}
              className={cn(
                'flex items-center gap-2.5 text-left rounded-lg p-1 transition-colors',
                canClick ? 'cursor-pointer hover:bg-surface-muted' : 'cursor-default'
              )}
            >
              <span
                className={cn(
                  'flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-semibold border transition-colors',
                  status === 'done' && 'border-success bg-success text-surface',
                  status === 'current' && 'border-brand bg-brand text-surface',
                  status === 'upcoming' && 'border-border bg-surface text-tertiary',
                  status === 'error' && 'border-danger bg-danger text-surface'
                )}
              >
                {status === 'done' ? (
                  <Check className="h-3.5 w-3.5" />
                ) : status === 'error' ? (
                  <AlertCircle className="h-3.5 w-3.5" />
                ) : (
                  idx + 1
                )}
              </span>
              <div>
                <div
                  className={cn(
                    'text-xs sm:text-sm font-medium',
                    status === 'upcoming' ? 'text-secondary' : 'text-primary'
                  )}
                >
                  {step.title}
                </div>
                {step.description && (
                  <div className="text-[11px] text-tertiary">{step.description}</div>
                )}
              </div>
            </button>
            {idx < steps.length - 1 && (
              <div className="hidden sm:block h-px w-8 bg-border" aria-hidden="true" />
            )}
          </li>
        );
      })}
    </ol>
  );
};
