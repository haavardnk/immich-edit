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
  const row = page.locator('div.group', {
    has: page.getByRole('button', { name: 'Exposure', exact: true })
  });
  await expect(slider).toHaveValue('0');
  await slider.fill('1');

  await expect(
    row.getByRole('button', { name: 'Exposure', exact: true }).locator('[aria-hidden]')
  ).toHaveCount(0);

  await expect
    .poll(() =>
      requests.some((r) => (r.edits as { basic: { exposure_ev: number } }).basic.exposure_ev === 1)
    )
    .toBe(true);
});

test('exact slider value entry commits through the shared control', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (req) => requests.push(req) });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Edit Exposure value' }).click();
  const value = page.getByRole('spinbutton', { name: 'Exposure value' });
  await value.fill('1.25');
  await value.press('Enter');

  await expect
    .poll(() =>
      requests.some(
        (request) =>
          (request.edits as { basic: { exposure_ev: number } }).basic.exposure_ev === 1.25
      )
    )
    .toBe(true);
  await expect(page.getByRole('button', { name: 'Edit Exposure value' })).toHaveText('1.25');
});

test('capture sharpening toggle is enabled only for raw assets', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { previewMeta: { is_raw: true }, onSave: (body) => saves.push(body) });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Detail', exact: true }).click();
  const toggle = page.getByRole('checkbox', { name: 'Capture Sharpening' });
  await expect(toggle).toBeEnabled();
  await expect(toggle).toBeChecked();

  await toggle.uncheck();
  await expect
    .poll(() => saves.some((s) => JSON.stringify(s).includes('"capture_sharpen"')))
    .toBe(true);

  await page.getByText('Capture Sharpening').click();
  await expect(toggle).toBeChecked();
});

test('capture sharpening toggle is disabled for non-raw assets', async ({ page }) => {
  await installMocks(page, { previewMeta: { is_raw: false } });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Detail', exact: true }).click();
  await expect(page.getByRole('checkbox', { name: 'Capture Sharpening' })).toBeDisabled();
});

test('histogram distinguishes loading from absent data', async ({ page }) => {
  let releaseMeta = (): void => {};
  const metaPending = new Promise<void>((resolve) => {
    releaseMeta = resolve;
  });
  await installMocks(page);
  await page.route('**/api/assets/*/preview/meta/*', async (route) => {
    await metaPending;
    await route.fallback();
  });
  await gotoAsset(page);

  await expect(page.getByText('Loading histogram…')).toBeVisible();
  releaseMeta();
  await expect(page.getByText('No histogram data')).toBeVisible();
});

test('lens profile failure can be retried', async ({ page }) => {
  let attempts = 0;
  await installMocks(page);
  await page.route('**/api/assets/*/lens-profile', (route) => {
    attempts += 1;
    if (attempts === 1) {
      return route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Lens profile unavailable' })
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ matched: false, lens: null, edits: null })
    });
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Lens Corrections' }).click();
  await page.getByRole('button', { name: 'Try again' }).click();

  await expect(page.getByText('No matching lens profile')).toBeVisible();
  expect(attempts).toBe(2);
});

test('pending saves guard browser unload', async ({ page }) => {
  let releaseSave = (): void => {};
  const savePending = new Promise<void>((resolve) => {
    releaseSave = resolve;
  });
  await installMocks(page);
  await page.route('**/api/assets/*/edits', async (route) => {
    if (route.request().method() === 'PUT') await savePending;
    await route.fallback();
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Edit Exposure value' }).click();
  const value = page.getByRole('spinbutton', { name: 'Exposure value' });
  await value.fill('1');
  await value.press('Enter');
  await expect(page.getByText('Saving…')).toBeVisible();

  const unloadPrevented = (): Promise<boolean> =>
    page.evaluate(() => {
      const event = new Event('beforeunload', { cancelable: true });
      window.dispatchEvent(event);
      return event.defaultPrevented;
    });
  expect(await unloadPrevented()).toBe(true);

  releaseSave();
  await expect(page.getByText('Saved')).toBeVisible();
  expect(await unloadPrevented()).toBe(false);
});

test('the soft proof popover survives opening its nested select', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'More editor actions' }).click();
  const warn = page.getByRole('checkbox', { name: 'Show gamut warning' });
  await expect(warn).toBeVisible();

  await page.getByRole('button', { name: 'Proof space' }).click();
  await expect(page.getByRole('option', { name: 'sRGB' })).toBeVisible();
  await page.keyboard.press('Escape');

  await expect(warn).toBeVisible();
  await warn.check();
  await expect(warn).toBeChecked();
});

