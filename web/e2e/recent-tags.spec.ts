import { expect, test, type Page } from '@playwright/test';
import { ASSET_EXIF, ASSET_ID, ASSET_SUMMARY, installMocks, type MockTag } from './helpers';
import { openTileLoupe } from './touch';

const SECOND_ID = '00000000-0000-0000-0000-000000000002';

const TAGS: MockTag[] = [
  { id: 'tag-places', name: 'Places', value: 'Places' },
  { id: 'tag-norway', name: 'Norway', value: 'Places/Norway' },
  { id: 'tag-landscape', name: 'Landscape', value: 'Landscape' }
];

type TagChange = { tagId: string; assetId: string; added: boolean };

async function openLoupe(page: Page, applied: MockTag[] = []): Promise<TagChange[]> {
  const changes: TagChange[] = [];
  await installMocks(page, {
    assets: [
      { ...ASSET_SUMMARY, tags: applied },
      { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW', exifInfo: ASSET_EXIF }
    ],
    tags: TAGS,
    onTagAsset: (change) => changes.push(change)
  });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`[role="group"] > a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await openTileLoupe(page, page.locator('[role="group"][data-name]').first());
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  return changes;
}

function rail(page: Page) {
  return page.getByRole('navigation', { name: 'Photo actions' });
}

test('a picked tag turns recent and Alt+1 adds it to the next photo', async ({ page }) => {
  const changes = await openLoupe(page);
  await rail(page).getByRole('button', { name: 'Tags (Shift+T)' }).click();
  const picker = page.locator('[data-popover-content]');
  await expect(picker.getByRole('textbox', { name: 'Add tags…' })).toBeFocused();
  await expect(picker.getByRole('region', { name: 'Recent' })).toHaveCount(0);

  await picker.getByRole('button', { name: 'Expand Places' }).click();
  await picker.getByRole('checkbox', { name: 'Norway' }).click();
  await expect
    .poll(() => changes)
    .toEqual([{ tagId: 'tag-norway', assetId: ASSET_ID, added: true }]);
  const recent = picker.getByRole('region', { name: 'Recent' });
  const chip = recent.getByRole('button', { name: 'Places/Norway' });
  await expect(chip).toHaveAttribute('aria-pressed', 'true');
  await expect(chip).toHaveText(/1\s*Norway/);
  await expect(rail(page).getByRole('button', { name: 'Remove Places/Norway' })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(picker).toBeHidden();
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('heading', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(rail(page).getByRole('button', { name: 'Remove Places/Norway' })).toHaveCount(0);

  await page.keyboard.press('Alt+Digit1');
  await expect
    .poll(() => changes)
    .toContainEqual({
      tagId: 'tag-norway',
      assetId: SECOND_ID,
      added: true
    });
  await expect(rail(page).getByRole('button', { name: 'Remove Places/Norway' })).toBeVisible();
});

test('Alt and a digit without a recent tag stays with the loupe', async ({ page }) => {
  const changes = await openLoupe(page);
  await page.evaluate(() => {
    const seen: boolean[] = [];
    Object.assign(window, { recentChord: seen });
    window.addEventListener('keydown', (event) => {
      if (event.code === 'Digit5') seen.push(event.defaultPrevented);
    });
  });
  await page.keyboard.press('Alt+Digit5');
  await expect
    .poll(() => page.evaluate(() => (window as unknown as { recentChord: boolean[] }).recentChord))
    .toEqual([true]);
  expect(changes).toEqual([]);
});

test.describe('the tags sheet on touch', { tag: '@touch' }, () => {
  test('shows the applied and recent tags', { tag: '@phone' }, async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem(
        'immich-edit:recentTags',
        JSON.stringify({ tags: [{ id: 'tag-norway', name: 'Norway', value: 'Places/Norway' }] })
      );
    });
    const changes = await openLoupe(page, [TAGS[2] as MockTag]);
    await rail(page).getByRole('button', { name: 'Tags (1)' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Tags' });
    await expect(sheet).toContainText('IMG_0001.ARW');

    const applied = sheet.getByRole('region', { name: 'Applied' });
    await expect(applied.getByRole('button', { name: 'Remove Landscape' })).toBeVisible();
    const chip = sheet.getByRole('region', { name: 'Recent' }).getByRole('button', {
      name: 'Places/Norway'
    });
    await expect(chip).toHaveText('Norway');
    await expect(chip).toHaveAttribute('aria-pressed', 'false');

    await chip.tap();
    await expect
      .poll(() => changes)
      .toEqual([{ tagId: 'tag-norway', assetId: ASSET_ID, added: true }]);
    await expect(applied.getByRole('button', { name: 'Remove Places/Norway' })).toBeVisible();
    await expect(chip).toHaveAttribute('aria-pressed', 'true');

    await sheet.getByRole('button', { name: 'Done' }).tap();
    await expect(sheet).toBeHidden();
  });
});
