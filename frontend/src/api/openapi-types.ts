/**
 * Canonical OpenAPI 3.1 TypeScript Schema Definitions for Phase 1.4
 * Core Data Model: Individuals, Families, Life Events, Places & Quick-Add.
 * Resolves TD-004 by providing strict schema-synchronized interfaces.
 */

import { GenDate } from '../types/gendate';

export type SexCode = 'M' | 'F' | 'U';

export type AltNameType =
  | 'married'
  | 'alias'
  | 'transliteration'
  | 'religious'
  | 'other';

export interface IndividualAltNameDto {
  id?: string;
  name_type: AltNameType;
  given_name?: string | null;
  patronymic?: string | null;
  surname?: string | null;
  lang?: string | null;
}

export type LifeEventType =
  | 'birth'
  | 'baptism'
  | 'marriage'
  | 'engagement'
  | 'divorce'
  | 'death'
  | 'burial'
  | 'cremation'
  | 'residence'
  | 'occupation'
  | 'education'
  | 'military'
  | 'emigration'
  | 'immigration'
  | 'naturalization'
  | 'census'
  | 'religion'
  | 'custom';

export interface LifeEventDto {
  id: string;
  gid: string;
  individual_id?: string | null;
  family_id?: string | null;
  event_type: LifeEventType | string;
  custom_type?: string | null;
  date?: GenDate | null;
  date_text?: string | null;
  date_sort?: number | null;
  place_id?: string | null;
  place_full_name?: string | null;
  description?: string | null;
  source_text?: string | null;
  media_ids?: string[];
  is_private: boolean;
  version: number;
}

export interface CreateLifeEventInput {
  individual_id?: string | null;
  family_id?: string | null;
  event_type: LifeEventType | string;
  custom_type?: string | null;
  date?: GenDate | null;
  date_text?: string | null;
  place_id?: string | null;
  place_path?: string | null;
  description?: string | null;
  source_text?: string | null;
  is_private?: boolean;
}

export interface UpdateLifeEventInput extends Partial<CreateLifeEventInput> {
  version: number;
}

export type RelationshipType =
  | 'married'
  | 'civil_union'
  | 'partners'
  | 'unknown'
  | string;

export type ParentChildRelation = 'birth' | 'adopted' | 'foster' | 'step';

export interface FamilyChildDto {
  child_id: string;
  individual_id?: string;
  child_name?: string;
  sort_order: number;
  rel_to_partner1?: ParentChildRelation | string;
  rel_to_partner2?: ParentChildRelation | string;
}

export interface FamilyDto {
  id: string;
  gid: string;
  partner1_id?: string | null;
  partner2_id?: string | null;
  relationship_type: RelationshipType;
  children: FamilyChildDto[];
  events: LifeEventDto[];
  is_private: boolean;
  version: number;
}

export interface CreateFamilyInput {
  partner1_id?: string | null;
  partner2_id?: string | null;
  relationship_type?: RelationshipType;
  children_ids?: string[];
  is_private?: boolean;
}

export interface UpdateFamilyInput extends Partial<CreateFamilyInput> {
  version: number;
}

export interface PersonDetailDto {
  id: string;
  gid: string;
  sex: SexCode;
  is_living: boolean;
  living_override?: boolean | null;
  is_private: boolean;
  given_name?: string | null;
  patronymic?: string | null;
  surname?: string | null;
  primary_name: {
    given_name: string;
    patronymic: string | null;
    surname: string;
  };
  birth_surname?: string | null;
  nickname?: string | null;
  name_prefix?: string | null;
  name_suffix?: string | null;
  birth: { date: string; place: string | null } | null;
  death: { date: string; place: string | null } | null;
  avatar_url: string | null;
  biography?: string | null;
  notes: string | null;
  version: number;
  alt_names: IndividualAltNameDto[];
  events: LifeEventDto[];
  families: FamilyDto[];
}

export interface CreatePersonInput {
  given_name: string;
  patronymic?: string | null;
  surname?: string | null;
  birth_surname?: string | null;
  nickname?: string | null;
  name_prefix?: string | null;
  name_suffix?: string | null;
  sex: SexCode;
  living_override?: boolean | null;
  is_private?: boolean;
  biography?: string | null;
  birth_date?: string | null;
  birth_place?: string | null;
  alt_names?: IndividualAltNameDto[];
}

export interface UpdatePersonInput extends Partial<CreatePersonInput> {
  version: number;
}

export type QuickAddRelation =
  | 'father'
  | 'mother'
  | 'spouse'
  | 'child'
  | 'sibling';

