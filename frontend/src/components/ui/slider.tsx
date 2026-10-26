import React, { useState } from 'react';
import { cn } from '../../lib/utils';

export interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  label?: string;
  disabled?: boolean;
  onChange?: (value: number) => void;
  className?: string;
}

export const Slider: React.FC<SliderProps> = ({
  value,
  defaultValue = 10,
  min = 0,
  max = 100,
  step = 1,
  unit = '',
  label,
  disabled = false,
  onChange,
  className,
}) => {
  const [internalVal, setInternalVal] = useState(defaultValue);
  const current = value !== undefined ? value : internalVal;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const next = Number(e.target.value);
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  return (
    <div className={cn('w-full space-y-1.5', disabled && 'opacity-50', className)}>
      <div className="flex items-center justify-between text-xs">
        <span className="text-secondary">{label || `${min}${unit}`}</span>
        <span className="rounded bg-surface-muted px-2 py-0.5 font-mono font-semibold text-primary border border-border">
          {current}
          {unit}
        </span>
        <span className="text-secondary">
          {max}
          {unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={current}
        disabled={disabled}
        aria-label={label || 'Value slider'}
        onChange={handleChange}
        className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-surface-muted accent-brand focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
      />
    </div>
  );
};

export interface RangeSliderProps {
  value?: [number, number];
  defaultValue?: [number, number];
  min?: number;
  max?: number;
  step?: number;
  label?: string;
  onChange?: (range: [number, number]) => void;
  className?: string;
}

export const RangeSlider: React.FC<RangeSliderProps> = ({
  value,
  defaultValue = [1850, 1950],
  min = 1700,
  max = 2026,
  step = 1,
  label,
  onChange,
  className,
}) => {
  const [internalRange, setInternalRange] = useState<[number, number]>(defaultValue);
  const [start, end] = value !== undefined ? value : internalRange;

  const update = (nextStart: number, nextEnd: number) => {
    const clampedStart = Math.max(min, Math.min(nextStart, nextEnd));
    const clampedEnd = Math.min(max, Math.max(nextEnd, clampedStart));
    const next: [number, number] = [clampedStart, clampedEnd];
    if (value === undefined) setInternalRange(next);
    onChange?.(next);
  };

  return (
    <div className={cn('space-y-2', className)}>
      {label && <div className="text-xs font-medium text-secondary">{label}</div>}
      <div className="flex items-center gap-3">
        <input
          type="number"
          min={min}
          max={end}
          step={step}
          value={start}
          aria-label={`${label || 'Range'} min`}
          onChange={(e) => update(Number(e.target.value), end)}
          className="w-20 rounded-lg border border-border bg-surface px-2.5 py-1.5 font-mono text-xs text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
        />
        <div className="flex flex-1 items-center gap-2">
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={start}
            aria-label={`${label || 'Range'} start slider`}
            onChange={(e) => update(Number(e.target.value), end)}
            className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-surface-muted accent-brand"
          />
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={end}
            aria-label={`${label || 'Range'} end slider`}
            onChange={(e) => update(start, Number(e.target.value))}
            className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-surface-muted accent-brand"
          />
        </div>
        <input
          type="number"
          min={start}
          max={max}
          step={step}
          value={end}
          aria-label={`${label || 'Range'} max`}
          onChange={(e) => update(start, Number(e.target.value))}
          className="w-20 rounded-lg border border-border bg-surface px-2.5 py-1.5 font-mono text-xs text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
        />
      </div>
    </div>
  );
};
