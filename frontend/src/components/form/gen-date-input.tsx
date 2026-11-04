import React, { useState, useEffect } from 'react';
import {
  Calendar as CalendarIcon,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Sparkles,
} from 'lucide-react';
import { cn } from '../../lib/utils';
import {
  GenDate,
  GenDateModifier,
  ParsedGenDateResult,
} from '../../types/gendate';
import {
  computeGenDateSortKey,
  convertJulianToGregorian,
  daysInMonthGregorian,
  formatGenDate,
  formatPartialDate,
  parseGenDate,
} from '../../lib/gendate-parser';
import { DropdownMenu } from '../ui/dropdown-menu';
import { Popover } from '../ui/popover';
import { Checkbox } from '../ui/checkbox';
import { Button } from '../ui/button';
import { IconButton } from '../ui/icon-button';
import { TextInput } from '../ui/input';
import { useI18n } from '../../i18n';

export interface GenDateInputProps {
  value?: GenDate | null;
  defaultValue?: GenDate | null;
  onChange?: (result: ParsedGenDateResult) => void;
  placeholder?: string;
  disabled?: boolean;
  invalid?: boolean;
  id?: string;
  className?: string;
}

const MODIFIER_LIST: Array<Exclude<GenDateModifier, 'phrase'>> = [
  'exact',
  'about',
  'estimated',
  'calculated',
  'before',
  'after',
  'between',
  'from_to',
];

