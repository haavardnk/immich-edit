import { expect, test } from '@playwright/test';

import type { Page } from '@playwright/test';

import { installMocks } from './helpers';

async function sidewaysScrollers(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const slack = 2;
    const root = document.documentElement;
    const out: string[] = [];
    if (root.scrollWidth > window.innerWidth + slack) out.push('document');
    if (root.scrollHeight > window.innerHeight + slack) out.push('document (vertical)');
    for (const el of Array.from(document.querySelectorAll<HTMLElement>('body *'))) {
      const overflowX = getComputedStyle(el).overflowX;
      if (['auto', 'scroll', 'hidden', 'clip'].includes(overflowX)) continue;
      if (el.scrollWidth <= el.clientWidth + slack) continue;
      out.push(`${el.tagName.toLowerCase()}.${el.className || '(no class)'}`);
    }
    return out;
  });
}

test('the mobile browse shell uses a toggleable 256px sidebar', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');

  const sidebar = page.getByRole('complementary');
  const main = page.getByRole('main');
  const visibleMainWidth = (): Promise<number> =>
    main.evaluate((element) => element.parentElement?.clientWidth ?? 0);
  const sidebarWidth = (): Promise<number> =>
    main.evaluate((element) => element.parentElement?.previousElementSibling?.clientWidth ?? 0);
  await expect(page.getByRole('textbox', { name: 'Search your photos' })).toBeVisible();
  await expect(sidebar).toBeHidden();
  await expect.poll(visibleMainWidth).toBeGreaterThan(380);
  await expect.poll(sidebarWidth).toBe(0);
  await expect
    .poll(() => page.evaluate(() => document.elementFromPoint(100, 500)?.closest('main') !== null))
    .toBe(true);

  await page.getByRole('button', { name: 'Library' }).click();
  await expect(sidebar).toBeVisible();
  await expect(sidebar).toHaveCSS('width', '256px');
  await expect.poll(sidebarWidth).toBeGreaterThan(254);
  expect(await sidewaysScrollers(page)).toEqual([]);

  await page.getByRole('button', { name: 'Library' }).click();
  await expect(sidebar).toBeHidden();
  await expect(page.getByRole('button', { name: 'Library' })).toBeFocused();
  await expect.poll(visibleMainWidth).toBeGreaterThan(380);
  await expect.poll(sidebarWidth).toBe(0);
  await expect
    .poll(() => page.evaluate(() => document.elementFromPoint(100, 500)?.closest('main') !== null))
    .toBe(true);

  await page.getByRole('button', { name: 'Library' }).click();
  await page.getByRole('link', { name: /^Favorites/ }).click();
  await expect(page).toHaveURL('/favorites');
  await expect(sidebar).toBeHidden();
});

test('the mobile bulk action bar keeps every action reachable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();

  const bulkBar = page.getByRole('navigation').filter({ hasText: '1 selected' });
  const bulkSurfaceFits = (): Promise<boolean> =>
    bulkBar.evaluate((element) => {
      const surface = element.parentElement;
      if (!surface) return false;
      const bounds = surface.getBoundingClientRect();
      return bounds.left >= 0 && bounds.right <= window.innerWidth;
    });
  const clearSelection = page.getByRole('button', { name: 'Clear selection' });
  const tags = page.getByRole('button', { name: 'Tags', exact: true });
  await expect.poll(bulkSurfaceFits).toBe(true);
  await expect(clearSelection).toBeInViewport();
  await tags.scrollIntoViewIfNeeded();
  await expect(tags).toBeInViewport();
  await expect(clearSelection).toBeInViewport();

  await tags.click();
  await expect(page.getByRole('button', { name: 'Add to selected' })).toBeInViewport();
  await expect(page.getByRole('button', { name: 'Remove from selected' })).toBeInViewport();
  await expect(clearSelection).toBeInViewport();
  await expect.poll(bulkSurfaceFits).toBe(true);
});
