import { expect, type Page, type Route } from '@playwright/test';

export const PNG_1X1: Buffer = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64'
);

export const JPEG_BLOB: Buffer = Buffer.from(
  '/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/2wBDAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAr/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFAEBAAAAAAAAAAAAAAAAAAAAAP/EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhEDEQA/AL+AB//Z',
  'base64'
);

export const ASSET_ID = '00000000-0000-0000-0000-000000000001';

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

export interface PreviewRequest {
  max_edge: number;
  edits: Record<string, unknown>;
  preview_mode: unknown;
}

export interface InstallOpts {
  assets?: Array<typeof ASSET_SUMMARY>;
  onExport?: (route: Route) => Promise<void> | void;
  onHistory?: (route: Route) => Promise<void> | void;
  onRestore?: (route: Route) => Promise<void> | void;
  onPreview?: (req: PreviewRequest) => void;
  onSmart?: (body: Record<string, unknown>) => void;
  onMetadata?: (body: Record<string, unknown>) => void;
  smartFails?: boolean;
}

export async function installMocks(page: Page, opts: InstallOpts = {}): Promise<void> {
  const assets = opts.assets ?? [ASSET_SUMMARY];

  await page.route('**/api/**', async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const p = url.pathname;
    const method = req.method();

    if (p === '/api/search/smart') {
      if (opts.onSmart) opts.onSmart((req.postDataJSON() as Record<string, unknown>) ?? {});
      if (opts.smartFails)
        return route.fulfill({ status: 500, contentType: 'application/json', body: '{}' });
      return route.fulfill(
        json({ items: assets, count: assets.length, total: assets.length, nextPage: null })
      );
    }
    if (p === '/api/search/metadata') {
      if (opts.onMetadata) opts.onMetadata((req.postDataJSON() as Record<string, unknown>) ?? {});
      return route.fulfill(
        json({ items: assets, count: assets.length, total: assets.length, nextPage: null })
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

    const editsMatch = p.match(/^\/api\/assets\/([^/]+)\/edits$/);
    if (editsMatch) {
      if (method === 'DELETE') return route.fulfill({ status: 204, body: '' });
      return route.fulfill(json({ ...NEUTRAL_RECORD, asset_id: editsMatch[1] }));
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

    if (p.match(/^\/api\/assets\/[^/]+\/preview$/)) {
      if (method === 'POST' && opts.onPreview) {
        const body = req.postDataJSON() as PreviewRequest | null;
        if (body) opts.onPreview(body);
      }
      return route.fulfill(png());
    }
    if (p.match(/^\/api\/assets\/[^/]+\/preview/)) return route.fulfill(png());
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
