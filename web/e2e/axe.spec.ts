import AxeBuilder from '@axe-core/playwright';

import { expect, test } from '@playwright/test';

import { ASSET_ID, installMocks, gotoAsset } from './helpers';

const IMPACTS = new Set(['serious', 'critical']);

async function audit(page: Parameters<typeof gotoAsset>[0]): Promise<string[]> {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze();
  return results.violations
    .filter((v) => IMPACTS.has(v.impact ?? ''))
    .map(
      (v) =>
        `${v.id}: ${v.nodes.map((n) => `${n.target.join(' ')} (${n.failureSummary})`).join(', ')}`
    );
}

test('the photo grid has no serious accessibility violations', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link').first()).toBeVisible();

  expect(await audit(page)).toEqual([]);
});

test('open browse filters have no serious accessibility violations', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await expect(page.getByRole('button', { name: 'Close filters' })).toBeVisible();
  expect(await audit(page)).toEqual([]);
});

test('the loupe and mobile action menu have no serious accessibility violations', async ({
  page
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();

  await page.getByLabel('Quick review').click();
  await page.getByRole('button', { name: 'More loupe actions' }).click();
  await expect(page.getByRole('button', { name: 'Clipping overlay', exact: true })).toBeVisible();

  expect(await audit(page)).toEqual([]);
});

test('visible feedback has accessible live-region semantics', async ({ page }) => {
  await installMocks(page, { smartFails: true });
  await page.goto('/search?q=red%20car');

  const message = 'Smart search unavailable, showing filename matches';
  const toast = page.getByRole('status').filter({ hasText: message });
  await expect(toast).toBeVisible();
  await expect(toast).toHaveAttribute('aria-live', 'polite');
  expect(await audit(page)).toEqual([]);
});
