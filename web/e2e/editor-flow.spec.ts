import { expect, test, type Route } from '@playwright/test';

const PNG_1X1: Buffer = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64'
);

const ASSET_ID: string = '00000000-0000-0000-0000-000000000001';

const ASSET_SUMMARY = {
  id: ASSET_ID,
  originalFileName: 'IMG_0001.ARW',
  type: 'IMAGE',
  fileCreatedAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  checksum: 'aaaa',
  isFavorite: false,
  exifInfo: null
};

const ASSET_DETAIL = {
  ...ASSET_SUMMARY,
  originalMimeType: 'image/x-sony-arw',
  tags: []
};

const EMPTY_EDITS = {
  schema_version: 1,
  asset_id: ASSET_ID,
  immich_updated_at: null,
  immich_checksum: null,
  renderer_version: 'test',
  manifest: { schema_version: 1, ops: {} },
  updated_at: '2024-01-01T00:00:00Z',
  hash: '0'
};

function json(data: unknown): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(data) };
}

function png(): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'image/png', body: PNG_1X1 };
}

test('photos → asset → export tab', async ({ page }) => {
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url());
    const p = url.pathname;
    if (!p.startsWith('/api/')) return route.continue();
    if (p === '/api/search/metadata')
      return route.fulfill(json({ items: [ASSET_SUMMARY], count: 1, total: 1, nextPage: null }));
    if (p === '/api/search/statistics') return route.fulfill(json({ total: 1 }));
    if (p === '/api/edits') return route.fulfill(json([]));
    if (p === '/api/folders/paths') return route.fulfill(json([]));
    if (p === '/api/albums') return route.fulfill(json([]));
    if (p === '/api/tags') return route.fulfill(json([]));
    if (p === '/api/people') return route.fulfill(json([]));
    if (p === `/api/assets/${ASSET_ID}`) return route.fulfill(json(ASSET_DETAIL));
    if (p === `/api/assets/${ASSET_ID}/edits`) return route.fulfill(json(EMPTY_EDITS));
    if (p.startsWith(`/api/assets/${ASSET_ID}/preview`)) return route.fulfill(png());
    if (p.endsWith('/thumbnail') || p.endsWith('/thumb') || p.endsWith('/edited-thumb'))
      return route.fulfill(png());
    return route.fulfill(json({}));
  });

  await page.goto('/photos');
  const tile = page.locator(`a[href="/assets/${ASSET_ID}"]`).first();
  await expect(tile).toBeVisible();
  await tile.click();
  await page.waitForURL(`**/assets/${ASSET_ID}`);

  await expect(page.getByTitle('Back')).toBeVisible();

  await page.getByRole('button', { name: 'Export', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Download' })).toBeVisible();
});
