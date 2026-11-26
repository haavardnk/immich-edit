import { expect, test, type Page } from '@playwright/test';
import { bulkBar, openBulkMetadata, openTouchMetadata, pick, tile } from './browse';
import { installMocks, numberedAssets } from './helpers';
import { selectTwo } from './touchShell';

const PHOTOS = numberedAssets(3).map((asset, index) => ({
  ...asset,
  exifInfo: {
    description: ['Fløyen at dusk', 'Fløyen at dusk', null][index] ?? null,
    latitude: index === 0 ? 60.3975 : null,
    longitude: index === 0 ? 5.324 : null,
    city: index === 0 ? 'Bergen' : null,
    rating: index === 0 ? 4 : null
  }
}));

interface Write {
  path: string;
  body: Record<string, unknown>;
}

async function open(page: Page): Promise<Write[]> {
  const writes: Write[] = [];
  page.on('request', (request) => {
    const path = new URL(request.url()).pathname;
    if (request.method() === 'PUT' && path.startsWith('/api/assets/')) {
      writes.push({ path, body: request.postDataJSON() as Record<string, unknown> });
    }
  });
  await installMocks(page, { assets: PHOTOS, metadataPushConsented: true });
  await page.goto('/photos');
  await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
  return writes;
}

test('a description replaces mixed ones and Undo puts them back', async ({ page }) => {
  const writes = await open(page);
  for (const asset of PHOTOS) await pick(page, asset.originalFileName);
  const menu = await openBulkMetadata(page);
  await menu.getByRole('button', { name: 'Description…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Description' });
  const field = dialog.getByRole('textbox', { name: 'Description' });
  await expect(field).toHaveAttribute('placeholder', 'Mixed');
  await dialog.getByRole('button', { name: /^Fløyen at dusk/ }).click();
  await expect(field).toHaveValue('Fløyen at dusk');
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(page.getByText('Changed the description of 1 photo')).toBeVisible();
  expect(writes).toEqual([
    { path: `/api/assets/${PHOTOS[2]!.id}`, body: { description: 'Fløyen at dusk' } }
  ]);
  await page.getByRole('button', { name: 'Undo' }).click();
  await expect.poll(() => writes.length).toBe(2);
  expect(writes[1]).toEqual({ path: `/api/assets/${PHOTOS[2]!.id}`, body: { description: '' } });
});

test('metadata copied from one photo pastes the chosen fields onto others', async ({ page }) => {
  const writes = await open(page);
  await pick(page, 'IMG_0001.ARW');
  let menu = await openBulkMetadata(page);
  await expect(menu.getByRole('button', { name: 'Paste metadata…' })).toHaveCount(0);
  await menu.getByRole('button', { name: 'Copy metadata' }).click();
  await expect(page.getByText('Copied the metadata of IMG_0001.ARW')).toBeVisible();
  await bulkBar(page).getByRole('button', { name: 'Clear selection' }).click();

  await pick(page, 'IMG_0002.ARW');
  await pick(page, 'IMG_0003.ARW');
  menu = await openBulkMetadata(page);
  await expect(menu.getByRole('button', { name: 'Copy metadata' })).toHaveCount(0);
  const paste = menu.getByRole('button', { name: 'Paste metadata…' });
  await expect(paste).toContainText('IMG_0001.ARW');
  await paste.click();
  const dialog = page.getByRole('dialog', { name: 'Paste metadata' });
  await expect(dialog.getByRole('checkbox', { name: 'Tags' })).toBeDisabled();
  await dialog.getByRole('checkbox', { name: 'Rating' }).check();
  await dialog.getByRole('button', { name: 'Paste', exact: true }).click();

  await expect(dialog).toHaveCount(0);
  await expect
    .poll(() => writes.filter((write) => write.path === `/api/assets/${PHOTOS[2]!.id}`).length)
    .toBe(2);
  expect(writes).toContainEqual({
    path: `/api/assets/${PHOTOS[2]!.id}`,
    body: { description: 'Fløyen at dusk' }
  });
  expect(writes).toContainEqual({ path: `/api/assets/${PHOTOS[1]!.id}`, body: { rating: 4 } });
  expect(writes.some((write) => write.path === '/api/assets/location')).toBe(true);
});

test.describe('metadata on touch', { tag: '@touch' }, () => {
  test('the Metadata sheet opens the description sheet', async ({ page }) => {
    const writes = await open(page);
    await page.goto('/photos');
    await selectTwo(page);
    const sheet = await openTouchMetadata(page);
    await expect(sheet.getByRole('button', { name: 'Copy metadata' })).toHaveCount(0);
    await expect(sheet.getByRole('button', { name: 'Paste metadata…' })).toHaveCount(0);
    await sheet.getByRole('button', { name: 'Description…' }).tap();
    const dialog = page.getByRole('dialog', { name: 'Description' });
    await dialog.getByRole('textbox', { name: 'Description' }).fill('Bryggen');
    await dialog.getByRole('button', { name: 'Save' }).tap();
    await expect.poll(() => writes.length).toBe(2);
  });
});
