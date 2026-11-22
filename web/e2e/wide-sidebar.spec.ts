import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets } from './helpers';

test.describe('wide sidebar on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('hides to give the page the full width and comes back from the header', async ({ page }) => {
    await installMocks(page, { assets: numberedAssets(6) });
    await page.goto('/photos');
    const sidebar = page.locator('[data-wide-sidebar]');
    const main = page.locator('main[data-page]');
    await expect(sidebar).toBeVisible();
    const narrow = (await main.boundingBox())!.width;

    const hide = sidebar.getByRole('button', { name: 'Hide sidebar' });
    await expect(hide).toHaveAttribute('aria-expanded', 'true');
    await hide.tap();
    await expect(sidebar).toHaveAttribute('aria-hidden', 'true');
    await expect.poll(async () => (await main.boundingBox())!.width).toBeGreaterThan(narrow + 200);
    const show = main.getByRole('button', { name: 'Show sidebar' });
    await expect(show).toBeVisible();
    await expect(show).toHaveAttribute('aria-expanded', 'false');

    await page.reload();
    await expect(main.getByRole('button', { name: 'Show sidebar' })).toBeVisible();
    await expect(sidebar).toHaveAttribute('aria-hidden', 'true');

    await main.getByRole('button', { name: 'Show sidebar' }).tap();
    await expect(sidebar).not.toHaveAttribute('aria-hidden');
    await expect(sidebar.getByRole('link', { name: /^Photos/ })).toBeVisible();
    await expect(main.getByRole('button', { name: 'Show sidebar' })).toHaveCount(0);
  });

  test('every page offers the way back while it is hidden', async ({ page }) => {
    await installMocks(page, { assets: numberedAssets(2) });
    await page.goto('/photos');
    await page.locator('[data-wide-sidebar]').getByRole('button', { name: 'Hide sidebar' }).tap();
    for (const path of [
      '/photos',
      '/favorites',
      '/albums',
      '/people',
      '/tags',
      '/folders',
      '/edited',
      '/search?q=IMG',
      '/settings/preferences',
      '/settings/account'
    ]) {
      await page.goto(path);
      await expect(
        page.locator('main[data-page]').getByRole('button', { name: 'Show sidebar' }),
        path
      ).toBeVisible();
    }
  });
});
