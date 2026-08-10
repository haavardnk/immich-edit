import { expect, type Page, type Route } from '@playwright/test';
import type { EditRecord } from '../src/lib/types/edits';
import type { PreviewMeta } from '../src/lib/types/preview';

export const PNG_1X1: Buffer = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64'
);

export const JPEG_BLOB: Buffer = Buffer.from(
  '/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/2wBDAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAr/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFAEBAAAAAAAAAAAAAAAAAAAAAP/EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhEDEQA/AL+AB//Z',
  'base64'
);

export const ASSET_ID = '00000000-0000-0000-0000-000000000001';
export const SESSION_USER = {
  id: '00000000-0000-0000-0000-0000000000aa',
  email: 'user@example.com',
  name: 'Test User',
  is_admin: true,
  auth_kind: 'password'
};

export const ASSET_SUMMARY = {
  id: ASSET_ID,
  originalFileName: 'IMG_0001.ARW',
  type: 'IMAGE',
  fileCreatedAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  checksum: 'aaaa',
  isFavorite: false,
  exifInfo: null
};

export const ASSET_DETAIL = {
  ...ASSET_SUMMARY,
  originalMimeType: 'image/x-sony-arw',
  tags: []
};

export const NEUTRAL_RECORD = {
  schema_version: 1,
  asset_id: ASSET_ID,
  immich_updated_at: null,
  immich_checksum: null,
  renderer_version: 'test',
  manifest: { schema_version: 1, ops: {} },
  updated_at: '2024-01-01T00:00:00Z',
  hash: 'hash-neutral'
};

export function json(data: unknown): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(data) };
}

export function png(): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'image/png', body: PNG_1X1 };
}

export const PNG_64: Buffer = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAATUlEQVR4nO3PMQEAAAgDoOU0sbGM4LUPGpAp27IICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICHwOg/Mxlm5wPxYAAAAASUVORK5CYII=',
  'base64'
);

export interface CopyRecord {
  id: string;
  source_asset_id: string;
  name: string | null;
  created_at: string;
}

export interface PreviewRequest {
  max_edge: number;
  edits: Record<string, unknown>;
  preview_mode: unknown;
}

export interface InstallOpts {
  assets?: Array<typeof ASSET_SUMMARY>;
  editRecord?: EditRecord;
  onExport?: (route: Route) => Promise<void> | void;
  onHistory?: (route: Route) => Promise<void> | void;
  onRestore?: (route: Route) => Promise<void> | void;
  onPreview?: (req: PreviewRequest) => void;
  onSave?: (body: Record<string, unknown>) => void;
  onSmart?: (body: Record<string, unknown>) => void;
  onMetadata?: (body: Record<string, unknown>) => void;
  smartFails?: boolean;
  previewBody?: Buffer;
  previewMeta?: Partial<PreviewMeta>;
}

const EMPTY_HISTOGRAM = { r: [], g: [], b: [], l: [] };

const PREVIEW_META: PreviewMeta = {
  asset_id: ASSET_ID,
  width: 64,
  height: 64,
  source_w: 64,
  source_h: 64,
  renderer: 'cpu',
  is_raw: false,
  histogram: EMPTY_HISTOGRAM
};

