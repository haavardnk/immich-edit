import { expect, test, type Locator, type Page } from '@playwright/test';
import { bulkBar, tile } from './browse';
import { datedAssets, dayShots, openTouchTimeline } from './timeline';
import { appBar, selectTwo, tapTile } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 10, 'Bergen'),
  ...dayShots('2026-10-30', 10, 'Ålesund')
]);

const MOCKS = { assets: ASSETS, metadataPushConsented: true };

test.use({ timezoneId: 'UTC' });

async function openSheet(page: Page, action: string, title: string): Promise<Locator> {
  await bulkBar(page).getByRole('button', { name: action, exact: true }).tap();
  const sheet = page.getByRole('dialog', { name: title });
  await expect(sheet).toBeVisible();
  await expect(sheet.getByRole('heading', { name: title })).toBeVisible();
  return sheet;
}

async function expectUnchanged(page: Page): Promise<void> {
  await expect(appBar(page)).toContainText('2 selected');
  await expect(bulkBar(page)).toBeVisible();
  await expect(tile(page, 'IMG_0003.ARW')).not.toHaveAttribute('data-selected');
  await expect(page.getByText(/^Updated \d+ assets?$/)).toHaveCount(0);
}

const SHEETS = [
  { action: 'Rate', title: 'Rate', clear: 'No rating' },
  { action: 'Label', title: 'Colour label', clear: 'No label' }
];

test.describe('touch control sheets', { tag: '@touch' }, () => {
  for (const { action, title, clear } of SHEETS) {
    test(`the ${title} sheet closes from its close button without assigning`, async ({ page }) => {
      await openTouchTimeline(page, MOCKS);
      await selectTwo(page);

      const sheet = await openSheet(page, action, title);
      await expect(sheet.getByRole('button', { name: clear, exact: true })).toBeVisible();
      await sheet.getByRole('button', { name: 'Close', exact: true }).tap();

      await expect(sheet).toBeHidden();
      await expectUnchanged(page);
    });

    test(`a tap outside the ${title} sheet closes it and leaves the tile alone`, async ({
      page
    }) => {
      await openTouchTimeline(page, MOCKS);
      await selectTwo(page);

      const sheet = await openSheet(page, action, title);
      const [box, sheetBox] = await Promise.all([
        tile(page, 'IMG_0003.ARW').boundingBox(),
        sheet.boundingBox()
      ]);
      expect((box?.y ?? 0) + (box?.height ?? 0) / 2).toBeLessThan(sheetBox?.y ?? 0);
      await tapTile(page, 'IMG_0003.ARW');

      await expect(sheet).toBeHidden();
      await expectUnchanged(page);
      await expect(page).toHaveURL(/\/photos$/);
    });
  }

  test('Escape closes the Rate sheet and keeps the selection', async ({ page }) => {
    await openTouchTimeline(page, MOCKS);
    await selectTwo(page);

    const sheet = await openSheet(page, 'Rate', 'Rate');
    await page.keyboard.press('Escape');

    await expect(sheet).toBeHidden();
    await expectUnchanged(page);
  });
});
