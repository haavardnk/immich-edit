import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks, NEUTRAL_RECORD, type PreviewRequest } from './helpers';

function exposureSlider(page: import('@playwright/test').Page) {
  return page
    .locator('div.group', {
      has: page.getByRole('button', { name: 'Exposure', exact: true })
    })
    .getByRole('slider');
}

function geometrySlider(page: import('@playwright/test').Page, label: string) {
  return page
    .locator('div.group', { has: page.getByRole('button', { name: label, exact: true }) })
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
    .poll(() =>
      requests.some((r) => (r.edits as { basic: { exposure_ev: number } }).basic.exposure_ev === 1)
    )
    .toBe(true);
});

test('color range eyedropper samples maskless preview', async ({ page }) => {
  const previews: PreviewRequest[] = [];
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    onPreview: (request) => previews.push(request),
    onSave: (body) => saves.push(body)
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Color range', exact: true }).click();

  const maskPreview = page.getByRole('button', { name: 'Toggle mask preview', exact: true });
  await maskPreview.click();
  await expect
    .poll(() =>
      previews.some(
        (request) =>
          typeof request.preview_mode === 'object' &&
          request.preview_mode !== null &&
          'mask_weight' in request.preview_mode
      )
    )
    .toBe(true);
  await maskPreview.click();

  const picker = page.getByRole('button', { name: 'Pick color from image' });
  await picker.click();
  await expect
    .poll(() =>
      previews.some((request) => {
        const edits = request.edits as { masks?: unknown[] };
        return Array.isArray(edits.masks) && edits.masks.length === 0;
      })
    )
    .toBe(true);

  await page.getByRole('button', { name: 'Sample mask color' }).click();
  await expect.poll(() => saves.length).toBeGreaterThan(1);
  const body = saves.at(-1) as {
    manifest?: {
      ops?: {
        masks?: {
          layers?: Array<{
            components?: Array<{ kind?: { kind?: string; sample_rgb?: number[] } }>;
          }>;
        };
      };
    };
  };
  const kind = body.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind;
  expect(kind?.kind).toBe('color_range');
  expect(kind?.sample_rgb).toHaveLength(3);
  expect(kind?.sample_rgb).not.toEqual([0.5, 0.5, 0.5]);
  await expect(page.getByRole('button', { name: 'Pick color from image' })).toBeVisible();

  await page.getByRole('button', { name: 'Pick color from image' }).click();
  await expect(page.getByRole('button', { name: 'Sample mask color' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: 'Sample mask color' })).toHaveCount(0);
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

test('keyboard help filters shortcuts and labels keys for the platform', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.keyboard.press('Shift+/');
  await expect(page.getByText('Undo', { exact: true })).toBeVisible();
  await expect(page.locator('kbd', { hasText: 'Ctrl+Z' }).first()).toBeVisible();

  await page.getByLabel('Filter shortcuts').fill('retouch');
  await expect(page.getByText('Open Retouch', { exact: true })).toBeVisible();
  await expect(page.getByText('Undo', { exact: true })).toHaveCount(0);
});

test('keyboard help splits binds by the current context', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.keyboard.press('Shift+/');
  await expect(page.getByRole('heading', { name: 'Available in the editor' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Elsewhere in the app' })).toBeVisible();

  await page.keyboard.press('Escape');
  await page.keyboard.press('m');
  await page.keyboard.press('Shift+/');
  await expect(page.getByRole('heading', { name: 'Available in the Masks panel' })).toBeVisible();
});

test('Q opens Retouch and M opens Masks', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.keyboard.press('q');
  await expect(page.getByRole('button', { name: 'Retouch' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );

  await page.keyboard.press('m');
  await expect(page.getByRole('button', { name: 'Masks' })).toHaveAttribute('aria-pressed', 'true');
});

test('tab hides the side panels and shift+tab hides all chrome', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await expect(page.getByRole('button', { name: 'collapse edit panel' })).toBeVisible();

  await page.keyboard.press('Tab');
  await expect(page.getByRole('button', { name: 'expand edit panel' })).toBeVisible();

  await page.keyboard.press('Tab');
  await expect(page.getByRole('button', { name: 'collapse edit panel' })).toBeVisible();

  await page.keyboard.press('Shift+Tab');
  await expect(page.getByRole('button', { name: 'expand edit panel' })).toBeVisible();
});

test('Geometry pane always exposes crop controls', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Geometry' }).click();
  await expect(page.getByRole('button', { name: 'Geometry' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'Angle', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await geometrySlider(page, 'Angle').fill('5');
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
  await expect(page.getByRole('button', { name: 'Develop' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
});

test('perspective sliders save into the transform op', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Geometry' }).click();
  await geometrySlider(page, 'Vertical').fill('40');
  await geometrySlider(page, 'Aspect').fill('-25');

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('button', { name: 'Develop' }).click();
  const request = await saved;
  const body = request.postDataJSON() as {
    manifest: { ops: { transform?: { perspective?: { vertical?: number; aspect?: number } } } };
  };
  expect(body.manifest.ops.transform?.perspective?.vertical).toBe(40);
  expect(body.manifest.ops.transform?.perspective?.aspect).toBe(-25);
});

test('perspective corner handles drag into the transform op', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Geometry' }).click();
  await expect(page.getByRole('button', { name: 'perspective corner 1' })).toHaveCount(0);

  await page.getByRole('button', { name: 'Corner handles' }).click();
  const handle = page.getByRole('button', { name: 'perspective corner 1' });
  await expect(handle).toBeVisible();

  const box = await handle.boundingBox();
  if (!box) throw new Error('corner handle has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 30, box.y + box.height / 2 + 20, { steps: 5 });
  await page.mouse.up();

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('button', { name: 'Develop' }).click();
  const request = await saved;
  const body = request.postDataJSON() as {
    manifest: { ops: { transform?: { perspective?: { corners?: number[][] } } } };
  };
  const corners = body.manifest.ops.transform?.perspective?.corners;
  expect(corners).toBeDefined();
  expect(corners?.[0][0]).toBeGreaterThan(0);
  expect(corners?.[0][1]).toBeGreaterThan(0);
});

test('R opens Geometry and Escape returns to Develop', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'collapse edit panel' }).click();
  await expect(page.getByRole('button', { name: 'expand edit panel' })).toBeVisible();

  await page.keyboard.press('r');
  await expect(page.getByRole('button', { name: 'Geometry' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByText('Angle', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: 'Develop' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
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
    page
      .locator('div.group', { has: page.getByRole('button', { name: 'Distortion' }) })
      .getByRole('slider')
  ).toHaveValue('100');
  await expect(
    page
      .locator('div.group', { has: page.getByRole('button', { name: 'Vignetting' }) })
      .getByRole('slider')
  ).toHaveValue('100');
});
