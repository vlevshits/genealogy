import { apiFetch } from "./core";
import type { UpdateProfileRequest, UserDto } from "./auth-api";

// =============================================================================
// Phase 1.4 Core Data Model: Persons, Families, Life Events & Places Endpoints
// =============================================================================

import {
  CreateFamilyInput,
  CreateLifeEventInput,
  CreateNoteInput,
  CreatePersonInput,
  CreatePlaceInput,
  DeleteImpactDto,
  FamilyDto,
  GlobalSearchQuery,
  IndividualDto,
  LifeEventDto,
  MergePlaceResponse,
  NoteDto,
  OnboardingStatusResponse,
  PatronymicSuggestionDto,
  PeopleDirectoryQuery,
  PeopleDirectoryResponse,
  PersonDetailDto,
  PersonProfileResponseDto,
  PlaceDetailDto,
  PlaceDto,
  PlacesListResponse,
  QuickAddRelativeInput,
  QuickAddRelativeResponse,
  SearchResultResponseDto,
  StartWithSelfInput,
  SurnamesIndexQuery,
  SurnamesIndexResponse,
  TreeViewQuery,
  TreeViewResponse,
  UpdateFamilyInput,
  UpdateLifeEventInput,
  UpdateNoteInput,
  UpdatePersonInput,
  UpdatePlaceInput,
} from './openapi-types';

export async function fetchPersonDetail(id: string): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>(`/persons/${encodeURIComponent(id)}`);
}

