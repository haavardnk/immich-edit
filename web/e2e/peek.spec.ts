import { expect, test, type Page } from '@playwright/test';
import { tile } from './browse';
import { installMocks, numberedAssets } from './helpers';

const ASSETS = numberedAssets(3);
const SECOND = ASSETS[1]?.id ?? '';

async function openGrid(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS, metadataPushConsented: true });
  await page.goto('/photos');
  await expect(tile(page, 'IMG_0003.ARW')).toBeVisible();
}

function peekView(page: Page) {
  return page.getByRole('figure', { name: 'Peek' });
}

test('holding W peeks at the hovered photo until the key is released', async ({ page }) => {
  await openGrid(page);
  const url = page.url();
  await tile(page, 'IMG_0002.ARW').hover();

  await page.keyboard.down('w');
  await expect(peekView(page)).toBeVisible();
  await expect(peekView(page).getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(peekView(page)).toContainText('IMG_0002.ARW');

  await page.keyboard.up('w');
  await expect(peekView(page)).toHaveCount(0);
  expect(page.url()).toBe(url);
});

test('rating keys rate the peeked photo, not the selection', async ({ page }) => {
  await openGrid(page);
  await tile(page, 'IMG_0001.ARW').getByRole('button', { name: 'Select' }).click();
  await tile(page, 'IMG_0002.ARW').hover();
  await page.keyboard.down('w');
  await expect(peekView(page)).toBeVisible();

  const write = page.waitForRequest((req) => req.method() === 'PUT');
  await page.keyboard.press('3');
  expect((await write).url()).toContain(`/api/assets/${SECOND}`);
  await expect(peekView(page).getByRole('img', { name: '3 stars' })).toBeVisible();
  await page.keyboard.up('w');
});

test('Space promotes the peek to the loupe', async ({ page }) => {
  await openGrid(page);
  await tile(page, 'IMG_0002.ARW').hover();
  await page.keyboard.down('w');
  await expect(peekView(page)).toBeVisible();

  await page.keyboard.press('Space');
  await page.keyboard.up('w');
  await expect(peekView(page)).toHaveCount(0);
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
});

test('W peeks at the selected photo when the pointer is off the grid', async ({ page }) => {
  await openGrid(page);
  await page.mouse.move(0, 0);
  await page.keyboard.down('w');
  await expect(peekView(page)).toHaveCount(0);
  await page.keyboard.up('w');

  await tile(page, 'IMG_0003.ARW').getByRole('button', { name: 'Select' }).click();
  await page.mouse.move(0, 0);
  await page.keyboard.down('w');
  await expect(peekView(page).getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
  await page.keyboard.up('w');
  await expect(peekView(page)).toHaveCount(0);
});