test('color range eyedropper samples maskless preview', async ({ page }) => {
  const previews: PreviewRequest[] = [];
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    onPreview: (request) => previews.push(request),
    onSave: (body) => saves.push(body)
  });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
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

test('dragging the radial centre handle moves the saved shape', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();

  const centre = page.getByRole('button', { name: 'Radial center' });
  await expect(centre).toBeVisible();
  const box = await centre.boundingBox();
  if (!box) throw new Error('radial centre handle has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 60, box.y + box.height / 2, { steps: 4 });
  await page.mouse.up();

  await expect
    .poll(() => {
      const body = saves.at(-1) as {
        manifest?: {
          ops?: {
            masks?: {
              layers?: Array<{
                components?: Array<{ kind?: { kind?: string; center?: { x: number } } }>;
              }>;
            };
          };
        };
      };
      return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind?.center?.x ?? 0.5;
    })
    .toBeGreaterThan(0.5);
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

test('keyboard help opens from the editor toolbar', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'More editor actions' }).click();
  await page.getByRole('button', { name: 'Keyboard shortcuts' }).click();
  await expect(page.getByRole('heading', { name: 'Keyboard shortcuts' })).toBeVisible();
  await expect(page.getByText('Open Geometry', { exact: true })).toBeVisible();
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

test('D opens Develop, Q opens Retouch and M opens Masks', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.keyboard.press('q');
  await expect(page.getByRole('tab', { name: 'Retouch' })).toHaveAttribute('aria-selected', 'true');

  await page.keyboard.press('m');
  await expect(page.getByRole('tab', { name: 'Masks' })).toHaveAttribute('aria-selected', 'true');

  await page.keyboard.press('d');
  await expect(page.getByRole('tab', { name: 'Develop' })).toHaveAttribute('aria-selected', 'true');
});

test('editor tool tabs switch panels with pointer clicks', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  for (const name of ['Masks', 'Retouch', 'Geometry', 'Export', 'Develop']) {
    const tab = page.getByRole('tab', { name });
    await tab.click();
    await expect(tab).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('tabpanel', { name })).toBeVisible();
  }
});

test('tab hides the side panels and shift+tab hides all chrome', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const controls = page.getByRole('complementary', { name: 'Editor controls' });
  await expect(controls).toBeVisible();

  await page.keyboard.press('Tab');
  await expect(controls).toBeHidden();

  await page.keyboard.press('Tab');
  await expect(controls).toBeVisible();

  await page.keyboard.press('Shift+Tab');
  await expect(controls).toBeHidden();
});

test('fullscreen leaves only the image and exit control', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: /^Fullscreen/ }).click();

  await expect(page.getByRole('application')).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Editor toolbar' })).toHaveCount(0);
  await expect(
    page.getByRole('navigation', { name: 'Editor status and view controls' })
  ).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Editor controls' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: /^Exit fullscreen/ })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('navigation', { name: 'Editor toolbar' })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Fullscreen/ })).toBeVisible();
});

