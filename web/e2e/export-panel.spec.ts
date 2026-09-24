import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';

test('a failed download export offers a retry beside the button', async ({ page }) => {
  let failures = 1;
  await installMocks(page, {
    onExport: async (route) => {
      if (failures-- > 0) return route.fulfill({ status: 500, body: '{}' });
      return route.fallback();
    }
  });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  await page.getByRole('button', { name: /Export JPEG/ }).click();
  const panel = page.getByRole('tabpanel');
  await expect(panel.getByText(/^Export failed/)).toBeVisible();

  const downloadPromise = page.waitForEvent('download');
  await panel.getByRole('button', { name: 'Retry' }).click();
  await downloadPromise;
  await expect(panel.getByText('Saved IMG_0001_edit.jpg')).toBeVisible();
});

test('export settings survive a reload', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  await page.getByRole('button', { name: 'Format' }).click();
  await page.getByRole('option', { name: 'AVIF' }).click();
  await expect(page.getByRole('button', { name: /Export AVIF/ })).toBeVisible();

  await page.reload();
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await expect(page.getByRole('button', { name: /Export AVIF/ })).toBeVisible();
});

test('a download carries the filename suffix', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  await page.getByLabel('Filename suffix').fill('_warm');
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  expect((await downloadPromise).suggestedFilename()).toBe('IMG_0001_warm.jpg');
});

test('export warns when it will not match the soft proof', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('button', { name: 'Color space' }).click();
  await page.getByRole('option', { name: 'Display P3' }).click();

  await page.getByRole('button', { name: 'More editor actions' }).click();
  await page.getByRole('checkbox', { name: 'Show gamut warning' }).check();
  await page.keyboard.press('Escape');

  const warning = page.getByText('Soft proofing sRGB, exporting Display P3');
  await expect(warning).toBeVisible();
  await page.getByRole('button', { name: 'Export sRGB' }).click();
  await expect(warning).toBeHidden();
  await expect(page.getByRole('button', { name: 'Color space' })).toHaveText('sRGB');
});
