import { expect, test } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

test('photos → asset → export tab', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  const tile = page.locator(`a[href="/assets/${ASSET_ID}"]`).first();
  await expect(tile).toBeVisible();
  const box = await tile.boundingBox();
  if (!box) throw new Error('tile has no bounding box');
  await tile.click({ position: { x: box.width - 6, y: box.height - 6 } });
  await page.waitForURL(`**/assets/${ASSET_ID}`);

  await expect(page.getByTitle('Back')).toBeVisible();

  await page.getByRole('button', { name: 'Export', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Download' })).toBeVisible();
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
  await page.getByTitle('Back').click();
  await page.waitForURL('**/photos');

  await expect.poll(async () => scroller.evaluate((el) => el.scrollTop)).toBeCloseTo(before, 0);
  await expect(link).toBeVisible();
});
