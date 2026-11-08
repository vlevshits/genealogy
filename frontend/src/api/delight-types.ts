import type {
  MediaAssetSummaryDto,
  PersonDetailDto,
  PlaceDto,
} from "./openapi-types";
import type { TreePersonNodeDto } from "./auth-api";
import type { ChangeLogTransactionGroupDto } from "./history-api";

// =============================================================================
// Phase 2 ("Delight") — Dashboard, Data Quality, Person Merge, Fan Chart,
// Kinship Path Finder & Clustered Places Map Endpoints
// =============================================================================

export type AnniversaryType = 'birthday' | 'memorial' | 'wedding';

export interface AnniversaryItemDto {
  id?: string;
  type?: AnniversaryType;
  kind?: AnniversaryType | string;
  person_id?: string | null;
  person?: {
    id: string;
    gid?: string;
    display_name: string;
    is_living?: boolean;
  } | null;
  partner_person_id?: string | null;
  family_id?: string | null;
  display_name?: string;
  secondary_name?: string | null;
  is_living?: boolean;
  date_month_day?: string;
  date_label?: string;
  next_occurrence_date?: string;
  years_count?: number | null;
  years_elapsed?: number | null;
  birth_year?: number | null;
  death_year?: number | null;
  days_until: number;
}

export interface TreeGlanceStatsDto {
  total_persons?: number;
  persons_count?: number;
  individuals_count?: number;
  total_families?: number;
  families_count?: number;
  total_photos?: number;
  media_count?: number;
  photos_count?: number;
  total_places?: number;
  places_count?: number;
  max_generations?: number;
  generations_count?: number;
  oldest_birth_year?: number | null;
  earliest_birth_year?: number | null;
  oldest_person_name?: string | null;
  oldest_ancestor?:
  | {
    id?: string;
    gid?: string;
    display_name?: string;
    birth_year?: number | null;
  }
  | string
  | null;
  open_warnings_count?: number;
  missing_birth_dates_count?: number;
  missing_birth_count?: number;
  quality_summary?: {
    total_open_warnings?: number;
  };
}

export interface DashboardPayloadDto {
  anniversaries: AnniversaryItemDto[];
  upcoming_anniversaries?: AnniversaryItemDto[];
  recent_activity: ChangeLogTransactionGroupDto[];
  stats: TreeGlanceStatsDto;
  tree_stats?: TreeGlanceStatsDto;
  latest_photos: MediaAssetSummaryDto[];
  resolved_home_person_id?: string | null;
}

export type QualitySeverity = 'critical' | 'warning' | 'info';

export type QualityRuleCode =
  | 'DEATH_BEFORE_BIRTH'
  | 'TOO_OLD'
  | 'PARENT_TOO_YOUNG'
  | 'MOTHER_TOO_OLD'
  | 'BORN_AFTER_PARENT_DEATH'
  | 'MARRIED_TOO_YOUNG'
  | 'EVENT_AFTER_DEATH'
  | 'POSSIBLE_DUPLICATE'
  | 'MISSING_BIRTH'
  | 'UNCONNECTED'
  | string;

export interface QualityWarningDto {
  id?: string;
  rule_code: QualityRuleCode;
  code?: QualityRuleCode;
  severity: QualitySeverity;
  entity_type?: string;
  entity_id: string;
  person_id?: string;
  person_name?: string;
  person_gid?: string;
  person?: {
    id: string;
    gid?: string;
    display_name: string;
  } | null;
  message?: string;
  details?: Record<string, unknown>;
}

export interface DuplicateCandidatePairDto {
  id?: string;
  person_a_id?: string;
  person_a_gid?: string;
  person_a_name?: string;
  person_a_birth?: string | null;
  person_a_death?: string | null;
  person_a?: {
    id: string;
    gid?: string;
    display_name: string;
    birth_date?: string | null;
    death_date?: string | null;
  } | null;
  person_b_id?: string;
  person_b_gid?: string;
  person_b_name?: string;
  person_b_birth?: string | null;
  person_b_death?: string | null;
  person_b?: {
    id: string;
    gid?: string;
    display_name: string;
    birth_date?: string | null;
    death_date?: string | null;
  } | null;
  score: number;
  reasons?: string[];
  is_detached?: boolean;
}

