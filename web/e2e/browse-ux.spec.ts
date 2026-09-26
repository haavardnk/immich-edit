import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json, numberedAssets } from './helpers';

const PAGED_ASSETS = numberedAssets(3);

test('a cold album link opens the albums section at the current album', async ({ page }) => {
  const album = {
    id: 'album-1',
    albumName: 'Review album',
    assetCount: 1,
    updatedAt: '2024-01-01T00:00:00Z'
  };
  await installMocks(page);
  await page.route('**/api/albums', (route) => route.fulfill(json([album])));
  await page.route('**/api/albums/album-1', (route) => route.fulfill(json(album)));

  await page.goto('/albums/album-1');

  const sidebar = page.getByRole('complementary', { name: 'Library' });
  const link = sidebar.getByRole('link', { name: /Review album/ });
  await expect(link).toBeVisible();
  await expect(link).toHaveAttribute('aria-current', 'page');
});

test('each thumbnail info mode shows more than the last', async ({ page }) => {
  await installMocks(page, {
    assets: [{ ...ASSET_SUMMARY, isFavorite: true, exifInfo: { rating: 3 } }]
  });
  await page.goto('/photos');
  const tile = page.locator(`div[title="${ASSET_SUMMARY.originalFileName}"]`);
  const heart = tile.getByRole('img', { name: 'Favorite' });
  const row = tile.getByTestId('tile-info');
  const name = tile.getByTestId('tile-name');
  await page.mouse.move(0, 0);

  await expect(page.getByRole('button', { name: 'Thumbnail info: badges' })).toBeVisible();
  await expect(heart).toHaveCSS('opacity', '1');
  await expect(row).toHaveCSS('opacity', '1');
  await expect(name).toHaveCSS('opacity', '0');

  await page.keyboard.press('Shift+i');
  await expect(page.getByRole('button', { name: 'Thumbnail info: name and date' })).toBeVisible();
  await expect(name).toHaveCSS('opacity', '1');
  await expect(name).toContainText(new Date('2024-01-01T00:00:00Z').toLocaleDateString());

  await page.keyboard.press('Shift+i');
  await expect(page.getByRole('button', { name: 'Thumbnail info: hover only' })).toBeVisible();
  await expect(heart).toHaveCSS('opacity', '0');
  await expect(row).toHaveCSS('opacity', '0');

  await tile.hover();
  await expect(heart).toHaveCSS('opacity', '1');
  await expect(name).toHaveCSS('opacity', '1');
});

test('grid shows total count without loaded progress', async ({ page }) => {
  await installMocks(page, { total: 1000 });

  await page.goto('/photos');

  await expect(page.getByText('1000 photos', { exact: true })).toBeVisible();
  await expect(page.getByText('1 of 1000', { exact: true })).toHaveCount(0);
  await expect(page.getByText('1 loaded', { exact: true })).toHaveCount(0);
});

test('active filters stay visible as removable chips', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('checkbox', { name: 'Favorites only' }).click();
  await page.getByLabel('Filename').fill('portrait');
  await page.getByRole('button', { name: 'Close filters' }).click();

  const chips = page.getByRole('list', { name: 'Active filters' });
  await expect(chips.getByRole('listitem')).toHaveText(['Favorites', 'Name: portrait']);
  await expect.poll(() => bodies.at(-1)?.originalFileName).toBe('portrait');

  await chips.getByRole('button', { name: 'Remove Favorites' }).click();

  await expect(chips.getByRole('listitem')).toHaveText(['Name: portrait']);
  await expect.poll(() => bodies.at(-1)?.isFavorite).toBeUndefined();
  expect(bodies.at(-1)?.originalFileName).toBe('portrait');
});

test('the count names photos hidden by the reject filter', async ({ page }) => {
  const [kept, dropped] = numberedAssets(2);
  if (!kept || !dropped) throw new Error('missing assets');
  const rejectedAsset = {
    ...dropped,
    tags: [{ id: 'r', name: 'reject', value: 'immich-edit/reject' }]
  };
  await installMocks(page, { assets: [kept, rejectedAsset] });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('button', { name: 'Rejected' }).click();
  await page.getByRole('option', { name: 'Hide' }).click();

  await expect(page.getByText('2 photos, 1 hidden', { exact: true })).toBeVisible();
});

test('a minimum rating filter asks Immich for that rating and up', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('button', { name: 'Rating' }).click();
  await page.getByRole('option', { name: '3 ★ and up' }).click();

  await expect.poll(() => bodies.at(-1)?.filter).toEqual({ rating: { gte: 3 } });
  expect(bodies.at(-1)).not.toHaveProperty('rating');
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['3+ stars']
  );
});

test('an Immich older than 3.2 offers only exact ratings', async ({ page }) => {
  await installMocks(page, { minRatingFilter: false });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('button', { name: 'Rating' }).click();
  await expect(page.getByRole('option', { name: '3 ★', exact: true })).toBeVisible();
  await expect(page.getByRole('option', { name: /and up/ })).toHaveCount(0);
});

test('select all loads every page and enables local actions', async ({ page }) => {
  const pages: Array<number | undefined> = [];
  const releasePages = new Map<number, () => void>();
  await installMocks(page, {
    assets: PAGED_ASSETS,
    searchPages: PAGED_ASSETS.map((asset) => [asset]),
    onMetadata: async (body) => {
      const pageNumber = typeof body.page === 'number' ? body.page : undefined;
      pages.push(pageNumber);
      if (pageNumber === undefined) return;
      await new Promise<void>((resolve) => releasePages.set(pageNumber, resolve));
    }
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Select all', exact: true }).click();

  await expect.poll(() => releasePages.has(2)).toBe(true);
  releasePages.get(2)?.();
  await expect(page.getByText('2 selected')).toBeVisible();
  await expect(page.locator('div[title="IMG_0002.ARW"]')).toHaveAttribute('data-selected', 'true');
  await expect.poll(() => releasePages.has(3)).toBe(true);
  releasePages.get(3)?.();
  await expect(page.getByText('3 selected')).toBeVisible();
  expect(pages).toEqual([undefined, 2, 3]);
  await expect(page.getByRole('button', { name: 'Select all', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Favorite', exact: true })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Edit and export selected' })).toBeEnabled();
});

test('changing a browse filter clears hidden selection', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await expect(page.getByText('1 selected')).toBeVisible();

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByLabel('Filename').fill('portrait');

  await expect(page.getByText('1 selected')).toBeHidden();
  await expect(page.getByRole('button', { name: 'Clear selection' })).toBeHidden();
});

test('thumbnail size controls remain available on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');

  for (const size of ['S', 'M', 'L', 'XL']) {
    await expect(page.getByRole('button', { name: `Thumbnail size ${size}` })).toBeInViewport();
  }
  await page.getByRole('button', { name: 'Thumbnail size XL' }).click();
  await expect(page.getByRole('button', { name: 'Thumbnail size XL' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});
