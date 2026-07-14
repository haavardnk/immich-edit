import { expect, test } from '@playwright/test';
import { ASSET_ID, NEUTRAL_RECORD, installMocks, json, gotoAsset } from './helpers';
import { neutralEdits } from '../src/lib/types/edits';

test('history popover expands details and restores a prior entry', async ({ page }) => {
  const latestEdits = neutralEdits();
  latestEdits.basic.exposure_ev = 1.25;
  latestEdits.geometry.rotate = 90;
  latestEdits.output.tonemap = 'agx';
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
  await expect(page.getByText('Tonemap: Default → AgX')).toBeVisible();
  expect(restoreCalled).toBe(false);
  await expect(page.getByText('Initial')).toBeVisible();
  await page.getByText('Initial').click();

  await expect.poll(() => restoreCalled).toBe(true);
});

test('export download triggers a file download', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Export', exact: true }).click();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: /Export JPEG/ }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/IMG_0001.*\.jpg$/);
});

test('split view toggle reveals before/after slider', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const splitButton = page.getByRole('button', { name: 'Before/After split' });
  await splitButton.click();

  await expect(page.getByRole('slider', { name: 'Before/after split' })).toBeVisible();
});

test('narrow viewports show the desktop-required guard', async ({ page }) => {
  await page.setViewportSize({ width: 600, height: 800 });
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);

  await expect(page.getByRole('heading', { name: 'Desktop required' })).toBeVisible();
  await expect(page.getByTitle('Back')).toHaveCount(0);
});