export async function createPerson(
  payload: CreatePersonInput
): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>('/persons', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updatePerson(
  id: string,
  payload: UpdatePersonInput
): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>(`/persons/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function fetchPersonDeleteImpact(
  id: string
): Promise<DeleteImpactDto> {
  return apiFetch<DeleteImpactDto>(
    `/persons/${encodeURIComponent(id)}/delete-impact`
  );
}

export async function deletePerson(
  id: string
): Promise<{ status: string; deleted_id?: string; tx_id?: string } | undefined> {
  return apiFetch<{ status: string; deleted_id?: string; tx_id?: string } | undefined>(
    `/persons/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function quickAddRelative(
  personId: string,
  payload: QuickAddRelativeInput
): Promise<QuickAddRelativeResponse> {
  return apiFetch<QuickAddRelativeResponse>(
    `/persons/${encodeURIComponent(personId)}/quick-add`,
    {
      method: 'POST',
      body: JSON.stringify(payload),
    }
  );
}

export async function suggestSlavicPatronymic(
  fatherGivenName: string,
  sex: 'M' | 'F' | 'U' = 'M'
): Promise<PatronymicSuggestionDto> {
  const params = new URLSearchParams({
    father_given_name: fatherGivenName,
    sex,
  });
  return apiFetch<PatronymicSuggestionDto>(
    `/persons/patronymic-suggest?${params.toString()}`
  );
}

export async function createLifeEvent(
  payload: CreateLifeEventInput
): Promise<LifeEventDto> {
  return apiFetch<LifeEventDto>('/events', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updateLifeEvent(
  id: string,
  payload: UpdateLifeEventInput
): Promise<LifeEventDto> {
  return apiFetch<LifeEventDto>(`/events/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deleteLifeEvent(
  id: string
): Promise<{ status: string; tx_id?: string } | undefined> {
  return apiFetch<{ status: string; tx_id?: string } | undefined>(`/events/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function fetchEventSources(
  query?: string
): Promise<{ items: string[] }> {
  const qs = query ? `?q=${encodeURIComponent(query)}` : '';
  return apiFetch<{ items: string[] }>(`/events/sources${qs}`);
}

export async function createFamily(
  payload: CreateFamilyInput
): Promise<FamilyDto> {
  return apiFetch<FamilyDto>('/families', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updateFamily(
  id: string,
  payload: UpdateFamilyInput
): Promise<FamilyDto> {
  return apiFetch<FamilyDto>(`/families/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deleteFamily(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/families/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function fetchPlaces(params?: {
  q?: string;
  parent_id?: string;
  missing_coords?: boolean;
}): Promise<PlacesListResponse> {
  const search = new URLSearchParams();
  if (params?.q) search.set('q', params.q);
  if (params?.parent_id) search.set('parent_id', params.parent_id);
  if (params?.missing_coords) search.set('missing_coords', 'true');
  const qs = search.toString();
  return apiFetch<PlacesListResponse>(`/places${qs ? `?${qs}` : ''}`);
}

export async function fetchPlaceDetail(id: string): Promise<PlaceDetailDto> {
  return apiFetch<PlaceDetailDto>(`/places/${encodeURIComponent(id)}`);
}

export async function createPlace(
  payload: CreatePlaceInput
): Promise<PlaceDto> {
  return apiFetch<PlaceDto>('/places', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updatePlace(
  id: string,
  payload: UpdatePlaceInput
): Promise<PlaceDto> {
  return apiFetch<PlaceDto>(`/places/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deletePlace(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/places/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function mergePlace(
  targetId: string,
  sourcePlaceId: string
): Promise<MergePlaceResponse> {
  return apiFetch<MergePlaceResponse>(
    `/places/${encodeURIComponent(targetId)}/merge`,
    {
      method: 'POST',
      body: JSON.stringify({ other_id: sourcePlaceId, source_place_id: sourcePlaceId }),
    }
  );
}

// =============================================================================
// Phase 1.5 — Empty-Tree Onboarding & Home Person Endpoints
// =============================================================================

export async function fetchOnboardingStatus(): Promise<OnboardingStatusResponse> {
  return apiFetch<OnboardingStatusResponse>('/onboarding/status');
}

export async function startWithSelf(
  data: StartWithSelfInput
): Promise<{ individual: IndividualDto; user: UserDto }> {
  return apiFetch<{ individual: IndividualDto; user: UserDto }>(
    '/onboarding/start-with-self',
    {
      method: 'POST',
      body: JSON.stringify(data),
    }
  );
}

export async function setHomePerson(
  personId: string | null
): Promise<UserDto> {
  return apiFetch<UserDto>('/me/home-person', {
    method: 'PUT',
    body: JSON.stringify({ person_id: personId }),
  });
}

export async function updateProfile(
  data: Partial<UserDto> | UpdateProfileRequest
): Promise<UserDto> {
  return apiFetch<UserDto>('/me', {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

// =============================================================================
// Phase 1.6 — Interactive Tree Canvas Endpoints
// =============================================================================

export async function fetchTreeView(
  params?: TreeViewQuery
): Promise<TreeViewResponse> {
  const search = new URLSearchParams();
  if (params?.focal_id) search.set('focal_id', params.focal_id);
  if (params?.mode) search.set('mode', params.mode);
  if (params?.generations !== undefined) {
    search.set('generations', String(params.generations));
  }
  if (params?.collateral !== undefined) {
    search.set('collateral', String(params.collateral));
  }
  const qs = search.toString();
  return apiFetch<TreeViewResponse>(`/tree/view${qs ? `?${qs}` : ''}`);
}

export const getTree = fetchTreeView;

// =============================================================================
// Phase 1.7 — Person Profile Page & Notes Endpoints
// =============================================================================

export async function fetchPersonProfile(
  id: string,
  include?: string
): Promise<PersonProfileResponseDto> {
  const qs = include ? `?include=${encodeURIComponent(include)}` : '';
  return apiFetch<PersonProfileResponseDto>(
    `/persons/${encodeURIComponent(id)}/profile${qs}`
  );
}

export async function fetchNotes(params?: {
  entity_type?: string;
  entity_id?: string;
}): Promise<{ items: NoteDto[]; total: number }> {
  const search = new URLSearchParams();
  if (params?.entity_type) search.set('entity_type', params.entity_type);
  if (params?.entity_id) search.set('entity_id', params.entity_id);
  const qs = search.toString();
  return apiFetch<{ items: NoteDto[]; total: number }>(
    `/notes${qs ? `?${qs}` : ''}`
  );
}

export async function createNote(data: CreateNoteInput): Promise<NoteDto> {
  return apiFetch<NoteDto>('/notes', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateNote(
  id: string,
  data: UpdateNoteInput
): Promise<NoteDto> {
  return apiFetch<NoteDto>(`/notes/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function deleteNote(
  id: string
): Promise<{ status?: string; tx_id?: string } | void> {
  return apiFetch<{ status?: string; tx_id?: string } | void>(
    `/notes/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

// =============================================================================
// Phase 1.8 — Global Search (Omnibox), People Directory & Surnames Index
// =============================================================================

export async function fetchGlobalSearch(
  params?: GlobalSearchQuery
): Promise<SearchResultResponseDto> {
  const search = new URLSearchParams();
  if (params?.q !== undefined) search.set('q', params.q);
  if (params?.types) search.set('types', params.types);
  if (params?.limit !== undefined) search.set('limit', String(params.limit));
  const qs = search.toString();
  return apiFetch<SearchResultResponseDto>(`/search${qs ? `?${qs}` : ''}`);
}

export async function fetchPeopleDirectory(
  params?: PeopleDirectoryQuery
): Promise<PeopleDirectoryResponse> {
  const search = new URLSearchParams();
  if (params?.q) search.set('q', params.q);
  if (params?.surname) search.set('surname', params.surname);
  if (params?.born_from !== undefined) {
    search.set('born_from', String(params.born_from));
  }
  if (params?.born_to !== undefined) {
    search.set('born_to', String(params.born_to));
  }
  if (params?.sex) search.set('sex', params.sex);
  if (params?.living) search.set('living', params.living);
  if (params?.has_photo !== undefined) {
    search.set('has_photo', String(params.has_photo));
  }
  if (params?.incomplete !== undefined) {
    search.set('incomplete', String(params.incomplete));
  }
  if (params?.sort) search.set('sort', params.sort);
  if (params?.cursor) search.set('cursor', params.cursor);
  if (params?.limit !== undefined) search.set('limit', String(params.limit));
  const qs = search.toString();
  return apiFetch<PeopleDirectoryResponse>(`/people${qs ? `?${qs}` : ''}`);
}

export async function fetchSurnamesIndex(
  params?: SurnamesIndexQuery
): Promise<SurnamesIndexResponse> {
  const search = new URLSearchParams();
  if (params?.letter) search.set('letter', params.letter);
  if (params?.min_count !== undefined) {
    search.set('min_count', String(params.min_count));
  }
  const qs = search.toString();
  return apiFetch<SurnamesIndexResponse>(
    `/people/surnames${qs ? `?${qs}` : ''}`
  );
}

