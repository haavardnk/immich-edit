import { expect, test } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

test('photos → asset → export tab', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  const tile = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(tile).toBeVisible();
  const box = await tile.boundingBox();
  if (!box) throw new Error('tile has no bounding box');
  await tile.click({ position: { x: box.width - 6, y: box.height - 6 } });
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));

  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await expect(page.getByRole('radio', { name: 'Download' })).toBeVisible();
});

test('photos grid restores scroll after editor back', async ({ page }) => {
  const assets = Array.from({ length: 80 }, (_, index) => {
    const assetNumber = index + 1;
    const suffix = String(assetNumber).padStart(12, '0');
    return {
      ...ASSET_SUMMARY,
      id: `00000000-0000-0000-0000-${suffix}`,
      originalFileName: `IMG_${String(assetNumber).padStart(4, '0')}.ARW`,
      checksum: suffix
    };
  });
  await installMocks(page, { assets });

  await page.goto('/photos');
  const scroller = page.getByRole('main').locator('.overflow-y-auto').first();
  await expect(scroller).toBeVisible();
  await scroller.evaluate((el) => {
    el.scrollTop = 900;
    el.dispatchEvent(new Event('scroll'));
  });
  await expect.poll(async () => scroller.evaluate((el) => el.scrollTop)).toBeGreaterThan(500);
  const link = page.getByRole('main').locator('a[href^="/assets/"]').first();
  await expect(link).toBeVisible();
  const href = await link.getAttribute('href');
  if (!href) throw new Error('visible tile has no href');
  const before = await scroller.evaluate((el) => el.scrollTop);

  await link.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(`**${href}`);
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expect.poll(async () => scroller.evaluate((el) => el.scrollTop)).toBeCloseTo(before, 0);
  await expect(link).toBeVisible();
});

test('back returns to the grid and selects the photo left open', async ({ page }) => {
  const second = '00000000-0000-0000-0000-000000000002';
  await installMocks(page, {
    assets: [
      ASSET_SUMMARY,
      { ...ASSET_SUMMARY, id: second, originalFileName: 'IMG_0002.ARW', checksum: 'bbbb' }
    ]
  });

  await page.goto('/search?q=IMG');
  const first = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await expect(first).toBeVisible();
  await first.evaluate((el) => (el as HTMLAnchorElement).click());
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));

  await page.locator(`a[href^="/assets/${second}?"]`).first().click();
  await page.waitForURL(new RegExp(`/assets/${second}(?:\\?.*)?$`));

  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/search?q=IMG');
  await expect(
    page.getByRole('main').locator(`div:has(> a[href^="/assets/${second}?"])`)
  ).toHaveClass(/ring-primary/);
});

test('back from a directly opened photo lands on a clean grid', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');
  await expect(page.getByRole('main').locator('.ring-primary')).toHaveCount(0);
});

test('the grid keeps its selection across an editor round trip', async ({ page }) => {
  const second = '00000000-0000-0000-0000-000000000002';
  await installMocks(page, {
    assets: [
      ASSET_SUMMARY,
      { ...ASSET_SUMMARY, id: second, originalFileName: 'IMG_0002.ARW', checksum: 'bbbb' }
    ]
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).nth(0).click();
  await page.getByRole('button', { name: 'Select', exact: true }).nth(0).click();
  await expect(page.getByText('2 selected')).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('d');
  await page.waitForURL('**/assets/**');
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expect(page.getByText('2 selected')).toBeVisible();
});
