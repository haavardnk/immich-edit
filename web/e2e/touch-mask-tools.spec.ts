import { expect, test, type Page } from '@playwright/test';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits } from '../src/lib/types/edits';
import { defaultRadial, makeComponent, makeLayer } from '../src/lib/types/masks';
import { makeStripesPng, NEUTRAL_RECORD } from './helpers';
import { drag, fingersDown, fingersMove, tapAt, touchDown, touchMoveTo, touchUp } from './touch';
import { addFromSheet, openMasks, shapes } from './touchMasks';

type Box = { x: number; y: number; width: number; height: number };

async function boxOf(page: Page, testId: string): Promise<Box> {
  const box = await page.getByTestId(testId).boundingBox();
  if (!box) throw new Error(`${testId} has no box`);
  return box;
}

function at(box: Box, u: number, v: number): { x: number; y: number } {
  return { x: box.x + box.width * u, y: box.y + box.height * v };
}

async function countRasters(page: Page): Promise<() => number> {
  let rasters = 0;
  await page.route(
    (url) => url.pathname === '/api/rasters',
    (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ raster_id: `raster-${++rasters}`, width: 64, height: 64 })
      })
  );
  return () => rasters;
}

function seeded(layers: number, shapesPerLayer: number) {
  const edits = neutralEdits();
  edits.masks = Array.from({ length: layers }, (_, i) => {
    const layer = makeLayer(`Mask ${i + 1}`, defaultRadial());
    for (let n = 1; n < shapesPerLayer; n += 1)
      layer.components.push(makeComponent(defaultRadial()));
    return layer;
  });
  return {
    ...NEUTRAL_RECORD,
    manifest: editsToManifest(edits),
    hash: `hash-${layers}-${shapesPerLayer}`
  };
}

const tool = (page: Page) => page.getByTestId('touch-shape-tool');

