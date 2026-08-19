import { expect, test, type Page } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

const OLD_ID = '00000000-0000-0000-0000-000000000001';
const MID_ID = '00000000-0000-0000-0000-000000000002';
const NEW_ID = '00000000-0000-0000-0000-000000000003';

const ASSETS = [
  {
    ...ASSET_SUMMARY,
    id: OLD_ID,
    originalFileName: 'IMG_0001.ARW',
    fileCreatedAt: '2024-01-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: MID_ID,
    originalFileName: 'IMG_0002.ARW',
    fileCreatedAt: '2024-02-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: NEW_ID,
    originalFileName: 'IMG_0003.ARW',
    fileCreatedAt: '2024-03-01T00:00:00Z'
  }
];

const OLDEST_FIRST = [OLD_ID, MID_ID, NEW_ID];
const NEWEST_FIRST = [NEW_ID, MID_ID, OLD_ID];

async function expectOrder(page: Page, ids: string[]): Promise<void> {
  const tiles = page.locator('a[href^="/assets/"]');
  await expect(tiles).toHaveCount(ids.length);
  await expect
    .poll(() =>
      tiles.evaluateAll((els) =>
        els.map((el) => (el as HTMLAnchorElement).getAttribute('href')?.slice('/assets/'.length))
      )
    )
    .toEqual(ids);
}

test('collections open oldest first while the timeline opens newest first', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto('/tags/tag-1');
  await expectOrder(page, OLDEST_FIRST);

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);
});

test('each browse family remembers its own sort direction', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto('/tags/tag-1');
  await page.getByTitle('Oldest first').click();
  await expectOrder(page, NEWEST_FIRST);

  await page.goto('/photos');
  await expectOrder(page, NEWEST_FIRST);
  await page.getByTitle('Newest first').click();
  await expectOrder(page, OLDEST_FIRST);

  await page.goto('/tags/tag-1');
  await expectOrder(page, NEWEST_FIRST);

  await page.reload();
  await expectOrder(page, NEWEST_FIRST);
  await page.goto('/photos');
  await expectOrder(page, OLDEST_FIRST);
});

test('loupe moves forward in time when culling a collection', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
    localStorage.setItem('immich-edit:browseView', JSON.stringify({ loupeAutoAdvance: true }));
  });

  await page.goto('/tags/tag-1');
  await expectOrder(page, OLDEST_FIRST);

  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();

  await page.keyboard.press('3');
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('edited view sorts by edit time', async ({ page }) => {
  await installMocks(page, {
    assets: ASSETS,
    edits: [
      { id: OLD_ID, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' },
      { id: MID_ID, hash: 'h2', updated_at: '2024-06-01T00:00:00Z' },
      { id: NEW_ID, hash: 'h3', updated_at: '2024-04-01T00:00:00Z' }
    ]
  });

  await page.goto('/edited');
  await expectOrder(page, [NEW_ID, OLD_ID, MID_ID]);

  await page.getByTitle('Oldest edit first').click();
  await expectOrder(page, [MID_ID, OLD_ID, NEW_ID]);
});
