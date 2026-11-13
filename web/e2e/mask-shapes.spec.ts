import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';

type SavedKind = {
  kind?: string;
  radius_xy?: { x: number; y: number };
  feather?: number;
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
