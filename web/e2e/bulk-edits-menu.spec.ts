import { expect, test, type Locator, type Page } from '@playwright/test';
import { bulkBar, openBulkEdits, pick } from './browse';
import {
  installMocks,
  jobsPanel,
  numberedAssets,
  openFilters,
  type MockAssetSummary
} from './helpers';

const [SOURCE, SECOND, THIRD] = numberedAssets(3) as [
  MockAssetSummary,
  MockAssetSummary,
  MockAssetSummary
];
if (!SOURCE || !SECOND || !THIRD) throw new Error('missing assets');

async function openPhotos(page: Page): Promise<void> {
  await installMocks(page, {
    assets: [SOURCE, SECOND, THIRD],
    edits: [{ id: SOURCE.id, hash: 'h1', updated_at: '2024-05-01T00:00:00Z' }],
    presets: [{ id: 'preset-warm', name: 'Warm', group_name: null, manifest: { ops: {} } }]
  });
  await page.goto('/photos');
}

function presetFlyout(page: Page): Locator {
  return page
    .locator('[data-popover-content]')
    .filter({ has: page.getByRole('combobox', { name: 'Select a preset…' }) });
}

function rows(menu: Locator): Locator {
  return menu.locator('[data-menu-row]');
}

test('the edits menu lists the bulk edit actions with their reasons and keys', async ({ page }) => {
  await openPhotos(page);
  await pick(page, SECOND.originalFileName);
  await pick(page, THIRD.originalFileName);
  const menu = await openBulkEdits(page);
  await expect(bulkBar(page).getByRole('button', { name: 'Edits', exact: true })).toHaveAttribute(
    'aria-expanded',
    'true'
  );

  await expect(rows(menu)).toHaveText([
    'Apply preset',
    'Paste edits Nothing copied',
    'Paste only… Nothing copied',
    'Copy edits Select one photo',
    'Reset edits'
  ]);
  for (const name of ['Paste edits', 'Paste only…', 'Copy edits', 'Reset edits']) {
    await expect(menu.getByRole('button', { name, exact: true })).toBeDisabled();
  }
  await page.keyboard.press('Escape');
  await expect(menu).toBeHidden();
});

test('copy from one edited photo, then paste from the menu onto a selection', async ({ page }) => {
  await openPhotos(page);
  await pick(page, SOURCE.originalFileName);
  let menu = await openBulkEdits(page);
  await menu.getByRole('button', { name: 'Copy edits', exact: true }).click();
  const copy = page.getByRole('dialog', { name: 'Copy settings' });
  await copy.getByRole('button', { name: 'Copy', exact: true }).click();
  await expect(copy).toBeHidden();

  await bulkBar(page).getByRole('button', { name: 'Clear selection' }).click();
  await expect(bulkBar(page)).toBeHidden();
  await pick(page, SECOND.originalFileName);
  await pick(page, THIRD.originalFileName);
  menu = await openBulkEdits(page);
  const paste = menu.getByRole('button', { name: 'Paste edits', exact: true });
  await expect(paste).toContainText('from IMG_0001');
  await expect(paste).toContainText(/(Ctrl\+|⌘)V/);
  await expect(menu.getByRole('button', { name: 'Copy edits', exact: true })).toContainText(
    'Select one photo'
  );
  const job = page.waitForRequest((r) => r.method() === 'POST' && r.url().endsWith('/api/jobs'));
  await paste.click();
  const sent = (await job).postDataJSON() as { kind: string; asset_ids: string[] };
  expect(sent.kind).toBe('paste_edits');
  expect(sent.asset_ids.sort()).toEqual([SECOND.id, THIRD.id].sort());
  await expect(menu).toBeHidden();
});

test('apply preset opens beside the menu and starting it opens Jobs', async ({ page }) => {
  await openPhotos(page);
  await pick(page, SECOND.originalFileName);
  await pick(page, THIRD.originalFileName);
  const menu = await openBulkEdits(page);
  const apply = menu.getByRole('button', { name: 'Apply preset', exact: true });
  await apply.click();
  const flyout = presetFlyout(page);
  await expect(flyout).toContainText('2 photos');
  await expect(apply).toHaveAttribute('aria-expanded', 'true');
  const side = await flyout.boundingBox();
  const list = await menu.boundingBox();
  expect(side?.x ?? 0).toBeGreaterThanOrEqual((list?.x ?? 0) + (list?.width ?? 0) - 1);

  await page.keyboard.press('Escape');
  await expect(flyout).toBeHidden();
  await expect(menu).toBeVisible();

  await apply.click();
  await flyout.getByRole('combobox', { name: 'Select a preset…' }).click();
  await page.getByRole('option', { name: 'Warm', exact: true }).click();
  await expect(menu).toBeVisible();
  await flyout.getByRole('button', { name: 'Apply Warm to 2' }).click();
  await expect(menu).toBeHidden();
  await expect(flyout).toBeHidden();
  await expect(jobsPanel(page)).toBeVisible();
});

test('a tight bar opens the same edits menu from More actions', async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 900 });
  await openPhotos(page);
  await openFilters(page);
  await pick(page, SOURCE.originalFileName);
  await pick(page, THIRD.originalFileName);
  const bar = bulkBar(page);
  await expect(bar.getByRole('button', { name: 'Edits', exact: true })).toBeHidden();
  await bar.getByRole('button', { name: 'More actions' }).click();
  const more = page.locator('[data-popover-content]').first();
  await more.getByRole('button', { name: 'Edits', exact: true }).click();
  const menu = page.getByRole('group', { name: 'Edits', exact: true });
  await expect(rows(menu)).toHaveCount(5);

  const job = page.waitForRequest((r) => r.method() === 'POST' && r.url().endsWith('/api/jobs'));
  await menu.getByRole('button', { name: 'Reset edits' }).click();
  expect(((await job).postDataJSON() as { kind: string }).kind).toBe('reset_edits');
  await expect(menu).toBeHidden();
  await expect(page.locator('[data-popover-content]')).toHaveCount(0);
});
