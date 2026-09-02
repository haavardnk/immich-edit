import { expect, test } from '@playwright/test';
import { installMocks } from './helpers';

test('grid shows total count without loaded progress', async ({ page }) => {
  await installMocks(page, { total: 1000 });

  await page.goto('/photos');

  await expect(page.getByText('1000 assets', { exact: true })).toBeVisible();
  await expect(page.getByText('1 of 1000', { exact: true })).toHaveCount(0);
  await expect(page.getByText('1 loaded', { exact: true })).toHaveCount(0);
});

test('select all keeps only whole-filter actions available', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Select loaded', exact: true })).toHaveCount(0);
  await page.getByRole('button', { name: 'Select all', exact: true }).click();

  await expect(
    page.getByText('Only job actions are available when all filtered assets are selected.')
  ).toBeVisible();
  await expect(
    page.getByRole('button', {
      name: 'Favorite unavailable when all filtered assets are selected',
      exact: true
    })
  ).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Edit and export selected' })).toBeEnabled();
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