export async function installMocks(page: Page, opts: InstallOpts = {}): Promise<void> {
  const assets = opts.assets ?? [ASSET_SUMMARY];
  const copies: CopyRecord[] = [];
  const expanded = () =>
    assets.flatMap((a) => [
      a,
      ...copies
        .filter((c) => c.source_asset_id === a.id)
        .map((c) => ({ ...a, id: c.id, copyOf: a.id, copyLabel: c.name }))
    ]);
  const previewPng = (): Parameters<Route['fulfill']>[0] => {
    const base = opts.previewBody
      ? { status: 200, contentType: 'image/png', body: opts.previewBody }
      : png();
    if (!opts.previewMeta) return base;
    return { ...base, headers: { 'x-preview-meta-id': 'meta-1' } };
  };

  await page.route('**/api/**', async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const p = url.pathname;
    const method = req.method();

    if (p === '/api/setup/status') return route.fulfill(json({ configured: true }));
    if (p === '/api/auth/me') return route.fulfill(json(SESSION_USER));

    if (p === '/api/search/smart') {
      if (opts.onSmart) opts.onSmart((req.postDataJSON() as Record<string, unknown>) ?? {});
      if (opts.smartFails)
        return route.fulfill({ status: 500, contentType: 'application/json', body: '{}' });
      const items = expanded();
      return route.fulfill(
        json({ items, count: items.length, total: items.length, nextPage: null })
      );
    }
    if (p === '/api/search/metadata') {
      if (opts.onMetadata) opts.onMetadata((req.postDataJSON() as Record<string, unknown>) ?? {});
      const items = expanded();
      return route.fulfill(
        json({ items, count: items.length, total: items.length, nextPage: null })
      );
    }
    if (p === '/api/search/statistics') return route.fulfill(json({ total: assets.length }));
    if (p === '/api/edits') return route.fulfill(json([]));
    if (p === '/api/folders/paths') return route.fulfill(json([]));
    if (p === '/api/albums') return route.fulfill(json([]));
    if (p === '/api/tags') return route.fulfill(json([]));
    if (p === '/api/people') return route.fulfill(json([]));

    const assetMatch = p.match(/^\/api\/assets\/([^/]+)$/);
    if (assetMatch) {
      const id = assetMatch[1];
      const asset = assets.find((a) => a.id === id) ?? ASSET_SUMMARY;
      return route.fulfill(
        json({
          ...ASSET_DETAIL,
          ...asset,
          originalMimeType: 'image/x-sony-arw',
          tags: []
        })
      );
    }

    const copiesMatch = p.match(/^\/api\/assets\/([^/]+)\/copies$/);
    if (copiesMatch) {
      const source = copiesMatch[1].split('_')[0];
      if (method === 'POST') {
        const body = (req.postDataJSON() as { name?: string } | null) ?? {};
        const record = {
          id: `${source}_${copies.filter((c) => c.source_asset_id === source).length + 1}`,
          source_asset_id: source,
          name: body.name ?? null,
          created_at: '2024-01-01T00:00:00Z'
        };
        copies.push(record);
        return route.fulfill({ ...json(record), status: 201 });
      }
      return route.fulfill(json(copies.filter((c) => c.source_asset_id === source)));
    }

    const copyMatch = p.match(/^\/api\/copies\/([^/]+)$/);
    if (copyMatch) {
      const idx = copies.findIndex((c) => c.id === copyMatch[1]);
      if (method === 'DELETE') {
        if (idx >= 0) copies.splice(idx, 1);
        return route.fulfill({ status: 204, body: '' });
      }
      const body = (req.postDataJSON() as { name?: string | null } | null) ?? {};
      if (idx >= 0) copies[idx] = { ...copies[idx], name: body.name ?? null };
      return route.fulfill(json(copies[idx]));
    }

    const editsMatch = p.match(/^\/api\/assets\/([^/]+)\/edits$/);
    if (editsMatch) {
      if (method === 'DELETE') return route.fulfill({ status: 204, body: '' });
      if (method === 'PUT') {
        const body = (req.postDataJSON() as Record<string, unknown> | null) ?? {};
        if (opts.onSave) opts.onSave(body);
        const record = opts.editRecord ?? NEUTRAL_RECORD;
        return route.fulfill(
          json({
            ...record,
            asset_id: editsMatch[1],
            manifest: body.manifest ?? record.manifest
          })
        );
      }
      return route.fulfill(
        json({ ...(opts.editRecord ?? NEUTRAL_RECORD), asset_id: editsMatch[1] })
      );
    }

    if (p.match(/^\/api\/assets\/[^/]+\/edits\/history$/)) {
      if (opts.onHistory) return opts.onHistory(route);
      return route.fulfill(json([]));
    }

    if (p.match(/^\/api\/assets\/[^/]+\/edits\/restore$/)) {
      if (opts.onRestore) return opts.onRestore(route);
      return route.fulfill(json(null));
    }

    if (p.match(/^\/api\/assets\/[^/]+\/export$/)) {
      if (opts.onExport) return opts.onExport(route);
      return route.fulfill({
        status: 200,
        contentType: 'image/jpeg',
        headers: { 'content-disposition': 'attachment; filename="IMG_0001_edit.jpg"' },
        body: JPEG_BLOB
      });
    }

    if (opts.previewMeta && p.match(/^\/api\/assets\/[^/]+\/preview\/meta\//))
      return route.fulfill(json({ ...PREVIEW_META, ...opts.previewMeta }));

    if (p.match(/^\/api\/assets\/[^/]+\/preview$/)) {
      if (method === 'POST' && opts.onPreview) {
        const body = req.postDataJSON() as PreviewRequest | null;
        if (body) opts.onPreview(body);
      }
      return route.fulfill(previewPng());
    }
    if (p.match(/^\/api\/assets\/[^/]+\/preview/)) return route.fulfill(previewPng());
    if (p.endsWith('/thumbnail') || p.endsWith('/thumb') || p.endsWith('/edited-thumb'))
      return route.fulfill(png());

    return route.fulfill(json({}));
  });
}

export async function gotoAsset(page: Page): Promise<void> {
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByTitle('Back')).toBeVisible();
  await expect(page.getByText(ASSET_SUMMARY.originalFileName)).toBeVisible();
}
