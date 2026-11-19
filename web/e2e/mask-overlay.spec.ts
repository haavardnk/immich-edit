import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';

function weightRender(req: PreviewRequest): Buffer {
  return makePng(60, 40, req.lane === 'weight' ? 255 : 90);
}

async function centrePixel(page: Page): Promise<number[]> {
  return page.getByTestId('mask-weight').evaluate((el) => {
    const canvas = el as HTMLCanvasElement;
    const ctx = canvas.getContext('2d');
    if (!ctx || !canvas.width || !canvas.height) return [];
    const x = Math.floor(canvas.width / 2);
    const y = Math.floor(canvas.height / 2);
    return [...ctx.getImageData(x, y, 1, 1).data];
  });
}

async function addRadial(page: Page): Promise<void> {
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Radial center' })).toBeVisible();
}

test('the overlay colour is saved to the account', async ({ page }) => {
  const patches: Record<string, unknown>[] = [];
  await installMocks(page, { previewRender: weightRender, onPreferences: (p) => patches.push(p) });
  await gotoAsset(page);
  await addRadial(page);
  await expect.poll(() => centrePixel(page)).toEqual([255, 59, 48, 255]);

  await page.getByRole('button', { name: 'Mask overlay options' }).click();
  await page.getByRole('radio', { name: 'Green' }).click();
  await expect.poll(() => patches.at(-1)).toEqual({ mask_overlay_color: '#34c759' });
  await expect.poll(() => centrePixel(page)).toEqual([52, 199, 89, 255]);
  await expect
    .poll(() =>
      page
        .getByTestId('mask-weight')
        .evaluate((el) => getComputedStyle(el).getPropertyValue('--color-mask-overlay').trim())
    )
    .toBe('#34c759');

  await page.getByRole('button', { name: 'White on black', exact: true }).click();
  await expect(page.getByTestId('mask-weight-backdrop')).toHaveAttribute(
    'data-mode',
    'whiteOnBlack'
  );
  await page.getByRole('button', { name: 'Show overlay', exact: true }).click();
  await expect(page.getByTestId('mask-weight')).toHaveCount(0);

  await page.getByRole('radio', { name: 'Red' }).click();
  await expect.poll(() => patches.at(-1)).toEqual({ mask_overlay_color: null });
});
