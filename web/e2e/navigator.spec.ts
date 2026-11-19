import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';
import { tapAt } from './touch';
import { open } from './touchShell';

const SOURCE = { w: 12000, h: 8000 };

function renderFor(req: PreviewRequest): Buffer {
  const roi = req.roi ?? [0, 0, 1, 1];
  const w = SOURCE.w * roi[2];
  const h = SOURCE.h * roi[3];
  const scale = Math.min(1, req.max_edge / Math.max(w, h));
  return makePng(Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
}

function zoomButton(page: Page): Locator {
  return page.getByRole('button', { name: 'Zoom', exact: true });
}

async function boxOf(target: Locator): Promise<{ x: number; y: number; w: number; h: number }> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x, y: box.y, w: box.width, h: box.height };
}

test.describe('navigator', () => {
  test.use({ viewport: { width: 1280, height: 900 } });

  test('shows when zoomed, pans the view by dragging and hides at fit', async ({ page }) => {
    await installMocks(page, { previewRender: renderFor, sourceSize: SOURCE });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    const navigator = page.getByTestId('navigator');
    await expect(navigator).toHaveCount(0);

    await page.keyboard.press('z');
    await expect(zoomButton(page)).toHaveText('100%');
    await expect(navigator).toBeVisible();

    const frame = await boxOf(navigator);
    const box = page.getByTestId('navigator-box');
    const before = await boxOf(box);
    expect(before.w).toBeLessThan(frame.w);
    expect(Math.abs(before.x + before.w / 2 - (frame.x + frame.w / 2))).toBeLessThan(2);

    const image = page.getByTestId('preview-image');
    const imageLeft = (await boxOf(image)).x;
    await page.mouse.move(before.x + before.w / 2, before.y + before.h / 2);
    await page.mouse.down();
    await page.mouse.move(before.x + before.w / 2 - 20, before.y + before.h / 2, { steps: 4 });
    await page.mouse.up();
    await expect.poll(async () => (await boxOf(box)).x).toBeLessThan(before.x - 10);
    await expect.poll(async () => (await boxOf(image)).x).toBeGreaterThan(imageLeft);
    await expect(zoomButton(page)).toHaveText('100%');

    await zoomButton(page).click();
    await page.getByRole('button', { name: /^Fit to screen/ }).click();
    await expect(zoomButton(page)).toHaveText('Fit');
    await expect(navigator).toHaveCount(0);
  });

  test('a press outside the box centres the view there', async ({ page }) => {
    await installMocks(page, { previewRender: renderFor, sourceSize: SOURCE });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await page.keyboard.press('z');
    const navigator = page.getByTestId('navigator');
    await expect(navigator).toBeVisible();

    const frame = await boxOf(navigator);
    await page.mouse.click(frame.x + 4, frame.y + 4);
    const box = page.getByTestId('navigator-box');
    await expect.poll(async () => (await boxOf(box)).x).toBeLessThan(frame.x + 1);
    await expect.poll(async () => (await boxOf(box)).y).toBeLessThan(frame.y + 1);
  });

  test('hides beside the side-by-side compare', async ({ page }) => {
    await installMocks(page, { previewRender: renderFor, sourceSize: SOURCE });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await page.keyboard.press('z');
    await expect(page.getByTestId('navigator')).toBeVisible();
    await page.keyboard.press('y');
    await page.keyboard.press('y');
    await expect(page.getByTestId('navigator')).toHaveCount(0);
  });
});

test.describe('navigator on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('a double tap zooms in and shows the navigator', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`, { previewRender: renderFor, sourceSize: SOURCE });
    await expect(page.getByTestId('view-render')).toBeVisible();
    const stage = await boxOf(page.locator('.editor-stage'));
    const centre = { x: stage.x + stage.w / 2, y: stage.y + stage.h / 2 };
    await tapAt(page, centre.x, centre.y);
    await tapAt(page, centre.x, centre.y);
    await expect(page.getByTestId('navigator')).toBeVisible();
    await tapAt(page, centre.x, centre.y);
    await tapAt(page, centre.x, centre.y);
    await expect(page.getByTestId('navigator')).toHaveCount(0);
  });
});
