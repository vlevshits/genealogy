import { EventType } from './common';
import { GenDate } from './gendate';
import { Place } from './place';

export interface LifeEvent {
  id: string;
  gid: string;
  individual_id?: string | null;
  family_id?: string | null;
  event_type: EventType;
  custom_type?: string | null;
  date?: GenDate | null;
  date_sort?: number | null;
  place_id?: string | null;
  place?: Place | null;
  description?: string | null;
  source_text?: string | null;
  has_scan?: boolean;
  scan_count?: number;
  is_private: boolean;
  age_at_event?: number | null;
  version: number;
}
