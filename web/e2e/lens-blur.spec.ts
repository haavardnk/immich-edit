import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, json, type PreviewRequest } from './helpers';

const DEPTH_MODEL = { id: 'depth-anything', kind: 'depth', installed: true };

async function routeModels(page: Page, models: unknown[]): Promise<void> {
  await page.route('**/api/masks/models', (route) =>
    route.fulfill(json({ runtime: 'cpu', enabled: true, models, active: {}, semantic_classes: [] }))
  );
}

async function routeDepth(
  page: Page,
  respond: () => Parameters<typeof json>[0] | null
): Promise<void> {
  await page.route('**/api/assets/*/depth', (route) => {
    const body = respond();
    if (body === null) {
      return route.fulfill(
        json({ code: 'bad_request', message: 'no model installed for depth' }, 400)
      );
    }
    return route.fulfill(json(body));
  });
}

function depthMap(id: string): Record<string, unknown> {
  return { raster_id: id };
}

function lensBlurOf(saves: Array<Record<string, unknown>>): Record<string, unknown> | undefined {
  const last = saves.at(-1) as { manifest?: { ops?: Record<string, unknown> } } | undefined;
  return last?.manifest?.ops?.lens_blur as Record<string, unknown> | undefined;
}

async function openLensBlur(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Lens Blur', exact: true }).click();
}

test('generating a depth map turns the blur on and every control saves and previews', async ({
  page
}) => {
  const saves: Array<Record<string, unknown>> = [];
  const previews: PreviewRequest[] = [];
  let requests = 0;
  await installMocks(page, {
    onSave: (body) => saves.push(body),
    onPreview: (req) => previews.push(req)
  });
  await routeModels(page, [DEPTH_MODEL]);
  await routeDepth(page, () => depthMap(`depth-${++requests}`));
  await gotoAsset(page);
  await openLensBlur(page);

  await expect(page.getByRole('status').filter({ hasText: 'No depth map' })).toBeVisible();
  await expect(page.getByRole('slider', { name: 'Boost' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Generate depth' }).click();

  await expect(page.getByRole('status').filter({ hasText: 'Depth map ready' })).toBeVisible();
  await expect
    .poll(() => lensBlurOf(saves))
    .toMatchObject({ amount: 50, depth_raster_id: 'depth-1' });
  await expect
    .poll(() => (previews.at(-1)?.edits.lens_blur as { amount?: number } | undefined)?.amount)
    .toBe(50);

  await page.getByRole('radio', { name: 'Hexagon' }).click();
  await expect.poll(() => lensBlurOf(saves)?.shape).toBe('hexagon');

  const far = page.getByRole('slider', { name: 'Far focus limit' });
  await expect(far).toHaveAttribute('aria-valuetext', '30% of the photo is nearer');
  await far.press('ArrowRight');
  await expect.poll(() => lensBlurOf(saves)?.focus_max).toBe(0.31);

  await page.getByRole('slider', { name: 'Boost' }).press('ArrowRight');
  await expect.poll(() => lensBlurOf(saves)?.boost).toBe(1);

  await page.getByRole('button', { name: 'Regenerate depth' }).click();
  await expect.poll(() => lensBlurOf(saves)?.depth_raster_id).toBe('depth-2');
  expect(lensBlurOf(saves)).toMatchObject({ amount: 50, shape: 'hexagon', focus_max: 0.31 });

  await page.getByRole('button', { name: 'Reset Lens Blur' }).first().click();
  await expect
    .poll(() => lensBlurOf(saves))
    .toEqual({
      amount: 0,
      focus_min: 0,
      focus_max: 0.3,
      shape: 'circle',
      boost: 0,
      depth_raster_id: 'depth-2'
    });
  await expect(page.getByRole('status').filter({ hasText: 'Depth map ready' })).toBeVisible();
});

test('a failed depth map explains itself and retries', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  let attempts = 0;
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await routeModels(page, [DEPTH_MODEL]);
  await routeDepth(page, () => (++attempts === 1 ? null : depthMap('depth-1')));
  await gotoAsset(page);
  await openLensBlur(page);

  await page.getByRole('button', { name: 'Generate depth' }).click();
  await expect(page.getByText('Depth map failed: no model installed for depth')).toBeVisible();
  await page.getByRole('button', { name: 'Try again' }).click();
  await expect(page.getByText(/^Depth map failed/)).toHaveCount(0);
  await expect.poll(() => lensBlurOf(saves)?.depth_raster_id).toBe('depth-1');
  expect(attempts).toBe(2);
});

test('without a depth model the generate button explains what is missing', async ({ page }) => {
  await installMocks(page);
  await routeModels(page, [{ ...DEPTH_MODEL, installed: false }]);
  await gotoAsset(page);
  await openLensBlur(page);

  await expect(page.getByRole('button', { name: 'Generate depth' })).toBeDisabled();
  await expect(page.getByText('Lens Blur needs a depth model.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Models settings' })).toHaveAttribute(
    'href',
    '/settings#models'
  );
});
