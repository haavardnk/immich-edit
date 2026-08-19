import { expect, type Page, type Route } from '@playwright/test';
import { deflateSync } from 'node:zlib';
import type { EditRecord } from '../src/lib/types/edits';

const CRC_TABLE: number[] = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc32(buf: Buffer): number {
  let c = 0xffffffff;
  for (const b of buf) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function pngChunk(type: string, data: Buffer): Buffer {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

export function makePng(width: number, height: number): Buffer {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  const raw = Buffer.alloc(height * (1 + width * 3));
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk('IHDR', ihdr),
    pngChunk('IDAT', deflateSync(raw)),
    pngChunk('IEND', Buffer.alloc(0))
  ]);
}

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
  lane?: string;
  roi?: [number, number, number, number] | null;
}

export interface InstallOpts {
  assets?: Array<typeof ASSET_SUMMARY>;
  edits?: Array<{ id: string; hash: string; updated_at: string }>;
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
  previewRender?: (req: PreviewRequest) => Buffer;
  sourceSize?: { w: number; h: number };
  previewMeta?: Record<string, unknown>;
}

export async function installMocks(page: Page, opts: InstallOpts = {}): Promise<void> {
  const assets = opts.assets ?? [ASSET_SUMMARY];
  const copies: CopyRecord[] = [];
  const ordered = (order: unknown): Array<typeof ASSET_SUMMARY> => {
    if (order !== 'asc' && order !== 'desc') return assets;
    const sign = order === 'asc' ? 1 : -1;
    return assets
      .slice()
      .sort((a, b) => (Date.parse(a.fileCreatedAt) - Date.parse(b.fileCreatedAt)) * sign);
  };
  const expanded = (list: Array<typeof ASSET_SUMMARY> = assets) =>
    list.flatMap((a) => [
      a,
      ...copies
        .filter((c) => c.source_asset_id === a.id)
        .map((c) => ({ ...a, id: c.id, copyOf: a.id, copyLabel: c.name }))
    ]);
  const previewPng = (): Parameters<Route['fulfill']>[0] =>
    opts.previewBody
      ? {
          status: 200,
          contentType: 'image/png',
          headers: { 'x-preview-meta-id': 'meta-1' },
          body: opts.previewBody
        }
      : { ...png(), headers: { 'x-preview-meta-id': 'meta-1' } };

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
      const body = (req.postDataJSON() as Record<string, unknown>) ?? {};
      if (opts.onMetadata) opts.onMetadata(body);
      const items = expanded(ordered(body.order));
      return route.fulfill(
        json({ items, count: items.length, total: items.length, nextPage: null })
      );
    }
    if (p === '/api/search/statistics') return route.fulfill(json({ total: assets.length }));
    if (p === '/api/edits') return route.fulfill(json(opts.edits ?? []));
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

    if (p.match(/^\/api\/assets\/[^/]+\/preview\/meta\//)) {
      const size = opts.sourceSize ?? { w: 6000, h: 4000 };
      return route.fulfill(
        json({
          width: size.w,
          height: size.h,
          source_w: size.w,
          source_h: size.h,
          renderer: 'cpu',
          is_raw: true,
          histogram: null,
          linear_histogram: null,
          ...(opts.previewMeta ?? {})
        })
      );
    }

    if (p.match(/^\/api\/assets\/[^/]+\/preview$/)) {
      if (method === 'POST') {
        const body = req.postDataJSON() as PreviewRequest | null;
        if (body && opts.onPreview) opts.onPreview(body);
        if (body && opts.previewRender) {
          return route.fulfill({
            status: 200,
            contentType: 'image/png',
            headers: { 'x-preview-meta-id': 'meta-1' },
            body: opts.previewRender(body)
          });
        }
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
