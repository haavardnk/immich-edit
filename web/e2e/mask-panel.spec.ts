import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';

type SavedRadial = { center?: { x: number; y: number }; feather?: number };

function lastRadial(saves: Array<Record<string, unknown>>): SavedRadial | undefined {
  const body = saves.at(-1) as {
    manifest?: {
      ops?: { masks?: { layers?: Array<{ components?: Array<{ kind?: SavedRadial }> }> } };
    };
  };
  return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind;
}

async function addRadial(page: Page): Promise<void> {
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Radial center' })).toBeVisible();
}

function weightRender(req: PreviewRequest): Buffer {
  return makePng(60, 40, req.lane === 'weight' ? 255 : 90);
}

async function paintedAlpha(page: Page, testId: string): Promise<number> {
  return page.getByTestId(testId).evaluate((el) => {
    const canvas = el as HTMLCanvasElement;
    const ctx = canvas.getContext('2d');
    if (!ctx || !canvas.width || !canvas.height) return 0;
    const px = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let sum = 0;
    for (let i = 3; i < px.length; i += 4) sum += px[i] ?? 0;
    return sum;
  });
}

async function dragBy(page: Page, name: string, to: { x: number; y: number }): Promise<void> {
  const box = await page.getByRole('button', { name }).boundingBox();
  if (!box) throw new Error(`${name} has no box`);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 5 });
  await page.mouse.up();
}

test('the radial feather handle stays grabbable at full feather', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);

  const centre = await page.getByRole('button', { name: 'Radial center' }).boundingBox();
  if (!centre) throw new Error('radial centre has no box');
  const middle = { x: centre.x + centre.width / 2, y: centre.y + centre.height / 2 };
  await dragBy(page, 'Radial feather', middle);
  await expect.poll(() => lastRadial(saves)?.feather).toBe(1);

  const feather = page.getByRole('button', { name: 'Radial feather' });
  await expect(feather).toBeVisible();
  const edge = await page.getByRole('button', { name: 'Radial radius x' }).first().boundingBox();
  if (!edge) throw new Error('radial edge has no box');
  await dragBy(page, 'Radial feather', {
    x: (middle.x + edge.x + edge.width / 2) / 2,
    y: middle.y
  });
  await expect.poll(() => lastRadial(saves)?.feather ?? 1).toBeLessThan(0.9);
});

test('Alt and the wheel resize the mask brush with a live cursor ring', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Brush', exact: true }).click();

  const image = await page.getByTestId('preview-image').boundingBox();
  if (!image) throw new Error('preview has no box');
  await page.mouse.move(image.x + image.width / 2, image.y + image.height / 2);
  const ring = page.getByTestId('brush-cursor');
  await expect(ring).toBeVisible();
  const ringWidth = async (): Promise<number> => (await ring.boundingBox())?.width ?? 0;
  const size = page.getByRole('button', { name: 'Edit Size value' });
  const before = await size.textContent();
  const startWidth = await ringWidth();

  await page.keyboard.down('Alt');
  await page.mouse.wheel(0, -300);
  await page.keyboard.up('Alt');

  await expect.poll(ringWidth).toBeGreaterThan(startWidth);
  await expect(size).not.toHaveText(before ?? '');
  await expect(page.getByRole('button', { name: 'Zoom', exact: true })).toHaveText('Fit');
});

test('arrow keys nudge the selected radial', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);
  await expect.poll(() => lastRadial(saves)?.center?.x).toBe(0.5);

  await page.keyboard.press('Shift+ArrowRight');
  await page.keyboard.press('ArrowUp');
  await expect.poll(() => lastRadial(saves)?.center?.x ?? 0).toBeGreaterThan(0.5);
  await expect.poll(() => lastRadial(saves)?.center?.y ?? 1).toBeLessThan(0.5);
  await expect(page).toHaveURL(/\/assets\/[^/?]+/);
});

test('the refine section keeps its state across a tab switch', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await addRadial(page);

  const refine = page.getByRole('button', { name: /^Refine/ });
  const reselect = async (): Promise<void> => {
    await page.keyboard.press('d');
    await page.keyboard.press('m');
    await page.getByRole('button', { name: 'Mask 1', exact: true }).click();
  };
  await reselect();
  await expect(refine).toHaveAttribute('aria-expanded', 'false');
  await refine.click();
  await expect(refine).toHaveAttribute('aria-expanded', 'true');

  await reselect();
  await expect(refine).toHaveAttribute('aria-expanded', 'true');
});

