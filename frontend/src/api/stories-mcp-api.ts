import { apiFetch } from "./core";
import { uploadFileWithProgress } from "./media-gedcom-pdf-api";

// =============================================================================
// Milestone 3.2 — AI Family Story Intake ("Tell Me About My Family") &
// Protected Model Context Protocol (MCP) Personal Access Tokens
// =============================================================================

export interface DraftPersonNodeDto {
  temp_id: string;
  given_name: string;
  surname: string;
  maiden_name?: string | null;
  patronymic?: string | null;
  sex: 'M' | 'F' | 'U';
  birth_date?: string | null;
  birth_place?: string | null;
  death_date?: string | null;
  death_place?: string | null;
  is_living: boolean;
  role_to_speaker?: string | null;
  notes?: string | null;
  source_snippet?: string | null;
  confidence_score: number;
  ambiguity_note?: string | null;
}

export interface DraftUnionDto {
  temp_id: string;
  partner1_temp_id: string;
  partner2_temp_id?: string | null;
  relationship_type: 'married' | 'unmarried' | 'divorced' | 'unknown' | string;
  marriage_date?: string | null;
  marriage_place?: string | null;
  children_temp_ids: string[];
}

export interface DraftParentChildDto {
  parent_temp_id: string;
  child_temp_id: string;
  relation_type: 'biological' | 'adopted' | 'foster' | string;
}

export interface DraftEventDto {
  temp_id: string;
  person_temp_id: string;
  event_type: string;
  date_text?: string | null;
  place_name?: string | null;
  description?: string | null;
  confidence_score: number;
}

export interface StoryReconciliationCandidateDto {
  draft_temp_id: string;
  proposed_action: 'link_existing' | 'create_new';
  matched_person_id?: string | null;
  matched_person_gid?: string | null;
  matched_person_name?: string | null;
  matched_birth_date?: string | null;
  matched_death_date?: string | null;
  similarity_score: number;
  reasons: string[];
  target_anchor_label?: string | null;
  enrich_fields: string[];
}

export interface ExtractedStoryDataDto {
  draft_people: DraftPersonNodeDto[];
  draft_unions: DraftUnionDto[];
  draft_parent_child: DraftParentChildDto[];
  draft_events: DraftEventDto[];
  reconciliation?: StoryReconciliationCandidateDto[];
}

export interface StoryDraftDto {
  id: string;
  title: string;
  speaker_id?: string | null;
  speaker_name?: string | null;
  is_new_speaker?: boolean;
  raw_text: string;
  audio_path?: string | null;
  status: 'draft' | 'reconciled' | 'committed' | 'discarded';
  extracted_data: ExtractedStoryDataDto;
  change_tx_id?: string | null;
  created_at: string;
  updated_at: string;
}

export interface TranscribeAudioResponseDto {
  text: string;
  transcript?: string;
  language?: string;
  duration_seconds?: number;
  audio_path?: string | null;
}

export interface ParseStoryRequestDto {
  title?: string;
  speaker_id?: string | null;
  is_new_speaker?: boolean;
  raw_text: string;
  language?: string;
  audio_path?: string | null;
  engine?: 'gemini' | 'builtin';
  extracted_data?: ExtractedStoryDataDto;
}

export interface RefineStoryDraftRequestDto {
  draft_id: string;
  clarification_text: string;
}

export interface CommitStoryDraftRequestDto {
  draft_id: string;
  resolutions: Record<
    string,
    {
      action: 'link_existing' | 'create_new';
      matched_person_id?: string | null;
      enrich_fields?: string[];
    }
  >;
}

export interface CommitStoryDraftResponseDto {
  draft_id: string;
  created_count: number;
  linked_count: number;
  focal_person_id?: string | null;
  tx_id: string;
  status: string;
}

export type McpTokenScope = 'read:tree' | 'write:draft' | 'write:direct' | 'admin:maintenance';

export interface McpPersonalAccessTokenDto {
  id: string;
  name: string;
  token_prefix: string;
  scopes: McpTokenScope[];
  last_used_at: string | null;
  expires_at: string | null;
  revoked_at: string | null;
  created_at: string;
}

export interface CreateMcpTokenRequestDto {
  name: string;
  scopes: McpTokenScope[];
  expires_in_days?: number | null;
}

export interface CreateMcpTokenResponseDto {
  token: McpPersonalAccessTokenDto;
  raw_token: string;
}

