import { expect, test, type Page } from '@playwright/test';
import { ASSET_EXIF, ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';
import type { MockAlbum } from './searchMocks';

const SECOND_ID = '00000000-0000-0000-0000-000000000002';
const PATH = '/library/2026/10/IMG_0001.ARW';

const PHOTO = {
  ...ASSET_SUMMARY,
  originalPath: PATH,
  exifInfo: {
    ...ASSET_EXIF,
    make: 'FUJIFILM',
    model: 'X-T5',
    lensModel: 'XF16-55mmF2.8 R LM WR',
    fNumber: 8,
    exposureTime: '0.004',
    iso: 160,
    focalLength: 23,
    fileSizeInByte: 54.2 * 1024 * 1024,
    city: 'Bergen',
    state: 'Vestland',
    country: 'Norway',
    latitude: 60.39,
    longitude: 5.32,
    description: 'Fløyen at dusk'
  }
};

const ASSETS = [
  PHOTO,
  { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW', exifInfo: ASSET_EXIF }
];

function albums(): MockAlbum[] {
  return [
    { id: 'album-bergen', albumName: 'Bergen 2026', assetIds: [ASSET_ID] },
    { id: 'album-autumn', albumName: 'Best of autumn', assetIds: [ASSET_ID, SECOND_ID] },
    { id: 'album-other', albumName: 'Elsewhere', assetIds: [SECOND_ID] }
  ];
}

async function openLoupe(page: Page, list: MockAlbum[] = albums()): Promise<void> {
  await installMocks(page, { assets: ASSETS, albums: list });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
}

function infoButton(page: Page) {
  return page.getByRole('navigation', { name: 'Loupe toolbar' }).getByRole('button', {
    name: /^Info/
  });
}

test('the info panel docks beside the photo with its details', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await openLoupe(page);
  await infoButton(page).click();
  await expect(infoButton(page)).toHaveAttribute('aria-pressed', 'true');

  const panel = page.getByRole('complementary', { name: 'Info' });
  await expect(panel).toBeVisible();
  await expect(panel.getByRole('textbox', { name: 'Description' })).toHaveValue('Fløyen at dusk');
  for (const value of ['f/8', '1/250s', '160', '23mm', 'FUJIFILM X-T5', '54.2 MB · ARW']) {
    await expect(panel).toContainText(value);
  }
  await expect(panel).toContainText('Vestland, Norway');
  await expect(panel.getByRole('link', { name: 'Open in map' })).toHaveAttribute(
    'href',
    /^https:\/\/www\.openstreetmap\.org\/\?mlat=60\.39000&mlon=5\.32000/
  );
  const albums = panel.getByRole('list', { name: 'Albums with this photo' });
  await expect(albums.getByRole('listitem')).toHaveText([/Bergen 2026/, /Best of autumn/]);
  await expect(panel).toContainText(PATH);

  const box = await panel.boundingBox();
  const image = await page.getByRole('img', { name: 'IMG_0001.ARW' }).boundingBox();
  expect(box?.width).toBe(360);
  expect((image?.x ?? 0) + (image?.width ?? 0)).toBeLessThanOrEqual((box?.x ?? 0) + 1);

  await panel.getByRole('button', { name: 'Copy path' }).click();
  await expect(page.getByText('Copied the file path')).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(PATH);

  await panel.getByRole('button', { name: 'Close info' }).click();
  await expect(panel).toBeHidden();
  await expect(infoButton(page)).toHaveAttribute('aria-pressed', 'false');
});

test('the info panel follows the photo and stays open across loupe visits', async ({ page }) => {
  await openLoupe(page);
  await page.keyboard.press('i');
  const panel = page.getByRole('complementary', { name: 'Info' });
  await expect(panel).toContainText(PATH);

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(panel.getByRole('list', { name: 'Albums with this photo' })).toHaveText(
    /Best of autumn.*Elsewhere/
  );
  await expect(panel).not.toContainText('Vestland');

  await page.keyboard.press('Escape');
  await expect(panel).toBeHidden();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('complementary', { name: 'Info' })).toBeVisible();
});

test('the description saves on Enter and Escape only reverts it', async ({ page }) => {
  await openLoupe(page);
  await infoButton(page).click();
  const field = page
    .getByRole('complementary', { name: 'Info' })
    .getByRole('textbox', { name: 'Description' });

  await field.fill('Draft that goes away');
  await field.press('Escape');
  await expect(field).toHaveValue('Fløyen at dusk');
  await expect(page.getByRole('navigation', { name: 'Loupe toolbar' })).toBeVisible();

  const write = page.waitForRequest(
    (req) => req.method() === 'PUT' && req.url().endsWith(`/api/assets/${ASSET_ID}`)
  );
  await field.fill('Bryggen in the rain');
  await field.press('Enter');
  expect((await write).postDataJSON()).toEqual({ description: 'Bryggen in the rain' });
  await expect(field).not.toBeFocused();
  await expect(field).toHaveValue('Bryggen in the rain');
});

test('Add puts the photo in another album', async ({ page }) => {
  const list = albums();
  await openLoupe(page, list);
  const added: unknown[] = [];
  await page.route('**/api/albums/album-other/assets', async (route) => {
    added.push(route.request().postDataJSON());
    list[2]?.assetIds.push(ASSET_ID);
    await route.fulfill({ json: [{ id: ASSET_ID, success: true }] });
  });
  await infoButton(page).click();
  const panel = page.getByRole('complementary', { name: 'Info' });
  await expect(panel.getByRole('listitem')).toHaveCount(2);

  await panel.getByRole('button', { name: 'Add to album' }).click();
  await page.getByRole('option', { name: 'Elsewhere' }).click();

  await expect.poll(() => added).toEqual([{ ids: [ASSET_ID] }]);
  await expect(page.getByText('Added 1 photo to the album')).toBeVisible();
  await expect(panel.getByRole('listitem')).toHaveText([
    /Bergen 2026/,
    /Best of autumn/,
    /Elsewhere/
  ]);
});

test.describe('loupe info on touch', { tag: '@touch' }, () => {
  test('opens as a sheet on a phone', { tag: '@phone' }, async ({ page }) => {
    await openLoupe(page);
    await infoButton(page).tap();
    const sheet = page.getByRole('dialog', { name: 'Info' });
    await expect(sheet).toBeVisible();
    await expect(sheet).toContainText('FUJIFILM X-T5');
    await expect(sheet).toContainText('Bergen');
    await expect(page.getByRole('complementary', { name: 'Info' })).toHaveCount(0);

    await sheet.getByRole('button', { name: 'Close info' }).tap();
    await expect(sheet).toBeHidden();
    await expect(infoButton(page)).toHaveAttribute('aria-pressed', 'false');
  });

  test('docks beside the photo on a tablet in landscape', { tag: '@tablet' }, async ({ page }) => {
    await openLoupe(page);
    await infoButton(page).tap();
    const panel = page.getByRole('complementary', { name: 'Info' });
    await expect(panel).toBeVisible();
    await expect(panel).toContainText('FUJIFILM X-T5');
    await expect(page.getByRole('dialog', { name: 'Info' })).toHaveCount(0);
    expect((await panel.boundingBox())?.width).toBe(360);
  });
});
