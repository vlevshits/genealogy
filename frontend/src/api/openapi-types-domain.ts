/**
 * Canonical OpenAPI 3.1 TypeScript Schema Definitions for Phases 1.7–1.10
 * Person Profile, Media Archive, Global Search, Directory & GEDCOM DTOs.
 */

import { GenDate } from '../types/gendate';
import type { TreePersonNodeDto } from './openapi-types';

// =============================================================================
// Phase 1.7 — Person Profile Page: Timeline, Family, Media & Notes DTOs
// =============================================================================

export interface FaceTagDto {
  id: string;
  person_id: string;
  person_name: string;
  is_living: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  is_primary_avatar?: boolean;
}

export interface MediaLinkDto {
  id: string;
  media_id: string;
  entity_type: 'individual' | 'family' | 'event' | 'place' | string;
  entity_id: string;
  role?: string | null;
}

export interface AvatarCropDto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface MediaAssetSummaryDto {
  id: string;
  gid: string;
  title: string;
  mime_type: string;
  file_url: string;
  file_name?: string | null;
  original_file_name?: string | null;
  file_size_bytes?: number | null;
  sha256?: string | null;
  width?: number | null;
  height?: number | null;
  description?: string | null;
  date_taken?: string | null;
  date_sort?: number | null;
  place_id?: string | null;
  place_full_name?: string | null;
  is_private: boolean;
  only_living_persons: boolean;
  linked_person_ids: string[];
  face_tags: FaceTagDto[];
  links?: MediaLinkDto[];
  version?: number;
  is_missing?: boolean;
}

export interface MediaBundleReconciliationReport {
  total_archive_files: number;
  total_extracted?: number;
  reconciled_stubs: number;
  newly_created: number;
  unmatched_files: number;
  unmatched_filenames: string[];
  remaining_missing_stubs: number;
  skipped_invalid_files: number;
  skipped_files?: number;
  total_bytes_extracted: number;
  duration_ms: number;
  errors?: string[];
}

export interface MediaArchiveResponse {
  items: MediaAssetSummaryDto[];
  next_cursor: string | null;
  total: number;
  total_unfiltered: number;
}

export interface MediaFilterQuery {
  q?: string;
  person_id?: string;
  decade?: string;
  type?: string;
  untagged?: boolean;
  is_missing?: boolean;
  cursor?: string;
  limit?: number;
}

export interface CreateMediaInput {
  file_name: string;
  mime_type: string;
  data?: string;
  title?: string | null;
  description?: string | null;
  date_taken?: string | null;
  place_id?: string | null;
  place_path?: string | null;
  is_private?: boolean;
  width?: number | null;
  height?: number | null;
  linked_person_ids?: string[];
  gid?: string | null;
}

export interface UpdateMediaInput {
  title?: string | null;
  description?: string | null;
  date_taken?: string | null;
  place_id?: string | null;
  place_path?: string | null;
  is_private?: boolean;
  version?: number;
}

export interface CreateFaceTagInput {
  person_id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  is_primary_avatar?: boolean;
}

export interface UpdateFaceTagInput {
  person_id?: string;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  is_primary_avatar?: boolean;
}

export interface CreateMediaLinkInput {
  entity_type: 'individual' | 'family' | 'event' | string;
  entity_id: string;
  role?: string | null;
}

export interface NoteDto {
  id: string;
  entity_type: 'individual' | 'family' | 'event' | 'place' | 'media' | string;
  entity_id: string;
  body: string;
  is_private: boolean;
  sort_order: number;
  version: number;
}

export interface CreateNoteInput {
  entity_type: 'individual' | 'family' | 'event' | 'place' | 'media' | string;
  entity_id: string;
  body: string;
  is_private?: boolean;
  sort_order?: number;
}

export interface UpdateNoteInput {
  body?: string;
  is_private?: boolean;
  sort_order?: number;
  version?: number;
}

export interface ProfileKinshipDto {
  home_person_id: string;
  target_person_id?: string;
  relation_key?: string;
  code?: string;
  label_en?: string;
  label_ru?: string;
  steps?: number;
  degree?: number;
  path_person_ids?: string[];
  home_person_name?: string;
}

export interface ProfileLifeEventDto {
  id: string;
  gid: string;
  individual_id?: string | null;
  family_id?: string | null;
  event_type: string;
  custom_type?: string | null;
  date?: GenDate | null;
  date_sort?: number | null;
  gedcom_date?: string | null;
  place_id?: string | null;
  place_full_name?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  description?: string | null;
  source_text?: string | null;
  media_ids: string[];
  is_private: boolean;
  age_at_event?: number | null;
  scans_count?: number;
  spouse_id?: string | null;
  spouse_name?: string | null;
  version: number;
}

export interface ProfileRelativeSummaryDto {
  id: string;
  gid: string;
  given_name?: string;
  patronymic?: string | null;
  surname?: string;
  display_name?: string;
  sex: string;
  is_living: boolean;
  birth_year?: number | null;
  death_year?: number | null;
  avatar_url?: string | null;
}

export interface ProfileChildSummaryDto {
  person?: ProfileRelativeSummaryDto;
  relative?: ProfileRelativeSummaryDto;
  sort_order: number;
  rel_to_partner1?: string;
  rel_to_partner2?: string;
  rel_to_target?: string;
}

export interface ProfileParentFamilyDto {
  family_id: string;
  gid: string;
  relationship_type?: string;
  father?: ProfileRelativeSummaryDto | null;
  mother?: ProfileRelativeSummaryDto | null;
  parents?: ProfileRelativeSummaryDto[];
  child_relation?: string;
  siblings: ProfileRelativeSummaryDto[];
  version?: number;
}

