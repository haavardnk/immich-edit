import { expect, test, type Page } from '@playwright/test';
import { bulkBar, pick, tile } from './browse';
import { installMocks, json, numberedAssets } from './helpers';
import type { MockAlbum } from './searchMocks';
import { selectTwo } from './touchShell';

const ASSETS = numberedAssets(3);
const IDS = ASSETS.map((asset) => asset.id);
const OTHER = '00000000-0000-0000-0000-0000000000bb';

const NORWAY: MockAlbum = {
  id: 'album-norway',
  albumName: 'Norway 2026',
  assetIds: IDS,
  updatedAt: '2026-11-01T00:00:00Z'
};

const FAMILY: MockAlbum = {
  id: 'album-family',
  albumName: 'Family',
  assetIds: NORWAY.assetIds,
  ownerId: OTHER,
  shared: true,
  updatedAt: '2026-10-01T00:00:00Z'
};

interface AlbumCall {
  method: string;
  ids: string[];
}

async function openAlbum(page: Page, album: MockAlbum, refused: string[] = []) {
  const calls: AlbumCall[] = [];
  await installMocks(page, {
    assets: ASSETS,
    albums: [NORWAY, FAMILY],
    metadataPushConsented: true
  });
  await page.route('**/api/albums/*/assets', (route) => {
    const request = route.request();
    const ids = (request.postDataJSON() as { ids: string[] }).ids;
    calls.push({ method: request.method(), ids });
    return route.fulfill(
      json(
        ids.map((id) =>
          refused.includes(id)
            ? { id, success: false, error: 'no_permission' }
            : { id, success: true }
        )
      )
    );
  });
  await page.goto(`/albums/${album.id}`);
  await expect(page.getByRole('heading', { name: album.albumName })).toBeVisible();
  return calls;
}

function toast(page: Page, text: string) {
  return page.getByRole('status').filter({ hasText: text });
}

function names(index: number): string {
  return ASSETS[index]?.originalFileName ?? '';
}

async function removeThroughMoreSheet(page: Page): Promise<void> {
  const calls = await openAlbum(page, NORWAY);
  await selectTwo(page);
  await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
  const sheet = page.getByRole('dialog', { name: 'More actions' });
  const remove = sheet.getByRole('button', { name: 'Remove from album' });
  await expect(remove).toBeVisible();
  const rows = sheet.locator('[data-menu-row]');
  const labels = await rows.allInnerTexts();
  const at = labels.findIndex((text) => text.includes('Remove from album'));
  expect(labels[at + 1]).toContain('Move to trash');

  await remove.tap();
  await expect(toast(page, 'Removed 2 photos from Norway 2026')).toBeVisible();
  await expect(sheet).toHaveCount(0);
  await expect(tile(page, names(0))).toHaveCount(0);
  await expect(tile(page, names(1))).toHaveCount(0);
  await expect(page.locator('[role="group"][data-selected]')).toHaveCount(0);
  expect(calls).toEqual([{ method: 'DELETE', ids: IDS.slice(0, 2) }]);
}

test.describe('remove from album', () => {
  test('removes the selection from the open album and Undo adds it back', async ({ page }) => {
    const calls = await openAlbum(page, NORWAY);
    await pick(page, names(0));
    await pick(page, names(1));
    await bulkBar(page).getByRole('button', { name: 'Remove from album' }).click();

    const removed = toast(page, 'Removed 2 photos from Norway 2026');
    await expect(removed).toBeVisible();
    await expect(tile(page, names(0))).toHaveCount(0);
    await expect(tile(page, names(1))).toHaveCount(0);
    await expect(tile(page, names(2))).toBeVisible();
    await expect(bulkBar(page)).toBeHidden();
    expect(calls).toEqual([{ method: 'DELETE', ids: IDS.slice(0, 2) }]);

    await removed.getByRole('button', { name: 'Undo' }).click();
    await expect(tile(page, names(0))).toBeVisible();
    await expect(tile(page, names(1))).toBeVisible();
    expect(calls.at(-1)).toEqual({ method: 'PUT', ids: IDS.slice(0, 2) });
  });

  test('counts the photos Immich keeps for their owner', async ({ page }) => {
    await openAlbum(page, NORWAY, IDS.slice(1, 2));
    await pick(page, names(0));
    await pick(page, names(1));
    await bulkBar(page).getByRole('button', { name: 'Remove from album' }).click();

    const removed = toast(page, 'Removed 1 photo from Norway 2026');
    await expect(removed).toContainText('1 belongs to someone else and stays');
    await expect(tile(page, names(0))).toHaveCount(0);
    await expect(tile(page, names(1))).toBeVisible();
  });

  test('is offered only on albums the user can edit', async ({ page }) => {
    await openAlbum(page, FAMILY);
    await pick(page, names(2));
    await expect(bulkBar(page).getByRole('button', { name: /^Move/ })).toBeVisible();
    await expect(bulkBar(page).getByRole('button', { name: 'Remove from album' })).toHaveCount(0);

    await page.goto('/photos');
    await pick(page, names(2));
    await expect(bulkBar(page).getByRole('button', { name: /^Move/ })).toBeVisible();
    await expect(bulkBar(page).getByRole('button', { name: 'Remove from album' })).toHaveCount(0);
  });

  test('@touch the More sheet removes the selection above Move to trash', async ({ page }) => {
    await removeThroughMoreSheet(page);
  });

  test('@touch viewers get no Remove from album row', async ({ page }) => {
    await openAlbum(page, FAMILY);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    const sheet = page.getByRole('dialog', { name: 'More actions' });
    await expect(sheet.getByRole('button', { name: 'Move to trash' })).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Remove from album' })).toHaveCount(0);
  });
});

test.describe('remove from album in phone landscape', { tag: ['@touch', '@phone'] }, () => {
  test.use({ viewport: { width: 915, height: 412 } });

  test('the More sheet removes the selection', async ({ page }) => {
    await removeThroughMoreSheet(page);
  });
});