test.describe('touch mask tools on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the brush tray has paint and erase with size on the photo edge', async ({ page }) => {
    await openMasks(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Brush');
    await expect(tool(page)).toContainText('Brush');
    await expect(tool(page).getByRole('button', { name: 'Paint', exact: true })).toBeVisible();
    await expect(tool(page).getByRole('slider', { name: 'Hardness' })).toBeVisible();
    await expect(tool(page).getByRole('slider', { name: 'Flow' })).toBeVisible();
    await expect(tool(page).getByRole('slider', { name: 'Size' })).toHaveCount(0);

    const rail = page.getByTestId('brush-size-rail');
    await expect(rail).toBeVisible();
    const before = await rail.textContent();
    const thumb = rail.getByRole('slider', { name: 'Brush size' });
    const box = await thumb.boundingBox();
    if (!box) throw new Error('size thumb has no box');
    const centre = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await touchDown(page, centre);
    for (let step = 1; step <= 10; step += 1) {
      await page.waitForTimeout(16);
      await touchMoveTo(page, { x: centre.x, y: centre.y - 3 * step });
    }
    await expect(page.locator('[data-photo-pill]')).toContainText('Brush');
    await page.waitForTimeout(150);
    await touchUp(page);
    await expect(rail).not.toHaveText(before ?? '');

    await tool(page).getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(rail).toHaveCount(0);
    await expect(shapes(page)).toHaveCount(1);
  });

  test('one finger paints and a second finger within 120 ms drops the stroke', async ({ page }) => {
    await openMasks(page);
    const rasters = await countRasters(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Brush');
    const canvas = await boxOf(page, 'brush-canvas');
    await expect.poll(rasters).toBeGreaterThan(0);
    const added = rasters();

    await touchDown(page, at(canvas, 0.3, 0.5));
    await touchMoveTo(page, at(canvas, 0.4, 0.5));
    await expect(page.getByTestId('touch-magnifier')).toBeVisible();
    await expect(page.locator('[data-photo-pill]')).toContainText('Brush');
    await touchMoveTo(page, at(canvas, 0.6, 0.5));
    await touchUp(page);
    await expect.poll(rasters).toBe(added + 1);
    await expect(page.getByTestId('touch-magnifier')).toHaveCount(0);

    const dab = at(canvas, 0.5, 0.3);
    await tapAt(page, dab.x, dab.y);
    await page.waitForTimeout(400);
    await expect(tool(page)).toBeVisible();
    await expect.poll(rasters).toBe(added + 2);

    const first = at(canvas, 0.4, 0.4);
    const second = at(canvas, 0.6, 0.6);
    await fingersDown(page, [first]);
    await fingersDown(page, [first, second]);
    await fingersMove(page, [
      { x: first.x - 20, y: first.y - 20 },
      { x: second.x + 20, y: second.y + 20 }
    ]);
    await touchUp(page);
    await page.waitForTimeout(400);
    expect(rasters()).toBe(added + 2);
    await expect(tool(page)).toBeVisible();
  });

  test('a radial opens its tool tray with mode, feather and invert', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openMasks(page, { onSave: (body) => saves.push(body) });
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Radial gradient');
    await expect(tool(page)).toContainText('Mask 1 · base shape');
    await expect(tool(page).getByRole('radiogroup', { name: 'Shape mode' })).toHaveCount(0);
    await expect(tool(page).getByRole('slider', { name: 'Feather' })).toBeVisible();
    const centre = page.getByRole('button', { name: 'Radial center' });
    const hit = await centre.boundingBox();
    expect(hit?.width ?? 0).toBeGreaterThanOrEqual(44);

    await tool(page).getByRole('switch', { name: 'Invert' }).tap();
    await expect
      .poll(() => {
        const body = saves.at(-1) as {
          manifest?: { ops?: { masks?: { layers?: { components?: { invert?: boolean }[] }[] } } };
        };
        return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.invert;
      })
      .toBe(true);
    await tool(page).getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(tool(page)).toHaveCount(0);
  });

  test('a polygon is placed by taps, steps back with Undo point and closes with Done', async ({
    page
  }) => {
    await openMasks(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Polygon');
    await expect(tool(page)).toContainText('Polygon');
    const undo = tool(page).getByRole('button', { name: 'Undo point' });
    const done = tool(page).getByRole('button', { name: 'Done', exact: true });
    await expect(undo).toBeDisabled();
    await expect(done).toBeDisabled();

    const photo = await boxOf(page, 'preview-image');
    for (const [u, v] of [
      [0.3, 0.3],
      [0.7, 0.3],
      [0.7, 0.7],
      [0.3, 0.7]
    ] as const) {
      const point = at(photo, u, v);
      await tapAt(page, point.x, point.y);
    }
    await expect(tool(page)).toContainText('4 corners');
    await undo.tap();
    await expect(tool(page)).toContainText('3 corners');
    await done.tap();
    await expect(tool(page)).toContainText('Mask 1 · base shape');
    await expect(page.getByRole('button', { name: 'Close polygon' })).toHaveCount(0);
    await tool(page).getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(shapes(page)).toHaveCount(1);
  });

  test('color range samples where the finger lifts', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openMasks(page, {
      onSave: (body) => saves.push(body),
      previewRender: () =>
        makeStripesPng(300, 200, [
          [200, 40, 40],
          [40, 40, 200]
        ])
    });
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Color range');
    await expect(tool(page)).toHaveCount(0);
    const sample = page.getByRole('button', { name: 'Sample color from the photo' });
    await sample.tap();
    const surface = await boxOf(page, 'range-sample-surface');

    await touchDown(page, at(surface, 0.2, 0.5));
    await touchMoveTo(page, at(surface, 0.8, 0.5));
    await expect(page.getByTestId('touch-magnifier')).toBeVisible();
    await expect(page.locator('[data-photo-pill]')).toContainText('Lift to sample');
    await expect(page.getByTestId('range-swatch-preview')).toBeVisible();
    await touchUp(page);

    await expect
      .poll(() => {
        const body = saves.at(-1) as {
          manifest?: {
            ops?: {
              masks?: { layers?: { components?: { kind?: { sample_rgb?: number[] } }[] }[] };
            };
          };
        };
        return body?.manifest?.ops?.masks?.layers?.[0]?.components?.[0]?.kind?.sample_rgb?.[2] ?? 0;
      })
      .toBeGreaterThan(0.6);
    await expect(sample).toHaveAttribute('aria-pressed', 'false');
  });

  test('the luminance range thumbs have touch targets', async ({ page }) => {
    await openMasks(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Luminance range');
    const thumb = page.getByRole('slider', { name: 'Darkest' });
    await expect(thumb).toBeVisible();
    expect((await thumb.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(22);
  });

  test('all eight masks and all 32 shapes say so under their buttons', async ({ page }) => {
    await openMasks(page, { editRecord: seeded(8, 4) });
    await expect(page.getByTestId('mask-layer-limit')).toHaveText(
      'You have used all 8 masks. Delete one to add another.'
    );
    await page.getByTestId('mask-layer-card').first().tap();
    await expect(page.getByTestId('mask-shape-limit')).toHaveText(
      'You have used all 32 shapes across every mask. Delete one to add another.'
    );
    await expect(page.getByRole('button', { name: 'Add', exact: true })).toBeDisabled();
  });

  test('a mask at eight shapes says it is full', async ({ page }) => {
    await openMasks(page, { editRecord: seeded(1, 8) });
    await page.getByTestId('mask-layer-card').first().tap();
    await expect(page.getByTestId('mask-shape-limit')).toHaveText(
      'This mask is full at 8 shapes. Delete one to add another.'
    );
  });
});

test.describe('touch mask tools on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the inspector keeps the brush size row and no edge rail', async ({ page }) => {
    await openMasks(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Brush');
    await expect(page.getByRole('slider', { name: 'Size' })).toBeVisible();
    await expect(page.getByTestId('brush-size-rail')).toHaveCount(0);
    await expect(tool(page)).toHaveCount(0);
  });

  test('a stroke on the tablet paints with one finger', async ({ page }) => {
    await openMasks(page);
    const rasters = await countRasters(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Brush');
    await expect.poll(rasters).toBeGreaterThan(0);
    const added = rasters();
    const canvas = await boxOf(page, 'brush-canvas');
    await drag(page, at(canvas, 0.3, 0.5), at(canvas, 0.6, 0.5));
    await expect.poll(rasters).toBe(added + 1);
  });
});
