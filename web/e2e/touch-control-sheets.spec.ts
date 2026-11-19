import { expect, test, type Locator, type Page } from '@playwright/test';
import { datedAssets, dayShots } from './timeline';
import { tapAt } from './touch';
import { open } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 10, 'Bergen'),
  ...dayShots('2026-10-30', 10, 'Ålesund')
]);

test.use({ timezoneId: 'UTC' });

async function openPhotos(page: Page): Promise<void> {
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await open(page, '/photos', { assets: ASSETS });
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

function tile(page: Page, name: string): Locator {
  return page.locator(`[data-name="${name}"]`);
}

async function tapTile(page: Page, name: string): Promise<void> {
  const box = await tile(page, name).boundingBox();
  if (!box) throw new Error(`no tile ${name}`);
  await tapAt(page, box.x + 20, box.y + 20);
}

function appBar(page: Page): Locator {
  return page.locator('header[data-app-bar]');
}

function bulkBar(page: Page): Locator {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

async function selectTwo(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Select photos' }).tap();
  await tapTile(page, 'IMG_0001.ARW');
  await tapTile(page, 'IMG_0002.ARW');
  await expect(appBar(page)).toContainText('2 selected');
}

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
      await openPhotos(page);
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
      await openPhotos(page);
      await selectTwo(page);

      const sheet = await openSheet(page, action, title);
      const [box, sheetBox] = await Promise.all([
        tile(page, 'IMG_0003.ARW').boundingBox(),
        sheet.boundingBox()
      ]);
      expect((box?.y ?? 0) + 20).toBeLessThan(sheetBox?.y ?? 0);
      await tapTile(page, 'IMG_0003.ARW');

      await expect(sheet).toBeHidden();
      await expectUnchanged(page);
      await expect(page).toHaveURL(/\/photos$/);
    });
  }

  test('Escape closes the Rate sheet and keeps the selection', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);

    const sheet = await openSheet(page, 'Rate', 'Rate');
    await page.keyboard.press('Escape');

    await expect(sheet).toBeHidden();
    await expectUnchanged(page);
  });
});
