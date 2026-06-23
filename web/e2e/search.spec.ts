import { expect, test } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

const TILE = `a[href="/assets/${ASSET_ID}"]`;

test('filename-like query uses metadata search', async ({ page }) => {
  let smartHit = false;
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    onSmart: () => {
      smartHit = true;
    },
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=DSC00195');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(smartHit).toBe(false);
  expect(metadataBody?.['originalFileName']).toBe('DSC00195');
});

test('description query uses smart search', async ({ page }) => {
  let smartBody: Record<string, unknown> | null = null;
  let metadataHit = false;
  await installMocks(page, {
    onSmart: (body) => {
      smartBody = body;
    },
    onMetadata: () => {
      metadataHit = true;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(smartBody?.['query']).toBe('red car');
  expect(metadataHit).toBe(false);
});

test('toggling to filename reruns as metadata search', async ({ page }) => {
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  await page.getByRole('button', { name: 'Filename', exact: true }).click();
  await page.waitForURL(/mode=filename/);

  await expect.poll(() => metadataBody?.['originalFileName']).toBe('red car');
});

test('smart failure falls back to filename search', async ({ page }) => {
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    smartFails: true,
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(metadataBody?.['originalFileName']).toBe('red car');
});
