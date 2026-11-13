import React from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ToastProvider } from '../../src/components/ui';
import { ProfileFamilyTab } from '../../src/components/profile';
import {
  ApiError,
  deleteFamily,
  detachFamilyChild,
  detachFamilyPartner,
  moveToFamily,
  updateFamily,
} from '../../src/api/client';
import { getMoveToFamilyErrorMessage } from '../../src/lib/family-move-errors';
import en from '../../src/i18n/en.json';
import ru from '../../src/i18n/ru.json';

function lookup(dict: unknown, key: string): string {
  const v = key
    .split('.')
    .reduce<unknown>((o, k) => (o as Record<string, unknown> | undefined)?.[k], dict);
  return typeof v === 'string' ? v : key;
}

const rel = (id: string, sex: string) => ({
  id,
  gid: `I${id}`,
  display_name: `Person ${id}`,
  sex,
  is_living: false,
});

const families = {
  parent_families: [
    {
      family_id: 'f1',
      gid: 'F1',
      father: rel('1', 'M'),
      mother: rel('2', 'F'),
      siblings: [],
    },
  ],
  spouse_families: [],
} as unknown as React.ComponentProps<typeof ProfileFamilyTab>['families'];

function render(canEdit: boolean, withHandlers: boolean) {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale="en">
        <ToastProvider>
          <ProfileFamilyTab
            families={families}
            canEdit={canEdit}
            personId="3"
            onSelectRelative={() => {}}
            onQuickAdd={() => {}}
            onUnlinkParent={withHandlers ? () => {} : undefined}
            onMoveToFamily={withHandlers ? async () => {} : undefined}
          />
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

afterEach(() => vi.unstubAllGlobals());

describe('family link fixes (GAP-1 / GAP-4)', () => {
  it('shows unlink buttons on both parent cards and the change-parents action for editors', () => {
    const html = render(true, true);
    expect(html.match(/aria-label="Unlink from this parent"/g)?.length).toBe(2);
    expect(html).toContain('Change parents');
  });

  it('hides unlink and change-parents for read-only viewers', () => {
    const html = render(false, true);
    expect(html).not.toContain('Unlink from this parent');
    expect(html).not.toContain('Change parents');
  });

  it('maps every move-to-family field code to a translation in EN and RU', () => {
    const codes = [
      'target_required',
      'duplicate_partner',
      'not_a_child_of_family',
      'ambiguous_source_family',
      'same_family',
      'already_child_of_family',
    ];
    for (const code of codes) {
      const err = new ApiError(400, {
        code: 'validation_failed',
        message: 'Validation failed',
        field_errors: { family_id: code },
      });
      const enMsg = getMoveToFamilyErrorMessage(err, (k) => lookup(en, k));
      const ruMsg = getMoveToFamilyErrorMessage(err, (k) => lookup(ru, k));
      expect(enMsg).not.toContain('profile.family.moveErrors');
      expect(ruMsg).not.toContain('profile.family.moveErrors');
      expect(ruMsg).toMatch(/[А-Яа-я]/);
    }
  });

  it('reads the undo id from X-Transaction-Id on deleteFamily', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(null, { status: 204, headers: { 'X-Transaction-Id': 'tx-9' } })
      )
    );
    expect(await deleteFamily('f1')).toEqual({ tx_id: 'tx-9' });
  });

  it('maps a 404 for a mistyped family id to a friendly EN/RU message', () => {
    const err = new ApiError(404, { code: 'not_found', message: 'nope' });
    const enMsg = getMoveToFamilyErrorMessage(err, (k) => lookup(en, k));
    const ruMsg = getMoveToFamilyErrorMessage(err, (k) => lookup(ru, k));
    expect(enMsg).toContain('Family not found');
    expect(ruMsg).toContain('Семья не найдена');
  });

  it('calls the detach child/partner endpoints with optional version', async () => {
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ id: 'f1', tx_id: 'tx-5', dissolved: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    );
    vi.stubGlobal('fetch', fetchMock);
    const child = await detachFamilyChild('f1', 'c1', 3);
    const partner = await detachFamilyPartner('f1', 'p1');
    expect(child.dissolved).toBe(true);
    expect(partner.tx_id).toBe('tx-5');
    const calls = fetchMock.mock.calls as unknown as [string, RequestInit][];
    expect(calls[0][0]).toBe('/api/v1/families/f1/children/c1?version=3');
    expect(calls[0][1].method).toBe('DELETE');
    expect(calls[1][0]).toBe('/api/v1/families/f1/partners/p1');
  });

  it('returns tx_id/dissolved from updateFamily and posts move-to-family', async () => {
    const fetchMock = vi.fn(async (url: string) =>
      new Response(
        JSON.stringify(
          String(url).includes('move-to-family')
            ? { person_id: 'p', tx_id: 'tx-2' }
            : { id: 'f1', tx_id: 'tx-1', dissolved: true }
        ),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    );
    vi.stubGlobal('fetch', fetchMock);
    const upd = await updateFamily('f1', { version: 1, children: [] });
    expect(upd.tx_id).toBe('tx-1');
    expect(upd.dissolved).toBe(true);
    const mv = await moveToFamily('p', { family_id: 'f2' });
    expect(mv.tx_id).toBe('tx-2');
    const call = fetchMock.mock.calls[1] as unknown as [string, RequestInit];
    expect(call[0]).toBe('/api/v1/persons/p/move-to-family');
    expect(call[1].method).toBe('POST');
  });
});
