import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, makePng, type PreviewRequest } from './helpers';

type SavedLayer = { name?: string; components?: { kind?: { kind?: string } }[] };

function savedLayers(saves: Array<Record<string, unknown>>): SavedLayer[] {
  const body = saves.at(-1) as { manifest?: { ops?: { masks?: { layers?: SavedLayer[] } } } };
  return body?.manifest?.ops?.masks?.layers ?? [];
}

function weightRender(req: PreviewRequest): Buffer {
  return makePng(60, 40, req.lane === 'weight' || req.lane === 'mask_thumb' ? 255 : 90);
}

async function addRadialMask(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).last().click();
  await expect(page.getByRole('button', { name: 'Radial center' })).toBeVisible();
}

async function twoMasks(page: Page, opts: Parameters<typeof installMocks>[1] = {}): Promise<void> {
  await installMocks(page, { previewRender: weightRender, ...opts });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await addRadialMask(page);
  await addRadialMask(page);
  await expect(page.getByTestId('mask-layer-row')).toHaveCount(2);
}

test('mask rows show a weight thumbnail rendered on its own lane', async ({ page }) => {
  const previews: PreviewRequest[] = [];
  await twoMasks(page, { onPreview: (req) => previews.push(req) });

  await expect(page.getByTestId('mask-thumb')).toHaveCount(2);
  const thumbs = (): PreviewRequest[] => previews.filter((req) => req.lane === 'mask_thumb');
  await expect.poll(() => thumbs().length).toBe(2);
  await expect(page.getByTestId('mask-layer-row').first()).toContainText('1 shape');

  await page.getByRole('button', { name: 'Invert', exact: true }).first().click();
  await expect.poll(() => thumbs().length).toBe(3);
});

test('hovering a mask or shape row previews its coverage', async ({ page }) => {
  await twoMasks(page);
  const weight = page.getByTestId('mask-weight');
  const pill = page.locator('[data-photo-pill]');
  await expect(weight).toBeVisible();
  const active = await weight.getAttribute('data-layer');

  await page.getByTestId('mask-layer-row').first().hover();
  await expect(pill).toContainText('Previewing Mask 1');
  await expect(weight).not.toHaveAttribute('data-layer', active ?? '');

  await page.getByTestId('mask-layer-row').nth(1).hover();
  await expect(pill).toHaveCount(0);
  await expect(weight).toHaveAttribute('data-layer', active ?? '');
  await page.getByRole('button', { name: 'Toggle mask preview', exact: true }).nth(1).click();
  await expect(weight).toHaveCount(0);
  await page.getByRole('button', { name: 'Toggle mask preview', exact: true }).nth(1).click();

  await page.getByTestId('mask-shape-row').first().hover();
  await expect(pill).toContainText('Previewing Radial gradient');
  await expect(weight).toHaveAttribute('data-layer', active ?? '');
  await expect(weight).toHaveAttribute('data-component', /.+/);

  await page.mouse.move(4, 4);
  await expect(pill).toHaveCount(0);
  await expect(weight).toHaveAttribute('data-layer', active ?? '');
  await expect(weight).not.toHaveAttribute('data-component', /.+/);
});

test('masks reorder by drag and shapes by the keyboard', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await twoMasks(page, { onSave: (body) => saves.push(body) });
  await expect
    .poll(() => savedLayers(saves).map((layer) => layer.name))
    .toEqual(['Mask 1', 'Mask 2']);

  const handle = await page.getByRole('button', { name: 'Reorder Mask 1' }).boundingBox();
  const target = await page.getByTestId('mask-layer-row').nth(1).boundingBox();
  if (!handle || !target) throw new Error('mask rows have no box');
  await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
  await page.mouse.down();
  await page.mouse.move(handle.x + handle.width / 2, target.y + target.height, { steps: 6 });
  await page.mouse.up();
  await expect
    .poll(() => savedLayers(saves).map((layer) => layer.name))
    .toEqual(['Mask 2', 'Mask 1']);

  await page.getByRole('button', { name: 'Add', exact: true }).click();
  await page.getByRole('button', { name: 'Linear gradient', exact: true }).click();
  await expect(page.getByTestId('mask-shape-row')).toHaveCount(2);
  await page.getByRole('button', { name: 'Reorder Linear gradient' }).press('ArrowUp');
  await expect
    .poll(() => savedLayers(saves)[0]?.components?.map((c) => c.kind?.kind))
    .toEqual(['linear', 'radial']);
  await expect(page.getByTestId('mask-shape-row').first()).toContainText('Base');
});
