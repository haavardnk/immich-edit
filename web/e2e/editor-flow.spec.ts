import { expect, test } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

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
