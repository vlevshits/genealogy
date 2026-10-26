export type GenDateModifier =
  | 'exact'
  | 'about'
  | 'estimated'
  | 'calculated'
  | 'before'
  | 'after'
  | 'between'
  | 'from_to'
  | 'phrase';

export type CalendarSystem = 'gregorian' | 'julian';

export interface PartialDate {
  y: number;
  m: number | null;
  d: number | null;
}

export interface GenDate {
  mod: GenDateModifier;
  start: PartialDate | null;
  end: PartialDate | null;
  cal: CalendarSystem;
  text: string;
}

export interface ParsedGenDateResult {
  value: GenDate;
  sortKey: number | null;
  isValid: boolean;
}
