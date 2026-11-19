import type { Page, Route } from '@playwright/test';
import { editsToManifest } from '../src/lib/edits/manifest';
import type { EditManifest, Edits } from '../src/lib/types/edits';
import { json, NEUTRAL_RECORD } from './helpers';

export interface HistorySeed {
  edits: Edits | null;
  minutesAgo: number;
  action?: string;
  undone?: boolean;
}

export interface MockHistoryEntry {
  id: number;
  manifest_hash: string;
  deleted: boolean;
  manifest: EditManifest | null;
  created_at: string;
  action: string | null;
  undone: boolean;
}

export interface HistoryMock {
  entries: MockHistoryEntry[];
  restores: number[];
  saves: number;
}

const EDITS_PATH = /^\/api\/assets\/([^/]+)\/edits(?:\/(history|restore))?$/;

function stamp(minutesAgo: number): string {
  return new Date(Date.now() - minutesAgo * 60_000).toISOString();
}

function record(assetId: string, entry: MockHistoryEntry | undefined) {
  if (!entry || entry.deleted || !entry.manifest) return { ...NEUTRAL_RECORD, asset_id: assetId };
  return {
    ...NEUTRAL_RECORD,
    asset_id: assetId,
    manifest: entry.manifest,
    hash: entry.manifest_hash,
    updated_at: entry.created_at
  };
}

export async function mockHistory(page: Page, seeds: HistorySeed[]): Promise<HistoryMock> {
  let nextId = 0;
  const entry = (
    edits: Edits | null,
    minutesAgo: number,
    action: string | null,
    undone: boolean
  ): MockHistoryEntry => {
    const id = ++nextId;
    return {
      id,
      manifest_hash: `hist-hash-${id}`,
      deleted: edits === null,
      manifest: edits ? editsToManifest(edits) : null,
      created_at: stamp(minutesAgo),
      action,
      undone
    };
  };
  const state: HistoryMock = {
    entries: seeds
      .map((seed) => entry(seed.edits, seed.minutesAgo, seed.action ?? null, !!seed.undone))
      .reverse(),
    restores: [],
    saves: 0
  };
  const current = (): MockHistoryEntry | undefined => state.entries.find((item) => !item.undone);

  const save = (manifest: EditManifest | null, action: string | null): MockHistoryEntry => {
    state.saves++;
    state.entries = state.entries.filter((item) => !item.undone);
    const saved = { ...entry(null, 0, action, false), deleted: manifest === null, manifest };
    state.entries.unshift(saved);
    return saved;
  };

  await page.route(/\/api\/assets\/[^/]+\/edits(\/(history|restore))?$/, async (route: Route) => {
    const request = route.request();
    const match = new URL(request.url()).pathname.match(EDITS_PATH);
    if (!match) return route.fallback();
    const [, assetId = '', sub] = match;
    const method = request.method();
    const body = (request.postDataJSON() as Record<string, unknown> | null) ?? {};

    if (sub === 'history') return route.fulfill(json(state.entries));
    if (sub === 'restore') {
      const target = Number(body.entry_id);
      const restored = state.entries.find((item) => item.id === target);
      if (!restored) return route.fulfill(json({ code: 'not_found', message: 'not found' }, 404));
      state.restores.push(target);
      state.entries = state.entries.map((item) => ({ ...item, undone: item.id > target }));
      if (restored.deleted) return route.fulfill({ status: 204, body: '' });
      return route.fulfill(json(record(assetId, restored)));
    }
    if (method === 'PUT') {
      const saved = save(body.manifest as EditManifest, (body.action as string | null) ?? null);
      return route.fulfill(json(record(assetId, saved)));
    }
    if (method === 'DELETE') {
      save(null, (body.action as string | null) ?? 'Reset to original');
      return route.fulfill({ status: 204, body: '' });
    }
    return route.fulfill(json(record(assetId, current())));
  });
  return state;
}
