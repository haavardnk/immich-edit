import AxeBuilder from '@axe-core/playwright';

import { expect, test } from '@playwright/test';

import { installMocks, gotoAsset } from './helpers';

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

test('open browse filters have no serious accessibility violations', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await expect(page.getByRole('button', { name: 'Close filters' })).toBeVisible();
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
