import { describe, expect, it, vi, afterEach } from 'vitest';
import {
  mcpApi,
  normalizeStoryDraft,
  storyApi,
} from '../../src/api/stories-mcp-api';

describe('stories-mcp-api contract & stability (STAB-01..STAB-04)', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('normalizes backend mcpApi.createToken response with token string and token_info object', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          token: 'mcp_pat_abc123_secret_xyz',
          token_info: {
            id: 'tok-99',
            name: 'Claude Desktop',
            token_prefix: 'mcp_pat_abc1',
            scopes: ['read:tree', 'write:draft'],
            last_used_at: null,
            expires_at: null,
            revoked_at: null,
            created_at: '2026-11-07T10:00:00Z',
          },
        }),
        { status: 201, headers: { 'Content-Type': 'application/json' } }
      )
    );

    const res = await mcpApi.createToken({
      name: 'Claude Desktop',
      scopes: ['read:tree', 'write:draft'],
    });

    expect(res.raw_token).toBe('mcp_pat_abc123_secret_xyz');
    expect(res.token.id).toBe('tok-99');
    expect(res.token.name).toBe('Claude Desktop');
    expect(res.token.token_prefix).toBe('mcp_pat_abc1');
    expect(res.token.scopes).toEqual(['read:tree', 'write:draft']);
  });

  it('normalizes story draft reconciliation link_enrich to link_existing and maps candidate fields', () => {
    const normalized = normalizeStoryDraft({
      id: 'draft-1',
      title: 'Story',
      speaker_person_id: 'person-speaker-1',
      raw_text: 'Sample narrative',
      status: 'reconciled',
      extracted_data: {
        persons: [
          {
            temp_id: 'p1',
            given_name: 'Anna',
            surname: 'Volkova',
            gender: 'F',
            birth_year: 1995,
          },
        ],
      },
      reconciliation_matches: [
        {
          draft_person_id: 'p1',
          recommended_action: 'link_enrich',
          candidate_person_id: 'person-speaker-1',
          candidate_display_name: 'Anna Volkova',
          confidence: 92,
          reasons: ['name_match', 'birth_year'],
        },
      ],
    });

    expect(normalized.speaker_id).toBe('person-speaker-1');
    expect(normalized.speaker_person_id).toBe('person-speaker-1');
    expect(normalized.extracted_data.reconciliation).toHaveLength(1);
    const rec = normalized.extracted_data.reconciliation![0];
    expect(rec.draft_temp_id).toBe('p1');
    expect(rec.proposed_action).toBe('link_existing');
    expect(rec.matched_person_id).toBe('person-speaker-1');
    expect(rec.matched_person_name).toBe('Anna Volkova');
    expect(rec.similarity_score).toBe(92);
  });

  it('sends match_overrides and normalizes counts, tx_id, and focal_person_id in storyApi.commitDraft', async () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          draft_id: 'draft-42',
          transaction_id: 'tx-999',
          created_person_ids: ['new-1', 'new-2'],
          updated_person_ids: ['existing-1'],
          status: 'committed',
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    );

    const res = await storyApi.commitDraft({
      draft_id: 'draft-42',
      resolutions: {
        'temp-1': {
          action: 'link_existing',
          matched_person_id: 'existing-1',
        },
        'temp-2': {
          action: 'create_new',
        },
        'temp-3': {
          action: 'create_new',
        },
      },
    });

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const [, init] = fetchSpy.mock.calls[0];
    const sentBody = JSON.parse(String(init?.body));
    expect(sentBody.match_overrides).toEqual([
      {
        draft_person_id: 'temp-1',
        selected_action: 'link_enrich',
        target_person_id: 'existing-1',
      },
      {
        draft_person_id: 'temp-2',
        selected_action: 'create_new',
        target_person_id: undefined,
      },
      {
        draft_person_id: 'temp-3',
        selected_action: 'create_new',
        target_person_id: undefined,
      },
    ]);

    expect(res.created_count).toBe(2);
    expect(res.linked_count).toBe(1);
    expect(res.tx_id).toBe('tx-999');
    expect(res.transaction_id).toBe('tx-999');
    expect(res.focal_person_id).toBe('new-1');
    expect(res.created_person_ids).toEqual(['new-1', 'new-2']);
    expect(res.updated_person_ids).toEqual(['existing-1']);
  });

  it('rethrows ApiError when HTTP status > 0 instead of silently falling back to demo data', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation(async () =>
      new Response(
        JSON.stringify({
          error: {
            code: 'forbidden',
            message: 'Insufficient permissions',
          },
        }),
        { status: 403, headers: { 'Content-Type': 'application/json' } }
      )
    );

    await expect(
      storyApi.parseStory({
        speaker_id: 'p-1',
        raw_text: 'Test story',
      })
    ).rejects.toMatchObject({ status: 403, code: 'forbidden' });

    await expect(
      storyApi.commitDraft({
        draft_id: 'd-1',
        resolutions: {},
      })
    ).rejects.toMatchObject({ status: 403, code: 'forbidden' });

    await expect(
      mcpApi.createToken({
        name: 'Test',
        scopes: ['read:tree'],
      })
    ).rejects.toMatchObject({ status: 403, code: 'forbidden' });
  });
});
