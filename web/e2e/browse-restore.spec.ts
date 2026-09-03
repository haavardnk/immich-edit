import { expect, test } from '@playwright/test';
import { ASSET_SUMMARY, installMocks } from './helpers';

const A = '00000000-0000-0000-0000-000000000001';
const B = '00000000-0000-0000-0000-000000000002';
const C = '00000000-0000-0000-0000-000000000003';

const ASSETS = [
  {
    ...ASSET_SUMMARY,
    id: A,
    originalFileName: 'IMG_0001.ARW',
    fileCreatedAt: '2024-01-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: B,
    originalFileName: 'IMG_0002.ARW',
    fileCreatedAt: '2024-02-01T00:00:00Z'
  },
  {
    ...ASSET_SUMMARY,
    id: C,
    originalFileName: 'IMG_0003.ARW',
    fileCreatedAt: '2024-03-01T00:00:00Z'
  }
];

test('opening the editor directly rebuilds the filmstrip from the return path', async ({
  page
}) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto(`/assets/${B}?from=%2Fphotos`);

  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('arrow keys move through the collection after a direct editor load', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });

  await page.goto(`/assets/${B}?from=%2Ftags%2Ftag-1`);
  await expect(page.getByRole('link', { name: 'IMG_0003.ARW' })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page).toHaveURL(new RegExp(C));

  await page.keyboard.press('ArrowLeft');
  await expect(page).toHaveURL(new RegExp(B));
});

test('a restored grid keeps the filters that were active when the editor opened', async ({
  page
}) => {
  await installMocks(page, { assets: ASSETS });

  const names: (string | undefined)[] = [];
  await page.route('**/api/search/metadata', async (route) => {
    names.push((route.request().postDataJSON() as { originalFileName?: string }).originalFileName);
    await route.fallback();
  });

  await page.goto('/photos');
  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByLabel('Filename').fill('IMG_0001');
  await expect.poll(() => names).toContain('IMG_0001');
  names.splice(0);

  await page.goto(`/assets/${B}?from=%2Fphotos`);
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();

  expect(names).toContain('IMG_0001');
});
