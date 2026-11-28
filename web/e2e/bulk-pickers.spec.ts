import { expect, test, type Page } from '@playwright/test';
import { pick } from './browse';
import { installMocks, json, numberedAssets, type MockAssetSummary } from './helpers';

const LANDSCAPE = { id: 'tag-1', name: 'Landscape', value: 'Landscape' };
const PORTRAIT = { id: 'tag-2', name: 'Portrait', value: 'Portrait' };
const LANDMARK = { id: 'tag-3', name: 'Landmark', value: 'Landmark' };
const [FIRST, SECOND] = numberedAssets(2) as [MockAssetSummary, MockAssetSummary];
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

function note(page: Page, name: string) {
  return page
    .locator('[data-bulk-pick] label')
    .filter({ has: page.getByText(name, { exact: true }) });
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
  await expect(note(page, 'Review')).toContainText('In 1 of 2');
  await expect(trips).toHaveAttribute('aria-checked', 'false');

  await trips.click();
  await expect(note(page, 'Trips')).toContainText('Add 2 photos');
  await review.click();
  await expect(review).toHaveAttribute('aria-checked', 'true');
  await review.click();
  await expect(review).toHaveAttribute('aria-checked', 'false');
  await expect(note(page, 'Review')).toContainText('Remove from 1 photo');

  await page.getByRole('button', { name: 'Apply', exact: true }).click();
  await expect(page.getByText('Updated albums for 2 photos')).toBeVisible();
  expect(calls).toEqual([
    { album: 'album-2', method: 'PUT', ids: IDS },
    { album: 'album-1', method: 'DELETE', ids: IDS }
  ]);
  await expect(page.locator('[data-bulk-pick]')).toHaveCount(0);
});

test('the albums page counts the photos just added to an album', async ({ page }) => {
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    albums: [{ id: 'album-1', albumName: 'Test', assetIds: [] }]
  });
  await page.goto('/albums');
  const card = page.getByRole('link', { name: /^Test/ });
  await expect(card).toContainText('0 photos');
  await page.getByRole('link', { name: 'Photos', exact: true }).click();
  await pick(page, FIRST.originalFileName);
  await pick(page, SECOND.originalFileName);
  await page.getByRole('button', { name: 'Albums', exact: true }).click();
  await row(page, 'Test').click();
  await page.getByRole('button', { name: 'Apply', exact: true }).click();
  await expect(page.getByText('Updated albums for 2 photos')).toBeVisible();

  await page.getByRole('link', { name: /^Albums \d+$/ }).click();
  await expect(card).toContainText('2 photos');
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

test('the tag checklist counts tags that only the membership route reports', async ({ page }) => {
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    tags: [LANDSCAPE, PORTRAIT]
  });
  await page.route('**/api/tags/membership', (route) =>
    route.fulfill(json({ [FIRST.id]: [LANDSCAPE], [SECOND.id]: [LANDSCAPE, PORTRAIT] }))
  );
  await selectBoth(page);
  await page.getByRole('button', { name: 'Tags', exact: true }).click();

  await expect(row(page, 'Landscape')).toHaveAttribute('aria-checked', 'true');
  await expect(note(page, 'Landscape')).toContainText('All 2 photos');
  await expect(row(page, 'Portrait')).toHaveAttribute('aria-checked', 'mixed');
  await expect(note(page, 'Portrait')).toContainText('In 1 of 2');
});

test('a failed membership load leaves the tag checklist adding only', async ({ page }) => {
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    tags: [LANDSCAPE]
  });
  const loads: string[][] = [];
  await page.route('**/api/tags/membership', (route) => {
    loads.push((route.request().postDataJSON() as { ids: string[] }).ids);
    return route.fulfill(json({ [FIRST.id]: [LANDSCAPE] }));
  });
  await selectBoth(page);
  await page.getByRole('button', { name: 'Tags', exact: true }).click();

  await expect.poll(() => loads).toEqual([IDS]);
  await expect(row(page, 'Landscape')).toHaveAttribute('aria-checked', 'false');
  await row(page, 'Landscape').click();
  await expect(note(page, 'Landscape')).toContainText('Add 2 photos');
});

test('a recent tag chip only stages an add, even for a tag every photo holds', async ({ page }) => {
  await page.addInitScript(
    (tag) => localStorage.setItem('immich-edit:recentTags', JSON.stringify({ tags: [tag] })),
    LANDSCAPE
  );
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    tags: [LANDSCAPE]
  });
  await page.route('**/api/tags/membership', (route) =>
    route.fulfill(json({ [FIRST.id]: [LANDSCAPE], [SECOND.id]: [LANDSCAPE] }))
  );
  await selectBoth(page);
  await page.getByRole('button', { name: 'Tags', exact: true }).click();
  await expect(row(page, 'Landscape')).toHaveAttribute('aria-checked', 'true');

  const chip = page.locator('[data-bulk-pick]').getByRole('button', { name: 'Landscape' });
  await chip.click();
  await expect(chip).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('[data-bulk-pick]')).toContainText('Add 1');
  await chip.click();
  await expect(chip).toHaveAttribute('aria-pressed', 'false');
  await expect(row(page, 'Landscape')).toHaveAttribute('aria-checked', 'true');
  await expect(page.locator('[data-bulk-pick]')).not.toContainText('Remove');
});

test('the tag checklist counts a photo selected while it is open', async ({ page }) => {
  await installMocks(page, {
    assets: [FIRST, SECOND],
    metadataPushConsented: true,
    tags: [LANDSCAPE]
  });
  await page.route('**/api/tags/membership', (route) => {
    const { ids } = route.request().postDataJSON() as { ids: string[] };
    return route.fulfill(json(Object.fromEntries(ids.map((id) => [id, [LANDSCAPE]]))));
  });
  await page.goto('/photos');
  await pick(page, FIRST.originalFileName);
  await page.getByRole('button', { name: 'Tags', exact: true }).click();
  await expect(note(page, 'Landscape')).toContainText('All 1 photo');

  await pick(page, SECOND.originalFileName);
  await expect(note(page, 'Landscape')).toContainText('All 2 photos');
});
