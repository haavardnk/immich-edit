import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks, NEUTRAL_RECORD, type PreviewRequest } from './helpers';

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

test('lens reset clears all profile edits', async ({ page }) => {
  await installMocks(page, {
    editRecord: {
      ...NEUTRAL_RECORD,
      hash: 'hash-lens',
      manifest: {
        schema_version: 3,
        ops: {
          lens_profile: {
            profile_enabled: true,
            ca_enabled: true,
            constrain_crop: true,
            distortion_amount: 70,
            vignette_amount: 80,
            k1: 0.1,
            k2: 0.2,
            k3: 0.3,
            vk1: 0.4,
            vk2: 0.5,
            vk3: 0.6,
            ca_red: 12,
            ca_blue: -8
          }
        }
      }
    }
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Lens Corrections' }).click();
  await expect(page.getByLabel('Enable Profile Corrections')).toBeChecked();
  await expect(page.getByLabel('Remove Chromatic Aberration')).toBeChecked();

  const deleted = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'DELETE'
  );
  await page.getByRole('button', { name: 'Reset Lens Corrections' }).click();
  await deleted;

  await expect(page.getByLabel('Enable Profile Corrections')).not.toBeChecked();
  await expect(page.getByLabel('Remove Chromatic Aberration')).not.toBeChecked();
  await expect(page.getByLabel('Constrain Crop')).not.toBeChecked();
  await expect(
    page.locator('div.group', { has: page.getByRole('button', { name: 'Distortion' }) }).getByRole('slider')
  ).toHaveValue('100');
  await expect(
    page.locator('div.group', { has: page.getByRole('button', { name: 'Vignetting' }) }).getByRole('slider')
  ).toHaveValue('100');
});
