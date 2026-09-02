import { expect, test } from '@playwright/test';
import { ASSET_ID, NEUTRAL_RECORD, installMocks, json, gotoAsset } from './helpers';
import { neutralEdits } from '../src/lib/types/edits';

test('editor toolbar shows the filename without a duplicate extension', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const toolbar = page.getByRole('navigation', { name: 'Editor toolbar' });
  await expect(toolbar.getByText('IMG_0001.ARW', { exact: true })).toBeVisible();
  const toolbarText = await toolbar.innerText();
  expect(toolbarText.match(/\bARW\b/g)).toHaveLength(1);
});

test('history popover expands details and restores a prior entry', async ({ page }) => {
  const latestEdits = neutralEdits();
  latestEdits.basic.exposure_ev = 1.25;
  latestEdits.geometry.rotate = 90;
  const entries = [
    {
      id: 2,
      manifest_hash: 'hash-neutral',
      deleted: false,
      edits: latestEdits,
      created_at: '2024-01-02T00:00:00Z',
      action: 'Latest'
    },
    {
      id: 1,
      manifest_hash: 'hash-prior',
      deleted: false,
      edits: neutralEdits(),
      created_at: '2024-01-01T00:00:00Z',
      action: 'Initial'
    }
  ];

  let restoreCalled = false;
  await installMocks(page, {
    onHistory: (route) => route.fulfill(json(entries)),
    onRestore: async (route) => {
      restoreCalled = true;
      const body = JSON.parse(route.request().postData() ?? '{}');
      expect(body.entry_id).toBe(1);
      await route.fulfill(json({ ...NEUTRAL_RECORD, hash: 'hash-restored' }));
    }
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Edit history' }).click();
  await page.getByRole('button', { name: 'Expand changes for Latest' }).click();
  await expect(page.getByText('0.00 → +1.25')).toBeVisible();
  await expect(page.getByText('Rotation: 0° → 90°')).toBeVisible();
  expect(restoreCalled).toBe(false);
  await expect(page.getByText('Initial')).toBeVisible();
  await page.getByText('Initial').click();

  await expect.poll(() => restoreCalled).toBe(true);
});

test('export download triggers a file download', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/IMG_0001.*\.jpg$/);
});

test('export quality uses the shared range control', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  const quality = page.getByRole('slider', { name: 'Quality' });
  await quality.fill('82');

  await expect(quality).toHaveValue('82');
  await expect(page.getByText('82', { exact: true })).toBeVisible();
});

test('split view toggle reveals before/after slider', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const splitButton = page.getByRole('button', { name: /^Before \/ after split/ });
  await splitButton.click();

  await expect(page.getByRole('slider', { name: 'Before/after split' })).toBeVisible();
});

test('editor and loupe keep independent filmstrip visibility', async ({ page }) => {
  await installMocks(page);
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: 'Collapse filmstrip' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await page.getByRole('button', { name: /^Edit/ }).click();

  await page.getByRole('button', { name: 'Hide filmstrip' }).click();
  await page.getByRole('button', { name: /^Back/ }).click();

  await expect(page.getByRole('button', { name: 'Collapse filmstrip' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'IMG_0001.ARW' })).toBeVisible();
});

test('editor and loupe layout choices survive reload', async ({ page }) => {
  await installMocks(page);
  await page.goto('/search?q=IMG');
  await page.getByLabel('Quick review').first().click();

  await page.getByRole('button', { name: 'Collapse filmstrip' }).click();
  await page.reload();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: 'Show filmstrip' })).toBeVisible();

  await page.getByRole('button', { name: /^Edit/ }).click();
  await page.getByRole('tab', { name: 'Develop' }).click();
  await page.getByRole('button', { name: 'Hide filmstrip' }).click();
  await page.reload();

  await expect(page.getByRole('complementary', { name: 'Editor controls' })).toBeHidden();
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.locator(`a[href^="/assets/${ASSET_ID}?"]`).evaluate((element) => element.click());
  await expect(page.getByRole('button', { name: 'Show filmstrip' })).toBeVisible();
});

