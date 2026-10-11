import { expect, test, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json, openFilters } from './helpers';

const OLD_ID = '00000000-0000-0000-0000-000000000001';
const MID_ID = '00000000-0000-0000-0000-000000000002';
const NEW_ID = '00000000-0000-0000-0000-000000000003';

const ASSETS = [
  {
    ...ASSET_SUMMARY,
    id: OLD_ID,
    originalFileName: 'IMG_0001.ARW',
    fileCreatedAt: '2024-01-01T00:00:00Z',
    createdAt: '2024-06-03T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: MID_ID,
    originalFileName: 'IMG_0002.ARW',
    fileCreatedAt: '2024-02-01T00:00:00Z',
    createdAt: '2024-06-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: NEW_ID,
    originalFileName: 'IMG_0003.ARW',
    fileCreatedAt: '2024-03-01T00:00:00Z',
    createdAt: '2024-06-02T00:00:00Z'
  }
];

const OLDEST_FIRST = [OLD_ID, MID_ID, NEW_ID];
const NEWEST_FIRST = [NEW_ID, MID_ID, OLD_ID];

async function expectOrder(page: Page, ids: string[]): Promise<void> {
  const tiles = page.locator('a[href^="/assets/"]');
  await expect(tiles).toHaveCount(ids.length);
  await expect
    .poll(() =>
      tiles.evaluateAll((els) =>
        els.map((el) => new URL((el as HTMLAnchorElement).href).pathname.slice('/assets/'.length))
      )
    )
    .toEqual(ids);
}

async function pickSort(page: Page, name: string): Promise<void> {
  await page.getByRole('button', { name: /^Sort:/ }).click();
  await page.locator('[data-popover-content]').getByRole('radio', { name, exact: true }).click();
  await page.keyboard.press('Escape');
}

test('collections open oldest first while the timeline opens newest first', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto('/tags/tag-1');
  await expectOrder(page, OLDEST_FIRST);

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);
});

test('each browse family remembers its own sort direction', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto('/tags/tag-1');
  await pickSort(page, 'Newest first');
  await expectOrder(page, NEWEST_FIRST);

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);
  await pickSort(page, 'Oldest first');
  await expectOrder(page, OLDEST_FIRST);

  await page.goto('/tags/tag-1');
  await expectOrder(page, NEWEST_FIRST);

  await page.reload();
  await expectOrder(page, NEWEST_FIRST);
  await page.goto('/photos');
  await expectOrder(page, OLDEST_FIRST);
});

test('loupe moves forward in time when culling a collection', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
    localStorage.setItem('immich-edit:browseView', JSON.stringify({ loupeAutoAdvance: true }));
  });

  await page.goto('/tags/tag-1');
  await expectOrder(page, OLDEST_FIRST);

  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();

  await page.keyboard.press('3');
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('the timeline can sort by date added', async ({ page }) => {
  const sorts: unknown[] = [];
  await installMocks(page, { assets: ASSETS, onMetadata: (body) => void sorts.push(body.sort) });

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);

  await pickSort(page, 'Date added');
  await expectOrder(page, [OLD_ID, NEW_ID, MID_ID]);
  expect(sorts.at(-1)).toEqual({ field: 'added', direction: 'desc' });

  await pickSort(page, 'Oldest first');
  await expectOrder(page, [MID_ID, NEW_ID, OLD_ID]);

  await page.reload();
  await expectOrder(page, [MID_ID, NEW_ID, OLD_ID]);
  await expect(page.getByRole('button', { name: 'Sort: date added, oldest first' })).toBeVisible();

  await page.goto('/tags/tag-1');
  await expectOrder(page, OLDEST_FIRST);
});

const RATED = [
  { ...ASSETS[0], exifInfo: { rating: 3 } },
  { ...ASSETS[1], exifInfo: { rating: null } },
  { ...ASSETS[2], exifInfo: { rating: 5 } }
];

test('structured servers sort the whole result by rating', async ({ page }) => {
  const sorts: unknown[] = [];
  await installMocks(page, {
    assets: RATED,
    structuredSearch: true,
    onMetadata: (body) => void sorts.push(body.sort)
  });

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);

  await pickSort(page, 'Rating');
  await expectOrder(page, [NEW_ID, OLD_ID, MID_ID]);
  expect(sorts.at(-1)).toEqual({ field: 'rating', direction: 'desc' });
  await expect(page.getByRole('button', { name: 'Sort: rating, highest first' })).toBeVisible();

  await pickSort(page, 'Lowest first');
  await expectOrder(page, [OLD_ID, NEW_ID, MID_ID]);
});

test('structured servers sort the whole result by file size', async ({ page }) => {
  const sorts: unknown[] = [];
  await installMocks(page, {
    assets: [
      { ...ASSETS[0], exifInfo: { fileSizeInByte: 30_000_000 } },
      { ...ASSETS[1], exifInfo: { fileSizeInByte: 10_000_000 } },
      { ...ASSETS[2], exifInfo: { fileSizeInByte: 20_000_000 } }
    ],
    structuredSearch: true,
    onMetadata: (body) => void sorts.push(body.sort)
  });

  await page.goto('/photos');
  await pickSort(page, 'Size');
  await expectOrder(page, [OLD_ID, NEW_ID, MID_ID]);
  expect(sorts.at(-1)).toEqual({ field: 'size', direction: 'desc' });
  await expect(page.getByRole('button', { name: 'Sort: size, largest first' })).toBeVisible();

  await pickSort(page, 'Smallest first');
  await expectOrder(page, [MID_ID, NEW_ID, OLD_ID]);
});

