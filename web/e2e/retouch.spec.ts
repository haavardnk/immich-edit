import { expect, test } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, NEUTRAL_RECORD, PNG_64 } from './helpers';

const RETOUCH_RECORD = {
  ...NEUTRAL_RECORD,
  manifest: {
    schema_version: 3,
    ops: {
      retouch: {
        strokes: [
          {
            id: 'spot-1',
            mode: 'clone',
            points: [{ x: 0.4, y: 0.4 }],
            radius: 0.05,
            hardness: 0.5,
            opacity: 1,
            source: { x: 0.7, y: 0.4 },
            enabled: true
          }
        ]
      }
    }
  }
};

test('painting a retouch stroke saves it and lists it in the panel', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { previewBody: PNG_64, onSave: (body) => saves.push(body) });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Retouch' }).click();
  const canvas = page.getByLabel('retouch canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('retouch canvas has no bounding box');

  const saved = page.waitForRequest(
    (request) => request.url().endsWith(`/assets/${ASSET_ID}/edits`) && request.method() === 'PUT'
  );
  await page.keyboard.down('Alt');
  await page.mouse.click(box.x + box.width * 0.7, box.y + box.height * 0.4);
  await page.keyboard.up('Alt');
  await page.mouse.move(box.x + box.width * 0.3, box.y + box.height * 0.4);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height * 0.5, { steps: 4 });
  await page.mouse.up();
  await saved;

  await expect
    .poll(() => {
      const ops = saves.at(-1)?.manifest as { ops?: { retouch?: { strokes?: unknown[] } } };
      return ops?.ops?.retouch?.strokes?.length ?? 0;
    })
    .toBeGreaterThan(0);
  await expect(page.getByRole('button', { name: 'Heal 1', exact: true })).toBeVisible();
});

test('painting without a sampled source does nothing', async ({ page }) => {
  await installMocks(page, { previewBody: PNG_64 });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Retouch' }).click();
  const canvas = page.getByLabel('retouch canvas');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('retouch canvas has no bounding box');

  await page.mouse.move(box.x + box.width * 0.3, box.y + box.height * 0.4);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height * 0.5, { steps: 4 });
  await page.mouse.up();

  await expect(page.getByRole('button', { name: 'Heal 1', exact: true })).toHaveCount(0);
});

test('painting while zoomed in paints instead of panning', async ({ page }) => {
  await installMocks(page, { previewBody: PNG_64 });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Retouch' }).click();
  const zoom = page.getByRole('button', { name: 'Zoom', exact: true });
  await zoom.click();
  await page.getByRole('button', { name: 'Zoom In', exact: true }).click();
  await page.getByRole('button', { name: 'Zoom In', exact: true }).click();
  await expect(zoom).toHaveText('150%');
  await zoom.click();

  const canvas = page.getByLabel('retouch canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('retouch canvas has no bounding box');
  const image = page.getByRole('img', { name: 'IMG_0001.ARW' });
  const before = await image.boundingBox();
  if (!before) throw new Error('preview image has no bounding box');

  await page.keyboard.down('Alt');
  await page.mouse.click(box.x + box.width * 0.7, box.y + box.height * 0.4);
  await page.keyboard.up('Alt');
  await page.mouse.move(box.x + box.width * 0.3, box.y + box.height * 0.4);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.4, box.y + box.height * 0.5, { steps: 4 });
  await page.mouse.up();

  await expect(page.getByRole('button', { name: 'Heal 1', exact: true })).toBeVisible();
  const after = await image.boundingBox();
  if (!after) throw new Error('preview image has no bounding box');
  expect(Math.abs(after.x - before.x)).toBeLessThan(1);
  expect(Math.abs(after.y - before.y)).toBeLessThan(1);
});

test('saved retouch strokes reappear after a reload', async ({ page }) => {
  await installMocks(page, { previewBody: PNG_64, editRecord: RETOUCH_RECORD });
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Retouch' }).click();
  await expect(page.getByRole('button', { name: 'Clone 1', exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Delete retouch stroke', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Clone 1', exact: true })).toHaveCount(0);
});
