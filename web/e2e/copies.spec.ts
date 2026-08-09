import { expect, test } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

test('create, rename and delete a virtual copy from the editor', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByTitle('Back')).toBeVisible();

  await page.getByTitle(/^Create a virtual copy/).click();
  await page.waitForURL(`**/assets/${ASSET_ID}_1`);

  await page.getByRole('button', { name: 'Versions', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Copy 1' })).toBeVisible();

  await page.getByRole('button', { name: 'Rename copy' }).click();
  await page.getByPlaceholder('Name').fill('Mono');
  await page.getByPlaceholder('Name').press('Enter');
  await expect(page.getByRole('link', { name: 'Mono' })).toBeVisible();

  await page.getByRole('button', { name: 'Delete copy' }).click();
  await page.waitForURL(`**/assets/${ASSET_ID}`);
  await expect(page.getByRole('link', { name: 'Mono' })).toHaveCount(0);
});

test('a virtual copy shows up beside its master in the grid', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByTitle('Back')).toBeVisible();
  await page.getByTitle(/^Create a virtual copy/).click();
  await page.waitForURL(`**/assets/${ASSET_ID}_1`);

  await page.goto('/photos');
  const master = page.locator(`a[href="/assets/${ASSET_ID}"]`).first();
  const copy = page.locator(`a[href="/assets/${ASSET_ID}_1"]`).first();
  await expect(master).toBeVisible();
  await expect(copy).toBeVisible();
});

test('the grid delete button drops a virtual copy after a confirm click', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByTitle('Back')).toBeVisible();
  await page.getByTitle(/^Create a virtual copy/).click();
  await page.waitForURL(`**/assets/${ASSET_ID}_1`);

  await page.goto('/photos');
  const copy = page.locator(`a[href="/assets/${ASSET_ID}_1"]`).first();
  await expect(copy).toBeVisible();
  await expect(page.locator(`a[href="/assets/${ASSET_ID}"]`).first()).toBeVisible();

  await page.getByRole('button', { name: 'Delete copy' }).click();
  await page.getByRole('button', { name: 'Confirm delete copy' }).click();

  await expect(copy).toHaveCount(0);
  await expect(page.locator(`a[href="/assets/${ASSET_ID}"]`).first()).toBeVisible();
});
