import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks, type InstallOpts } from './helpers';

const SECOND_ID = '00000000-0000-0000-0000-000000000002';
const ASSETS = [
  ASSET_SUMMARY,
  { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW' }
];

const LOUPE_IMAGE = 'IMG_0001.ARW';
const NEXT_IMAGE = 'IMG_0002.ARW';

async function openLoupe(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, { assets: ASSETS, ...opts });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
}

test('loupe navigates with the arrow keys and closes with escape', async ({ page }) => {
  await openLoupe(page);
  await expect(page.getByRole('img', { name: LOUPE_IMAGE })).toBeVisible();

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('img', { name: NEXT_IMAGE })).toBeVisible();

  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('img', { name: LOUPE_IMAGE })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: /^Back/ })).toBeHidden();

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

test('shift+f toggles loupe fullscreen', async ({ page }) => {
  await openLoupe(page);

  await page.keyboard.press('Shift+f');
  await expect(page.getByRole('navigation', { name: 'Loupe toolbar' })).toHaveCount(0);
  await expect(page.getByRole('navigation', { name: 'Photo actions' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: /^Exit fullscreen/ })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('navigation', { name: 'Loupe toolbar' })).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Photo actions' })).toBeVisible();
});

test('zoom cycles through detected faces before returning to fit', async ({ page }) => {
  await openLoupe(page, {
    faces: [
      { source_w: 1000, source_h: 800, x: 0.05, y: 0.05, w: 0.3, h: 0.3 },
      { source_w: 1000, source_h: 800, x: 0.7, y: 0.7, w: 0.2, h: 0.2 }
    ]
  });
  const image = page.getByRole('img', { name: LOUPE_IMAGE });

  await page.keyboard.press('z');
  await expect(image).toHaveAttribute('style', /scale\(2\.5\) translate\(/);
  const first = await image.getAttribute('style');

  await page.keyboard.press('z');
  await expect(image).toHaveAttribute('style', /scale\(2\.5\) translate\(/);
  expect(await image.getAttribute('style')).not.toBe(first);

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

  await expect(image).toHaveAttribute('src', /clip=false/);

  await page.keyboard.press('j');
  await expect(image).toHaveAttribute('src', /clip=true/);

  await page.keyboard.press('j');
  await expect(image).toHaveAttribute('src', /clip=false/);
});