test('older servers hide the rating and size sorts and recall the default', async ({ page }) => {
  const sorts: unknown[] = [];
  await installMocks(page, { assets: RATED, onMetadata: (body) => void sorts.push(body.sort) });
  await page.addInitScript(() =>
    localStorage.setItem(
      'immich-edit:browseSort',
      JSON.stringify({ timeline: { field: 'rating', direction: 'asc' } })
    )
  );

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);
  expect(sorts.every((sort) => (sort as { field: string }).field === 'taken')).toBe(true);

  await page.getByRole('button', { name: 'Sort: date taken, newest first' }).click();
  const menu = page.locator('[data-popover-content]');
  await expect(menu.getByRole('radio', { name: 'Date added', exact: true })).toBeVisible();
  await expect(menu.getByRole('radio', { name: 'Rating', exact: true })).toHaveCount(0);
  await expect(menu.getByRole('radio', { name: 'Size', exact: true })).toHaveCount(0);
});

test('edited view sorts by edit time', async ({ page }) => {
  await installMocks(page, {
    assets: ASSETS,
    edits: [
      { id: OLD_ID, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' },
      { id: MID_ID, hash: 'h2', updated_at: '2024-06-01T00:00:00Z' },
      { id: NEW_ID, hash: 'h3', updated_at: '2024-04-01T00:00:00Z' }
    ]
  });

  await page.goto('/edited');
  await expectOrder(page, [NEW_ID, OLD_ID, MID_ID]);
  await expect(page.getByRole('button', { name: /^Sort:/ })).toHaveCount(0);

  await page.getByRole('button', { name: 'Oldest edit first' }).click();
  await expectOrder(page, [MID_ID, OLD_ID, NEW_ID]);
});

test('edited view names photos from one enriched request', async ({ page }) => {
  let assetDetailCalls = 0;
  await installMocks(page, {
    assets: ASSETS,
    edits: [
      { id: OLD_ID, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' },
      { id: MID_ID, hash: 'h2', updated_at: '2024-06-01T00:00:00Z' }
    ]
  });
  await page.route('**/api/assets/*', async (route) => {
    if (route.request().method() === 'GET') assetDetailCalls += 1;
    await route.fallback();
  });

  await page.goto('/edited');
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'IMG_0002.ARW' })).toBeVisible();
  expect(assetDetailCalls).toBe(0);
});

test('edited view filters the loaded photos', async ({ page }) => {
  await installMocks(page, {
    assets: [
      { ...ASSETS[0], exifInfo: { fileSizeInByte: 30_000_000 } },
      { ...ASSETS[1], exifInfo: { fileSizeInByte: 10_000_000 } },
      { ...ASSETS[2], exifInfo: { fileSizeInByte: 20_000_000 } }
    ],
    edits: [
      { id: OLD_ID, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' },
      { id: MID_ID, hash: 'h2', updated_at: '2024-06-01T00:00:00Z' },
      { id: NEW_ID, hash: 'h3', updated_at: '2024-04-01T00:00:00Z' }
    ]
  });

  await page.goto('/edited?minsize=15&visibility=archive');
  await expectOrder(page, [NEW_ID, OLD_ID]);
  await expect(page).not.toHaveURL(/visibility/);

  const panel = await openFilters(page, 'Library', 'File');
  await expect(panel.getByRole('button', { name: 'Rejected', exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Visibility', exact: true })).toHaveCount(0);
  await expect(panel.getByLabel('Min size (MB)')).toHaveValue('15');
  await panel.getByLabel('Filename').fill('0003');
  await expectOrder(page, [NEW_ID]);
});

test('edited view counts and steps through only the photos a filter keeps', async ({ page }) => {
  await installMocks(page, {
    assets: [
      { ...ASSETS[0], exifInfo: { make: 'FUJIFILM' } },
      { ...ASSETS[1], exifInfo: { make: 'SONY' } },
      { ...ASSETS[2], exifInfo: { make: 'SONY' } }
    ],
    edits: [
      { id: OLD_ID, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' },
      { id: MID_ID, hash: 'h2', updated_at: '2024-06-01T00:00:00Z' },
      { id: NEW_ID, hash: 'h3', updated_at: '2024-04-01T00:00:00Z' }
    ]
  });

  await page.goto('/edited?make=SONY');
  await expectOrder(page, [NEW_ID, MID_ID]);
  await expect(page.getByText('2 photos', { exact: true })).toBeVisible();

  await page.getByLabel('Quick review').first().click();
  await expect(page.getByText('1 of 2', { exact: true })).toBeVisible();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(page.getByText('2 of 2', { exact: true })).toBeVisible();
});

test('folder view counts only the photos a filter keeps', async ({ page }) => {
  const assets = [
    { ...ASSETS[0], exifInfo: { make: 'FUJIFILM' } },
    { ...ASSETS[1], exifInfo: { make: 'SONY' } },
    { ...ASSETS[2], exifInfo: { make: 'SONY' } }
  ];
  await installMocks(page, { assets });
  await page.route('**/api/folders/assets?*', (route) => route.fulfill(json(assets)));

  await page.goto('/folders?path=%2Fphotos&make=SONY');
  await expectOrder(page, [MID_ID, NEW_ID]);
  await expect(page.getByText('2 photos', { exact: true })).toBeVisible();
});
