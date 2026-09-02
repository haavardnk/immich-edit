import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json } from './helpers';

const GRID_ASSETS = Array.from({ length: 10 }, (_, index) => {
  const number = index + 1;
  return {
    ...ASSET_SUMMARY,
    id: `00000000-0000-0000-0000-${String(number).padStart(12, '0')}`,
    originalFileName: `IMG_${String(number).padStart(4, '0')}.ARW`,
    exifInfo: { exifImageWidth: 6000, exifImageHeight: 4000 }
  };
});

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

test('grid selection previews and commits a shift range', async ({ page }) => {
  await installMocks(page, { assets: GRID_ASSETS.slice(0, 3) });
  await page.goto('/photos');

  const firstTile = page.locator('div[title="IMG_0001.ARW"]');
  const secondTile = page.locator('div[title="IMG_0002.ARW"]');
  const thirdTile = page.locator('div[title="IMG_0003.ARW"]');
  const firstSelector = firstTile.getByRole('button', { name: 'Select', exact: true });

  await expect(firstSelector).toHaveCSS('opacity', '0');
  await firstTile.hover();
  await expect(firstSelector).toHaveCSS('opacity', '1');
  await expect(firstSelector).toHaveCSS('width', '24px');
  await expect(firstSelector).toHaveCSS('height', '24px');

  await firstSelector.click();
  await expect(firstTile).toHaveAttribute('data-selected', 'true');
  await expect(firstTile.locator('a')).toHaveCSS('padding', '8px');
  await expect(firstTile.locator('img')).toHaveCSS('transform', 'none');

  await page.keyboard.down('Shift');
  await thirdTile.hover();
  await expect(secondTile).toHaveAttribute('data-range-preview', 'true');
  await expect(thirdTile).toHaveAttribute('data-range-preview', 'true');
  await expect(page.getByText('1 selected')).toBeVisible();

  await thirdTile.getByRole('button', { name: 'Select', exact: true }).click();
  await page.keyboard.up('Shift');
  await expect(page.getByText('3 selected')).toBeVisible();
  await expect(secondTile).toHaveAttribute('data-selected', 'true');
  await expect(thirdTile).toHaveAttribute('data-selected', 'true');
  await expect(page.locator('[data-range-preview="true"]')).toHaveCount(0);
});

test('compare and survey enforce their selection limits', async ({ page }) => {
  await installMocks(page, { assets: GRID_ASSETS });
  await page.goto('/photos');

  const compare = page.getByRole('button', { name: 'Compare selected' });
  const survey = page.getByRole('button', { name: 'Survey selected' });
  const select = (name: string) =>
    page.locator(`div[title="${name}"]`).getByRole('button', { name: 'Select', exact: true });

  await select('IMG_0001.ARW').click();
  await select('IMG_0002.ARW').click();
  await expect(compare).toBeEnabled();
  await expect(survey).toBeEnabled();

  await select('IMG_0003.ARW').click();
  await expect(compare).toBeDisabled();
  await expect(survey).toBeEnabled();

  await page.keyboard.press('ControlOrMeta+a');
  await expect(page.getByText('10 selected')).toBeVisible();
  await expect(compare).toBeDisabled();
  await expect(survey).toBeDisabled();
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
