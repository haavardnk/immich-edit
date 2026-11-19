import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, makeStripesPng } from './helpers';

type SavedKind = {
  kind?: string;
  radius_xy?: { x: number; y: number };
  feather?: number;
  min?: number;
  max?: number;
  tolerance?: number;
  sample_rgb?: number[];
};

function lastKind(saves: Array<Record<string, unknown>>): SavedKind | undefined {
  const body = saves.at(-1) as {
    manifest?: {
      ops?: { masks?: { layers?: Array<{ components?: Array<{ kind?: SavedKind }> }> } };
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

async function overPhoto(page: Page): Promise<void> {
  const image = await page.getByTestId('preview-image').boundingBox();
  if (!image) throw new Error('preview has no box');
  await page.mouse.move(image.x + image.width / 4, image.y + image.height / 4);
}

test('Alt and the wheel resize the selected shape, with Shift its feather', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);
  await expect.poll(() => lastKind(saves)?.radius_xy?.x).toBeGreaterThan(0);
  const start = lastKind(saves);
  const saved = saves.length;

  await overPhoto(page);
  await page.keyboard.down('Alt');
  await page.mouse.wheel(0, -200);
  await page.keyboard.up('Alt');

  const pill = page.locator('[data-photo-pill]');
  await expect(pill).toContainText('Size');
  await expect
    .poll(() => lastKind(saves)?.radius_xy?.x ?? 0)
    .toBeGreaterThan(start?.radius_xy?.x ?? 1);
  expect(saves.length).toBe(saved + 1);
  expect(lastKind(saves)?.feather).toBe(start?.feather);

  await page.keyboard.down('Alt');
  await page.keyboard.down('Shift');
  await page.mouse.wheel(0, -100);
  await page.keyboard.up('Shift');
  await page.keyboard.up('Alt');

  await expect(pill).toContainText('Feather');
  await expect.poll(() => lastKind(saves)?.feather ?? 0).toBeGreaterThan(start?.feather ?? 1);
  await expect(page.getByRole('button', { name: 'Zoom', exact: true })).toHaveText('Fit');
});

test('a plain wheel over a selected shape leaves it alone', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);
  await addRadial(page);
  await expect.poll(() => lastKind(saves)?.radius_xy?.x).toBeGreaterThan(0);
  const saved = saves.length;

  await overPhoto(page);
  await page.mouse.wheel(0, -200);
  await page.waitForTimeout(600);
  expect(saves.length).toBe(saved);
});

test('the selected shape lists its wheel keys', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await addRadial(page);
  const hints = page.getByTestId('mask-wheel-hints');
  await expect(hints).toContainText('Size');
  await expect(hints).toContainText('Feather');
});

async function strokeAlpha(page: Page, alt: boolean): Promise<number> {
  const image = await page.getByTestId('preview-image').boundingBox();
  if (!image) throw new Error('preview has no box');
  const y = image.y + image.height * 0.5;
  if (alt) await page.keyboard.down('Alt');
  await page.mouse.move(image.x + image.width * 0.3, y);
  await page.mouse.down();
  await page.mouse.move(image.x + image.width * 0.6, y, { steps: 6 });
  const alpha = await page.getByTestId('brush-canvas').evaluate((el) => {
    const canvas = el as HTMLCanvasElement;
    const ctx = canvas.getContext('2d');
    if (!ctx || !canvas.width || !canvas.height) return 0;
    const px = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let sum = 0;
    for (let i = 3; i < px.length; i += 4) sum += px[i] ?? 0;
    return sum;
  });
  await page.mouse.up();
  if (alt) await page.keyboard.up('Alt');
  return alpha;
}

test('holding Alt erases with the mask brush', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Brush', exact: true }).click();
  await overPhoto(page);

  const erase = page.getByRole('button', { name: 'Erase', exact: true });
  const cursor = page.getByTestId('brush-cursor');
  await page.keyboard.down('Alt');
  await expect(erase).toHaveAttribute('data-held', 'true');
  await expect(cursor).toHaveAttribute('data-erase', 'true');
  await expect(page.locator('[data-photo-pill]')).toContainText('Erase');
  await page.keyboard.up('Alt');
  await expect(erase).not.toHaveAttribute('data-held');
  await expect(cursor).not.toHaveAttribute('data-erase');
  await expect(page.getByRole('button', { name: 'Paint', exact: true })).toBeVisible();

  expect(await strokeAlpha(page, true)).toBe(0);
  expect(await strokeAlpha(page, false)).toBeGreaterThan(0);
});

