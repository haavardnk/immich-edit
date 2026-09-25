import { expect, test } from '@playwright/test';
import { gotoAsset, installMocks, makePng, NEUTRAL_RECORD, type PreviewRequest } from './helpers';

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

function lastExposure(requests: PreviewRequest[]): number | null {
  const last = requests.at(-1);
  return last ? (last.edits as { basic: { exposure_ev: number } }).basic.exposure_ev : null;
}

async function pickScope(page: import('@playwright/test').Page, label: string): Promise<void> {
  await page.getByRole('button', { name: 'Scope', exact: true }).click();
  await page.getByRole('option', { name: label, exact: true }).click();
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

test('shift with an arrow key moves a slider by ten steps', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (req) => requests.push(req) });
  await gotoAsset(page);

  const slider = exposureSlider(page);
  await slider.focus();
  await slider.press('ArrowRight');
  await expect(page.getByRole('button', { name: 'Edit Exposure value' })).toHaveText('0.05');

  await slider.press('Shift+ArrowRight');
  await expect(page.getByRole('button', { name: 'Edit Exposure value' })).toHaveText('0.55');

  await expect
    .poll(() =>
      requests.some(
        (request) =>
          (request.edits as { basic: { exposure_ev: number } }).basic.exposure_ev === 0.55
      )
    )
    .toBe(true);
});

test('holding a section header renders that section bypassed', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (req) => requests.push(req) });
  await gotoAsset(page);

  const bypass = page.getByRole('button', { name: 'Bypass Tone' });
  await expect(bypass).toHaveCount(0);

  await exposureSlider(page).fill('1');
  await expect.poll(() => lastExposure(requests)).toBe(1);

  await bypass.hover();
  await page.mouse.down();
  await expect.poll(() => lastExposure(requests)).toBe(0);
  await expect(page.getByRole('button', { name: 'Edit Exposure value' })).toHaveText('1.00');

  await page.mouse.up();
  await expect.poll(() => lastExposure(requests)).toBe(1);
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

test('the scopes panel distinguishes loading from absent data', async ({ page }) => {
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

  await expect(page.getByText('Loading…')).toBeVisible();
  releaseMeta();
  await expect(page.getByText('No data')).toBeVisible();
});

