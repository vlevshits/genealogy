import { PlaceType } from './common';

export interface Place {
  id: string;
  gid: string;
  name: string;
  place_type: PlaceType;
  parent_id?: string | null;
  hierarchy_names?: string[];
  latitude?: number | null;
  longitude?: number | null;
  events_count?: number;
  version: number;
}