test('a popover closes on escape and on an outside click, returning focus', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const zoom = page.getByRole('button', { name: 'Zoom', exact: true });
  const zoomIn = page.getByRole('button', { name: 'Zoom In' });

  await zoom.click();
  await expect(zoomIn).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(zoomIn).toBeHidden();
  await expect(zoom).toBeFocused();

  await zoom.click();
  await expect(zoomIn).toBeVisible();

  await page.getByRole('button', { name: /^Before \/ after split/ }).click();
  await expect(zoomIn).toBeHidden();
});

test('the copy dialog is a named modal that cancels back to the editor', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const slider = page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
  await slider.fill('1');

  const copy = page.getByRole('button', { name: 'Copy edits' });
  await expect(copy).toBeEnabled();
  await copy.click();

  const dialog = page.getByRole('dialog', { name: 'Copy settings' });
  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();
});

test('narrow viewports show the desktop-required guard', async ({ page }) => {
  await page.setViewportSize({ width: 600, height: 800 });
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);

  await expect(page.getByRole('heading', { name: 'Desktop required' })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Back/ })).toHaveCount(0);
});

test('deleting a preset requires explicit confirmation', async ({ page }) => {
  const deleted: string[] = [];
  await installMocks(page, {
    presets: [
      {
        id: 'preset-1',
        name: 'Warm film',
        group_name: null,
        manifest: { schema_version: 1, ops: {} },
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ],
    onPresetDelete: (id) => deleted.push(id)
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Presets', exact: true }).click();
  const preset = page.getByRole('combobox', { name: 'Select a preset…' });
  await preset.click();
  await page.getByRole('option', { name: 'Warm film', exact: true }).click();
  await expect(preset).toHaveValue('Warm film');

  await page.getByRole('button', { name: 'Delete preset' }).click();
  const confirm = page.getByRole('button', { name: 'Confirm delete preset' });
  await expect(confirm).toBeVisible();
  expect(deleted).toEqual([]);

  await page.getByRole('button', { name: 'Cancel delete' }).click();
  await expect(confirm).toHaveCount(0);
  expect(deleted).toEqual([]);

  await page.getByRole('button', { name: 'Delete preset' }).click();
  await confirm.click();
  await expect.poll(() => deleted).toEqual(['preset-1']);
});

test('preset rename commits with Enter and cancels with Escape', async ({ page }) => {
  const updates: Array<[string, Record<string, unknown>]> = [];
  await installMocks(page, {
    presets: [
      {
        id: 'preset-1',
        name: 'Warm film',
        group_name: null,
        manifest: { schema_version: 1, ops: {} },
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }
    ],
    onPresetUpdate: (id, body) => updates.push([id, body])
  });
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Presets', exact: true }).click();
  await page.getByRole('combobox', { name: 'Select a preset…' }).click();
  await page.getByRole('option', { name: 'Warm film', exact: true }).click();

  await page.getByRole('button', { name: 'Rename preset' }).click();
  const name = page.getByRole('textbox', { name: 'Preset name' });
  await expect(name).toBeFocused();
  await expect(name).toHaveValue('Warm film');
  await name.fill('Cool film');
  await name.press('Enter');
  await expect.poll(() => updates.map(([, body]) => body.name)).toEqual(['Cool film']);

  await page.getByRole('button', { name: 'Rename preset' }).click();
  await name.fill('Discarded');
  await name.press('Escape');
  await expect(name).toHaveCount(0);
  expect(updates).toHaveLength(1);
});

test('stack primary is a keyboard radio group', async ({ page }) => {
  await installMocks(page, {});
  await gotoAsset(page);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await page.getByRole('checkbox', { name: 'Stack with original' }).click();

  const original = page.getByRole('radio', { name: 'Original primary' });
  await expect(page.getByRole('radiogroup', { name: 'Stack primary' })).toHaveCSS('height', '28px');
  await expect(page.getByRole('radio', { name: 'Edit primary' })).toBeChecked();
  await original.click();
  await expect(original).toBeChecked();
});

test('editor toggles report their pressed state', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const split = page.getByRole('button', { name: /^Before \/ after split/ });
  await expect(split).toHaveAttribute('aria-pressed', 'false');
  await split.click();
  await expect(split).toHaveAttribute('aria-pressed', 'true');
});
