import { expect, test, type Page, type Route } from '@playwright/test';

const PNG_1X1: Buffer = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64'
);
const JPEG_BLOB: Buffer = Buffer.from(
  '/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/2wBDAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQH/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAr/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFAEBAAAAAAAAAAAAAAAAAAAAAP/EABQRAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhEDEQA/AL+AB//Z',
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

const NEUTRAL_RECORD = {
  schema_version: 1,
  asset_id: ASSET_ID,
  immich_updated_at: null,
  immich_checksum: null,
  renderer_version: 'test',
  manifest: { schema_version: 1, ops: {} },
  updated_at: '2024-01-01T00:00:00Z',
  hash: 'hash-neutral'
};

function json(data: unknown): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'application/json', body: JSON.stringify(data) };
}

function png(): Parameters<Route['fulfill']>[0] {
  return { status: 200, contentType: 'image/png', body: PNG_1X1 };
}

interface InstallOpts {
  onExport?: (route: Route) => Promise<void> | void;
  onHistory?: (route: Route) => Promise<void> | void;
  onRestore?: (route: Route) => Promise<void> | void;
}

async function installMocks(page: Page, opts: InstallOpts = {}): Promise<void> {
  await page.route('**/api/**', async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const p = url.pathname;
    const method = req.method();

    if (p === '/api/search/metadata')
      return route.fulfill(json({ items: [ASSET_SUMMARY], count: 1, total: 1, nextPage: null }));
    if (p === '/api/search/statistics') return route.fulfill(json({ total: 1 }));
    if (p === '/api/edits') return route.fulfill(json([]));
    if (p === '/api/folders/paths') return route.fulfill(json([]));
    if (p === '/api/albums') return route.fulfill(json([]));
    if (p === '/api/tags') return route.fulfill(json([]));
    if (p === '/api/people') return route.fulfill(json([]));

    if (p === `/api/assets/${ASSET_ID}`) return route.fulfill(json(ASSET_DETAIL));

    if (p === `/api/assets/${ASSET_ID}/edits`) {
      if (method === 'DELETE') return route.fulfill({ status: 204, body: '' });
      return route.fulfill(json(NEUTRAL_RECORD));
    }

    if (p === `/api/assets/${ASSET_ID}/edits/history`) {
      if (opts.onHistory) return opts.onHistory(route);
      return route.fulfill(json([]));
    }

    if (p === `/api/assets/${ASSET_ID}/edits/restore`) {
      if (opts.onRestore) return opts.onRestore(route);
      return route.fulfill(json(null));
    }

    if (p === `/api/assets/${ASSET_ID}/export`) {
      if (opts.onExport) return opts.onExport(route);
      return route.fulfill({
        status: 200,
        contentType: 'image/jpeg',
        headers: { 'content-disposition': 'attachment; filename="IMG_0001_edit.jpg"' },
        body: JPEG_BLOB
      });
    }

    if (p.startsWith(`/api/assets/${ASSET_ID}/preview`)) return route.fulfill(png());
    if (p.endsWith('/thumbnail') || p.endsWith('/thumb') || p.endsWith('/edited-thumb'))
      return route.fulfill(png());

    return route.fulfill(json({}));
  });
}

async function gotoAsset(page: Page): Promise<void> {
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByTitle('Back')).toBeVisible();
  await expect(page.getByText('Saved')).toBeVisible();
}

test('history popover restores a prior entry', async ({ page }) => {
  const entries = [
    {
      id: 2,
      manifest_hash: 'hash-neutral',
      deleted: false,
      edits: null,
      created_at: '2024-01-02T00:00:00Z',
      action: 'Latest'
    },
    {
      id: 1,
      manifest_hash: 'hash-prior',
      deleted: false,
      edits: null,
      created_at: '2024-01-01T00:00:00Z',
      action: 'Initial'
    }
  ];

  let restoreCalled = false;
  await installMocks(page, {
    onHistory: (route) => route.fulfill(json(entries)),
    onRestore: async (route) => {
      restoreCalled = true;
      const body = JSON.parse(route.request().postData() ?? '{}');
      expect(body.entry_id).toBe(1);
      await route.fulfill(json({ ...NEUTRAL_RECORD, hash: 'hash-restored' }));
    }
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Edit history' }).click();
  await expect(page.getByText('Initial')).toBeVisible();
  await page.getByText('Initial').click();

  await expect.poll(() => restoreCalled).toBe(true);
});

test('export download triggers a file download', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Export', exact: true }).click();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/IMG_0001.*\.jpg$/);
});

test('split view toggle reveals before/after slider', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const splitButton = page.getByRole('button', { name: 'Before/After split' });
  await splitButton.click();

  await expect(page.getByRole('slider', { name: 'Before/after split' })).toBeVisible();
});

test('narrow viewports show the desktop-required guard', async ({ page }) => {
  await page.setViewportSize({ width: 600, height: 800 });
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);

  await expect(page.getByRole('heading', { name: 'Desktop required' })).toBeVisible();
  await expect(page.getByTitle('Back')).toHaveCount(0);
});
