import type { Page } from '@playwright/test';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits, type EditManifest } from '../src/lib/types/edits';
import { json, NEUTRAL_RECORD } from './helpers';

export interface MockSnapshot {
  id: string;
  name: string;
  manifest_hash: string;
  manifest: EditManifest;
  created_at: string;
  updated_at: string;
}

export interface SnapshotSeed {
  name: string;
  exposure: number;
  minutesAgo: number;
}

export interface SnapshotMock {
  items: MockSnapshot[];
  restores: Array<{ id: string; ifMatch: string | undefined }>;
  updates: Array<{ id: string; body: Record<string, unknown> }>;
}

export function exposureManifest(exposure: number): EditManifest {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return editsToManifest(edits);
}

const SNAPSHOT_PATH = /^\/api\/assets\/([^/]+)\/snapshots(?:\/([^/]+))?(\/restore)?$/;

export async function mockSnapshots(page: Page, seeds: SnapshotSeed[] = []): Promise<SnapshotMock> {
  let seq = 0;
  const stamp = (minutesAgo = 0): string =>
    new Date(Date.now() - minutesAgo * 60_000).toISOString();
  const nextHash = (): string => `snap-hash-${++seq}`;
  const state: SnapshotMock = {
    items: seeds.map((seed, index) => ({
      id: `snap-${index + 1}`,
      name: seed.name,
      manifest_hash: nextHash(),
      manifest: exposureManifest(seed.exposure),
      created_at: stamp(seed.minutesAgo),
      updated_at: stamp(seed.minutesAgo)
    })),
    restores: [],
    updates: []
  };
  let created = state.items.length;

  await page.route(/\/api\/assets\/[^/]+\/snapshots/, async (route) => {
    const request = route.request();
    const match = new URL(request.url()).pathname.match(SNAPSHOT_PATH);
    if (!match) return route.fallback();
    const [, assetId, id, restore] = match;
    const method = request.method();
    const index = state.items.findIndex((item) => item.id === id);
    const item = state.items[index];

    if (!id && method === 'GET') return route.fulfill(json(state.items));
    if (!id && method === 'POST') {
      const body = request.postDataJSON() as { name: string; manifest: EditManifest };
      const snapshot: MockSnapshot = {
        id: `snap-${++created}`,
        name: body.name,
        manifest_hash: nextHash(),
        manifest: body.manifest,
        created_at: stamp(),
        updated_at: stamp()
      };
      state.items.unshift(snapshot);
      return route.fulfill(json(snapshot));
    }
    if (!item) return route.fulfill(json({ code: 'not_found', message: 'not found' }, 404));
    if (restore) {
      state.restores.push({ id: item.id, ifMatch: request.headers()['if-match'] });
      return route.fulfill(
        json({
          ...NEUTRAL_RECORD,
          asset_id: assetId,
          manifest: item.manifest,
          hash: item.manifest_hash,
          updated_at: stamp()
        })
      );
    }
    if (method === 'DELETE') {
      state.items.splice(index, 1);
      return route.fulfill({ status: 204, body: '' });
    }
    const body = request.postDataJSON() as { name?: string; manifest?: EditManifest };
    state.updates.push({ id: item.id, body });
    const updated: MockSnapshot = {
      ...item,
      name: body.name ?? item.name,
      manifest: body.manifest ?? item.manifest,
      manifest_hash: body.manifest ? nextHash() : item.manifest_hash,
      updated_at: stamp()
    };
    state.items[index] = updated;
    return route.fulfill(json(updated));
  });
  return state;
}
