import { apiFetch } from "./core";
import {
  exportGedcom,
  exportPrintablePdf,
  fetchPdfPreviewInfo,
  getGedcomExportUrl,
} from "./media-gedcom-pdf-api";
import type {
  BatchFixPlacesRequest,
  BatchFixPlacesResponse,
  DashboardPayloadDto,
  DuplicateCandidatePairDto,
  DuplicateQuickMatchDto,
  FanChartResponseDto,
  FixPlaceHierarchyRequest,
  FixPlaceHierarchyResponse,
  KinshipPathResponseDto,
  MergePeopleInput,
  MergePeopleResponseDto,
  PersonMergePreviewDto,
  PlaceHierarchyAnalysisResponseDto,
  PlaceHierarchyIssueDto,
  PlacesMapResponseDto,
  QualitySummaryResponse,
  QualityWarningDto,
  QualityWarningsResponseDto,
} from "./delight-types";

export const dashboardApi = {
  async getDashboard(
    scope: 'all' | 'close' = 'all'
  ): Promise<DashboardPayloadDto> {
    return apiFetch<DashboardPayloadDto>(
      `/dashboard?scope=${encodeURIComponent(scope)}`
    );
  },
};


function isUnknownPlaceLabel(name?: string | null): boolean {
  if (!name) return true;
  const normalized = name.trim().toLowerCase();
  return normalized === 'unknown' || normalized === 'неизвестно';
}

export function normalizePlaceHierarchyCategory(
  rawCategory?: string | null,
  ruleCode?: string | null
): string {
  const cat = (rawCategory || '').trim();
  switch (cat) {
    case 'unparsed_path':
    case 'unparsed_paths':
      return 'unparsed_paths';
    case 'orphan_hierarchy':
    case 'orphans':
      return 'orphans';
    case 'type_mismatch':
    case 'type_mismatches':
      return 'type_mismatches';
    case 'duplicate_node':
    case 'duplicates':
      return 'duplicates';
    default: {
      switch (ruleCode) {
        case 'PLACE_UNPARSED_PATH':
          return 'unparsed_paths';
        case 'PLACE_ORPHAN_HIERARCHY':
          return 'orphans';
        case 'PLACE_TYPE_MISMATCH':
          return 'type_mismatches';
        case 'PLACE_DUPLICATE_CANDIDATE':
          return 'duplicates';
        default:
          return cat;
      }
    }
  }
}

export function normalizePlaceHierarchyIssue(
  raw: PlaceHierarchyIssueDto
): PlaceHierarchyIssueDto {
  const confidence_score =
    typeof raw.confidence_score === 'number' && !Number.isNaN(raw.confidence_score)
      ? raw.confidence_score
      : typeof raw.confidence === 'number' && !Number.isNaN(raw.confidence)
        ? raw.confidence
        : 85;

  const category = normalizePlaceHierarchyCategory(
    raw.category || raw.issue_type,
    raw.rule_code
  );

  const rationale = raw.rationale || raw.reason || '';

  const filteredChain = (raw.proposed_chain ?? []).filter(
    (step) => !isUnknownPlaceLabel(step.name)
  );

  const proposed_chain = filteredChain.map((step, idx) => {
    const rawAction = String(step.action || '');
    const mappedAction =
      rawAction === 'reuse' || rawAction === 'match_existing'
        ? 'reuse'
        : rawAction === 'create' || rawAction === 'create_new'
          ? 'create'
          : rawAction === 'consolidate'
            ? 'consolidate'
            : rawAction === 'clean_leaf' || rawAction === 'update_leaf'
              ? 'clean_leaf'
              : null;

    const validAction: 'reuse' | 'create' | 'clean_leaf' | 'consolidate' =
      mappedAction
        ? mappedAction
        : idx < filteredChain.length - 1
          ? step.is_new || !step.existing_place_id
            ? 'create'
            : 'reuse'
          : raw.consolidate_with_place_id || category === 'duplicates'
            ? 'consolidate'
            : 'clean_leaf';

    return {
      ...step,
      action: validAction,
    };
  });

  const rawCandidates = raw.candidate_parents || raw.parent_candidates;
  const candidate_parents = rawCandidates
    ? rawCandidates
        .filter((c) => !isUnknownPlaceLabel(c.name))
        .map((c) => ({
          ...c,
          place_gid: c.place_gid || c.gid,
          score:
            typeof c.score === 'number' && !Number.isNaN(c.score)
              ? c.score
              : typeof c.confidence === 'number' && !Number.isNaN(c.confidence)
                ? c.confidence
                : 80,
        }))
    : undefined;

  return {
    ...raw,
    confidence_score,
    category,
    rationale,
    proposed_chain,
    candidate_parents,
  };
}

