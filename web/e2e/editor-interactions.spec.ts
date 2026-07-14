import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks, type PreviewRequest } from './helpers';

function exposureSlider(page: import('@playwright/test').Page) {
  return page
    .locator('div.group', {
      has: page.getByRole('button', { name: 'Exposure', exact: true })
    })
    .getByRole('slider');
}

test('adjusting a slider requests a live preview with the new edit', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (req) => requests.push(req) });
  await gotoAsset(page);

  const slider = exposureSlider(page);
  await expect(slider).toHaveValue('0');
  await slider.fill('1');

  await expect
    .poll(() => requests.some((r) => (r.edits as { basic: { exposure_ev: number } }).basic.exposure_ev === 1))
    .toBe(true);
});

test('keyboard help modal toggles with the ? key', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.keyboard.press('Shift+/');
  await expect(page.getByRole('heading', { name: 'Keyboard shortcuts' })).toBeVisible();
  await expect(page.getByText('Open Geometry', { exact: true })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('heading', { name: 'Keyboard shortcuts' })).toHaveCount(0);
});

test('Geometry pane always exposes crop controls', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Geometry' }).click();
  await expect(page.getByRole('button', { name: 'Geometry' })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByText('Angle', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await page.getByRole('slider', { name: 'Angle' }).fill('5');
  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('button', { name: 'Develop' }).click();
  const request = await saved;
  const body = request.postDataJSON() as {
    manifest: { ops: { transform?: { angle?: number } } };
    action: string | null;
  };
  expect(body.manifest.ops.transform?.angle).toBe(5);
  expect(body.action).toBe('Geometry');
  await expect(page.getByRole('button', { name: 'Develop' })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
});

test('C opens Geometry and Escape returns to Develop', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'collapse edit panel' }).click();
  await expect(page.getByRole('button', { name: 'expand edit panel' })).toBeVisible();

  await page.keyboard.press('c');
  await expect(page.getByRole('button', { name: 'Geometry' })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByText('Angle', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: 'Develop' })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
});
