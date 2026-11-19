import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';
import { open } from './touchShell';

const SOURCE = { w: 12000, h: 8000 };

function renderFor(req: PreviewRequest): Buffer {
  const roi = req.roi ?? [0, 0, 1, 1];
  const w = SOURCE.w * roi[2];
  const h = SOURCE.h * roi[3];
  const scale = Math.min(1, req.max_edge / Math.max(w, h));
  return makePng(Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
}

async function boxOf(target: Locator): Promise<{ x: number; y: number; w: number; h: number }> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x, y: box.y, w: box.width, h: box.height };
}

function overlay(page: Page): Locator {
  return page.getByTestId('floating-scope');
}

test.describe('floating scope', () => {
  test.use({ viewport: { width: 1280, height: 900 } });

  test('S shows it, a click switches the plot and a drag snaps it to a corner', async ({
    page
  }) => {
    await installMocks(page, { previewRender: renderFor, sourceSize: SOURCE });
    await gotoAsset(page);
    await expect(page.getByTestId('view-render')).toBeVisible();
    await expect(overlay(page)).toHaveCount(0);

    await page.keyboard.press('s');
    await expect(overlay(page)).toBeVisible();
    await expect(overlay(page)).toHaveAttribute('data-corner', 'tl');
    await expect(overlay(page)).toHaveAccessibleName('Histogram overlay');

    await overlay(page).click();
    await expect(overlay(page)).toHaveAccessibleName('Waveform overlay');
    await overlay(page).click();
    await expect(overlay(page)).toHaveAccessibleName('Histogram overlay');

    const stage = await boxOf(page.locator('.editor-stage'));
    const start = await boxOf(overlay(page));
    await page.mouse.move(start.x + start.w / 2, start.y + start.h / 2);
    await page.mouse.down();
    await page.mouse.move(stage.x + stage.w - 60, stage.y + stage.h - 40, { steps: 8 });
    await page.mouse.up();
    await expect(overlay(page)).toHaveAttribute('data-corner', 'br');
    await expect(overlay(page)).toHaveAccessibleName('Histogram overlay');
    const settled = await boxOf(overlay(page));
    expect(settled.x + settled.w).toBeGreaterThan(stage.x + stage.w - 30);

    await page.keyboard.press('z');
    const navigator = page.getByTestId('navigator');
    await expect(navigator).toBeVisible();
    expect((await boxOf(navigator)).x).toBeLessThan(stage.x + stage.w / 2);

    await page.getByRole('button', { name: 'More editor actions' }).click();
    await page.getByRole('button', { name: 'Scope overlay' }).click();
    await expect(overlay(page)).toHaveCount(0);
    await expect(navigator).toBeVisible();
    expect((await boxOf(navigator)).x).toBeGreaterThan(stage.x + stage.w / 2);
  });
});

test.describe('floating histogram on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the More sheet Histogram switch shows the mini histogram', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`);
    await expect(page.getByTestId('preview-image')).toBeVisible();
    await expect(overlay(page)).toHaveCount(0);

    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await page.getByRole('button', { name: 'Histogram', exact: true }).tap();
    await expect(overlay(page)).toBeVisible();
    await expect(overlay(page)).toHaveAccessibleName('Histogram overlay');
    const box = await boxOf(overlay(page));
    expect(box.w).toBeLessThan(160);
  });
});

test.describe('floating histogram on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the More sheet has no Histogram switch', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`);
    await expect(page.getByTestId('preview-image')).toBeVisible();
    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await expect(page.getByRole('button', { name: 'Grey background' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Histogram', exact: true })).toHaveCount(0);
  });
});
