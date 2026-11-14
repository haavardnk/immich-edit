import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, installMocks, makePng } from './helpers';

const BINS = Array.from({ length: 256 }, (_, i) => 200 + Math.round(150 * Math.sin(i / 20)));
const HISTOGRAM = { r: BINS, g: BINS, b: BINS, l: BINS };

async function openHistogram(page: Page): Promise<Locator> {
  await installMocks(page, {
    previewRender: () => makePng(48, 32, 120),
    previewMeta: { histogram: HISTOGRAM, linear_histogram: HISTOGRAM }
  });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  const zones = page.getByTestId('histogram-zones');
  await expect(zones).toBeVisible();
  return zones;
}

async function at(zones: Locator, x: number): Promise<{ x: number; y: number }> {
  const box = await zones.boundingBox();
  if (!box) throw new Error('no histogram');
  return { x: box.x + box.width * x, y: box.y + box.height * 0.4 };
}

test.describe('draggable histogram zones', () => {
  test('hovering names the zone and a horizontal drag moves its slider in one step', async ({
    page
  }) => {
    const zones = await openHistogram(page);
    const label = page.getByTestId('histogram-zone');

    const exposure = await at(zones, 0.5);
    await page.mouse.move(exposure.x, exposure.y);
    await expect(label).toHaveText(/Exposure\s+0\.00/);

    const shadows = await at(zones, 0.2);
    await page.mouse.move(shadows.x, shadows.y);
    await expect(label).toHaveText(/Shadows\s+0/);

    const box = await zones.boundingBox();
    if (!box) throw new Error('no histogram');
    await page.mouse.down();
    await page.mouse.move(shadows.x + box.width * 0.25, shadows.y, { steps: 8 });
    await expect(label).toHaveText(/Shadows\s+25/);
    await page.mouse.up();
    await expect(page.getByRole('slider', { name: 'Shadows' })).toHaveValue('25');

    await page.mouse.move(box.x + box.width * 0.2, box.y + box.height + 120);
    await expect(label).toHaveCount(0);
    await page.getByRole('button', { name: /^Undo/ }).click();
    await expect(page.getByRole('slider', { name: 'Shadows' })).toHaveValue('0');
  });
});
