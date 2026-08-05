import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

const SECOND_ID = '00000000-0000-0000-0000-000000000002';
const ASSETS = [
  ASSET_SUMMARY,
  { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW' }
];

const LOUPE_IMAGE = 'IMG_0001.ARW';
const NEXT_IMAGE = 'IMG_0002.ARW';

async function openLoupe(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.addInitScript(() => {
    localStorage.setItem(
      'immich-edit:settings',
      JSON.stringify({ metadataPushConsented: true })
    );
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href="/assets/${ASSET_ID}"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: /^Close/ })).toBeVisible();
}

test('loupe navigates with the arrow keys and closes with escape', async ({ page }) => {
  await openLoupe(page);
  await expect(page.getByRole('img', { name: LOUPE_IMAGE })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: NEXT_IMAGE })).toBeVisible();

  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('img', { name: LOUPE_IMAGE })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: /^Close/ })).toBeHidden();

  await page.keyboard.press(' ');
  await expect(page.getByRole('img', { name: LOUPE_IMAGE })).toBeVisible();
});

test('z toggles loupe zoom', async ({ page }) => {
  await openLoupe(page);
  const image = page.getByRole('img', { name: LOUPE_IMAGE });

  await page.keyboard.press('z');
  await expect(image).toHaveAttribute('style', /scale\(2\.5\)/);

  await page.keyboard.press('z');
  await expect(image).not.toHaveAttribute('style', /scale\(2\.5\)/);
});

test('number keys rate the loupe asset', async ({ page }) => {
  await openLoupe(page);

  await page.keyboard.press('3');
  await expect(page.getByRole('radio', { name: '3 stars' })).toBeChecked();
});

test('j toggles the clipping overlay', async ({ page }) => {
  await openLoupe(page);
  const image = page.getByRole('img', { name: LOUPE_IMAGE });
  const button = page.getByRole('button', { name: /^Clipping overlay/ });

  await expect(image).toHaveAttribute('src', /clip=false/);
  await expect(button).toHaveAttribute('aria-pressed', 'false');

  await page.keyboard.press('j');
  await expect(image).toHaveAttribute('src', /clip=true/);
  await expect(button).toHaveAttribute('aria-pressed', 'true');

  await page.keyboard.press('j');
  await expect(image).toHaveAttribute('src', /clip=false/);
});
