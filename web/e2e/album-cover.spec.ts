import { expect, test, type Locator, type Page } from '@playwright/test';
import { loupeToolbar, tile } from './browse';
import { installMocks, numberedAssets, openLoupeFromGrid } from './helpers';
import type { MockAlbum } from './searchMocks';
import { tapTile } from './touchShell';

const ASSETS = numberedAssets(3);
const IDS = ASSETS.map((asset) => asset.id);
const OTHER = '00000000-0000-0000-0000-0000000000bb';

const NORWAY: MockAlbum = {
  id: 'album-norway',
  albumName: 'Norway 2026',
  assetIds: IDS,
  albumThumbnailAssetId: IDS[0],
  updatedAt: '2026-11-01T00:00:00Z'
};

const FAMILY: MockAlbum = {
  id: 'album-family',
  albumName: 'Family',
  assetIds: IDS,
  ownerId: OTHER,
  shared: true,
  updatedAt: '2026-10-01T00:00:00Z'
};

async function openAlbum(page: Page, album: MockAlbum): Promise<unknown[]> {
  const patches: unknown[] = [];
  page.on('request', (request) => {
    if (request.method() === 'PATCH' && request.url().endsWith(`/api/albums/${album.id}`)) {
      patches.push(request.postDataJSON());
    }
  });
  await installMocks(page, {
    assets: ASSETS,
    albums: [{ ...NORWAY }, { ...FAMILY }],
    metadataPushConsented: true
  });
  await page.goto(`/albums/${album.id}`);
  await expect(page.getByRole('heading', { name: album.albumName })).toBeVisible();
  return patches;
}

function moreButton(page: Page): Locator {
  return page.getByRole('button', { name: 'More loupe actions' });
}

function coverRow(page: Page): Locator {
  return page.getByRole('button', { name: 'Set as album cover', exact: true });
}

async function expectBeforeTrash(page: Page): Promise<void> {
  const labels = await page.locator('[data-menu-row]').allInnerTexts();
  const at = labels.findIndex((text) => text.includes('Set as album cover'));
  expect(at).toBeGreaterThanOrEqual(0);
  expect(labels[at + 1]).toContain('Move to Immich trash');
}

async function openTouchLoupe(page: Page, album: MockAlbum): Promise<unknown[]> {
  const patches = await openAlbum(page, album);
  await tapTile(page, 'IMG_0002.ARW');
  await expect(loupeToolbar(page).getByRole('heading')).toHaveText('IMG_0002.ARW');
  return patches;
}

async function setCoverByTouch(page: Page): Promise<void> {
  const patches = await openTouchLoupe(page, NORWAY);
  await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
  const sheet = page.getByRole('dialog', { name: 'Menu' });
  await expect(sheet.getByRole('button', { name: 'Set as album cover' })).toBeVisible();
  await expectBeforeTrash(page);

  await sheet.getByRole('button', { name: 'Set as album cover' }).tap();
  await expect(
    page.getByRole('status').filter({ hasText: 'Set as cover of Norway 2026' })
  ).toBeVisible();
  await expect(sheet).toBeHidden();
  expect(patches).toEqual([{ albumThumbnailAssetId: IDS[1] }]);

  await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
  await expect(sheet.getByRole('button', { name: 'Set as album cover' })).toBeDisabled();
}

test.describe('set as album cover', () => {
  test('the loupe More menu makes the photo on screen the album cover', async ({ page }) => {
    const patches = await openAlbum(page, NORWAY);
    await openLoupeFromGrid(page, 1);
    await expect(page.getByRole('heading', { name: 'IMG_0002.ARW' })).toBeVisible();
    await moreButton(page).click();
    await expect(coverRow(page)).toBeEnabled();
    await expectBeforeTrash(page);

    await coverRow(page).click();
    await expect(
      page.getByRole('status').filter({ hasText: 'Set as cover of Norway 2026' })
    ).toBeVisible();
    expect(patches).toEqual([{ albumThumbnailAssetId: IDS[1] }]);

    await moreButton(page).click();
    await expect(coverRow(page)).toBeDisabled();

    await page.keyboard.press('Escape');
    await expect(coverRow(page)).toHaveCount(0);
    await page.keyboard.press('Escape');
    await expect(moreButton(page)).toHaveCount(0);
    await page.getByRole('main').getByRole('link', { name: 'Albums', exact: true }).click();
    const card = page.getByRole('list', { name: 'Albums' }).getByRole('link', { name: /Norway/ });
    await expect(card.locator('img')).toHaveAttribute('src', new RegExp(IDS[1] ?? ''));
  });

  test('is offered only in albums the user can edit', async ({ page }) => {
    await openAlbum(page, FAMILY);
    await openLoupeFromGrid(page, 1);
    await moreButton(page).click();
    await expect(page.getByRole('button', { name: /^Fullscreen/ })).toBeVisible();
    await expect(coverRow(page)).toHaveCount(0);

    await page.goto('/photos');
    await expect(tile(page, 'IMG_0002.ARW')).toBeVisible();
    await openLoupeFromGrid(page, 1);
    await moreButton(page).click();
    await expect(page.getByRole('button', { name: /^Fullscreen/ })).toBeVisible();
    await expect(coverRow(page)).toHaveCount(0);
  });

  test('@touch the loupe More sheet sets the cover above Move to trash', async ({ page }) => {
    await setCoverByTouch(page);
  });

  test('@touch viewers get no Set as album cover row', async ({ page }) => {
    await openTouchLoupe(page, FAMILY);
    await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Menu' });
    await expect(sheet.getByRole('button', { name: 'Move to Immich trash' })).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Set as album cover' })).toHaveCount(0);
  });
});

test.describe('set as album cover in phone landscape', { tag: ['@touch', '@phone'] }, () => {
  test.use({ viewport: { width: 915, height: 412 } });

  test('the loupe More sheet sets the cover', async ({ page }) => {
    await setCoverByTouch(page);
  });
});
