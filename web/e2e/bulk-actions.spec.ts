import { expect, test, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks, json, type MockAssetSummary } from './helpers';

const ASSETS: MockAssetSummary[] = Array.from({ length: 2 }, (_, index) => {
  const number = index + 1;
  return {
    ...ASSET_SUMMARY,
    id: `00000000-0000-0000-0000-${String(number).padStart(12, '0')}`,
    originalFileName: `IMG_${String(number).padStart(4, '0')}.ARW`
  };
});

interface AssetUpdate {
  id: string;
  isFavorite?: boolean;
  rating?: number | null;
}

async function installBulkMocks(page: Page): Promise<AssetUpdate[]> {
  const updates: AssetUpdate[] = [];
  const favorites = new Map<string, boolean>();
  await installMocks(page, { assets: ASSETS });
  await page.route('**/api/assets/*', async (route) => {
    const request = route.request();
    const id = new URL(request.url()).pathname.split('/').pop() ?? '';
    if (request.method() !== 'PUT') return route.fallback();
    const body = (request.postDataJSON() as Omit<AssetUpdate, 'id'>) ?? {};
    updates.push({ id, ...body });
    if (body.isFavorite !== undefined) favorites.set(id, body.isFavorite);
    const asset = ASSETS.find((a) => a.id === id) ?? ASSETS[0];
    return route.fulfill(json({ ...asset, isFavorite: favorites.get(id) ?? false, tags: [] }));
  });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  return updates;
}

async function selectBoth(page: Page): Promise<void> {
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  await expect(page.getByText('2 selected')).toBeVisible();
}

test('the bulk bar favorites every selected asset', async ({ page }) => {
  const updates = await installBulkMocks(page);

  await page.goto('/photos');
  await selectBoth(page);
  await page.getByRole('button', { name: 'Favorite', exact: true }).click();

  await expect(page.getByText('Updated 2 assets')).toBeVisible();
  expect(updates.map((update) => update.id).sort()).toEqual(ASSETS.map((a) => a.id).sort());
  expect(updates.every((update) => update.isFavorite === true)).toBe(true);
  await expect(page.getByRole('img', { name: 'Favorite' })).toHaveCount(2);
});

test('the bulk bar rates and then clears the rating of a selection', async ({ page }) => {
  const updates = await installBulkMocks(page);

  await page.goto('/photos');
  await selectBoth(page);
  await page.getByRole('button', { name: 'Rate 3', exact: true }).click();
  await expect(page.getByText('Updated 2 assets')).toBeVisible();

  await page.getByRole('button', { name: 'Clear rating', exact: true }).click();
  await expect.poll(() => updates.length).toBe(4);
  expect(updates.slice(0, 2).every((update) => update.rating === 3)).toBe(true);
  expect(updates.slice(2).every((update) => update.rating === 0)).toBe(true);
});
