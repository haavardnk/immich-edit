import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

const PAGED_ASSETS = Array.from({ length: 3 }, (_, index) => {
  const number = index + 1;
  return {
    ...ASSET_SUMMARY,
    id: `00000000-0000-0000-0000-${String(number).padStart(12, '0')}`,
    originalFileName: `IMG_${String(number).padStart(4, '0')}.ARW`
  };
});

test('grid shows total count without loaded progress', async ({ page }) => {
  await installMocks(page, { total: 1000 });

  await page.goto('/photos');

  await expect(page.getByText('1000 assets', { exact: true })).toBeVisible();
  await expect(page.getByText('1 of 1000', { exact: true })).toHaveCount(0);
  await expect(page.getByText('1 loaded', { exact: true })).toHaveCount(0);
});

test('select all loads every page and enables local actions', async ({ page }) => {
  const pages: Array<number | undefined> = [];
  const releasePages = new Map<number, () => void>();
  await installMocks(page, {
    assets: PAGED_ASSETS,
    searchPages: PAGED_ASSETS.map((asset) => [asset]),
    onMetadata: async (body) => {
      const pageNumber = typeof body.page === 'number' ? body.page : undefined;
      pages.push(pageNumber);
      if (pageNumber === undefined) return;
      await new Promise<void>((resolve) => releasePages.set(pageNumber, resolve));
    }
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Select all', exact: true }).click();

  await expect.poll(() => releasePages.has(2)).toBe(true);
  releasePages.get(2)?.();
  await expect(page.getByText('2 selected')).toBeVisible();
  await expect(page.locator('div[title="IMG_0002.ARW"]')).toHaveAttribute('data-selected', 'true');
  await expect.poll(() => releasePages.has(3)).toBe(true);
  releasePages.get(3)?.();
  await expect(page.getByText('3 selected')).toBeVisible();
  expect(pages).toEqual([undefined, 2, 3]);
  await expect(page.getByRole('button', { name: 'Select all', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Favorite', exact: true })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Edit and export selected' })).toBeEnabled();
});

test('changing a browse filter clears hidden selection', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await expect(page.getByText('1 selected')).toBeVisible();

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByLabel('Filename').fill('portrait');

  await expect(page.getByText('1 selected')).toBeHidden();
  await expect(page.getByRole('button', { name: 'Clear selection' })).toBeHidden();
});

test('thumbnail size controls remain available on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await installMocks(page);

  await page.goto('/photos');

  for (const size of ['S', 'M', 'L', 'XL']) {
    await expect(page.getByRole('button', { name: `Thumbnail size ${size}` })).toBeInViewport();
  }
  await page.getByRole('button', { name: 'Thumbnail size XL' }).click();
  await expect(page.getByRole('button', { name: 'Thumbnail size XL' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});