const STRIPES = makeStripesPng(300, 200, [
  [20, 20, 20],
  [230, 230, 230]
]);

async function addRange(page: Page, tool: 'Luminance range' | 'Color range'): Promise<void> {
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: tool, exact: true }).click();
}

async function photoAt(page: Page, u: number, v: number): Promise<{ x: number; y: number }> {
  const box = await page.getByTestId('range-sample-surface').boundingBox();
  if (!box) throw new Error('sample surface has no box');
  return { x: box.x + box.width * u, y: box.y + box.height * v };
}

test('a drag with Sample armed sets the luminance range from the area', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body), previewRender: () => STRIPES });
  await gotoAsset(page);
  await addRange(page, 'Luminance range');
  await expect(page.getByTestId('luma-range')).toBeVisible();
  await expect(page.getByRole('slider', { name: 'Darkest' })).toBeVisible();
  await expect(page.getByRole('slider', { name: 'Min' })).toHaveCount(0);

  const sample = page.getByRole('button', { name: 'Sample luminance from the photo' });
  await sample.click();
  await expect(sample).toHaveAttribute('aria-pressed', 'true');
  const from = await photoAt(page, 0.3, 0.3);
  const to = await photoAt(page, 0.7, 0.7);
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 5 });
  await expect(page.getByTestId('range-sample-box')).toBeVisible();
  await page.mouse.up();

  await expect.poll(() => lastKind(saves)?.max ?? 0).toBeGreaterThan(0.85);
  expect(lastKind(saves)?.min ?? 1).toBeLessThan(0.1);
  await expect(page.locator('[data-photo-pill]')).toContainText('1 sample');
  await expect(sample).toHaveAttribute('aria-pressed', 'true');
});

test('Shift-click widens a sampled luminance range and Clear starts over', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body), previewRender: () => STRIPES });
  await gotoAsset(page);
  await addRange(page, 'Luminance range');
  await page.getByRole('button', { name: 'Sample luminance from the photo' }).click();

  const dark = await photoAt(page, 0.25, 0.5);
  await page.mouse.click(dark.x, dark.y);
  await expect.poll(() => lastKind(saves)?.max ?? 1).toBeLessThan(0.1);
  expect(lastKind(saves)?.min).toBe(0.07);

  const light = await photoAt(page, 0.75, 0.5);
  await page.keyboard.down('Shift');
  await page.mouse.click(light.x, light.y);
  await page.keyboard.up('Shift');
  await expect.poll(() => lastKind(saves)?.max ?? 0).toBeGreaterThan(0.85);
  expect(lastKind(saves)?.min).toBe(0.07);
  await expect(
    page.getByRole('tabpanel', { name: 'Masks' }).getByText('2 samples', { exact: true })
  ).toBeVisible();

  await page.getByRole('button', { name: 'Clear', exact: true }).click();
  await expect(
    page.getByRole('tabpanel', { name: 'Masks' }).getByText('2 samples', { exact: true })
  ).toHaveCount(0);
  await page.mouse.click(light.x, light.y);
  await expect.poll(() => lastKind(saves)?.min ?? 0).toBeGreaterThan(0.85);
});

test('Shift-click widens a color range tolerance to reach every sample', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body), previewRender: () => STRIPES });
  await gotoAsset(page);
  await addRange(page, 'Color range');
  await page.getByRole('button', { name: 'Sample color from the photo' }).click();

  const dark = await photoAt(page, 0.25, 0.5);
  await page.mouse.click(dark.x, dark.y);
  await expect.poll(() => lastKind(saves)?.sample_rgb?.[0] ?? 1).toBeLessThan(0.1);
  const tolerance = lastKind(saves)?.tolerance ?? 0;

  const light = await photoAt(page, 0.75, 0.5);
  await page.keyboard.down('Shift');
  await page.mouse.click(light.x, light.y);
  await page.keyboard.up('Shift');
  await expect.poll(() => lastKind(saves)?.tolerance ?? 0).toBeGreaterThan(tolerance);
  expect(lastKind(saves)?.sample_rgb?.[0] ?? 0).toBeGreaterThan(0.1);
});