test('scope modes draw from the cached grids and survive a reload', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await pickScope(page, 'Waveform');
  await expect(page.getByRole('img', { name: 'Waveform' })).toBeVisible();

  await pickScope(page, 'Vectorscope');
  await expect(page.getByRole('img', { name: 'Vectorscope' })).toBeVisible();

  await page.reload();
  await expect(page.getByRole('img', { name: 'Vectorscope' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Scope', exact: true })).toHaveText('Vectorscope');
});

test('pinned scopes stay above the scrolling panels across a reload', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await installMocks(page);
  await gotoAsset(page);

  const scopesToggle = page.getByRole('button', { name: 'Scopes', exact: true });
  const pin = page.getByRole('button', { name: 'Pin scopes' });
  await expect(pin).toHaveAttribute('aria-pressed', 'false');
  await pin.click();
  await expect(pin).toHaveAttribute('aria-pressed', 'true');
  await expect(scopesToggle).toHaveCount(0);

  const list = page.getByRole('button', { name: 'Versions', exact: true });
  await list.scrollIntoViewIfNeeded();
  await expect(pin).toBeInViewport();
  const scroller = page.locator('.scrollbar-hidden').filter({ has: list });
  expect(await scroller.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);

  await page.reload();
  await expect(page.getByRole('button', { name: 'Pin scopes' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(scopesToggle).toHaveCount(0);

  await pin.click();
  await expect(scroller.getByRole('button', { name: 'Scopes', exact: true })).toBeVisible();
});

test('pinned scopes stay expanded when the panel was saved collapsed', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:scopes', JSON.stringify({ pinned: true }));
    localStorage.setItem('immich-edit:editorUi', JSON.stringify({ developOpenPanels: [] }));
  });
  await installMocks(page);
  await gotoAsset(page);

  await expect(page.getByRole('button', { name: 'Pin scopes' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'Scope', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Scopes', exact: true })).toHaveCount(0);
});

test('the scope height resizes by drag and keyboard and persists', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const status = page.getByRole('status').filter({ hasText: /No data|Loading/ });
  const height = (): Promise<number> =>
    status.evaluate((element) => Math.round(element.getBoundingClientRect().height));
  await expect.poll(height).toBe(128);

  const handle = page.getByRole('slider', { name: 'Resize scopes' });
  await handle.focus();
  await page.keyboard.press('ArrowDown');
  await expect.poll(height).toBe(136);

  const box = await handle.boundingBox();
  if (!box) throw new Error('resize handle has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 64, { steps: 4 });
  await page.mouse.up();
  await expect.poll(height).toBe(200);

  await page.reload();
  await expect.poll(height).toBe(200);
  await expect(page.getByRole('slider', { name: 'Resize scopes' })).toHaveAttribute(
    'aria-valuenow',
    '200'
  );
});

test('a scope mode re-renders the preview when the cached meta has no scopes', async ({ page }) => {
  const requests: PreviewRequest[] = [];
  await installMocks(page, {
    previewMeta: { has_scopes: false },
    onPreview: (req) => requests.push(req)
  });
  await gotoAsset(page);
  await expect(page.getByText('No data')).toBeVisible();

  await pickScope(page, 'Parade');

  await expect.poll(() => requests.some((req) => req.scopes === true)).toBe(true);
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

  await page.getByRole('button', { name: 'Lens Corrections', exact: true }).click();
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

test('only sliders that drive a preview hint the alt drag', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Detail', exact: true }).click();
  await expect(geometrySlider(page, 'Radius')).toHaveAttribute(
    'title',
    /^(⌥|Alt) \+ drag to preview$/
  );
  await expect(exposureSlider(page)).not.toHaveAttribute('title');
});

test('the clipping overlay toggles from the toolbar at desktop width', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const clip = page.getByRole('button', { name: 'Clipping overlay' });
  await expect(clip).toHaveAttribute('aria-pressed', 'false');
  await clip.click();
  await expect(clip).toHaveAttribute('aria-pressed', 'true');
});

test('the grey background preference survives a reload', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const stage = page.locator('.editor-stage');
  await expect(stage).toHaveCSS('background-color', 'rgb(0, 0, 0)');
  await page.getByRole('button', { name: 'More editor actions' }).click();
  await page.getByRole('button', { name: 'Grey background' }).click();
  await expect(stage).toHaveCSS('background-color', 'rgb(118, 118, 118)');

  await page.reload();
  await expect(page.locator('.editor-stage')).toHaveCSS('background-color', 'rgb(118, 118, 118)');
  await page.getByRole('button', { name: 'More editor actions' }).click();
  await expect(page.getByRole('button', { name: 'Grey background' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});

test('the auto adjust shortcut requests suggested edits', async ({ page }) => {
  await installMocks(page);
  let auto = 0;
  await page.route('**/api/assets/*/edits/auto', async (route) => {
    auto += 1;
    const body = JSON.parse(route.request().postData() ?? '{}');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...body, basic: { ...body.basic, exposure_ev: 0.75 } })
    });
  });
  await gotoAsset(page);

  await page.keyboard.press('ControlOrMeta+u');
  await expect(exposureSlider(page)).toHaveValue('0.75');
  expect(auto).toBe(1);
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
  const resetMask = page.getByRole('button', { name: 'Reset mask adjustments' });
  for (const control of [picker, resetMask]) {
    const bounds = await control.boundingBox();
    if (!bounds) throw new Error('Mask control has no bounds');
    expect(bounds.width).toBeGreaterThanOrEqual(24);
    expect(bounds.height).toBeGreaterThanOrEqual(24);
  }
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

test('mask preview survives a clipping toggle and a held original', async ({ page }) => {
  const previews: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (request) => previews.push(request) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Color range', exact: true }).click();
  await page.getByRole('button', { name: 'Toggle mask preview', exact: true }).click();
  const isMaskWeight = (request: PreviewRequest | undefined): boolean =>
    typeof request?.preview_mode === 'object' &&
    request.preview_mode !== null &&
    'mask_weight' in request.preview_mode;
  await expect.poll(() => isMaskWeight(previews.at(-1))).toBe(true);

  const toggled = previews.length;
  await page.getByRole('button', { name: 'Clipping overlay' }).click();
  await expect.poll(() => previews.length).toBeGreaterThan(toggled);
  expect(isMaskWeight(previews.at(-1))).toBe(true);

  const held = previews.length;
  await page.locator('.editor-stage').hover();
  await page.keyboard.down('\\');
  await expect.poll(() => previews.length).toBeGreaterThan(held);
  await page.keyboard.up('\\');
  await expect.poll(() => previews.length).toBeGreaterThan(held + 1);
  await page.waitForTimeout(500);
  expect(isMaskWeight(previews.at(-1))).toBe(true);
  expect(previews.slice(held).filter((request) => request.lane === 'roi')).toEqual([]);
});

test('color range eyedropper keeps the maskless preview through a clipping toggle', async ({
  page
}) => {
  const previews: PreviewRequest[] = [];
  await installMocks(page, { onPreview: (request) => previews.push(request) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Color range', exact: true }).click();
  await page.getByRole('button', { name: 'Pick color from image' }).click();
  const maskCount = (request: PreviewRequest | undefined): number =>
    ((request?.edits as { masks?: unknown[] } | undefined)?.masks ?? []).length;
  await expect.poll(() => previews.length > 0 && maskCount(previews.at(-1)) === 0).toBe(true);

  const toggled = previews.length;
  await page.getByRole('button', { name: 'Clipping overlay' }).click();
  await expect.poll(() => previews.length).toBeGreaterThan(toggled);
  expect(previews.slice(toggled).map(maskCount)).toEqual(previews.slice(toggled).map(() => 0));
});

test('white balance eyedropper applies the sampled neutral to both sliders', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  const sampled: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    onSave: (body) => saves.push(body),
    onWhiteBalance: (route) => {
      sampled.push((route.request().postDataJSON() as Record<string, unknown>) ?? {});
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ wb_temp: -19, wb_tint: 6 })
      });
    }
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Pick white balance' }).click();
  const surface = page.getByTestId('wb-picker-surface');
  await expect(surface).toBeVisible();
  const box = await surface.boundingBox();
  if (!box) throw new Error('white balance picker has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.up();

  await expect(surface).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Edit Temperature value' })).toHaveText('-19');
  await expect(page.getByRole('button', { name: 'Edit Tint value' })).toHaveText('6');
  expect(sampled).toHaveLength(1);
  expect(sampled[0]?.u).toBeCloseTo(0.5, 1);
  await expect.poll(() => saves.length).toBeGreaterThan(0);
});

test('auto white balance solves without touching exposure', async ({ page }) => {
  await installMocks(page, {
    onWhiteBalance: (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ wb_temp: 12, wb_tint: -4 })
      })
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Auto white balance' }).click();

  await expect(page.getByRole('button', { name: 'Edit Temperature value' })).toHaveText('12');
  await expect(page.getByRole('button', { name: 'Edit Tint value' })).toHaveText('-4');
  await expect(page.getByRole('button', { name: 'Edit Exposure value' })).toHaveText('0.00');
});

test('mask rename commits when its field loses focus', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Linear gradient', exact: true }).click();
  await page.getByRole('button', { name: 'Mask 1' }).dblclick();
  const name = page.getByRole('textbox', { name: 'Mask name' });
  await expect(name).toBeFocused();
  await name.fill('Sky');
  await name.evaluate((input) => input.blur());

  await expect(page.getByRole('button', { name: 'Sky' })).toBeVisible();
  await expect
    .poll(() => {
      const body = saves.at(-1) as {
        manifest?: { ops?: { masks?: { layers?: Array<{ name?: string }> } } };
      };
      return body.manifest?.ops?.masks?.layers?.[0]?.name;
    })
    .toBe('Sky');
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

test('the radial rotation grip saves the angle', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body), previewBody: makePng(60, 40) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();

  const centre = await page.getByRole('button', { name: 'Radial center' }).boundingBox();
  const grip = await page.getByRole('button', { name: 'Radial rotation' }).boundingBox();
  if (!centre || !grip) throw new Error('radial handles have no box');
  const cx = centre.x + centre.width / 2;
  const cy = centre.y + centre.height / 2;
  await page.mouse.move(grip.x + grip.width / 2, grip.y + grip.height / 2);
  await page.mouse.down();
  await page.mouse.move(cx, cy + 80, { steps: 6 });
  await page.mouse.up();

  await expect
    .poll(() => {
      const body = saves.at(-1) as {
        manifest?: {
          ops?: {
            masks?: { layers?: Array<{ components?: Array<{ kind?: { angle?: number } }> }> };
          };
        };
      };
      return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind?.angle ?? 0;
    })
    .toBeCloseTo(90, 0);
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
  for (const label of [
    'Keep a radial round while resizing it',
    'Snap a linear gradient to 45° steps',
    'Move a polygon corner straight across or straight up and down'
  ]) {
    await expect(page.getByText(label, { exact: true })).toBeVisible();
  }
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
  await page.getByRole('tab', { name: 'Masks' }).click();
  const controls = page.locator('aside[aria-label="Editor controls"]');
  const viewer = page.getByRole('application');
  const openWidth = await controls.evaluate((element) => element.getBoundingClientRect().width);
  const viewerWidth = await viewer.evaluate((element) => element.getBoundingClientRect().width);
  expect(openWidth).toBe(384);

  await page.getByRole('tab', { name: 'Masks' }).click();
  await expect(controls).toBeHidden();
  expect(await controls.evaluate((element) => element.getBoundingClientRect().width)).toBe(0);
  expect(await viewer.evaluate((element) => element.getBoundingClientRect().width)).toBe(
    viewerWidth + openWidth
  );

  await page.getByRole('tab', { name: 'Masks' }).click();
  await expect(page.getByRole('tab', { name: 'Masks' })).toHaveAttribute('aria-selected', 'true');
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

async function dragCropCorner(page: import('@playwright/test').Page): Promise<void> {
  const handle = page.getByRole('button', { name: 'resize nw' });
  const box = await handle.boundingBox();
  if (!box) throw new Error('crop handle has no box');
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 40, box.y + box.height / 2 + 30, { steps: 5 });
  await page.mouse.up();
}

test('o cycles the crop guide and the choice survives a reload', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.keyboard.press('r');
  const guide = page.getByTestId('crop-guide');
  await expect(guide).toHaveAttribute('data-mode', 'thirds');
  for (const mode of ['golden', 'diagonal', 'grid', 'off']) {
    await page.keyboard.press('o');
    await expect(guide).toHaveAttribute('data-mode', mode);
  }
  await page.keyboard.press('o');
  await page.keyboard.press('o');
  await expect(guide).toHaveAttribute('data-mode', 'golden');

  await page.reload();
  await page.keyboard.press('r');
  await expect(page.getByTestId('crop-guide')).toHaveAttribute('data-mode', 'golden');
  await expect(page.getByRole('button', { name: 'Crop guide: Golden ratio' })).toBeVisible();
});

