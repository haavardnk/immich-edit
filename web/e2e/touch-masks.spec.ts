import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, json, type InstallOpts } from './helpers';
import { mockLibrary } from './touchShell';

const MODELS = [
  { id: 'ormbg', kind: 'subject', installed: true },
  { id: 'skyseg', kind: 'sky', installed: true },
  { id: 'depth', kind: 'depth', installed: false },
  { id: 'sam', kind: 'click', installed: true }
];

async function openMasks(page: Page, opts: InstallOpts = {}): Promise<void> {
  await mockLibrary(page, opts);
  await page.route('**/api/masks/models', (route) =>
    route.fulfill(
      json({ runtime: 'cpu', enabled: true, models: MODELS, active: {}, semantic_classes: [] })
    )
  );
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await modes(page).getByRole('tab', { name: 'Masks' }).tap();
  await expect(page.getByRole('button', { name: 'New mask' })).toBeVisible();
}

function modes(page: Page): Locator {
  return page.getByRole('tablist', { name: 'Editor modes' });
}

function sheet(page: Page): Locator {
  return page.getByRole('dialog');
}

function cards(page: Page): Locator {
  return page.getByTestId('mask-layer-card');
}

function shapes(page: Page): Locator {
  return page.getByTestId('mask-shape-row');
}

async function addFromSheet(page: Page, trigger: Locator, tool: string): Promise<void> {
  await trigger.tap();
  await sheet(page).getByRole('button', { name: tool, exact: true }).tap();
  await expect(sheet(page)).toHaveCount(0);
}

type SavedLayer = { name?: string; invert?: boolean; components?: { invert?: boolean }[] };

function savedLayers(saves: Array<Record<string, unknown>>): SavedLayer[] {
  const body = saves.at(-1) as { manifest?: { ops?: { masks?: { layers?: SavedLayer[] } } } };
  return body?.manifest?.ops?.masks?.layers ?? [];
}

test.describe('touch mask layers on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the add sheet lists AI tools then shapes in the desktop order', async ({ page }) => {
    await openMasks(page);
    await page.getByRole('button', { name: 'New mask' }).tap();
    const dialog = sheet(page);
    await expect(dialog.getByText('Pick a tool to start a mask.')).toBeVisible();
    const tools = dialog.locator('[data-menu-row]');
    const names = await tools.evaluateAll((rows) =>
      rows.map((row) => row.getAttribute('aria-label'))
    );
    expect(names).toEqual([
      'Subject',
      'Background',
      'Sky',
      'Depth',
      'Click to select',
      'Box select',
      'Linear gradient',
      'Radial gradient',
      'Brush',
      'Polygon',
      'Luminance range',
      'Color range'
    ]);
    await expect(dialog.getByRole('button', { name: 'Depth' })).toContainText(
      'Needs a model download'
    );
    await dialog.getByRole('button', { name: 'Cancel' }).tap();
    await expect(dialog).toHaveCount(0);
  });

  test('layers, shapes and adjustments are edited from the tray', async ({ page }) => {
    const saves: Array<Record<string, unknown>> = [];
    await openMasks(page, { onSave: (body) => saves.push(body) });

    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Radial gradient');
    await expect(cards(page)).toHaveCount(1);
    await expect(cards(page).first()).toContainText('Mask 1');
    await expect(cards(page).first()).toContainText('1 shape');
    await expect(shapes(page).first()).toContainText('Base');
    await expect(page.getByText('1 of 8')).toBeVisible();

    await page.getByRole('button', { name: 'Subtract', exact: true }).tap();
    await expect(page.getByRole('dialog', { name: 'Subtract from “Mask 1”' })).toBeVisible();
    await sheet(page).getByRole('button', { name: 'Linear gradient', exact: true }).tap();
    await expect(shapes(page)).toHaveCount(2);
    await expect(shapes(page).nth(1)).toContainText('Subtract');

    await page.getByRole('button', { name: 'Linear gradient actions' }).tap();
    await sheet(page).getByRole('button', { name: 'Invert', exact: true }).tap();
    await expect(shapes(page).nth(1)).toContainText('Inverted');
    await expect.poll(() => savedLayers(saves)[0]?.components?.[1]?.invert).toBe(true);

    await page.getByRole('tab', { name: 'Adjust' }).tap();
    await expect(page.getByRole('slider', { name: 'Amount' })).toBeVisible();
    await page.getByRole('tab', { name: 'Shapes' }).tap();

    await page.getByRole('button', { name: 'Mask 1 actions' }).tap();
    await sheet(page).getByRole('button', { name: 'Duplicate', exact: true }).tap();
    await expect(cards(page)).toHaveCount(2);
    await page.getByRole('button', { name: 'Mask 1 actions' }).tap();
    await sheet(page).getByRole('button', { name: 'Delete', exact: true }).tap();
    await expect(cards(page)).toHaveCount(1);
    await expect.poll(() => savedLayers(saves).length).toBe(1);
  });

  test('masks keep the tray height of the other modes', async ({ page }) => {
    await openMasks(page);
    const tray = page.getByRole('region', { name: 'Adjustments' });
    await expect(tray).not.toHaveAttribute('data-snapping');
    const masks = (await tray.boundingBox())?.height;
    const detent = await tray.getAttribute('data-detent');
    await modes(page).getByRole('tab', { name: 'Develop' }).tap();
    await expect(tray).toHaveAttribute('data-detent', detent ?? '');
    await expect(tray).not.toHaveAttribute('data-snapping');
    expect((await tray.boundingBox())?.height).toBe(masks);
  });
});

test.describe('touch mask layers on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the inspector stacks layers, shapes and adjustments in one scroll', async ({ page }) => {
    await openMasks(page);
    await addFromSheet(page, page.getByRole('button', { name: 'New mask' }), 'Radial gradient');
    await expect(shapes(page)).toHaveCount(1);
    await expect(page.getByRole('tablist', { name: 'Mask views' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Intersect', exact: true })).toBeVisible();
    await expect(page.getByRole('slider', { name: 'Amount' })).toBeAttached();
  });
});
