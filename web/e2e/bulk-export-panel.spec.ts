import { expect, test, type Page } from '@playwright/test';
import { bulkBar, openBulkExport, pick } from './browse';
import { installMocks, jobsPanel, numberedAssets, openFilters } from './helpers';

const ASSETS = numberedAssets(4);

async function selectTwo(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/photos');
  await pick(page, 'IMG_0001.ARW');
  await pick(page, 'IMG_0002.ARW');
  await expect(bulkBar(page).getByText('2 selected')).toBeVisible();
}

function exportButton(page: Page) {
  return bulkBar(page).getByRole('button', { name: 'Export selected photos' });
}

test('export docks beside the grid and follows the selection', async ({ page }) => {
  await selectTwo(page);
  const button = exportButton(page);
  await expect(button).toHaveAttribute('aria-expanded', 'false');

  const panel = await openBulkExport(page);
  await expect(button).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(panel.getByText('2 photos', { exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Download 2 as JPEG ZIP' })).toBeVisible();
  await expect.poll(async () => Math.round((await panel.boundingBox())?.width ?? 0)).toBe(384);
  await expect
    .poll(async () => {
      const grid = await page.getByTestId('grid-column').boundingBox();
      const side = await panel.boundingBox();
      return (grid?.x ?? 0) + (grid?.width ?? 0) <= (side?.x ?? 0) + 0.5;
    })
    .toBe(true);

  await pick(page, 'IMG_0003.ARW');
  await expect(panel.getByText('3 photos', { exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Download 3 as JPEG ZIP' })).toBeVisible();

  await bulkBar(page).getByRole('button', { name: 'Clear selection' }).click();
  await expect(panel).toBeHidden();
});

test('escape and the X close the panel and return focus to the bar', async ({ page }) => {
  await selectTwo(page);
  const panel = await openBulkExport(page);
  await panel.getByRole('combobox', { name: 'Format' }).focus();
  await page.keyboard.press('Escape');
  await expect(panel).toBeHidden();
  await expect(exportButton(page)).toBeFocused();
  await expect(bulkBar(page).getByText('2 selected')).toBeVisible();

  await openBulkExport(page);
  await panel.getByRole('button', { name: 'Close export' }).click();
  await expect(panel).toBeHidden();
  await expect(exportButton(page)).toBeFocused();
});

test('shift+E toggles the panel from the grid', async ({ page }) => {
  await selectTwo(page);
  const panel = page.getByRole('complementary', { name: 'Export', exact: true });
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press('Shift+E');
  await expect(panel).toBeVisible();
  await page.keyboard.press('Shift+E');
  await expect(panel).toBeHidden();
});

test('one side panel at a time: export, filters and jobs replace each other', async ({ page }) => {
  await selectTwo(page);
  const filters = await openFilters(page);
  const panel = await openBulkExport(page);
  await expect(filters).toBeHidden();

  await page.getByRole('button', { name: 'Jobs', exact: true }).click();
  await expect(jobsPanel(page)).toBeVisible();
  await expect(panel).toBeHidden();
  await expect(exportButton(page)).toHaveAttribute('aria-expanded', 'false');

  await openBulkExport(page);
  await expect(jobsPanel(page)).toBeHidden();

  await openFilters(page);
  await expect(panel).toBeHidden();
});

test('the bar keeps export and folds the organise tools beside the panel', async ({ page }) => {
  for (const width of [1440, 1024]) {
    await page.setViewportSize({ width, height: 900 });
    await selectTwo(page);
    await openBulkExport(page);
    const bar = bulkBar(page);
    await expect(bar.getByRole('button', { name: 'More actions' })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Edits', exact: true })).toBeHidden();
    await expect(exportButton(page)).toBeVisible();
    const box = await bar.boundingBox();
    for (const button of await bar.getByRole('button').all()) {
      if (!(await button.isVisible())) continue;
      const r = await button.boundingBox();
      expect((r?.x ?? 0) + (r?.width ?? 0)).toBeLessThanOrEqual((box?.x ?? 0) + (box?.width ?? 0));
      expect(r?.y ?? 0).toBeGreaterThanOrEqual((box?.y ?? 0) - 0.5);
    }
    const rows = new Set(
      await Promise.all(
        ['Compare selected', 'Colour label', 'More actions'].map(
          async (name) => (await bar.getByRole('button', { name }).boundingBox())?.y
        )
      )
    );
    expect(rows.size).toBe(1);
  }
});

test('a narrow window in the desktop layout lays the panel over the grid', async ({ page }) => {
  await page.setViewportSize({ width: 760, height: 900 });
  await page.addInitScript(() =>
    localStorage.setItem('immich-edit:layout', JSON.stringify({ choice: 'desktop' }))
  );
  await selectTwo(page);
  const panel = await openBulkExport(page);
  await expect(panel.locator('xpath=..')).toHaveCSS('position', 'fixed');
  await expect
    .poll(async () => {
      const box = await panel.boundingBox();
      return Math.round((box?.x ?? 0) + (box?.width ?? 0));
    })
    .toBe(760);
});

test('leaving the grid closes the panel', async ({ page }) => {
  await selectTwo(page);
  const panel = await openBulkExport(page);
  await page.getByRole('link', { name: /^Favorites/ }).click();
  await expect(page).toHaveURL(/\/favorites/);
  await expect(panel).toBeHidden();
});
