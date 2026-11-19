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

test('shift O cycles the mask overlay modes with a pill', async ({ page }) => {
  await installMocks(page, { previewRender: weightRender });
  await gotoAsset(page);
  await addRadial(page);

  const backdrop = page.getByTestId('mask-weight-backdrop');
  const pill = page.locator('[data-photo-pill]');
  await expect(backdrop).toHaveAttribute('data-mode', 'color');
  await expect.poll(() => centrePixel(page)).toEqual([255, 59, 48, 255]);

  await page.keyboard.press('Shift+O');
  await expect(backdrop).toHaveAttribute('data-mode', 'colorOnBw');
  await expect(pill).toContainText('Color on black and white');
  await expect(pill).toContainText('cycles overlay');
  await expect
    .poll(() => backdrop.evaluate((el) => getComputedStyle(el).backdropFilter))
    .toContain('grayscale');

  await page.keyboard.press('Shift+O');
  await expect(backdrop).toHaveAttribute('data-mode', 'whiteOnBlack');
  await expect(pill).toContainText('White on black');
  await expect.poll(() => centrePixel(page)).toEqual([255, 255, 255, 255]);
  await expect(page.getByTestId('mask-weight')).toHaveCSS('opacity', '1');
  await expect(pill).toHaveCount(0, { timeout: 4000 });

  await page.reload();
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'Mask overlay options' }).click();
  await expect(page.getByRole('button', { name: 'White on black', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await page.keyboard.press('Escape');
  await page.keyboard.press('Shift+O');
  await page.getByRole('button', { name: 'Mask overlay options' }).click();
  await expect(page.getByRole('button', { name: 'Color', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});

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
