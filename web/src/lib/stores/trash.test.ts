import { deleteCopy } from '$lib/api/copies';
import { restoreAssets, trashAssets } from '$lib/api/trash';
import type { AssetSummary } from '$lib/types/album';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { browseView } from './browseView.svelte';
import { browsing } from './browsing.svelte';
import { immichCapabilities } from './immichCapabilities.svelte';
import { selection } from './selection.svelte';
import { session } from './session.svelte';
import { toasts } from './toasts.svelte';
import { trash } from './trash.svelte';

vi.mock('$lib/api/trash', () => ({
  trashAssets: vi.fn().mockResolvedValue({ count: 1 }),
  restoreAssets: vi.fn().mockResolvedValue({ count: 1 })
}));
vi.mock('$lib/api/copies', () => ({ deleteCopy: vi.fn().mockResolvedValue(undefined) }));
vi.mock('$lib/api/immich', () => ({
  getImmichCapabilities: vi.fn().mockResolvedValue({
    immich_version: '3.3.0',
    structured_search: true,
    smart_search: true,
    ocr: true,
    trash: false,
    trash_days: null
  })
}));

const A = '11111111-1111-1111-1111-111111111111';
const B = '22222222-2222-2222-2222-222222222222';
const C = '33333333-3333-3333-3333-333333333333';

function asset(id: string, extra: Partial<AssetSummary> = {}): AssetSummary {
  return { id, originalFileName: `${id}.arw`, ownerId: 'me', ...extra } as AssetSummary;
}

async function confirmed(picked: AssetSummary[]): Promise<boolean> {
  const done = trash.request(picked);
  await vi.waitFor(() => expect(trash.plan).not.toBeNull());
  trash.confirm();
  return done;
}

beforeEach(() => {
  vi.clearAllMocks();
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  immichCapabilities.trash = true;
  toasts.items = [];
  selection.clear();
  browseView.closeLoupe();
  browsing.set([asset(A), asset(`${A}_1`, { copyOf: A }), asset(B), asset(C)]);
});

describe('trash flow', () => {
  it('trashes after confirmation, leaves the grid in place and undoes', async () => {
    const feed = browsing.assets;
    selection.selectLoaded([A, B]);
    browseView.openLoupe(A);

    expect(await confirmed([asset(A)])).toBe(true);

    expect(trashAssets).toHaveBeenCalledWith([A]);
    expect(browsing.assets).toBe(feed);
    expect(browsing.assets.map((a) => a.id)).toEqual([B, C]);
    expect([...selection.selected]).toEqual([B]);
    expect(browseView.loupeId).toBe(B);

    toasts.items.at(-1)?.action?.run();
    await vi.waitFor(() => expect(restoreAssets).toHaveBeenCalledWith([A]));
    await vi.waitFor(() => expect(browsing.assets.map((a) => a.id)).toEqual([A, `${A}_1`, B, C]));
  });

  it('does nothing when the dialog is cancelled', async () => {
    const done = trash.request([asset(A)]);
    await vi.waitFor(() => expect(trash.plan).not.toBeNull());
    trash.cancel();

    expect(await done).toBe(false);
    expect(trashAssets).not.toHaveBeenCalled();
    expect(browsing.assets).toHaveLength(4);
  });

  it('deletes copies locally without the Immich trash', async () => {
    expect(await confirmed([asset(`${A}_1`, { copyOf: A })])).toBe(true);

    expect(deleteCopy).toHaveBeenCalledWith(`${A}_1`);
    expect(trashAssets).not.toHaveBeenCalled();
    expect(browsing.assets.map((a) => a.id)).toEqual([A, B, C]);
  });

  it('refuses masters while the Immich trash is off', async () => {
    immichCapabilities.trash = false;

    expect(await trash.request([asset(A)])).toBe(false);

    expect(trash.plan).toBeNull();
    expect(trashAssets).not.toHaveBeenCalled();
    expect(toasts.items.at(-1)?.kind).toBe('warn');
  });

  it('keeps what Immich refused in the grid', async () => {
    vi.mocked(trashAssets).mockRejectedValueOnce(new Error('Missing required permission'));

    expect(await confirmed([asset(A)])).toBe(false);

    expect(browsing.assets).toHaveLength(4);
    expect(toasts.items.at(-1)?.kind).toBe('error');
  });
});