test('collapsed editor controls leave the canvas and retain their state', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await expect(page.getByRole('navigation', { name: 'Global navigation' })).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Library' })).toHaveCount(0);
  await page.getByRole('tab', { name: 'Geometry' }).click();
  const controls = page.locator('aside[aria-label="Editor controls"]');
  const viewer = page.getByRole('application');
  const openWidth = await controls.evaluate((element) => element.getBoundingClientRect().width);
  const viewerWidth = await viewer.evaluate((element) => element.getBoundingClientRect().width);
  expect(openWidth).toBe(384);

  await page.getByRole('tab', { name: 'Geometry' }).click();
  await expect(controls).toBeHidden();
  expect(await controls.evaluate((element) => element.getBoundingClientRect().width)).toBe(0);
  expect(await viewer.evaluate((element) => element.getBoundingClientRect().width)).toBe(
    viewerWidth + openWidth
  );

  await page.getByRole('tab', { name: 'Geometry' }).click();
  await expect(page.getByRole('tab', { name: 'Geometry' })).toHaveAttribute(
    'aria-selected',
    'true'
  );
  expect(await controls.evaluate((element) => element.getBoundingClientRect().width)).toBe(384);
});

test('Geometry pane always exposes crop controls', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Geometry' }).click();
  await expect(page.getByRole('tab', { name: 'Geometry' })).toHaveAttribute(
    'aria-selected',
    'true'
  );
  await expect(page.getByRole('button', { name: 'Angle', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await geometrySlider(page, 'Angle').fill('5');
  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('tab', { name: 'Develop' }).click();
  const request = await saved;
  const body = request.postDataJSON() as {
    manifest: { ops: { transform?: { angle?: number } } };
    action: string | null;
  };
  expect(body.manifest.ops.transform?.angle).toBe(5);
  expect(body.action).toBe('Geometry');
  await expect(page.getByRole('tab', { name: 'Develop' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
});

test('perspective sliders save into the transform op', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Geometry' }).click();
  await geometrySlider(page, 'Vertical').fill('40');
  await geometrySlider(page, 'Aspect').fill('-25');

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await page.getByRole('tab', { name: 'Develop' }).click();
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

  await page.getByRole('tab', { name: 'Geometry' }).click();
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
  await page.getByRole('tab', { name: 'Develop' }).click();
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

  await page.getByRole('tab', { name: 'Develop' }).click();
  await expect(page.getByRole('complementary', { name: 'Editor controls' })).toBeHidden();

  await page.keyboard.press('r');
  await expect(page.getByRole('tab', { name: 'Geometry' })).toHaveAttribute(
    'aria-selected',
    'true'
  );
  await expect(page.getByText('Angle', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('tab', { name: 'Develop' })).toHaveAttribute('aria-selected', 'true');
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
  const reset = page.getByRole('button', { name: 'Reset Lens Corrections' });
  const resetBounds = await reset.boundingBox();
  if (!resetBounds) throw new Error('Lens reset has no bounds');
  expect(resetBounds.width).toBeGreaterThanOrEqual(24);
  expect(resetBounds.height).toBeGreaterThanOrEqual(24);
  await expect(page.getByLabel('Enable Profile Corrections')).toBeChecked();
  await expect(page.getByLabel('Remove Chromatic Aberration')).toBeChecked();

  const deleted = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'DELETE'
  );
  await reset.click();
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

test('a matched profile corrects a raw file until the user says otherwise', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    previewMeta: { is_raw: true },
    editRecord: { ...NEUTRAL_RECORD, manifest: { schema_version: 4, ops: {} } },
    onSave: (body) => saves.push(body)
  });
  await page.route('**/api/assets/*/lens-profile', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        matched: true,
        lens: 'Sony FE 35mm f/1.8',
        focal_length: 35,
        aperture: 1.8,
        edits: {
          k1: -0.1,
          k2: 0,
          k3: 0,
          vk1: -0.3,
          vk2: 0,
          vk3: 0,
          ca_red_scale_x10000: 0,
          ca_blue_scale_x10000: 0
        }
      })
    })
  );
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Lens Corrections' }).click();
  const toggle = page.getByLabel('Enable Profile Corrections');
  await expect(toggle).toBeChecked();
  await expect(page.getByText('· Auto')).toBeVisible();

  await toggle.uncheck();
  await expect
    .poll(() => saves.some((s) => JSON.stringify(s).includes('"profile_enabled":false')))
    .toBe(true);
});
