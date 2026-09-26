import { expect, test, type Locator, type Page } from '@playwright/test';
import {
  ASSET_ID,
  focusRingClipped,
  gotoAsset,
  installMocks,
  numberedAssets,
  type InstallOpts
} from './helpers';

type TagChange = { tagId: string; assetId: string; added: boolean };

async function openGrid(page: Page, opts: InstallOpts = {}): Promise<TagChange[]> {
  const changes: TagChange[] = [];
  await installMocks(page, {
    assets: numberedAssets(2),
    onTagAsset: (change) => void changes.push(change),
    ...opts
  });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/photos');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  return changes;
}

const firstTile = (page: Page) => page.locator('div[title="IMG_0001.ARW"]');

test('number keys set, swap and clear a label on the selected photo', async ({ page }) => {
  const changes = await openGrid(page);
  await page.keyboard.press('ArrowRight');

  await page.keyboard.press('6');
  await expect(firstTile(page).getByRole('img', { name: 'Red label' })).toBeVisible();
  await page.keyboard.press('9');
  await expect(firstTile(page).getByRole('img', { name: 'Blue label' })).toBeVisible();
  await expect(firstTile(page).getByRole('img', { name: 'Red label' })).toHaveCount(0);
  await page.keyboard.press('9');
  await expect(firstTile(page).getByRole('img', { name: /label$/ })).toHaveCount(0);

  await expect
    .poll(() => changes.map((c) => `${c.added ? '+' : '-'}${c.tagId}`))
    .toEqual(['+mock-tag-1', '-mock-tag-1', '+mock-tag-2', '-mock-tag-2']);
});

test('the label filter hides other photos and names what it hides', async ({ page }) => {
  await openGrid(page, {
    tags: [{ id: 'green', name: 'green', value: 'immich-edit/label/green' }],
    assets: numberedAssets(2).map((a, i) =>
      i === 0
        ? { ...a, tags: [{ id: 'green', name: 'green', value: 'immich-edit/label/green' }] }
        : a
    )
  });
  await expect(firstTile(page).getByRole('img', { name: 'Green label' })).toBeVisible();

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('radio', { name: 'Green label' }).click();
  await page.getByRole('button', { name: 'Close filters' }).click();

  const chips = page.getByRole('list', { name: 'Active filters' });
  await expect(chips).toContainText('Green label');
  await expect(chips.locator('svg.text-green-500')).toHaveCount(1);
  await expect(page.getByText('2 photos, 1 hidden', { exact: true })).toBeVisible();
  await expect(page.locator('div[title="IMG_0002.ARW"]')).toHaveCount(0);
});

test('the loupe picker sets purple, which has no key', async ({ page }) => {
  await openGrid(page);
  await page.getByLabel('Quick review').first().click();
  const rail = page.getByRole('navigation', { name: 'Photo actions' });

  await rail.getByRole('button', { name: 'Colour label' }).click();
  await page.getByRole('button', { name: 'Purple label' }).click();

  await expect(rail.getByRole('button', { name: 'Purple label' })).toBeVisible();
});

test('the bulk bar labels every selected photo', async ({ page }) => {
  const changes = await openGrid(page);
  for (const name of ['IMG_0001.ARW', 'IMG_0002.ARW']) {
    await page.locator(`div[title="${name}"]`).getByRole('button', { name: 'Select' }).click();
  }

  const toolbar = page.getByRole('toolbar', { name: 'Selection actions' });
  await toolbar.getByRole('button', { name: 'Colour label' }).click();
  await page.getByRole('button', { name: 'Yellow label' }).click();

  await expect(page.getByText('Yellow label on 2 photos')).toBeVisible();
  expect(changes.filter((c) => c.added)).toHaveLength(2);
  await expect(page.getByRole('img', { name: 'Yellow label' })).toHaveCount(2);
});

test('the editor sets a label with its key and shows its colour', async ({ page }) => {
  await installMocks(page);
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await gotoAsset(page);

  await page.keyboard.press('8');

  const bar = page.getByRole('navigation', { name: 'Editor status and view controls' });
  const label = bar.getByRole('button', { name: 'Green label' });
  await expect(label).toBeVisible();
  const color = (target: Locator) => target.evaluate((el) => getComputedStyle(el).color);
  expect(await color(label)).not.toBe(await color(bar.getByRole('button', { name: /^Favorite/ })));

  expect(await focusRingClipped(bar.getByRole('radiogroup', { name: 'Rating' }))).toBe('');
  expect(await focusRingClipped(label)).toBe('');
});
