import { expect, test, type Page } from '@playwright/test';
import { pick } from './browse';
import { installMocks, json, numberedAssets } from './helpers';

const LANDSCAPE = { id: 'tag-1', name: 'Landscape', value: 'Landscape' };
const PORTRAIT = { id: 'tag-2', name: 'Portrait', value: 'Portrait' };
const LANDMARK = { id: 'tag-3', name: 'Landmark', value: 'Landmark' };
const [FIRST, SECOND] = numberedAssets(2);
if (!FIRST || !SECOND) throw new Error('missing assets');
const IDS = [FIRST.id, SECOND.id];

async function selectBoth(page: Page): Promise<void> {
  await page.goto('/photos');
  await pick(page, FIRST.originalFileName);
  await pick(page, SECOND.originalFileName);
}

function row(page: Page, name: string) {
  return page.locator('[data-bulk-pick]').getByRole('checkbox', { name: new RegExp(`^${name}`) });
}

test('the album checklist stages adds and removals and applies them together', async ({ page }) => {
  const calls: Array<{ album: string; method: string; ids: string[] }> = [];
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    albums: [
      { id: 'album-1', albumName: 'Review', assetIds: [FIRST.id] },
      { id: 'album-2', albumName: 'Trips', assetIds: [] }
    ]
  });
  await page.route('**/api/albums/*/assets', (route) => {
    const request = route.request();
    const ids = (request.postDataJSON() as { ids: string[] }).ids;
    calls.push({ album: request.url().split('/').at(-2) ?? '', method: request.method(), ids });
    return route.fulfill(json(ids.map((id) => ({ id, success: true }))));
  });
  await selectBoth(page);
  await page.getByRole('button', { name: 'Albums', exact: true }).click();

  const review = row(page, 'Review');
  const trips = row(page, 'Trips');
  await expect(review).toHaveAttribute('aria-checked', 'mixed');
  await expect(review).toContainText('In 1 of 2');
  await expect(trips).toHaveAttribute('aria-checked', 'false');

  await trips.click();
  await expect(trips).toContainText('Add 2 photos');
  await review.click();
  await expect(review).toHaveAttribute('aria-checked', 'true');
  await review.click();
  await expect(review).toHaveAttribute('aria-checked', 'false');
  await expect(review).toContainText('Remove from 1 photo');

  await page.getByRole('button', { name: 'Apply', exact: true }).click();
  await expect(page.getByText('Updated albums for 2 photos')).toBeVisible();
  expect(calls).toEqual([
    { album: 'album-2', method: 'PUT', ids: IDS },
    { album: 'album-1', method: 'DELETE', ids: IDS }
  ]);
  await expect(page.locator('[data-bulk-pick]')).toHaveCount(0);
});

test('the tag checklist searches, adds one tag and removes another', async ({ page }) => {
  const changes: Array<{ tagId: string; assetId: string; added: boolean }> = [];
  await installMocks(page, {
    assets: [{ ...FIRST, tags: [LANDSCAPE] }, SECOND],
    metadataPushConsented: true,
    tags: [LANDSCAPE, PORTRAIT, LANDMARK],
    onTagAsset: (change) => changes.push(change)
  });
  await selectBoth(page);
  await page.getByRole('button', { name: 'Tags', exact: true }).click();

  await page.getByRole('textbox', { name: 'Search tags' }).fill('land');
  await expect(row(page, 'Portrait')).toHaveCount(0);
  await row(page, 'Landmark').click();
  await row(page, 'Landscape').click();
  await row(page, 'Landscape').click();
  await expect(page.locator('[data-bulk-pick]')).toContainText('Add 1 · remove 1');

  await page.getByRole('button', { name: 'Apply', exact: true }).click();
  await expect.poll(() => changes.length).toBe(4);
  expect(changes.filter((c) => c.added).map((c) => c.tagId)).toEqual(['tag-3', 'tag-3']);
  expect(changes.filter((c) => !c.added).map((c) => c.tagId)).toEqual(['tag-1', 'tag-1']);
});
