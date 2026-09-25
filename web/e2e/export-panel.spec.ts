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

test('a download carries the output sharpening settings', async ({ page }) => {
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

  await page.getByRole('button', { name: 'Sharpen', exact: true }).click();
  await page.getByRole('option', { name: 'Matte paper' }).click();
  await page.getByRole('button', { name: 'Amount', exact: true }).click();
  await page.getByRole('option', { name: 'High' }).click();
  const ppi = page.getByLabel('Print PPI');
  await expect(ppi).toHaveValue('300');
  const exportButton = page.getByRole('button', { name: /Export JPEG/ });

  await ppi.fill('40');
  await expect(page.getByText('Enter a whole number from 72 to 1200')).toBeVisible();
  await expect(exportButton).toBeDisabled();

  await ppi.fill('240');
  const downloadPromise = page.waitForEvent('download');
  await exportButton.click();
  await downloadPromise;
  expect(
    ['output_sharpen_media', 'output_sharpen_amount', 'output_sharpen_ppi'].map((key) =>
      String(sent[0]?.[key])
    )
  ).toEqual(['matte', 'high', '240']);

  await page.getByRole('button', { name: 'Sharpen', exact: true }).click();
  await page.getByRole('option', { name: 'Screen' }).click();
  await expect(ppi).toBeHidden();
  const second = page.waitForEvent('download');
  await exportButton.click();
  await second;
  expect(sent[1]?.output_sharpen_media).toBe('screen');
  expect(sent[1]).not.toHaveProperty('output_sharpen_ppi');
});

test('every export option label fits on one line', async ({ page }) => {
  await page.addInitScript(() =>
    localStorage.setItem('immich-edit:editorUi', JSON.stringify({ inspectorWidth: 320 }))
  );
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await page.getByRole('button', { name: 'Resize' }).click();
  await page.getByRole('option', { name: 'Dimensions' }).click();
  await page.getByRole('button', { name: 'Sharpen', exact: true }).click();
  await page.getByRole('option', { name: 'Matte paper' }).click();
  await expect(page.getByLabel('Print PPI')).toBeVisible();

  const labels = page.getByRole('tabpanel').locator('label, .editor-compact-label');
  const lines = await labels.evaluateAll((nodes) =>
    nodes
      .filter((node) => node.getClientRects().length > 0)
      .map((node) => {
        const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
        const range = document.createRange();
        const tops: number[] = [];
        for (let text = walker.nextNode(); text; text = walker.nextNode()) {
          if (!text.textContent?.trim()) continue;
          range.selectNodeContents(text);
          tops.push(...[...range.getClientRects()].map((rect) => rect.top));
        }
        tops.sort((a, b) => a - b);
        const count = tops.filter((top, i) => i === 0 || top - (tops[i - 1] ?? top) > 4).length;
        return [node.textContent?.trim(), count] as const;
      })
  );
  expect(lines.length).toBeGreaterThan(8);
  expect(lines.filter(([, count]) => count > 1)).toEqual([]);
});

test('a download carries the chosen watermark', async ({ page }) => {
  const sent: Array<Record<string, unknown>> = [];
  await page.addInitScript(() =>
    localStorage.setItem('immich-edit:editorUi', JSON.stringify({ inspectorWidth: 320 }))
  );
  await installMocks(page, {
    watermarks: [
      {
        id: 'wm-signature',
        name: 'Signature',
        width: 64,
        height: 32,
        size: 10,
        created_at: '2024-01-01T00:00:00Z'
      }
    ],
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
  const exportButton = page.getByRole('button', { name: /Export JPEG/ });
  const picker = page
    .getByRole('region', { name: 'Watermark' })
    .getByRole('button', { name: 'Image', exact: true });

  await picker.click();
  await page.getByRole('option', { name: 'Signature' }).click();
  await page.getByRole('slider', { name: 'Watermark opacity' }).fill('50');
  const topLeft = page.getByRole('button', { name: 'Top left' });
  await topLeft.click();
  await expect(topLeft).toHaveAttribute('aria-pressed', 'true');
  const first = page.waitForEvent('download');
  await exportButton.click();
  await first;
  expect(
    [
      'watermark_id',
      'watermark_size',
      'watermark_opacity',
      'watermark_anchor',
      'watermark_inset'
    ].map((key) => String(sent[0]?.[key]))
  ).toEqual(['wm-signature', '0.2', '0.5', 'top_left', '0.03']);

  await page.locator('input[type="file"][accept="image/png"]').setInputFiles({
    name: 'logo.png',
    mimeType: 'image/png',
    buffer: Buffer.from([0x89, 0x50, 0x4e, 0x47])
  });
  await expect(picker).toContainText('logo');
  await page.getByRole('button', { name: 'Delete watermark', exact: true }).click();
  const region = page.getByRole('region', { name: 'Watermark' });
  const rowButtons = region.locator('.panel-row').first().getByRole('button');
  const boxes = await Promise.all(
    [region.getByText('Image', { exact: true }), ...(await rowButtons.all())].map((locator) =>
      locator.boundingBox()
    )
  );
  const edges = boxes.map((box) => {
    if (!box) throw new Error('watermark row is not laid out');
    return [box.x, box.x + box.width] as const;
  });
  expect(edges.length).toBe(4);
  expect(edges.slice(1).every(([start], i) => start >= (edges[i]?.[1] ?? Infinity))).toBe(true);
  await page.getByRole('button', { name: 'Confirm delete watermark' }).click();
  await expect(page.getByRole('slider', { name: 'Watermark size' })).toBeHidden();
  const second = page.waitForEvent('download');
  await exportButton.click();
  await second;
  expect(sent[1]).not.toHaveProperty('watermark_id');
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

test('export options sit in the same sections in the editor and the bulk dialog', async ({
  page
}) => {
  const titles = ['File', 'Size', 'Watermark', 'Name'];
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  const panel = page.getByRole('tabpanel');
  const headings = panel.locator('section[aria-labelledby] > h3');
  await expect(headings).toHaveText(titles);
  await expect(panel.getByRole('region', { name: 'Size' })).toContainText('6000 × 4000 px');
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await expect(headings).toHaveText([...titles, 'Immich']);
  await expect(
    panel.getByRole('region', { name: 'Immich' }).getByLabel('Add album…')
  ).toBeVisible();

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  await dialog.getByRole('tab', { name: 'Export' }).click();
  await expect(dialog.locator('section[aria-labelledby] > h3')).toHaveText([...titles, 'Immich']);
});