export const GenDateInput: React.FC<GenDateInputProps> = ({
  value,
  defaultValue = null,
  onChange,
  placeholder,
  disabled = false,
  invalid = false,
  id,
  className,
}) => {
  const { t, locale } = useI18n();
  const initialDate = value ?? defaultValue;

  const [textInput, setTextInput] = useState<string>(initialDate?.text ?? '');
  const [secondDateText, setSecondDateText] = useState<string>(
    initialDate?.end ? String(initialDate.end.y) : ''
  );
  const [parsedResult, setParsedResult] = useState<ParsedGenDateResult>(() =>
    initialDate
      ? {
          value: initialDate,
          sortKey: computeGenDateSortKey(initialDate),
          isValid: true,
        }
      : parseGenDate('', locale)
  );

  // Calendar popover state with fast century/decade navigation
  const [calendarOpen, setCalendarOpen] = useState(false);
  const [calYear, setCalYear] = useState<number>(
    initialDate?.start?.y ?? 1912
  );
  const [calMonth, setCalMonth] = useState<number>(
    initialDate?.start?.m ?? 7
  );

  useEffect(() => {
    if (value && value.text !== textInput) {
      setTextInput(value.text);
      setParsedResult({
        value,
        sortKey: computeGenDateSortKey(value),
        isValid: true,
      });
      if (value.start?.y) setCalYear(value.start.y);
      if (value.start?.m) setCalMonth(value.start.m);
    } else if (value === null && textInput !== '') {
      setTextInput('');
      setSecondDateText('');
      setParsedResult(parseGenDate('', locale));
    }
  }, [value]);

  const emitAndSet = (nextResult: ParsedGenDateResult, updateRawText = true) => {
    setParsedResult(nextResult);
    if (updateRawText) {
      setTextInput(nextResult.value.text);
    }
    onChange?.(nextResult);
  };

  const handleTextChange = (raw: string) => {
    setTextInput(raw);
    const res = parseGenDate(raw, locale);
    setParsedResult(res);
    if (res.value.start?.y) setCalYear(res.value.start.y);
    if (res.value.start?.m) setCalMonth(res.value.start.m);
    onChange?.(res);
  };

  const handleModifierSelect = (mod: Exclude<GenDateModifier, 'phrase'>) => {
    const current = parsedResult.value;
    const start = current.start ?? { y: calYear, m: null, d: null };
    const end =
      mod === 'between' || mod === 'from_to'
        ? current.end ?? { y: start.y + 4, m: null, d: null }
        : null;

    const nextDate: GenDate = {
      mod,
      start,
      end,
      cal: current.cal,
      text: '',
    };
    nextDate.text = formatGenDate(nextDate, locale);
    if (end) setSecondDateText(String(end.y));
    emitAndSet({
      value: nextDate,
      sortKey: computeGenDateSortKey(nextDate),
      isValid: true,
    });
  };

  const handleSecondDateChange = (rawEnd: string) => {
    setSecondDateText(rawEnd);
    const parsedEnd = parseGenDate(rawEnd, locale);
    if (parsedEnd.value.start && parsedResult.value.start) {
      const nextDate: GenDate = {
        ...parsedResult.value,
        end: parsedEnd.value.start,
        text: '',
      };
      nextDate.text = formatGenDate(nextDate, locale);
      emitAndSet({
        value: nextDate,
        sortKey: computeGenDateSortKey(nextDate),
        isValid: true,
      });
    }
  };

  const handleCalendarToggle = (isJulian: boolean) => {
    const nextCal = isJulian ? 'julian' : 'gregorian';
    const nextDate: GenDate = {
      ...parsedResult.value,
      cal: nextCal,
      text: '',
    };
    nextDate.text = formatGenDate(nextDate, locale);
    emitAndSet({
      value: nextDate,
      sortKey: computeGenDateSortKey(nextDate),
      isValid: true,
    });
  };

  const applyCalendarPick = (y: number, m: number | null, d: number | null) => {
    const currentMod =
      parsedResult.value.mod === 'phrase' ? 'exact' : parsedResult.value.mod;
    const nextDate: GenDate = {
      mod: currentMod,
      start: { y, m, d },
      end: parsedResult.value.end,
      cal: parsedResult.value.cal,
      text: '',
    };
    nextDate.text = formatGenDate(nextDate, locale);
    emitAndSet({
      value: nextDate,
      sortKey: computeGenDateSortKey(nextDate),
      isValid: true,
    });
    setCalendarOpen(false);
  };

  const gd = parsedResult.value;
  const isRange = gd.mod === 'between' || gd.mod === 'from_to';
  const isPhrase = gd.mod === 'phrase' && Boolean(gd.text);
  const showJulianControl =
    Boolean(gd.start && gd.start.y <= 1918) || gd.cal === 'julian';

  const daysInSelectedMonth = daysInMonthGregorian(calYear, calMonth);
  const monthLabel = t(`gendate.months.${calMonth}` as string);

  return (
    <div className={cn('space-y-2', className)}>
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
        <div className="relative flex-1 flex items-center">
          <TextInput
            id={id}
            type="text"
            value={textInput}
            disabled={disabled}
            invalid={invalid}
            placeholder={placeholder || t('gendate.placeholder')}
            onChange={(e) => handleTextChange(e.target.value)}
            className="pr-20 font-mono text-xs sm:text-sm placeholder:font-sans"
          />

          <div className="absolute right-1.5 flex items-center gap-1">
            {/* Calendar Popover with Fast Century/Decade Jump */}
            <Popover
              open={calendarOpen}
              onOpenChange={setCalendarOpen}
              align="end"
              className="w-76 p-3"
              trigger={
                <IconButton
                  size="sm"
                  variant="ghost"
                  disabled={disabled}
                  label={t('gendate.openCalendar')}
                  className="h-7 w-7 min-h-0 min-w-0 text-secondary hover:text-primary"
                >
                  <CalendarIcon className="h-4 w-4" />
                </IconButton>
              }
            >
              <div className="space-y-3">
                {/* Fast Century & Decade Jump Controls */}
                <div className="flex items-center justify-between gap-1 border-b border-border pb-2">
                  <IconButton
                    size="sm"
                    variant="ghost"
                    label={t('gendate.prevCentury')}
                    onClick={() => setCalYear((y) => Math.max(1000, y - 100))}
                    className="h-7 w-7 min-h-0 min-w-0 text-secondary hover:text-primary"
                  >
                    <ChevronsLeft className="h-4 w-4" />
                  </IconButton>
                  <IconButton
                    size="sm"
                    variant="ghost"
                    label={t('gendate.prevDecade')}
                    onClick={() => setCalYear((y) => Math.max(1000, y - 10))}
                    className="h-7 w-7 min-h-0 min-w-0 text-secondary hover:text-primary"
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </IconButton>

                  <div className="w-20">
                    <TextInput
                      type="number"
                      size="sm"
                      aria-label={t('gendate.yearLabel')}
                      value={calYear}
                      onChange={(e) => {
                        const val = parseInt(e.target.value, 10);
                        if (!Number.isNaN(val)) setCalYear(val);
                      }}
                      className="text-center font-mono text-xs font-bold px-1.5 py-1"
                    />
                  </div>

                  <IconButton
                    size="sm"
                    variant="ghost"
                    label={t('gendate.nextDecade')}
                    onClick={() => setCalYear((y) => Math.min(2099, y + 10))}
                    className="h-7 w-7 min-h-0 min-w-0 text-secondary hover:text-primary"
                  >
                    <ChevronRight className="h-4 w-4" />
                  </IconButton>
                  <IconButton
                    size="sm"
                    variant="ghost"
                    label={t('gendate.nextCentury')}
                    onClick={() => setCalYear((y) => Math.min(2099, y + 100))}
                    className="h-7 w-7 min-h-0 min-w-0 text-secondary hover:text-primary"
                  >
                    <ChevronsRight className="h-4 w-4" />
                  </IconButton>
                </div>

                {/* Month Grid */}
                <div className="grid grid-cols-4 gap-1">
                  {Array.from({ length: 12 }, (_, idx) => idx + 1).map((m) => (
                    <Button
                      key={m}
                      size="sm"
                      variant={calMonth === m ? 'primary' : 'secondary'}
                      onClick={() => setCalMonth(m)}
                      className="h-7 min-h-0 px-1 py-1 text-[11px]"
                    >
                      {t(`gendate.months.${m}`).slice(0, 3)}
                    </Button>
                  ))}
                </div>

                {/* Day Grid */}
                <div className="grid grid-cols-7 gap-1 pt-1">
                  {Array.from({ length: daysInSelectedMonth }, (_, i) => i + 1).map((d) => {
                    const isSelectedDay =
                      gd.start?.y === calYear &&
                      gd.start?.m === calMonth &&
                      gd.start?.d === d;
                    return (
                      <Button
                        key={d}
                        size="sm"
                        variant={isSelectedDay ? 'primary' : 'ghost'}
                        onClick={() => applyCalendarPick(calYear, calMonth, d)}
                        className="h-7 w-7 min-h-0 min-w-0 p-0 font-mono text-xs"
                      >
                        {d}
                      </Button>
                    );
                  })}
                </div>

                {/* Partial Date Shortcuts (Year only / Month + Year) */}
                <div className="flex items-center justify-between gap-2 border-t border-border pt-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => applyCalendarPick(calYear, null, null)}
                  >
                    {t('gendate.pickYearOnly', { year: calYear })}
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => applyCalendarPick(calYear, calMonth, null)}
                  >
                    {t('gendate.pickMonthYear', { month: monthLabel.slice(0, 3), year: calYear })}
                  </Button>
                </div>
              </div>
            </Popover>

            {/* Modifier Menu (▾) */}
            <DropdownMenu
              ariaLabel={t('gendate.selectModifier')}
              trigger={
                <span
                  title={t('gendate.selectModifier')}
                  className="inline-flex h-7 items-center gap-0.5 rounded-md border border-border bg-surface-muted px-1.5 text-[11px] font-medium text-secondary hover:text-primary"
                >
                  <span>{gd.mod.toUpperCase().slice(0, 3)}</span>
                  <ChevronDown className="h-3 w-3" />
                </span>
              }
              items={MODIFIER_LIST.map((m) => ({
                id: m,
                label: t(`gendate.modifiers.${m}`),
                checked: gd.mod === m,
                onSelect: () => handleModifierSelect(m),
              }))}
            />
          </div>
        </div>

        {/* Second field shown when Between or From–To is selected */}
        {isRange && (
          <div className="sm:w-44">
            <TextInput
              type="text"
              value={secondDateText}
              aria-label={t('gendate.secondDatePlaceholder')}
              placeholder={t('gendate.secondDatePlaceholder')}
              onChange={(e) => handleSecondDateChange(e.target.value)}
              className="font-mono text-xs sm:text-sm placeholder:font-sans"
            />
          </div>
        )}
      </div>

      {/* Live Parse Preview & Julian Calendar Toggle */}
      {textInput.trim().length > 0 && (
        <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border/70 bg-surface-muted/50 px-3 py-1.5 text-xs">
          <div className="flex items-center gap-2">
            <Sparkles className="h-3.5 w-3.5 text-brand shrink-0" />
            {isPhrase ? (
              <span className="text-warning font-medium">
                {t('gendate.phraseWarning')}: «{gd.text}»
              </span>
            ) : (
              <span className="text-secondary">
                {t('gendate.interpretedAs')}{' '}
                <strong className="text-primary">{formatGenDate(gd, locale)}</strong>
                {parsedResult.sortKey !== null && (
                  <span className="ml-2 font-mono text-[11px] text-tertiary">
                    [sort: {parsedResult.sortKey}]
                  </span>
                )}
              </span>
            )}
          </div>

          {showJulianControl && !isPhrase && (
            <div className="flex items-center gap-3">
              <Checkbox
                checked={gd.cal === 'julian'}
                onCheckedChange={handleCalendarToggle}
                label={t('gendate.julianToggle')}
              />
              {gd.cal === 'julian' && gd.start?.m && gd.start?.d && (
                <span className="font-mono text-[11px] text-brand">
                  {t('gendate.convertedGregorian', {
                    date: formatPartialDate(convertJulianToGregorian(gd.start), locale),
                  })}
                </span>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
