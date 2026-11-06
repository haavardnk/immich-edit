import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_EXIF, ASSET_SUMMARY, installMocks, numberedAssets, openFilters } from './helpers';
import { drag, longPress } from './touch';
import { open } from './touchShell';

const CAMERA_ASSETS = numberedAssets(3).map((asset, index) => ({
  ...asset,
  exifInfo: { make: index === 1 ? 'FUJIFILM' : 'SONY' }
}));

const SECOND_ID = '00000000-0000-0000-0000-000000000002';
const LOUPE_ASSETS = [
  { ...ASSET_SUMMARY, exifInfo: ASSET_EXIF },
  { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW', exifInfo: ASSET_EXIF }
];

async function openBrowse(page: Page, assets?: typeof CAMERA_ASSETS): Promise<void> {
  await installMocks(page, assets ? { assets } : {});
  await page.goto('/photos');
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
}

async function openLoupe(page: Page): Promise<void> {
  await installMocks(page, { assets: LOUPE_ASSETS });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
  await expect(page.locator(`a[href^="/assets/${ASSET_SUMMARY.id}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
}

async function openShortcuts(page: Page): Promise<Locator> {
  await page.keyboard.press('?');
  const sheet = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
  await expect(sheet).toBeVisible();
  return sheet;
}

async function expectAtBottom(page: Page, sheet: Locator): Promise<void> {
  const box = await sheet.boundingBox();
  const viewport = page.viewportSize();
  expect(box).not.toBeNull();
  expect(viewport).not.toBeNull();
  if (!box || !viewport) return;
  expect(Math.abs(box.y + box.height - viewport.height)).toBeLessThan(2);
  expect(box.width).toBeGreaterThan(viewport.width - 2);
}

async function settledHandle(sheet: Locator): Promise<{ x: number; y: number }> {
  const handle = sheet.locator('[data-sheet-handle]');
  const height = sheet.page().viewportSize()?.height ?? Number.POSITIVE_INFINITY;
  let previous = Number.NaN;
  await expect
    .poll(async () => {
      const box = await handle.boundingBox();
      const y = box?.y ?? Number.NaN;
      const still = Math.abs(y - previous) < 0.5 && y + (box?.height ?? 0) <= height;
      previous = y;
      return still;
    })
    .toBe(true);
  const box = await handle.boundingBox();
  expect(box).not.toBeNull();
  return box ? { x: box.x + box.width / 2, y: box.y + box.height / 2 } : { x: 0, y: 0 };
}

async function dragHandle(page: Page, sheet: Locator, distance: number): Promise<void> {
  const start = await settledHandle(sheet);
  await drag(page, start, { x: start.x, y: start.y + distance });
}

test.describe('touch sheets', { tag: '@touch' }, () => {
  test('a panel popover opens as a bottom sheet with Done', async ({ page }) => {
    await open(page, '/albums');

    await page.getByRole('button', { name: /^Sort albums/ }).tap();
    const sheet = page.getByRole('dialog', { name: 'Sort albums' });
    await expect(sheet).toBeVisible();
    await expect(sheet).toHaveAttribute('data-popover-content');
    await expect(sheet.locator('[data-sheet-handle]')).toBeAttached();
    await expect(sheet.getByRole('button', { name: 'Cancel', exact: true })).toHaveCount(0);
    await expectAtBottom(page, sheet);

    const count = sheet.getByRole('radio', { name: 'Photo count', exact: true });
    await count.tap();
    await expect(count).toBeChecked();

    await sheet.getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(sheet).toBeHidden();
  });

  test('a command menu opens as an action sheet with Cancel', async ({ page }) => {
    await openLoupe(page);

    await page.getByRole('button', { name: 'More loupe actions', exact: true }).tap();
    const sheet = page.getByRole('dialog', { name: 'Menu' });
    await expect(sheet).toBeVisible();
    await expect(sheet).toHaveAttribute('data-popover-content');
    await expect(sheet.locator('[data-sheet-handle]')).toHaveCount(0);
    const firstRow = sheet.locator('[data-menu-row]').first();
    await expect(firstRow).toBeVisible();
    expect((await firstRow.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);

    await sheet.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await expect(sheet).toBeHidden();
  });

  test('long-pressing a filmstrip thumbnail opens its menu as an action sheet', async ({
    page
  }) => {
    await openLoupe(page);

    const thumb = page
      .getByTestId('filmstrip-scroll')
      .getByRole('button', { name: 'IMG_0002.ARW', exact: true });
    await expect(thumb).toBeVisible();
    const box = await thumb.boundingBox();
    expect(box).not.toBeNull();
    if (!box) return;
    await longPress(page, box.x + box.width / 2, box.y + box.height / 2);

    const sheet = page.getByRole('dialog', { name: 'Actions' });
    await expect(sheet).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Open in editor' })).toBeVisible();
    await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

    await sheet.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await expect(sheet).toBeHidden();
  });

  test('a dialog opens as a bottom sheet that Escape closes', async ({ page }) => {
    await openBrowse(page);

    const sheet = await openShortcuts(page);
    await expect(sheet).toHaveAttribute('aria-modal', 'true');
    await expect(sheet.locator('[data-sheet-handle]')).toBeAttached();
    await expectAtBottom(page, sheet);

    await page.keyboard.press('Escape');
    await expect(sheet).toBeHidden();
  });

  test('dragging the handle down dismisses a dialog sheet', async ({ page }) => {
    await openBrowse(page);

    const sheet = await openShortcuts(page);
    await dragHandle(page, sheet, 400);
    await expect(sheet).toBeHidden();
  });

  test('a short drag on the handle settles the sheet back', async ({ page }) => {
    await openBrowse(page);

    const sheet = await openShortcuts(page);
    const handle = sheet.locator('[data-sheet-handle]');
    const start = await settledHandle(sheet);
    await drag(page, start, { x: start.x, y: start.y + 30 }, { steps: 30 });

    await expect(sheet).toBeVisible();
    await expect
      .poll(async () => {
        const box = await handle.boundingBox();
        return box ? box.y + box.height / 2 : 0;
      })
      .toBeCloseTo(start.y, 0);
  });

  test('a searchable select opens a full-height picker with its search field', async ({ page }) => {
    await openBrowse(page, CAMERA_ASSETS);

    await openFilters(page, 'Camera');
    await page.getByRole('button', { name: 'Make', exact: true }).tap();

    const sheet = page.getByRole('dialog', { name: 'Make' });
    await expect(sheet).toBeVisible();
    await expectAtBottom(page, sheet);
    const box = await sheet.boundingBox();
    const viewport = page.viewportSize();
    expect((box?.height ?? 0) / (viewport?.height ?? 1)).toBeGreaterThan(0.85);

    const search = sheet.getByRole('combobox', { name: 'Make' });
    await expect(search).toBeVisible();

    await search.fill('fuji');
    await expect(sheet.getByRole('option')).toHaveText(['FUJIFILM']);
    await sheet.getByRole('option', { name: 'FUJIFILM' }).tap();

    await expect(sheet).toBeHidden();
    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText(['Make: FUJIFILM']);
  });

  test('a multi-select picker keeps its search after a pick', async ({ page }) => {
    await installMocks(page, { structuredSearch: true, assets: CAMERA_ASSETS });
    await page.goto('/photos');
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    await openFilters(page, 'Camera');
    const make = page.getByRole('button', { name: 'Make', exact: true });
    await make.tap();
    const sheet = page.getByRole('dialog', { name: 'Make' });
    const search = sheet.getByRole('combobox', { name: 'Make' });
    await search.fill('son');
    await sheet.getByRole('option', { name: 'SONY' }).tap();

    await expect(search).toHaveValue('son');
    await expect(sheet.getByRole('option')).toHaveText(['SONY']);
    await sheet.getByRole('button', { name: 'close', exact: true }).tap();
    await expect(sheet).toBeHidden();

    await make.tap();
    await expect(search).toHaveValue('');
    await expect(sheet.getByRole('option')).toHaveText(['SONY', 'FUJIFILM']);
  });
});

test.describe('touch sheets on WebKit', { tag: '@touch-smoke' }, () => {
  test('a dialog opens as a sheet and closes from its close button', async ({ page }) => {
    await openBrowse(page);

    const sheet = await openShortcuts(page);
    await expect(sheet.locator('[data-sheet-handle]')).toBeAttached();
    await sheet.getByRole('button', { name: 'close', exact: true }).tap();
    await expect(sheet).toBeHidden();
  });
});