async function cropRatio(page: import('@playwright/test').Page): Promise<number> {
  const text = (await page.getByTestId('crop-output-size').textContent()) ?? '';
  const [w, h] = text.replace(' px', '').split(' × ').map(Number);
  return (w ?? 0) / (h ?? 1);
}

test('aspect presets, Shift+X and a custom ratio drive the output size', async ({ page }) => {
  await installMocks(page, { previewBody: makePng(60, 40) });
  await gotoAsset(page);
  await page.keyboard.press('r');
  await expect(page.getByTestId('crop-output-size')).toHaveText('6000 × 4000 px');

  await page.getByRole('button', { name: 'Aspect Ratio' }).click();
  await page.getByRole('option', { name: '5:4', exact: true }).click();
  await expect.poll(() => cropRatio(page)).toBeCloseTo(5 / 4, 2);

  await page.locator('.cursor-move').click();
  await page.keyboard.press('Shift+X');
  await expect.poll(() => cropRatio(page)).toBeCloseTo(4 / 5, 2);
  await expect(page.getByRole('button', { name: 'Aspect Ratio' })).toHaveText(/4:5/);

  await page.getByRole('button', { name: 'Aspect Ratio' }).click();
  await page.getByRole('option', { name: 'Custom…', exact: true }).click();
  const widthBox = await page.getByRole('textbox', { name: 'Width' }).boundingBox();
  const heightBox = await page.getByRole('textbox', { name: 'Height' }).boundingBox();
  const rowEnd = await page
    .getByRole('button', { name: /Switch to (landscape|portrait)/ })
    .boundingBox();
  if (!widthBox || !heightBox || !rowEnd) throw new Error('custom ratio row not laid out');
  expect(Math.abs(widthBox.y - heightBox.y)).toBeLessThan(1);
  expect(heightBox.x + heightBox.width).toBeLessThanOrEqual(rowEnd.x + rowEnd.width + 1);
  await page.getByRole('textbox', { name: 'Width' }).fill('7');
  await page.getByRole('textbox', { name: 'Width' }).press('Tab');
  await page.getByRole('textbox', { name: 'Height' }).fill('3');
  await page.getByRole('textbox', { name: 'Height' }).press('Tab');
  await expect.poll(() => cropRatio(page)).toBeCloseTo(7 / 3, 2);
});

