import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

async function createCopy(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'More editor actions' }).click();
  await page.getByRole('button', { name: 'Create virtual copy' }).click();
  await page.waitForURL(new RegExp(`/assets/${ASSET_ID}_1(?:\\?.*)?$`));
}

test('create, rename and delete a virtual copy from the editor', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}?from=${encodeURIComponent('/photos')}`);
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();

  await createCopy(page);
  await expect(page).toHaveURL(/\?from=%2Fphotos$/);

  await page.getByRole('button', { name: 'Versions', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Copy 1' })).toBeVisible();

  await page.getByRole('link', { name: 'Original' }).click();
  await page.waitForURL(`**/assets/${ASSET_ID}`);
  await page.getByRole('link', { name: 'Copy 1' }).click();
  await page.waitForURL(`**/assets/${ASSET_ID}_1`);

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
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await createCopy(page);

  await page.goto('/photos');
  const master = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  const copy = page.locator(`a[href^="/assets/${ASSET_ID}_1?"]`).first();
  await expect(master).toBeVisible();
  await expect(copy).toBeVisible();
});

test('the loupe header marks a virtual copy', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await createCopy(page);

  await page.goto('/photos');
  const copy = page.locator(`a[href^="/assets/${ASSET_ID}_1?"]`).first();
  await expect(copy).toBeVisible();

  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await expect(page.getByRole('status', { name: 'Virtual copy', exact: true })).toHaveCount(0);

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('status', { name: 'Virtual copy', exact: true })).toHaveText(
    'Copy 1'
  );
});

test('the grid delete button drops a virtual copy after a confirm click', async ({ page }) => {
  await installMocks(page);

  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await createCopy(page);

  await page.goto('/photos');
  const copy = page.locator(`a[href^="/assets/${ASSET_ID}_1?"]`).first();
  await expect(copy).toBeVisible();
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first()).toBeVisible();

  await page.getByRole('button', { name: 'Delete copy' }).click();
  await page.getByRole('button', { name: 'Confirm delete copy' }).click();

  await expect(copy).toHaveCount(0);
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first()).toBeVisible();
});
