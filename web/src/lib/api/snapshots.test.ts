import { editsToManifest } from '#lib/edits/manifest.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ConflictError } from '#lib/api/client.ts';
import { createSnapshot, restoreSnapshot, updateSnapshot } from '#lib/api/snapshots.ts';

interface Sent {
  url: string;
  method?: string;
  headers: Record<string, string>;
  body: unknown;
}

function stubFetch(response: unknown, status = 200): () => Sent {
  let sent: Sent = { url: '', headers: {}, body: null };
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: RequestInfo, init?: RequestInit) => {
      sent = {
        url: String(input),
        method: init?.method,
        headers: (init?.headers ?? {}) as Record<string, string>,
        body: init?.body ? JSON.parse(String(init.body)) : null
      };
      return new Response(JSON.stringify(response), {
        status,
        headers: { 'content-type': 'application/json' }
      });
    })
  );
  return () => sent;
}

const edits = neutralEdits();
edits.basic.exposure_ev = 0.5;
const manifest = editsToManifest(edits);
const item = {
  id: 'snap-1',
  name: 'Warm',
  manifest_hash: 'h1',
  manifest,
  created_at: '2026-11-01T00:00:00Z',
  updated_at: '2026-11-01T00:00:00Z'
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('snapshots api', () => {
  it('creates a snapshot from edits and returns edits', async () => {
    const sent = stubFetch(item);
    const created = await createSnapshot('asset 1', 'Warm', edits);
    expect(sent()).toMatchObject({
      url: '/api/assets/asset%201/snapshots',
      method: 'POST',
      body: { name: 'Warm', manifest }
    });
    expect(created.edits.basic.exposure_ev).toBe(0.5);
    expect(created).not.toHaveProperty('manifest');
  });

  it('patches only the given fields', async () => {
    const sent = stubFetch(item);
    await updateSnapshot('a', 'snap-1', { name: 'Cool' });
    expect(sent()).toMatchObject({ method: 'PATCH', body: { name: 'Cool' } });
    await updateSnapshot('a', 'snap-1', { edits });
    expect(sent().body).toEqual({ manifest });
  });

  it('restores with the base hash and announces the save', async () => {
    const sent = stubFetch({ hash: 'h2', updated_at: 'now', manifest });
    const saved = vi.fn();
    const target = new EventTarget();
    target.addEventListener('immich-edit:edits-saved', saved);
    vi.stubGlobal('window', target);
    await restoreSnapshot('a', 'snap-1', 'h0');
    expect(sent()).toMatchObject({
      url: '/api/assets/a/snapshots/snap-1/restore',
      method: 'POST',
      headers: { 'if-match': 'h0' }
    });
    expect(saved).toHaveBeenCalledTimes(1);
  });

  it('turns a 409 restore into a conflict', async () => {
    stubFetch({ hash: 'remote', manifest }, 409);
    await expect(restoreSnapshot('a', 'snap-1', 'h0')).rejects.toBeInstanceOf(ConflictError);
  });
});
