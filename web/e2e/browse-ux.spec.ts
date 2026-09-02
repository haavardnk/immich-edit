import { expect, test } from '@playwright/test';
import { installMocks } from './helpers';

test('select all explains why loaded-only actions are unavailable', async ({ page }) => {
  await installMocks(page);

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Select all', exact: true }).click();

  await expect(
    page.getByText(
      'Job actions only. Select loaded assets to use metadata, compare, tags, or copies.'
    )
  ).toBeVisible();
  await expect(
    page.getByRole('button', {
      name: 'Favorite unavailable: select loaded assets',
      exact: true
    })
  ).toBeDisabled();
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
