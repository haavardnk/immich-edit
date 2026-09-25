import { expect, test, type Page } from '@playwright/test';
import { installMocks, json, numberedAssets } from './helpers';

const ASSETS = numberedAssets(2);

interface AssetUpdate {
  id: string;
  isFavorite?: boolean;
  rating?: number | null;
}

async function installBulkMocks(page: Page, assets = ASSETS): Promise<AssetUpdate[]> {
  const updates: AssetUpdate[] = [];
  const favorites = new Map(assets.map((a) => [a.id, a.isFavorite]));
  const ratings = new Map<string, number | null | undefined>(
    assets.map((a) => [a.id, a.exifInfo?.rating])
  );
  await installMocks(page, { assets });
  await page.route('**/api/assets/*', async (route) => {
    const request = route.request();
    const id = new URL(request.url()).pathname.split('/').pop() ?? '';
    if (request.method() !== 'PUT') return route.fallback();
    const body = (request.postDataJSON() as Omit<AssetUpdate, 'id'>) ?? {};
    updates.push({ id, ...body });
    if (body.isFavorite !== undefined) favorites.set(id, body.isFavorite);
    if (body.rating !== undefined) ratings.set(id, body.rating);
    const asset = assets.find((a) => a.id === id) ?? assets[0];
    return route.fulfill(
      json({
        ...asset,
        isFavorite: favorites.get(id) ?? false,
        exifInfo: { ...asset?.exifInfo, rating: ratings.get(id) ?? null },
        tags: []
      })
    );
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
  const heart = page.getByRole('toolbar', { name: 'Selection actions' }).getByRole('button', {
    name: /^Favorite \(/
  });
  await expect(heart).toHaveAttribute('aria-pressed', 'false');
  await heart.click();

  await expect(page.getByText('Updated 2 assets')).toBeVisible();
  expect(updates.map((update) => update.id).sort()).toEqual(ASSETS.map((a) => a.id).sort());
  expect(updates.every((update) => update.isFavorite === true)).toBe(true);
  await expect(page.getByRole('img', { name: 'Favorite' })).toHaveCount(2);
  await expect(
    page.getByRole('toolbar', { name: 'Selection actions' }).getByRole('button', {
      name: /^Unfavorite \(/
    })
  ).toHaveAttribute('aria-pressed', 'true');
});

test('a mixed selection shows mixed state and sets every asset the same way', async ({ page }) => {
  const assets = numberedAssets(2).map((asset, index) =>
    index === 0
      ? { ...asset, isFavorite: true, exifInfo: { rating: 2 } }
      : { ...asset, exifInfo: { rating: 0 } }
  );
  const updates = await installBulkMocks(page, assets);

  await page.goto('/photos');
  await selectBoth(page);
  const bar = page.getByRole('toolbar', { name: 'Selection actions' });
  const heart = bar.getByRole('button', { name: /^Favorite \(/ });
  await expect(heart).toHaveAttribute('aria-pressed', 'mixed');
  await expect(bar.getByRole('radiogroup', { name: 'Rating, mixed' })).toBeVisible();

  await heart.click();
  await expect(page.getByText('Updated 2 assets')).toBeVisible();
  expect(updates.map((update) => update.isFavorite)).toEqual([true, true]);
  await expect(bar.getByRole('button', { name: /^Unfavorite \(/ })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});

test('the bulk bar rates and then clears the rating of a selection', async ({ page }) => {
  const updates = await installBulkMocks(page);

  await page.goto('/photos');
  await selectBoth(page);
  const bar = page.getByRole('toolbar', { name: 'Selection actions' });
  await bar.getByRole('radio', { name: '3 stars', exact: true }).click();
  await expect(page.getByText('Updated 2 assets')).toBeVisible();
  await expect(bar.getByRole('radio', { name: '3 stars', exact: true })).toBeChecked();

  await bar.getByRole('radio', { name: '3 stars', exact: true }).click();
  await expect.poll(() => updates.length).toBe(4);
  expect(updates.slice(0, 2).every((update) => update.rating === 3)).toBe(true);
  expect(updates.slice(2).every((update) => update.rating === 0)).toBe(true);
  await expect(bar.getByRole('radio', { name: '3 stars', exact: true })).not.toBeChecked();
});

test('the preset picker opens above the bulk dialog', async ({ page }) => {
  await installMocks(page, {
    assets: ASSETS,
    presets: [{ id: 'preset-warm', name: 'Warm', group_name: null, manifest: { ops: {} } }]
  });

  await page.goto('/photos');
  await selectBoth(page);
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  const section = dialog.getByRole('button', { name: 'Presets', exact: true });
  if ((await section.getAttribute('aria-expanded')) !== 'true') await section.click();
  await dialog.getByRole('combobox', { name: 'Select a preset…' }).click();
  await page.getByRole('option', { name: 'Warm', exact: true }).click();

  await expect(dialog.getByRole('button', { name: 'Apply Warm to 2' })).toBeEnabled();
});