export interface ProfileSpouseFamilyDto {
  family_id: string;
  gid: string;
  relationship_type: string;
  spouse: ProfileRelativeSummaryDto | null;
  marriage_date?: string | null;
  marriage_place?: string | null;
  marriage_event?: { date?: string | null; place?: string | null } | null;
  children: ProfileChildSummaryDto[];
  version?: number;
}

export interface ProfileFamiliesDto {
  parent_families: ProfileParentFamilyDto[];
  spouse_families: ProfileSpouseFamilyDto[];
}

export interface ProfileSourceSummaryDto {
  source_text: string;
  event_ids: string[];
  events_count?: number;
  count?: number;
}

export interface ProfileCapabilitiesDto {
  can_edit: boolean;
  can_delete: boolean;
  can_merge: boolean;
  is_home_person: boolean;
}

export interface PersonProfileResponseDto {
  person: TreePersonNodeDto;
  kinship: ProfileKinshipDto | null;
  events: ProfileLifeEventDto[];
  families: ProfileFamiliesDto;
  media: MediaAssetSummaryDto[];
  notes: NoteDto[];
  sources_summary: ProfileSourceSummaryDto[];
  capabilities: ProfileCapabilitiesDto;
}

// =============================================================================
// Phase 1.8 — Global Search (Omnibox), People Directory & Surnames Index DTOs
// =============================================================================

export interface SearchPersonResultDto {
  id: string;
  gid: string;
  display_name: string;
  given_name: string;
  patronymic: string | null;
  surname: string;
  sex: string;
  birth_year: number | null;
  death_year: number | null;
  birth_place: string | null;
  avatar_url: string | null;
  is_living: boolean;
  is_private: boolean;
  score: number;
}

export interface SearchPlaceResultDto {
  id: string;
  gid: string;
  name: string;
  full_name: string;
  place_type: string;
  events_count: number;
  has_coordinates: boolean;
}

export interface SearchMediaResultDto {
  id: string;
  gid: string;
  title: string;
  mime_type: string;
  file_url: string;
  is_private: boolean;
}

export interface SearchActionDto {
  id: string;
  label: string;
  hint: string;
  path: string;
  icon: string;
  required_role: string;
}

export interface SearchResultResponseDto {
  query: string;
  people: SearchPersonResultDto[];
  places: SearchPlaceResultDto[];
  media: SearchMediaResultDto[];
  actions: SearchActionDto[];
  timing_ms: number;
}

export interface GlobalSearchQuery {
  q?: string;
  types?: string;
  limit?: number;
}

export interface PeopleDirectoryItemDto {
  id: string;
  gid: string;
  given_name: string;
  patronymic: string | null;
  surname: string;
  birth_surname?: string | null;
  display_name: string;
  sex: string;
  birth_date: string | null;
  birth_year: number | null;
  birth_place: string | null;
  death_date: string | null;
  death_year: number | null;
  death_place: string | null;
  avatar_url: string | null;
  is_living: boolean;
  is_private: boolean;
  has_parents: boolean;
  has_birth_date: boolean;
  version: number;
  updated_at?: string | null;
}

export interface PeopleDirectoryResponse {
  items: PeopleDirectoryItemDto[];
  next_cursor: string | null;
  total: number;
  total_unfiltered: number;
}

export interface PeopleDirectoryQuery {
  q?: string;
  surname?: string;
  born_from?: number;
  born_to?: number;
  sex?: string;
  living?: string;
  has_photo?: boolean;
  incomplete?: boolean;
  sort?: string;
  cursor?: string;
  limit?: number;
}

export interface SurnameIndexEntryDto {
  canonical_surname: string;
  variants: string[];
  count: number;
  earliest_year: number | null;
  latest_year: number | null;
}

export interface SurnameLetterGroupDto {
  letter: string;
  items: SurnameIndexEntryDto[];
}

export interface SurnamesIndexResponse {
  groups: SurnameLetterGroupDto[];
  total_surnames: number;
  total_people: number;
}

export interface SurnamesIndexQuery {
  letter?: string;
  min_count?: number;
}

// =============================================================================
// Phase 1.10 — GEDCOM 5.5.1 + 7.0 Import & Export DTOs
// =============================================================================

export interface GedcomDialectSummary {
  coordinates_normalized: number;
  uids_preserved: number;
  vendor_upd_filtered: number;
  married_names_resolved: number;
  parental_asso_deduplicated: number;
  missing_media_stubs: number;
}

export interface GedcomAnalysisReport {
  total_individuals: number;
  total_families: number;
  total_events: number;
  total_places: number;
  total_media: number;
  missing_media_count: number;
  detected_software: string;
  gedcom_version: string;
  character_encoding: string;
  dialect_summary: GedcomDialectSummary;
  warnings: string[];
  file_name: string;
  file_size_bytes: number;
}

export interface AnalyzeGedcomInput {
  file_name: string;
  data?: string;
}

export type GedcomImportMode = 'clean_slate' | 'replace' | 'new' | 'merge';

export interface ImportGedcomInput {
  mode: GedcomImportMode;
  confirm_text?: string;
  file_name: string;
  data?: string;
}

export interface GedcomImportResult {
  imported_individuals: number;
  imported_families: number;
  imported_events: number;
  imported_places: number;
  imported_media: number;
  missing_media_count: number;
  snapshot_id: string;
  prompt_home_person: boolean;
  warnings: string[];
}

export type GedcomVersionOption = '5.5.1' | '7.0';

export interface GedcomExportOptions {
  version: GedcomVersionOption;
  redact_living: boolean;
  include_media: boolean;
  include_private_notes: boolean;
}
