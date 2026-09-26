import { expect, test, type Page } from '@playwright/test';
import { installMocks, json, numberedAssets } from './helpers';

const ASSETS = numberedAssets(2);

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

const ALBUM = {
  id: 'album-1',
  albumName: 'Review album',
  assetCount: 2,
  updatedAt: '2024-01-01T00:00:00Z'
};

async function installAlbumMocks(page: Page): Promise<Array<{ method: string; ids: string[] }>> {
  const calls: Array<{ method: string; ids: string[] }> = [];
  await installBulkMocks(page);
  await page.route('**/api/albums', (route) => route.fulfill(json([ALBUM])));
  await page.route('**/api/albums/album-1', (route) => route.fulfill(json(ALBUM)));
  await page.route('**/api/albums/album-1/assets', (route) => {
    const request = route.request();
    const ids = (request.postDataJSON() as { ids: string[] }).ids;
    calls.push({ method: request.method(), ids });
    return route.fulfill(json(ids.map((id) => ({ id, success: true }))));
  });
  return calls;
}

test('the bulk bar adds a selection to an album', async ({ page }) => {
  const calls = await installAlbumMocks(page);
  await page.goto('/photos');
  await selectBoth(page);

  await page.getByRole('button', { name: 'Albums', exact: true }).click();
  await expect(page.getByRole('button', { name: /^Remove from/ })).toHaveCount(0);
  await page.getByLabel('Choose albums…').fill('Review');
  await page.getByRole('option', { name: 'Review album' }).click();
  await expect(page.getByRole('button', { name: 'Remove Review album' })).toBeVisible();
  await page.getByRole('button', { name: 'Add to albums' }).click();

  await expect(page.getByText('Added 2 selected photos to the album')).toBeVisible();
  expect(calls).toEqual([{ method: 'PUT', ids: ASSETS.map((a) => a.id) }]);
});

test('removing a selection from the open album drops its tiles', async ({ page }) => {
  const calls = await installAlbumMocks(page);
  await page.goto('/albums/album-1');
  await selectBoth(page);

  await page.getByRole('button', { name: 'Albums', exact: true }).click();
  await page.getByRole('button', { name: 'Remove from Review album' }).click();

  await expect(page.getByText('Removed 2 from Review album')).toBeVisible();
  expect(calls).toEqual([{ method: 'DELETE', ids: ASSETS.map((a) => a.id) }]);
  for (const asset of ASSETS) {
    await expect(page.locator(`div[title="${asset.originalFileName}"]`)).toHaveCount(0);
  }
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

test('grid shortcuts copy edits from one photo and paste them onto a selection', async ({
  page
}) => {
  const [source, second, third] = numberedAssets(3);
  if (!source || !second || !third) throw new Error('missing assets');
  await installMocks(page, {
    assets: [source, second, third],
    edits: [{ id: source.id, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' }]
  });
  await page.goto('/photos');
  const tile = (name: string) => page.locator(`div[title="${name}"]`);
  await expect(tile(source.originalFileName)).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ControlOrMeta+Shift+c');
  const dialog = page.getByRole('dialog', { name: 'Copy settings' });
  await dialog.getByRole('button', { name: 'Copy', exact: true }).click();
  await expect(dialog).toBeHidden();
  await page.keyboard.press('Escape');
  await expect(page.getByText('1 selected')).toBeHidden();

  await tile(second.originalFileName).getByRole('button', { name: 'Select' }).click();
  await tile(third.originalFileName).getByRole('button', { name: 'Select' }).click();
  await expect(page.getByText('2 selected')).toBeVisible();

  const job = page.waitForRequest((r) => r.method() === 'POST' && r.url().endsWith('/api/jobs'));
  await page.keyboard.press('ControlOrMeta+Shift+v');
  const body = (await job).postDataJSON() as { kind: string; asset_ids: string[] };
  expect(body.kind).toBe('paste_edits');
  expect(body.asset_ids.sort()).toEqual([second.id, third.id].sort());
});
