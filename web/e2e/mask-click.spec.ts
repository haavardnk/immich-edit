import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, json } from './helpers';

type ClickBody = { points: { x: number; y: number; positive: boolean }[] };

async function mockClickModel(page: Page, bodies: ClickBody[]): Promise<void> {
  await page.route('**/api/masks/models', (route) =>
    route.fulfill(
      json({
        runtime: 'cpu',
        enabled: true,
        models: [{ id: 'sam', kind: 'click', installed: true }],
        active: {},
        semantic_classes: []
      })
    )
  );
  await page.route('**/api/assets/*/masks/click', (route) => {
    bodies.push(route.request().postDataJSON() as ClickBody);
    const n = bodies.length;
    return route.fulfill(
      json({
        raster_id: `click-${n}`,
        prob_raster_id: `prob-${n}`,
        width: 60,
        height: 40,
        model_id: 'sam',
        backend: 'cpu',
        elapsed_ms: 1
      })
    );
  });
  await page.route('**/api/rasters/*', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/octet-stream',
      headers: { 'x-raster-width': '60', 'x-raster-height': '40' },
      body: Buffer.alloc(60 * 40, 255)
    })
  );
}

async function clickPhoto(
  page: Page,
  fx: number,
  fy: number,
  opts: { modifiers?: ('Shift' | 'Alt')[] } = {}
): Promise<void> {
  const box = await page.getByTestId('preview-image').boundingBox();
  if (!box) throw new Error('preview has no box');
  await page.mouse.move(box.x + box.width * fx, box.y + box.height * fy);
  for (const key of opts.modifiers ?? []) await page.keyboard.down(key);
  await page.mouse.down();
  await page.mouse.up();
  for (const key of opts.modifiers ?? []) await page.keyboard.up(key);
}

test('click to select shows include and exclude from the first click', async ({ page }) => {
  const bodies: ClickBody[] = [];
  await installMocks(page);
  await mockClickModel(page, bodies);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Click to select', exact: true }).click();

  const include = page.getByRole('button', { name: 'Include', exact: true });
  const exclude = page.getByRole('button', { name: 'Exclude', exact: true });
  const pill = page.locator('[data-photo-pill]');
  await expect(include).toHaveAttribute('aria-pressed', 'true');
  await expect(exclude).toBeDisabled();
  await expect(page.getByText('Click the photo to select an area.')).toBeVisible();
  await expect(pill).toContainText('Click to select');

  await clickPhoto(page, 0.3, 0.4);
  await expect.poll(() => bodies.length).toBe(1);
  expect(bodies[0]?.points.map((p) => p.positive)).toEqual([true]);
  await expect(include).toHaveAttribute('aria-pressed', 'true');
  await expect(exclude).toBeEnabled();
  await expect(pill).toContainText('1 point');

  await exclude.click();
  await expect(exclude).toHaveAttribute('aria-pressed', 'true');
  await clickPhoto(page, 0.6, 0.5);
  await expect.poll(() => bodies.length).toBe(2);
  expect(bodies[1]?.points.map((p) => p.positive)).toEqual([true, false]);
  await expect(page.getByText(/1 included, 1 excluded\.$/)).toBeVisible();

  await page.mouse.move(10, 10);
  await clickPhoto(page, 0.7, 0.6);
  await expect(page.getByTestId('click-cursor-tag')).toHaveText(
    /^Exclude · \S+ or right-click includes$/
  );
  await expect.poll(() => bodies.length).toBe(3);
  await clickPhoto(page, 0.4, 0.7, { modifiers: ['Shift'] });
  await expect.poll(() => bodies.length).toBe(4);
  expect(bodies[3]?.points.map((p) => p.positive)).toEqual([true, false, false, true]);
  await expect(pill).toContainText('4 points');

  await page.keyboard.press('Escape');
  await expect(pill).toHaveCount(0);
  await expect(include).toHaveAttribute('aria-pressed', 'false');
  await expect(page.getByRole('button', { name: 'Done', exact: true })).toHaveCount(0);
  await clickPhoto(page, 0.5, 0.2);
  expect(bodies).toHaveLength(4);
});

test('Done ends click to select and keeps the shape', async ({ page }) => {
  const bodies: ClickBody[] = [];
  await installMocks(page);
  await mockClickModel(page, bodies);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Click to select', exact: true }).click();
  await clickPhoto(page, 0.5, 0.5, { modifiers: ['Shift'] });
  await expect.poll(() => bodies.length).toBe(1);
  expect(bodies[0]?.points.map((p) => p.positive)).toEqual([true]);

  await page.getByRole('button', { name: 'Done', exact: true }).click();
  await expect(page.locator('[data-photo-pill]')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Mask 1', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Include', exact: true })).toHaveAttribute(
    'aria-pressed',
    'false'
  );
});