test('mask shortcuts are shown where the actions live', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();

  await page.getByRole('button', { name: 'Toggle mask overlays' }).hover();
  await expect(page.getByText('Hide mask overlays (O)')).toBeVisible();
  await page.getByRole('button', { name: 'New mask' }).click();
  await expect(page.getByRole('button', { name: 'Linear gradient', exact: true })).toContainText(
    /Shift\+L|⇧L/
  );
  await expect(page.getByRole('button', { name: 'Radial gradient', exact: true })).toContainText(
    /Shift\+M|⇧M/
  );
  await expect(page.getByRole('button', { name: 'Brush', exact: true })).toContainText('K');
  await page.keyboard.press('Escape');

  await page.keyboard.press('Shift+L');
  await expect(page.getByRole('button', { name: 'Linear start' })).toBeVisible();

  await page.keyboard.press('Shift+/');
  await expect(page.getByText('Add a polygon corner', { exact: true })).toBeVisible();
  await expect(page.getByText('Remove a polygon corner', { exact: true })).toBeVisible();
});

test('the mask row toggles a weight overlay that hides while adjusting', async ({ page }) => {
  const previews: PreviewRequest[] = [];
  await installMocks(page, {
    onPreview: (req) => previews.push(req),
    previewRender: weightRender
  });
  await gotoAsset(page);
  await addRadial(page);

  const weight = page.getByTestId('mask-weight');
  const row = page.getByRole('button', { name: 'Toggle mask preview', exact: true });
  await expect(row).toHaveAttribute('aria-pressed', 'true');
  await expect.poll(() => paintedAlpha(page, 'mask-weight')).toBeGreaterThan(0);
  const weights = (): PreviewRequest[] => previews.filter((req) => req.lane === 'weight');
  expect(weights().every((req) => typeof req.preview_mode === 'object')).toBe(true);
  expect(
    previews.filter((req) => req.lane !== 'weight').every((req) => req.preview_mode === 'none')
  ).toBe(true);

  await row.click();
  await expect(weight).toHaveCount(0);
  await expect(row).toHaveAttribute('aria-pressed', 'false');
  await row.click();
  await expect(weight).toBeVisible();

  const rendered = weights().length;
  const slider = await page.getByRole('slider', { name: 'Exposure' }).boundingBox();
  if (!slider) throw new Error('exposure slider has no box');
  await page.mouse.move(slider.x + slider.width / 2, slider.y + slider.height / 2);
  await page.mouse.down();
  await page.mouse.move(slider.x + slider.width * 0.7, slider.y + slider.height / 2, { steps: 4 });
  await expect(weight).toHaveCount(0);
  await page.mouse.up();
  await expect(weight).toBeVisible();
  expect(weights()).toHaveLength(rendered);

  await page.getByRole('button', { name: 'Toggle mask overlays' }).click();
  await expect(weight).toHaveCount(0);
  await expect(row).toHaveAttribute('aria-pressed', 'false');
});

for (const shown of [true, false]) {
  test(`a brush stroke keeps its tint until the weight is current, overlay ${
    shown ? 'shown' : 'hidden'
  }`, async ({ page }) => {
    let rasters = 0;
    let gate: Promise<void> | null = null;
    let open = (): void => {};
    await installMocks(page, { previewRender: weightRender });
    await page.route(
      (url) => url.pathname === '/api/rasters',
      (route) =>
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ raster_id: `raster-${++rasters}`, width: 64, height: 64 })
        })
    );
    await page.route(
      (url) => /^\/api\/assets\/[^/]+\/preview$/.test(url.pathname),
      async (route) => {
        const body = route.request().postDataJSON() as PreviewRequest | null;
        if (body?.lane === 'weight' && gate) await gate;
        await route.fallback();
      }
    );
    await gotoAsset(page);
    await page.getByRole('tab', { name: 'Masks' }).click();
    await page.getByRole('button', { name: 'New mask' }).click();
    await page.getByRole('button', { name: 'Brush', exact: true }).click();
    const weight = page.getByTestId('mask-weight');
    if (shown) await expect(weight).toBeVisible();
    else await page.getByRole('button', { name: 'Toggle mask overlays' }).click();

    const image = await page.getByTestId('preview-image').boundingBox();
    if (!image) throw new Error('preview has no box');
    const y = image.y + image.height / 2;
    await page.mouse.move(image.x + image.width * 0.4, y);
    await page.mouse.down();
    await page.mouse.move(image.x + image.width * 0.6, y, { steps: 5 });
    await expect.poll(() => paintedAlpha(page, 'brush-canvas')).toBeGreaterThan(0);
    await expect(weight).toHaveCount(0);

    gate = new Promise((resolve) => (open = resolve));
    const uploaded = rasters;
    await page.mouse.up();
    await expect.poll(() => rasters).toBeGreaterThan(uploaded);
    if (shown) {
      await page.waitForTimeout(300);
      expect(await paintedAlpha(page, 'brush-canvas')).toBeGreaterThan(0);
      await expect(weight).toHaveCount(0);
    }
    open();
    await expect.poll(() => paintedAlpha(page, 'brush-canvas')).toBe(0);
    await expect(weight).toHaveCount(shown ? 1 : 0);
  });
}