for (const how of ['tool', 'shift'] as const) {
  test(`a line drawn with the straighten ${how} levels the photo`, async ({ page }) => {
    await installMocks(page);
    await gotoAsset(page);
    await page.keyboard.press('r');
    await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();

    const straighten = page.getByRole('button', { name: 'Straighten' });
    if (how === 'tool') {
      await page.keyboard.press('Shift+A');
      await expect(straighten).toHaveAttribute('aria-pressed', 'true');
    }
    const surface =
      how === 'tool' ? page.getByTestId('straighten-surface') : page.locator('.cursor-move');
    const box = await surface.boundingBox();
    if (!box) throw new Error('no straighten surface');
    const x = box.x + box.width * 0.25;
    const y = box.y + box.height * 0.5;
    if (how === 'shift') await page.keyboard.down('Shift');
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x + 200, y + 200 * Math.tan((10 * Math.PI) / 180), { steps: 6 });
    await page.mouse.up();
    if (how === 'shift') await page.keyboard.up('Shift');

    await expect
      .poll(async () => Number(await geometrySlider(page, 'Angle').inputValue()))
      .toBeCloseTo(-10, 0);
    await expect(straighten).toHaveAttribute('aria-pressed', 'false');
  });
}

test('R opens Geometry and Escape drops the crop', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
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
  await dragCropCorner(page);

  await page.keyboard.press('Escape');
  await expect(page.getByRole('tab', { name: 'Develop' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
  await page.waitForTimeout(300);
  expect(saves.filter((body) => JSON.stringify(body).includes('"crop"'))).toEqual([]);
});

for (const [open, apply] of [
  ['r', 'Enter'],
  ['rail', 'Enter'],
  ['rail', 'rail']
] as const) {
  test(`Geometry opened by ${open} applies the crop on ${apply} and returns to Develop`, async ({
    page
  }) => {
    await installMocks(page);
    await gotoAsset(page);
    const geometryTab = page.getByRole('tab', { name: 'Geometry' });

    if (open === 'rail') await geometryTab.click();
    else await page.keyboard.press('r');
    await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();
    await dragCropCorner(page);

    const saved = page.waitForRequest(
      (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
    );
    if (apply === 'rail') await geometryTab.click();
    else await page.keyboard.press('Enter');
    await expect(page.getByRole('tab', { name: 'Develop' })).toHaveAttribute(
      'aria-selected',
      'true'
    );
    await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);
    await expect(geometryTab).not.toBeFocused();
    const body = (await saved).postDataJSON() as {
      manifest: { ops: { transform?: { crop?: { w: number } } } };
    };
    expect(body.manifest.ops.transform?.crop?.w).toBeLessThan(1);
  });
}

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

  await page.getByRole('button', { name: 'Lens Corrections', exact: true }).click();
  const reset = page.getByRole('button', { name: 'Reset Lens Corrections', exact: true });
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

  await page.getByRole('button', { name: 'Lens Corrections', exact: true }).click();
  const toggle = page.getByLabel('Enable Profile Corrections');
  await expect(toggle).toBeChecked();
  await expect(page.getByText('· Auto')).toBeVisible();

  await toggle.uncheck();
  await expect
    .poll(() => saves.some((s) => JSON.stringify(s).includes('"profile_enabled":false')))
    .toBe(true);
});
