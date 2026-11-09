import { Sex } from './common';
import { GenDate } from './gendate';

export interface AvatarCrop {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Individual {
  id: string;
  gid: string;
  display_name?: string | null;
  given_name?: string | null;
  patronymic?: string | null;
  surname?: string | null;
  birth_surname?: string | null;
  nickname?: string | null;
  name_prefix?: string | null;
  name_suffix?: string | null;
  sex: Sex;
  living_override?: boolean | null;
  is_living: boolean;
  is_private: boolean;
  is_redacted?: boolean;
  biography?: string | null;
  avatar_url?: string | null;
  avatar_crop?: AvatarCrop | null;
  birth_date?: GenDate | null;
  death_date?: GenDate | null;
  birth_sort?: number | null;
  death_sort?: number | null;
  birth_place?: string | null;
  death_place?: string | null;
  version: number;
}