export interface QuickAddRelativeInput {
  relation: QuickAddRelation;
  existing_person_id?: string | null;
  given_name?: string;
  patronymic?: string | null;
  surname?: string | null;
  sex?: SexCode;
  birth_date?: string | null;
  birth_place?: string | null;
  living_override?: boolean | null;
  family_id?: string | null;
}

export interface QuickAddRelativeResponse {
  person: PersonDetailDto;
  family?: FamilyDto | null;
}

export interface DeleteImpactDto {
  person_id: string;
  gid: string;
  display_name: string;
  events_count: number;
  families_unlinked_count: number;
  empty_families_deleted_count: number;
  notes_count: number;
  media_links_count: number;
}

export interface PatronymicSuggestionDto {
  father_given_name: string;
  male_patronymic: string;
  female_patronymic: string;
}

export type PlaceType =
  | 'country'
  | 'region'
  | 'district'
  | 'city'
  | 'village'
  | 'parish'
  | 'cemetery'
  | 'address'
  | 'other';

export interface PlaceAltNameDto {
  id?: string;
  name: string;
  lang?: string | null;
  note?: string | null;
}

export interface PlaceDto {
  id: string;
  gid: string;
  name: string;
  place_type: PlaceType;
  parent_id: string | null;
  full_name: string;
  latitude: number | null;
  longitude: number | null;
  alt_names: PlaceAltNameDto[];
  events_count: number;
  children_count: number;
  version: number;
}

export interface PlaceBreadcrumbDto {
  id: string;
  name: string;
  place_type: PlaceType | string;
}

export interface PlaceDetailDto {
  place: PlaceDto;
  breadcrumbs: PlaceBreadcrumbDto[];
  children: PlaceDto[];
  events: LifeEventDto[];
}

export interface PlacesListResponse {
  items: PlaceDto[];
  map_tile_url?: string;
  map_attribution?: string;
  geocoding_enabled?: boolean;
}

export interface CreatePlaceInput {
  name?: string;
  path?: string;
  place_type?: PlaceType;
  parent_id?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  alt_names?: PlaceAltNameDto[];
}

export interface UpdatePlaceInput {
  name?: string;
  place_type?: PlaceType;
  parent_id?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  alt_names?: PlaceAltNameDto[];
  version: number;
}

export type MergePlaceResponse =
  | PlaceDto
  | {
      target_place: PlaceDto;
      repointed_events_count?: number;
      repointed_children_count?: number;
      message?: string;
    };

// =============================================================================
// Phase 1.5 — Empty-Tree Onboarding & Home Person DTOs
// =============================================================================

export type IndividualDto = PersonDetailDto;

export interface OnboardingStatusResponse {
  is_empty: boolean;
  total_individuals: number;
  can_onboard: boolean;
  resolved_home_person_id: string | null;
  default_home_person_id: string | null;
  user_home_person_id: string | null;
  user_person_id: string | null;
  start_page: 'tree' | 'dashboard';
}

export interface StartWithSelfInput {
  given_name?: string;
  patronymic?: string | null;
  surname?: string | null;
  sex: SexCode;
  birth_date?: string | null;
  birth_place?: string | null;
}

// =============================================================================
// Phase 1.6 — Interactive Tree Canvas (Hourglass, Pedigree, Descendants) DTOs
// =============================================================================

export type TreeProjectionMode = 'hourglass' | 'pedigree' | 'descendant';

export interface TreePersonNodeDto {
  id: string;
  gid: string;
  sex: SexCode;
  is_living: boolean;
  is_private: boolean;
  primary_name: {
    given_name: string;
    patronymic: string | null;
    surname: string;
  };
  birth: { date: string; place: string | null } | null;
  death: { date: string; place: string | null } | null;
  avatar_url: string | null;
  notes: string | null;
}

export interface TreeEdgeDto {
  id: string;
  source: string;
  target: string;
  kind: 'spouse' | 'parent_child' | string;
  family_id: string;
}

export interface TreeViewQuery {
  focal_id?: string;
  mode?: TreeProjectionMode | string;
  generations?: number;
}

export interface TreeViewResponse {
  access_tier: 'public_anonymous' | 'viewer' | 'editor' | 'admin' | string;
  tree_title: string;
  focal_id: string | null;
  mode: TreeProjectionMode | string;
  generations: number;
  nodes: TreePersonNodeDto[];
  families: FamilyDto[];
  edges: TreeEdgeDto[];
  resolved_home_person_id: string | null;
  total_visible: number;
}