export const qualityApi = {
  async getSummary(): Promise<QualitySummaryResponse> {
    return apiFetch<QualitySummaryResponse>('/quality/summary');
  },

  async getWarnings(params?: {
    severity?: string;
    code?: string;
    cursor?: string;
  }): Promise<QualityWarningsResponseDto> {
    const search = new URLSearchParams();
    if (params?.severity && params.severity !== 'all') {
      search.set('severity', params.severity);
    }
    if (params?.code) search.set('code', params.code);
    if (params?.cursor) search.set('cursor', params.cursor);
    const qs = search.toString();
    const res = await apiFetch<
      QualityWarningsResponseDto | { items: QualityWarningDto[] }
    >(`/quality/warnings${qs ? `?${qs}` : ''}`);
    if ('warnings' in res) return res;
    return {
      warnings: res.items ?? [],
    };
  },

  async getDuplicates(): Promise<DuplicateCandidatePairDto[]> {
    const res = await apiFetch<
      DuplicateCandidatePairDto[] | { items: DuplicateCandidatePairDto[] }
    >('/quality/duplicates');
    if (Array.isArray(res)) return res;
    return res.items ?? [];
  },

  async dismissWarning(
    ruleCode: string,
    entityId: string
  ): Promise<{ status: string }> {
    return apiFetch<{ status: string }>('/quality/warnings/dismiss', {
      method: 'POST',
      body: JSON.stringify({
        rule_code: ruleCode,
        code: ruleCode,
        entity_id: entityId,
      }),
    });
  },

  async getPlaceHierarchyAnalysis(params?: {
    category?: string;
    min_confidence?: number;
  }): Promise<PlaceHierarchyAnalysisResponseDto> {
    const search = new URLSearchParams();
    if (params?.category && params.category !== 'all') {
      search.set('category', params.category);
    }
    if (params?.min_confidence !== undefined) {
      search.set('min_confidence', String(params.min_confidence));
    }
    const qs = search.toString();
    const res = await apiFetch<PlaceHierarchyAnalysisResponseDto>(
      `/quality/places/analysis${qs ? `?${qs}` : ''}`
    );
    const items = (res.items ?? []).map(normalizePlaceHierarchyIssue);
    const rawCounts = res.counts;
    return {
      ...res,
      items,
      counts: {
        total: rawCounts?.total ?? items.length,
        high_confidence:
          rawCounts?.high_confidence ??
          items.filter(
            (i) => i.confidence_tier === 'high' || i.confidence_score >= 85
          ).length,
        medium_confidence:
          rawCounts?.medium_confidence ??
          items.filter((i) => i.confidence_tier === 'medium').length,
        low_confidence:
          rawCounts?.low_confidence ??
          items.filter((i) => i.confidence_tier === 'low').length,
        unparsed_paths:
          rawCounts?.unparsed_paths ||
          items.filter((i) => i.category === 'unparsed_paths').length,
        orphans:
          rawCounts?.orphans ||
          items.filter((i) => i.category === 'orphans').length,
        type_mismatches:
          rawCounts?.type_mismatches ||
          items.filter((i) => i.category === 'type_mismatches').length,
        duplicates:
          rawCounts?.duplicates ||
          items.filter((i) => i.category === 'duplicates').length,
      },
    };
  },

  async getPlaceHierarchyPreview(
    placeId: string
  ): Promise<PlaceHierarchyIssueDto> {
    const search = new URLSearchParams({ id: placeId });
    const raw = await apiFetch<PlaceHierarchyIssueDto>(
      `/quality/places/preview?${search.toString()}`
    );
    return normalizePlaceHierarchyIssue(raw);
  },

  async fixPlaceHierarchy(
    body: FixPlaceHierarchyRequest
  ): Promise<FixPlaceHierarchyResponse> {
    return apiFetch<FixPlaceHierarchyResponse>('/quality/places/fix', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async batchFixPlaces(
    body: BatchFixPlacesRequest
  ): Promise<BatchFixPlacesResponse> {
    return apiFetch<BatchFixPlacesResponse>('/quality/places/batch-fix', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async dismissPlaceQuality(
    placeId: string,
    ruleCode: string = 'PLACE_HIERARCHY'
  ): Promise<{ status: string }> {
    return apiFetch<{ status: string }>('/quality/places/dismiss', {
      method: 'POST',
      body: JSON.stringify({
        place_id: placeId,
        entity_id: placeId,
        rule_code: ruleCode,
        code: ruleCode,
      }),
    });
  },
};

export const peopleApi = {
  async findDuplicates(
    name: string,
    birthYear?: number | string | null,
    gender?: string | null
  ): Promise<DuplicateQuickMatchDto[]> {
    const search = new URLSearchParams();
    const trimmed = name.trim();
    const parts = trimmed.split(/\s+/);
    const given = parts[0] || '';
    const surname = parts.length > 1 ? parts[parts.length - 1] : '';
    search.set('q', trimmed);
    if (given) search.set('given', given);
    if (surname) search.set('surname', surname);
    if (birthYear !== undefined && birthYear !== null && String(birthYear).trim()) {
      search.set('birth', String(birthYear).trim());
      search.set('birth_year', String(birthYear).trim());
    }
    if (gender) search.set('sex', gender);
    const res = await apiFetch<
      DuplicateQuickMatchDto[] | { items: DuplicateQuickMatchDto[] }
    >(`/people/match?${search.toString()}`);
    const rawItems = Array.isArray(res) ? res : (res.items ?? []);
    return rawItems.map((item) => ({
      ...item,
      id: item.person?.id ?? item.id,
      gid: item.person?.gid ?? item.gid,
      display_name: item.person?.display_name ?? item.display_name,
      given_name: item.person?.given_name ?? item.given_name ?? '',
      patronymic: item.person?.patronymic ?? item.patronymic ?? null,
      surname: item.person?.surname ?? item.surname ?? '',
      sex: item.person?.sex ?? item.sex ?? 'U',
      birth_date: item.person?.birth_date ?? item.birth_date ?? null,
      death_date: item.person?.death_date ?? item.death_date ?? null,
      is_living: item.person?.is_living ?? item.is_living ?? false,
      score: item.score,
    }));
  },

  async getMergePreview(
    targetId: string,
    sourceId: string
  ): Promise<PersonMergePreviewDto> {
    const search = new URLSearchParams({
      target: targetId,
      source: sourceId,
      keep_id: targetId,
      merge_id: sourceId,
      a: targetId,
      b: sourceId,
    });
    return apiFetch<PersonMergePreviewDto>(
      `/people/merge/preview?${search.toString()}`
    );
  },

  async merge(body: MergePeopleInput): Promise<MergePeopleResponseDto> {
    return apiFetch<MergePeopleResponseDto>('/people/merge', {
      method: 'POST',
      body: JSON.stringify({
        ...body,
        keep_id: body.keep_id ?? body.target_id,
        merge_id: body.merge_id ?? body.source_id,
      }),
    });
  },

  async markNotDuplicate(
    personA: string,
    personB: string
  ): Promise<{ status: string }> {
    return apiFetch<{ status: string }>('/quality/duplicates/not-duplicate', {
      method: 'POST',
      body: JSON.stringify({
        person_a: personA,
        person_b: personB,
        person_a_id: personA,
        person_b_id: personB,
      }),
    });
  },
};

export const treeApi = {
  async getFanChart(
    rootId?: string | null,
    generations: number = 5
  ): Promise<FanChartResponseDto> {
    const search = new URLSearchParams({
      mode: 'fan',
      generations: String(generations),
    });
    if (rootId) {
      search.set('root_id', rootId);
      search.set('focal_id', rootId);
    }
    return apiFetch<FanChartResponseDto>(`/tree/fan?${search.toString()}`);
  },
};

export async function fetchKinshipPath(
  fromId: string,
  toId: string,
  maxTraces?: number
): Promise<KinshipPathResponseDto> {
  const search = new URLSearchParams({
    from: fromId,
    to: toId,
    a: fromId,
    b: toId,
  });
  if (maxTraces !== undefined && maxTraces !== null) {
    search.set('max_traces', String(maxTraces));
  }
  return apiFetch<KinshipPathResponseDto>(`/kinship?${search.toString()}`);
}

export const kinshipApi = {
  async getKinship(
    fromId: string,
    toId: string,
    maxTraces?: number
  ): Promise<KinshipPathResponseDto> {
    return fetchKinshipPath(fromId, toId, maxTraces);
  },
};

export const placesApi = {
  async getPlacesMap(): Promise<PlacesMapResponseDto> {
    return apiFetch<PlacesMapResponseDto>('/places/map');
  },
};

export const exportApi = {
  exportGedcom,
  getGedcomExportUrl,
  exportPrintablePdf,
  fetchPdfPreviewInfo,
};

