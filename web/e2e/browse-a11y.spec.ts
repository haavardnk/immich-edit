import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json } from './helpers';

test('grid tiles and counts are readable by assistive tech', async ({ page }) => {
  await installMocks(page, {
    assets: [{ ...ASSET_SUMMARY, isFavorite: true }]
  });

  await page.goto('/photos');
  await expect(page.getByText('1 asset', { exact: true })).toBeVisible();

  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
  await expect(page.getByRole('img', { name: 'Favorite' })).toBeVisible();

  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await expect(page.getByText('1 selected')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Deselect', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});

test('the library sidebar exposes the current page and section state', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');

  await expect(page.getByRole('link', { name: /^Photos/ })).toHaveAttribute('aria-current', 'page');
  await expect(page.getByRole('link', { name: /^Favorites/ })).not.toHaveAttribute(
    'aria-current',
    'page'
  );

  const albums = page.getByRole('button', { name: /^Albums/ });
  await expect(albums).toHaveAttribute('aria-expanded', 'false');
  await albums.click();
  await expect(albums).toHaveAttribute('aria-expanded', 'true');
});

test('collection rows expose current state only on their own route', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/tags', (route) =>
    route.fulfill(json([{ id: 'shared-id', name: 'Travel', value: 'Travel', assetCount: 3 }]))
  );
  await page.route('**/api/albums', (route) =>
    route.fulfill(
      json([
        {
          id: 'shared-id',
          albumName: 'Trips',
          assetCount: 3,
          albumThumbnailAssetId: null
        }
      ])
    )
  );

  await page.goto('/tags/shared-id');
  await page.getByRole('button', { name: /^Tags/ }).click();
  await expect(page.getByRole('link', { name: 'Travel' })).toHaveAttribute('aria-current', 'page');

  await page.getByRole('button', { name: /^Albums/ }).click();
  await expect(page.getByRole('link', { name: /Trips/ })).not.toHaveAttribute(
    'aria-current',
    'page'
  );
});

test('an empty browse route keeps its header and names the empty state', async ({ page }) => {
  await installMocks(page, { assets: [] });

  await page.goto('/photos');

  await expect(page.getByRole('heading', { name: 'Photos', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'No photos' })).toBeVisible();
  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toHaveCount(0);
});