export interface QualitySummaryResponse {
  total_open_warnings: number;
  critical_count: number;
  warning_count: number;
  info_count: number;
  duplicate_candidates_count: number;
  missing_birth_dates_count?: number;
  place_hierarchy_issues?: number;
  place_hierarchy_high_confidence?: number;
}

export interface ProposedHierarchyStepDto {
  name: string;
  place_type: string;
  existing_place_id?: string | null;
  existing_place_gid?: string | null;
  is_new?: boolean;
  action?: 'reuse' | 'create' | 'clean_leaf' | 'consolidate';
}

export interface PlaceHierarchyCandidateParentDto {
  place_id: string;
  place_gid?: string;
  gid?: string;
  name: string;
  full_name: string;
  place_type: string;
  distance_km?: number | null;
  score: number;
  confidence?: number;
}

export interface PlaceHierarchyIssueDto {
  place_id: string;
  place_gid: string;
  current_name: string;
  current_full_name: string;
  current_place_type: string;
  current_parent_id?: string | null;
  current_parent_name?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  events_count: number;
  rule_code:
    | 'PLACE_UNPARSED_PATH'
    | 'PLACE_ORPHAN_HIERARCHY'
    | 'PLACE_TYPE_MISMATCH'
    | 'PLACE_DUPLICATE_CANDIDATE'
    | string;
  category:
    | 'unparsed_paths'
    | 'orphans'
    | 'type_mismatches'
    | 'duplicates'
    | string;
  issue_type?: string;
  confidence_score: number;
  confidence?: number;
  confidence_tier: 'high' | 'medium' | 'low';
  rationale: string;
  reason?: string;
  proposed_clean_name: string;
  proposed_place_type: string;
  proposed_parent_id?: string | null;
  proposed_parent_name?: string | null;
  consolidate_with_place_id?: string | null;
  proposed_chain: ProposedHierarchyStepDto[];
  candidate_parents?: PlaceHierarchyCandidateParentDto[];
  parent_candidates?: PlaceHierarchyCandidateParentDto[];
}

export interface PlaceQualityCountsDto {
  total: number;
  high_confidence: number;
  medium_confidence: number;
  low_confidence: number;
  unparsed_paths: number;
  orphans: number;
  type_mismatches: number;
  duplicates: number;
}

export interface PlaceHierarchyAnalysisResponseDto {
  items: PlaceHierarchyIssueDto[];
  counts: PlaceQualityCountsDto;
}

export interface FixPlaceHierarchyRequest {
  place_id: string;
  parent_id?: string | null;
  clean_name?: string;
  place_type?: string;
  create_missing_parents?: boolean;
  proposed_chain?: ProposedHierarchyStepDto[];
}

export interface FixPlaceHierarchyResponse {
  place: PlaceDto;
  created_parents_count: number;
  consolidated_into_id?: string | null;
  tx_id?: string;
  status: string;
}

export interface BatchFixPlacesRequest {
  place_ids?: string[];
  min_confidence?: number;
  category?: string;
  dry_run?: boolean;
}

export interface BatchFixPlacesResponse {
  fixed_count: number;
  created_parents_count: number;
  consolidated_count: number;
  tx_id?: string;
  status: string;
}

export interface QualityWarningsResponseDto {
  warnings: QualityWarningDto[];
  duplicates?: DuplicateCandidatePairDto[];
  counts?: {
    total: number;
    critical: number;
    warning: number;
    info: number;
    duplicates: number;
  };
}

export interface DuplicateQuickMatchDto {
  id: string;
  gid: string;
  display_name: string;
  given_name: string;
  patronymic?: string | null;
  surname: string;
  sex: 'M' | 'F' | 'U';
  birth_date?: string | null;
  death_date?: string | null;
  is_living: boolean;
  score: number;
  person?: {
    id: string;
    gid: string;
    display_name: string;
    given_name?: string;
    patronymic?: string | null;
    surname?: string;
    sex?: 'M' | 'F' | 'U';
    birth_date?: string | null;
    death_date?: string | null;
    is_living?: boolean;
  } | null;
}

