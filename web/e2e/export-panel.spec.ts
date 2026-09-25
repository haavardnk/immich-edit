import { expect, test } from '@playwright/test';
import { JPEG_BLOB, gotoAsset, installMocks } from './helpers';

test('a failed download export offers a retry beside the button', async ({ page }) => {
  let failures = 1;
  await installMocks(page, {
    onExport: async (route) => {
      if (failures-- > 0) return route.fulfill({ status: 500, body: '{}' });
      return route.fulfill({
        status: 200,
        contentType: 'image/jpeg',
        headers: { 'content-disposition': 'attachment; filename="IMG_0001_edit.jpg"' },
        body: JPEG_BLOB
      });
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

test('a download is named by the filename template', async ({ page }) => {
  const templates: Array<string | null> = [];
  await installMocks(page, {
    onExport: (route) => {
      const request = route.request();
      templates.push(
        new URL(request.url()).searchParams.get('filename_template') ??
          (request.postDataJSON() as { filename_template?: string } | null)?.filename_template ??
          null
      );
      return route.fulfill({
        status: 200,
        contentType: 'image/jpeg',
        headers: {
          'content-disposition': `attachment; filename="2024-01-01_IMG_0001.jpg"; filename*=UTF-8''2024-01-01_IMG_0001.jpg`
        },
        body: JPEG_BLOB
      });
    }
  });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  const field = page.getByLabel('Filename', { exact: true });
  await expect(field).toHaveValue('{name}_edit');
  await expect(page.getByText('IMG_0001_edit.jpg', { exact: true })).toBeVisible();

  await field.fill('{name');
  await expect(page.getByText('Filename template has an unmatched {')).toBeVisible();
  await expect(page.getByRole('button', { name: /Export JPEG/ })).toBeDisabled();

  await field.fill('{date}_{name}');
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  expect((await downloadPromise).suggestedFilename()).toBe('2024-01-01_IMG_0001.jpg');
  expect(templates).toEqual(['{date}_{name}']);
  await expect(page.getByText('Saved 2024-01-01_IMG_0001.jpg')).toBeVisible();
});

test('a download carries the resize settings', async ({ page }) => {
  const sent: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    onExport: (route) => {
      const request = route.request();
      const query = Object.fromEntries(new URL(request.url()).searchParams);
      sent.push(
        request.method() === 'POST' ? (request.postDataJSON() as Record<string, unknown>) : query
      );
      return route.fulfill({
        status: 200,
        contentType: 'image/jpeg',
        headers: { 'content-disposition': 'attachment; filename="IMG_0001_edit.jpg"' },
        body: JPEG_BLOB
      });
    }
  });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  await page.getByRole('button', { name: 'Resize' }).click();
  await page.getByRole('option', { name: 'Dimensions' }).click();
  const width = page.getByRole('spinbutton', { name: 'Width' });
  const height = page.getByRole('spinbutton', { name: 'Height' });
  await expect(width).toHaveValue('6000');
  await expect(height).toHaveValue('4000');
  const exportButton = page.getByRole('button', { name: /Export JPEG/ });

  await width.fill('0');
  await expect(page.getByText('Enter whole pixels from 1 to 65535')).toBeVisible();
  await expect(exportButton).toBeDisabled();

  await width.fill('1600');
  await expect(height).toHaveValue('1067');
  await expect(page.getByText('1600 × 1067 px')).toBeVisible();
  await height.fill('500');
  await expect(width).toHaveValue('750');
  const dontEnlarge = page.getByRole('checkbox', { name: "Don't enlarge" });
  await expect(dontEnlarge).toBeChecked();
  await dontEnlarge.click();
  const downloadPromise = page.waitForEvent('download');
  await exportButton.click();
  await downloadPromise;
  expect(
    ['resize_mode', 'resize_width', 'resize_height', 'resize_enlarge'].map((key) =>
      String(sent[0]?.[key])
    )
  ).toEqual(['dimensions', '750', '500', 'true']);

  await page.getByRole('button', { name: 'Resize' }).click();
  await page.getByRole('option', { name: 'Megapixels' }).click();
  await page.getByRole('spinbutton', { name: 'Megapixels' }).fill('6');
  await expect(page.getByText('3000 × 2000 px')).toBeVisible();
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