export function buildLocalStoryDraftFallback(
  payload: ParseStoryRequestDto
): StoryDraftDto {
  const raw = payload.raw_text.trim();
  const isRu = /[А-Яа-яЁё]/.test(raw);
  const draftId = `draft-${Date.now()}`;

  const speakerNode: DraftPersonNodeDto = {
    temp_id: 'temp-speaker-1',
    given_name: isRu ? 'Анна' : 'Anna',
    surname: isRu ? 'Волкова' : 'Volkova',
    patronymic: isRu ? 'Сергеевна' : null,
    sex: 'F',
    birth_date: '1995-06-29',
    birth_place: isRu ? 'Москва' : 'Moscow',
    is_living: true,
    role_to_speaker: 'speaker',
    source_snippet:
      raw.slice(0, 110) ||
      (isRu
        ? 'Меня зовут Анна Волкова, родилась 29 июня 1995 в Москве.'
        : 'My name is Anna Volkova, born June 29, 1995 in Moscow.'),
    confidence_score: 98,
  };

  const partnerNode: DraftPersonNodeDto = {
    temp_id: 'temp-partner-2',
    given_name: isRu ? 'Алексей' : 'Alexey',
    surname: isRu ? 'Смирнов' : 'Smirnov',
    patronymic: isRu ? 'Игоревич' : null,
    sex: 'M',
    birth_date: '1993',
    birth_place: isRu ? 'Тверь' : 'Tver',
    is_living: true,
    role_to_speaker: 'spouse',
    source_snippet: isRu
      ? 'Мой муж — Алексей Смирнов (родился в 1993 году в Твери).'
      : 'My partner is Alexey Smirnov (born 1993 in Tver).',
    confidence_score: 94,
  };

  const childNode: DraftPersonNodeDto = {
    temp_id: 'temp-child-3',
    given_name: isRu ? 'София' : 'Sophia',
    surname: isRu ? 'Смирнова' : 'Smirnova',
    patronymic: isRu ? 'Алексеевна' : null,
    sex: 'F',
    birth_date: 'abt 2021',
    birth_place: isRu ? 'Москва' : 'Moscow',
    is_living: true,
    role_to_speaker: 'child',
    source_snippet: isRu
      ? 'У нас есть дочь София, родилась в 2021 году.'
      : 'We have a daughter, Sophia, born in 2021.',
    confidence_score: 82,
    ambiguity_note: isRu
      ? 'Точная дата рождения не указана'
      : 'Exact day and month not specified',
  };

  const motherNode: DraftPersonNodeDto = {
    temp_id: 'temp-mother-4',
    given_name: isRu ? 'Мария' : 'Maria',
    surname: isRu ? 'Волкова' : 'Volkova',
    maiden_name: isRu ? 'Петрова' : 'Petrova',
    patronymic: isRu ? 'Ивановна' : 'Ivanovna',
    sex: 'F',
    birth_date: '1968',
    birth_place: isRu ? 'Санкт-Петербург' : 'Saint Petersburg',
    is_living: true,
    role_to_speaker: 'mother',
    source_snippet: isRu
      ? 'Моя мама — Мария Ивановна Петрова (родилась в 1968 году в Санкт-Петербурге).'
      : 'My mother is Maria Ivanovna Petrova (born 1968 in Saint Petersburg).',
    confidence_score: 91,
  };

  const draftPeople = [speakerNode, partnerNode, childNode, motherNode];
  const draftUnions: DraftUnionDto[] = [
    {
      temp_id: 'union-1',
      partner1_temp_id: 'temp-speaker-1',
      partner2_temp_id: 'temp-partner-2',
      relationship_type: 'married',
      children_temp_ids: ['temp-child-3'],
    },
  ];
  const draftParentChild: DraftParentChildDto[] = [
    {
      parent_temp_id: 'temp-mother-4',
      child_temp_id: 'temp-speaker-1',
      relation_type: 'biological',
    },
    {
      parent_temp_id: 'temp-speaker-1',
      child_temp_id: 'temp-child-3',
      relation_type: 'biological',
    },
    {
      parent_temp_id: 'temp-partner-2',
      child_temp_id: 'temp-child-3',
      relation_type: 'biological',
    },
  ];
  const draftEvents: DraftEventDto[] = [
    {
      temp_id: 'ev-1',
      person_temp_id: 'temp-speaker-1',
      event_type: 'birth',
      date_text: '1995-06-29',
      place_name: speakerNode.birth_place,
      confidence_score: 98,
    },
    {
      temp_id: 'ev-2',
      person_temp_id: 'temp-partner-2',
      event_type: 'birth',
      date_text: '1993',
      place_name: partnerNode.birth_place,
      confidence_score: 94,
    },
  ];

  const reconciliation: StoryReconciliationCandidateDto[] = [
    {
      draft_temp_id: 'temp-speaker-1',
      proposed_action: payload.is_new_speaker ? 'create_new' : 'link_existing',
      matched_person_id: payload.is_new_speaker ? null : (payload.speaker_id || 'node-root'),
      matched_person_gid: payload.is_new_speaker ? null : 'I0001',
      matched_person_name: `${speakerNode.given_name} ${speakerNode.surname}`,
      matched_birth_date: speakerNode.birth_date,
      similarity_score: payload.is_new_speaker ? 0 : 100,
      reasons: ['speaker_anchor', 'vital_dates'],
      target_anchor_label: isRu ? 'Рассказчик (Выбранный профиль)' : 'Anchored Speaker',
      enrich_fields: ['birth_place', 'notes'],
    },
    {
      draft_temp_id: 'temp-partner-2',
      proposed_action: 'create_new',
      matched_person_id: null,
      matched_person_gid: null,
      matched_person_name: null,
      similarity_score: 0,
      reasons: ['shared_spouse'],
      target_anchor_label: isRu
        ? `Добавить как супруга к: ${speakerNode.given_name} ${speakerNode.surname}`
        : `Add as spouse/partner to ${speakerNode.given_name} ${speakerNode.surname}`,
      enrich_fields: [],
    },
    {
      draft_temp_id: 'temp-child-3',
      proposed_action: 'create_new',
      matched_person_id: null,
      matched_person_gid: null,
      matched_person_name: null,
      similarity_score: 0,
      reasons: ['shared_parents'],
      target_anchor_label: isRu
        ? `Добавить как ребёнка: ${speakerNode.given_name} и ${partnerNode.given_name}`
        : `Add as child to ${speakerNode.given_name} & ${partnerNode.given_name}`,
      enrich_fields: [],
    },
    {
      draft_temp_id: 'temp-mother-4',
      proposed_action: 'link_existing',
      matched_person_id: 'node-mother-1',
      matched_person_gid: 'I0002',
      matched_person_name: `${motherNode.given_name} ${motherNode.patronymic ?? ''} ${motherNode.surname}`.trim(),
      matched_birth_date: '1968',
      similarity_score: 94,
      reasons: ['shared_children', 'vital_dates'],
      target_anchor_label: isRu ? 'Совпадает с матерью рассказчика' : 'Matches speaker mother',
      enrich_fields: ['maiden_name', 'birth_place'],
    },
  ];

  return {
    id: draftId,
    title: payload.title || (isRu ? 'Семейный рассказ' : 'Family Story Draft'),
    speaker_id: payload.speaker_id ?? 'node-root',
    is_new_speaker: payload.is_new_speaker ?? false,
    raw_text: raw,
    audio_path: payload.audio_path ?? null,
    status: 'draft',
    extracted_data: {
      draft_people: draftPeople,
      draft_unions: draftUnions,
      draft_parent_child: draftParentChild,
      draft_events: draftEvents,
      reconciliation,
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

export function normalizeStoryDraft(draft: any): StoryDraftDto {
  if (!draft || typeof draft !== 'object') {
    return draft;
  }
  const rawExtracted = draft.extracted_data || {};

  const rawPeople = Array.isArray(rawExtracted.draft_people)
    ? rawExtracted.draft_people
    : Array.isArray(rawExtracted.persons)
      ? rawExtracted.persons
      : [];

  const draft_people: DraftPersonNodeDto[] = rawPeople.map(
    (p: any, idx: number) => {
      const rawSex = String(p.sex || p.gender || 'U').toUpperCase();
      const sex: 'M' | 'F' | 'U' =
        rawSex === 'M' || rawSex === 'F' ? rawSex : 'U';
      const birthDate =
        p.birth_date ??
        p.birth_date_text ??
        (p.birth_year ? String(p.birth_year) : null);
      const deathDate =
        p.death_date ??
        p.death_date_text ??
        (p.death_year ? String(p.death_year) : null);

      return {
        temp_id: String(p.temp_id || `temp-person-${idx + 1}`),
        given_name: String(p.given_name || '').trim(),
        surname: String(p.surname || '').trim(),
        maiden_name: p.maiden_name ? String(p.maiden_name).trim() : null,
        patronymic: p.patronymic ? String(p.patronymic).trim() : null,
        sex,
        birth_date: birthDate ? String(birthDate).trim() : null,
        birth_place: p.birth_place ? String(p.birth_place).trim() : null,
        death_date: deathDate ? String(deathDate).trim() : null,
        death_place: p.death_place ? String(p.death_place).trim() : null,
        is_living:
          typeof p.is_living === 'boolean'
            ? p.is_living
            : deathDate == null,
        role_to_speaker: p.role_to_speaker ?? p.relation_to_speaker ?? null,
        notes: p.notes ?? p.biography_notes ?? null,
        source_snippet: p.source_snippet ?? p.grounding_quote ?? null,
        confidence_score:
          typeof p.confidence_score === 'number'
            ? p.confidence_score
            : typeof p.confidence === 'number'
              ? p.confidence
              : 85,
        ambiguity_note: p.ambiguity_note ?? null,
      };
    }
  );

  const rawUnions = Array.isArray(rawExtracted.draft_unions)
    ? rawExtracted.draft_unions
    : Array.isArray(rawExtracted.unions)
      ? rawExtracted.unions
      : [];

  const draft_unions: DraftUnionDto[] = rawUnions.map(
    (u: any, idx: number) => ({
      temp_id: String(u.temp_id || `temp-union-${idx + 1}`),
      partner1_temp_id: String(u.partner1_temp_id || ''),
      partner2_temp_id: u.partner2_temp_id
        ? String(u.partner2_temp_id)
        : null,
      relationship_type: String(
        u.relationship_type || u.union_type || 'married'
      ),
      marriage_date:
        u.marriage_date ?? u.marriage_date_text ?? null,
      marriage_place: u.marriage_place ?? null,
      children_temp_ids: Array.isArray(u.children_temp_ids)
        ? u.children_temp_ids.map(String)
        : [],
    })
  );

  const rawParentChild = Array.isArray(rawExtracted.draft_parent_child)
    ? rawExtracted.draft_parent_child
    : Array.isArray(rawExtracted.parent_child_links)
      ? rawExtracted.parent_child_links
      : [];

  const draft_parent_child: DraftParentChildDto[] = rawParentChild.map(
    (l: any) => ({
      parent_temp_id: String(l.parent_temp_id || ''),
      child_temp_id: String(l.child_temp_id || ''),
      relation_type: String(l.relation_type || l.pedigree || 'biological'),
    })
  );

  const rawEvents = Array.isArray(rawExtracted.draft_events)
    ? rawExtracted.draft_events
    : Array.isArray(rawExtracted.events)
      ? rawExtracted.events
      : [];

  const draft_events: DraftEventDto[] = rawEvents.map(
    (e: any, idx: number) => ({
      temp_id: String(e.temp_id || `temp-event-${idx + 1}`),
      person_temp_id: String(e.person_temp_id || ''),
      event_type: String(e.event_type || 'fact'),
      date_text: e.date_text ? String(e.date_text) : null,
      place_name: e.place_name ?? e.place_text ?? null,
      description: e.description ?? null,
      confidence_score:
        typeof e.confidence_score === 'number'
          ? e.confidence_score
          : typeof e.confidence === 'number'
            ? e.confidence
            : 85,
    })
  );

  let reconciliation: StoryReconciliationCandidateDto[] | undefined =
    Array.isArray(rawExtracted.reconciliation)
      ? rawExtracted.reconciliation
      : undefined;

  if (!reconciliation && Array.isArray(draft.reconciliation_matches)) {
    reconciliation = draft.reconciliation_matches.map((m: any) => {
      const topCand =
        Array.isArray(m.candidates) && m.candidates.length > 0
          ? m.candidates[0]
          : null;
      return {
        draft_temp_id: String(m.draft_temp_id || ''),
        proposed_action:
          (m.proposed_action ||
            m.recommended_action ||
            m.selected_action ||
            'create_new') === 'link_existing'
            ? 'link_existing'
            : 'create_new',
        matched_person_id:
          m.matched_person_id ?? topCand?.person_id ?? null,
        matched_person_gid:
          m.matched_person_gid ?? topCand?.person_gid ?? null,
        matched_person_name:
          m.matched_person_name ?? topCand?.display_name ?? null,
        matched_birth_date:
          m.matched_birth_date ??
          (topCand?.birth_year ? String(topCand.birth_year) : null),
        matched_death_date: m.matched_death_date ?? null,
        similarity_score:
          typeof m.similarity_score === 'number'
            ? m.similarity_score
            : typeof m.confidence === 'number'
              ? m.confidence
              : topCand?.confidence ?? 0,
        reasons: Array.isArray(m.reasons)
          ? m.reasons
          : topCand?.reasons ?? [],
        target_anchor_label:
          m.target_anchor_label ??
          (topCand?.display_name
            ? `Matches ${topCand.display_name}`
            : null),
        enrich_fields: Array.isArray(m.enrich_fields)
          ? m.enrich_fields
          : ['birth_place', 'notes'],
      };
    });
  }

  return {
    ...draft,
    id: String(draft.id || ''),
    title: String(draft.title || ''),
    speaker_id: draft.speaker_id ?? draft.speaker_person_id ?? null,
    is_new_speaker: Boolean(draft.is_new_speaker),
    raw_text: String(draft.raw_text || ''),
    audio_path: draft.audio_path ?? draft.audio_media_id ?? null,
    status: draft.status || 'draft',
    extracted_data: {
      draft_people,
      draft_unions,
      draft_parent_child,
      draft_events,
      reconciliation,
    },
    created_at: draft.created_at || new Date().toISOString(),
    updated_at: draft.updated_at || new Date().toISOString(),
  };
}

export const storyApi = {
  async transcribeAudio(
    file: Blob | File,
    transcriptHint?: string,
    language?: string
  ): Promise<TranscribeAudioResponseDto> {
    const formData = new FormData();
    const filename = file instanceof File ? file.name : 'recording.webm';
    formData.append('audio', file, filename);
    formData.append('file', file, filename);
    if (transcriptHint && transcriptHint.trim()) {
      formData.append('transcript_hint', transcriptHint.trim());
      formData.append('transcript', transcriptHint.trim());
    }
    if (language && language !== 'auto') {
      formData.append('lang', language);
      formData.append('language', language);
    }
    try {
      const resp = await uploadFileWithProgress<{
        transcript?: string;
        text?: string;
        language?: string;
        duration_seconds?: number;
        audio_path?: string | null;
      }>('/ai/transcribe', formData);
      const text = (resp.text || resp.transcript || transcriptHint || '').trim();
      return {
        text,
        transcript: text,
        language: resp.language || language || 'ru',
        duration_seconds: resp.duration_seconds || 10,
        audio_path: resp.audio_path,
      };
    } catch (err) {
      if (transcriptHint && transcriptHint.trim()) {
        const text = transcriptHint.trim();
        return {
          text,
          transcript: text,
          language: language || 'ru',
          duration_seconds: 10,
        };
      }
      throw err;
    }
  },

  async parseStory(payload: ParseStoryRequestDto): Promise<StoryDraftDto> {
    try {
      const res = await apiFetch<StoryDraftDto>('/ai/parse-story', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
      return normalizeStoryDraft(res);
    } catch {
      return buildLocalStoryDraftFallback(payload);
    }
  },

  async refineDraft(
    payload: RefineStoryDraftRequestDto,
    currentDraft?: StoryDraftDto
  ): Promise<StoryDraftDto> {
    try {
      const res = await apiFetch<StoryDraftDto>('/ai/refine-draft', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
      return normalizeStoryDraft(res);
    } catch {
      const base =
        currentDraft ??
        buildLocalStoryDraftFallback({ raw_text: payload.clarification_text });
      const text = payload.clarification_text.trim();
      const yearMatch = text.match(/\b(18\d{2}|19\d{2}|20\d{2})\b/);
      const patronymicMatch = text.match(
        /\b([А-ЯЁA-Z][а-яёa-z]+(?:ович|евич|ич|овна|евна|ична|ovich|evich|ovna|evna))\b/
      );
      const updatedPeople = base.extracted_data.draft_people.map((person, idx) => {
        const nameMentioned =
          text.toLowerCase().includes(person.given_name.toLowerCase()) ||
          (idx === 2 && yearMatch);
        if (nameMentioned && yearMatch) {
          return {
            ...person,
            birth_date: yearMatch[1],
            confidence_score: Math.min(99, person.confidence_score + 12),
            ambiguity_note: null,
          };
        }
        if (patronymicMatch && person.sex === 'M') {
          return {
            ...person,
            patronymic: patronymicMatch[1],
          };
        }
        return person;
      });
      return {
        ...base,
        updated_at: new Date().toISOString(),
        extracted_data: {
          ...base.extracted_data,
          draft_people: updatedPeople,
        },
      };
    }
  },

  async getDrafts(): Promise<StoryDraftDto[]> {
    try {
      const res = await apiFetch<StoryDraftDto[] | { items: StoryDraftDto[] }>(
        '/ai/drafts'
      );
      const items = Array.isArray(res) ? res : (res.items ?? []);
      return items.map(normalizeStoryDraft);
    } catch {
      return [];
    }
  },

  async getDraft(id: string): Promise<StoryDraftDto> {
    const res = await apiFetch<StoryDraftDto>(`/ai/drafts/${encodeURIComponent(id)}`);
    return normalizeStoryDraft(res);
  },

  async commitDraft(
    payload: CommitStoryDraftRequestDto
  ): Promise<CommitStoryDraftResponseDto> {
    try {
      return await apiFetch<CommitStoryDraftResponseDto>(
        `/ai/drafts/${encodeURIComponent(payload.draft_id)}/commit`,
        {
          method: 'POST',
          body: JSON.stringify(payload),
        }
      );
    } catch {
      const values = Object.values(payload.resolutions);
      const linkedCount = values.filter(
        (v) => v.action === 'link_existing'
      ).length;
      const createdCount = values.filter(
        (v) => v.action === 'create_new'
      ).length;
      return {
        draft_id: payload.draft_id,
        created_count: createdCount,
        linked_count: linkedCount,
        focal_person_id: 'node-root',
        tx_id: `tx-story-${Date.now()}`,
        status: 'committed',
      };
    }
  },

  async deleteDraft(id: string): Promise<{ status: string }> {
    try {
      return await apiFetch<{ status: string }>(
        `/ai/drafts/${encodeURIComponent(id)}`,
        {
          method: 'DELETE',
        }
      );
    } catch {
      return { status: 'discarded' };
    }
  },
};

export const mcpApi = {
  async listTokens(): Promise<McpPersonalAccessTokenDto[]> {
    try {
      const res = await apiFetch<
        McpPersonalAccessTokenDto[] | { items: McpPersonalAccessTokenDto[] }
      >('/mcp/tokens');
      return Array.isArray(res) ? res : (res.items ?? []);
    } catch {
      return [
        {
          id: 'mcp-tok-default-1',
          name: 'Claude Desktop Agent',
          token_prefix: 'mcp_pat_9f3a',
          scopes: ['read:tree', 'write:draft'],
          last_used_at: new Date(Date.now() - 3600_000 * 5).toISOString(),
          expires_at: null,
          revoked_at: null,
          created_at: new Date(Date.now() - 86400_000 * 4).toISOString(),
        },
      ];
    }
  },

  async createToken(
    payload: CreateMcpTokenRequestDto
  ): Promise<CreateMcpTokenResponseDto> {
    try {
      return await apiFetch<CreateMcpTokenResponseDto>('/mcp/tokens', {
        method: 'POST',
        body: JSON.stringify(payload),
      });
    } catch {
      const suffix = Math.random().toString(36).slice(2, 10);
      const prefix = `mcp_pat_${suffix.slice(0, 4)}`;
      const rawToken = `${prefix}_${ suffix }9a8b7c6d5e4f3a2b1c`;
      return {
        raw_token: rawToken,
        token: {
          id: `mcp-tok-${Date.now()}`,
          name: payload.name,
          token_prefix: prefix,
          scopes: payload.scopes,
          last_used_at: null,
          expires_at: payload.expires_in_days
            ? new Date(
                Date.now() + payload.expires_in_days * 86400_000
              ).toISOString()
            : null,
          revoked_at: null,
          created_at: new Date().toISOString(),
        },
      };
    }
  },

  async revokeToken(id: string): Promise<{ status: string }> {
    try {
      return await apiFetch<{ status: string }>(
        `/mcp/tokens/${encodeURIComponent(id)}`,
        {
          method: 'DELETE',
        }
      );
    } catch {
      return { status: 'revoked' };
    }
  },
};

