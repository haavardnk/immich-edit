import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID } from './helpers';
import { mockLibrary } from './touchShell';

const sheet = (page: Page) => page.getByRole('dialog', { name: 'Export' });

const SIGNATURE = {
  id: 'wm-signature',
  name: 'Signature',
  width: 64,
  height: 32,
  size: 10,
  created_at: '2024-01-01T00:00:00Z'
};

async function openExport(page: Page): Promise<void> {
  await mockLibrary(page, { watermarks: [SIGNATURE] });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await page.getByRole('button', { name: 'Share', exact: true }).tap();
  await expect(sheet(page)).toBeVisible();
}

async function stubShare(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const shared: string[] = [];
    Object.assign(window, { shared });
    navigator.canShare = () => true;
    navigator.share = async (data?: ShareData) => {
      shared.push(...(data?.files ?? []).map((file) => file.name));
    };
  });
}

async function sharedNames(page: Page): Promise<string[]> {
  return page.evaluate(() => (window as unknown as { shared: string[] }).shared);
}

for (const device of ['phone', 'tablet'] as const) {
  test.describe(`touch export on a ${device}`, { tag: ['@touch', `@${device}`] }, () => {
    test('the sheet sums up the output and turns into the ready file', async ({ page }) => {
      await openExport(page);
      const dialog = sheet(page);
      await expect(dialog.getByRole('radio', { name: 'Device' })).toBeChecked();
      await expect(dialog.getByRole('button', { name: 'Export presets' })).toBeVisible();
      await expect(dialog.getByText('6000 × 4000 px', { exact: true }).last()).toBeVisible();
      await expect(dialog.getByText('JPEG · 90 · sRGB', { exact: true })).toBeVisible();

      await dialog.getByRole('button', { name: /Export JPEG/ }).tap();
      await expect(dialog.getByText('Ready to save')).toBeVisible();
      await expect(dialog.getByText(/^Rendered in \d+\.\d s$/)).toBeVisible();
      await expect(dialog.getByRole('group', { name: 'Exported file' })).toContainText(
        'IMG_0001_edit.jpg'
      );
      await expect(dialog.getByRole('combobox', { name: 'Format' })).toHaveCount(0);

      const downloadPromise = page.waitForEvent('download');
      await dialog.getByRole('button', { name: 'Save', exact: true }).tap();
      expect((await downloadPromise).suggestedFilename()).toBe('IMG_0001_edit.jpg');
      await expect(dialog.getByText('Saved', { exact: true }).first()).toBeVisible();

      await dialog.getByRole('button', { name: 'Settings' }).tap();
      await expect(dialog.getByRole('combobox', { name: 'Format' })).toBeVisible();
      await dialog.getByRole('button', { name: 'File', exact: true }).tap();
      await expect(dialog.getByRole('group', { name: 'Exported file' })).toBeVisible();
      await dialog.getByRole('button', { name: 'Settings' }).tap();
      await dialog.getByRole('button', { name: 'Close export' }).tap();
      await expect(dialog).toBeHidden();
    });

    test('picking a watermark lowers the sheet over its preview', async ({ page }) => {
      await openExport(page);
      const dialog = sheet(page);
      const full = (await dialog.boundingBox())?.height ?? 0;
      await dialog
        .getByRole('radiogroup', { name: 'Watermark' })
        .getByRole('radio', { name: 'Signature' })
        .tap();
      await expect
        .poll(async () => (await dialog.boundingBox())?.height ?? 0)
        .toBeLessThan(full * 0.7);
      await expect(page.getByTestId('watermark-preview')).toBeVisible();
      await expect(dialog.getByRole('slider', { name: 'Watermark size' })).toBeAttached();
    });

    test('a ready file shares on a fresh tap', async ({ page }) => {
      await stubShare(page);
      await openExport(page);
      const dialog = sheet(page);
      await dialog.getByRole('button', { name: /Export JPEG/ }).tap();
      await expect(dialog.getByText('Ready to share')).toBeVisible();
      expect(await sharedNames(page)).toEqual([]);

      await dialog.getByRole('button', { name: 'Share', exact: true }).tap();
      await expect.poll(() => sharedNames(page)).toEqual(['IMG_0001_edit.jpg']);
    });
  });
}