export interface MergeScalarFieldDiffDto {
  field: string;
  target_value: string | null;
  source_value: string | null;
}

export interface PersonMergePreviewDto {
  target: TreePersonNodeDto & Partial<PersonDetailDto>;
  source: TreePersonNodeDto & Partial<PersonDetailDto>;
  keep_person?: TreePersonNodeDto & Partial<PersonDetailDto> & {
    display_name?: string;
    birth_date?: string | null;
    birth_place?: string | null;
    death_date?: string | null;
    death_place?: string | null;
  };
  merge_person?: TreePersonNodeDto & Partial<PersonDetailDto> & {
    display_name?: string;
    birth_date?: string | null;
    birth_place?: string | null;
    death_date?: string | null;
    death_place?: string | null;
  };
  fields?: MergeScalarFieldDiffDto[];
  events_to_transfer?: unknown[] | number;
  media_to_transfer?: unknown[] | number;
  families_to_transfer?: unknown[] | number;
  notes_to_transfer?: unknown[] | number;
  combined_counts: {
    events: number;
    media: number;
    families: number;
    notes: number;
  };
}

export interface MergePeopleInput {
  target_id: string;
  source_id: string;
  keep_id?: string;
  merge_id?: string;
  field_choices: Record<string, 'target' | 'source' | unknown>;
}

export interface MergePeopleResponseDto {
  surviving_person_id?: string;
  surviving_person?: {
    id: string;
    gid?: string;
    display_name?: string;
  } | null;
  merged_source_id?: string;
  tx_id?: string;
  status?: string;
}

export interface FanChartNodeDto {
  ahnentafel: number;
  generation: number;
  quadrant: 'paternal_gf' | 'paternal_gm' | 'maternal_gf' | 'maternal_gm' | 'root';
  person: TreePersonNodeDto;
}

export interface FanChartResponseDto {
  root_id: string;
  generations: number;
  nodes: FanChartNodeDto[];
}

export interface KinshipStepDto {
  person_id?: string;
  from_id?: string;
  to_id?: string;
  edge_type?: string;
  gid?: string;
  display_name?: string;
  sex?: 'M' | 'F' | 'U';
  is_living?: boolean;
  relation_to_next?: string | null;
  is_mrca?: boolean;
}

export interface KinshipTraceStepDto {
  person_id: string;
  relation_to_next?: string | null;
  relation_to_next_ru?: string | null;
}

export interface KinshipTraceDto {
  id: string;
  relationship_key: string;
  label_en: string;
  label_ru: string;
  degree_of_kinship: number;
  is_blood_relation: boolean;
  common_ancestor_ids: string[];
  path_person_ids: string[];
  steps: KinshipTraceStepDto[];
}

export interface KinshipPathResponseDto {
  from_id?: string;
  to_id?: string;
  connected: boolean;
  relationship_label?: string;
  label_en?: string;
  label_ru?: string;
  relationship_key?: string;
  degree?: number;
  degree_of_kinship?: number;
  mrca_ids?: string[];
  steps: KinshipStepDto[];
  path_nodes?: Array<
    TreePersonNodeDto & {
      display_name?: string;
      given_name?: string;
      surname?: string;
    }
  >;
  traces_count?: number;
  traces?: KinshipTraceDto[];
}

export interface PlaceMapPinDto {
  id: string;
  gid: string;
  name: string;
  full_name: string;
  place_type: string;
  latitude: number;
  longitude: number;
  events_count: number;
  persons_count: number;
  sample_people?: Array<{
    id: string;
    display_name: string;
    event_type?: string;
    year?: number | null;
  }>;
}

export interface PlacesMapResponseDto {
  items: PlaceMapPinDto[];
  pins?: PlaceMapPinDto[];
  map_tile_url?: string;
  map_attribution?: string;
}
