import { expect, test, type Page } from '@playwright/test';
import {
  ASSET_EXIF,
  ASSET_ID,
  ASSET_SUMMARY,
  installMocks,
  numberedAssets,
  type InstallOpts
} from './helpers';

const SECOND_ID = '00000000-0000-0000-0000-000000000002';
const ASSETS = [
  { ...ASSET_SUMMARY, exifInfo: ASSET_EXIF },
  { ...ASSET_SUMMARY, id: SECOND_ID, originalFileName: 'IMG_0002.ARW', exifInfo: ASSET_EXIF }
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

test('loupe walks past the last loaded page', async ({ page }) => {
  const assets = numberedAssets(80);
  const pages: unknown[] = [];
  await openLoupe(page, {
    assets,
    searchPages: [assets.slice(0, 40), assets.slice(40)],
    onMetadata: (body) => void pages.push(body.page),
    onSmart: (body) => void pages.push(body.page)
  });
  expect(pages).toEqual([undefined]);

  for (let step = 0; step < 41; step += 1) await page.keyboard.press('ArrowRight');

  await expect(page.getByRole('img', { name: 'IMG_0042.ARW' })).toBeVisible();
  expect(pages).toEqual([undefined, 2]);
});

test('a filmstrip scrolled away stays put when the next page loads', async ({ page }) => {
  const assets = numberedAssets(80);
  const pages: unknown[] = [];
  await openLoupe(page, {
    assets,
    searchPages: [assets.slice(0, 40), assets.slice(40)],
    onMetadata: (body) => void pages.push(body.page),
    onSmart: (body) => void pages.push(body.page)
  });
  const strip = page.getByTestId('filmstrip-scroll');
  const width = (): Promise<number> => strip.evaluate((el) => el.scrollWidth);
  const loadedWidth = await width();

  await strip.evaluate((el) => el.scrollTo({ left: el.scrollWidth }));
  await expect.poll(() => pages).toEqual([undefined, 2]);
  await expect.poll(width).toBeGreaterThan(loadedWidth);
  await page.waitForTimeout(600);

  const left = await strip.evaluate((el) => el.scrollLeft);
  expect(left).toBeGreaterThan(loadedWidth / 2);
});

test('z toggles loupe zoom', async ({ page }) => {
  await openLoupe(page);
  const image = page.getByRole('img', { name: LOUPE_IMAGE });

  await page.keyboard.press('z');
  await expect(image).toHaveAttribute('style', /scale\(/);

  await page.keyboard.press('z');
  await expect(image).not.toHaveAttribute('style', /scale\(/);
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

test('shift+t hides the loupe filmstrip', async ({ page }) => {
  await openLoupe(page);
  await expect(page.getByTestId('filmstrip-scroll')).toBeVisible();

  await page.keyboard.press('Shift+t');

  await expect(page.getByTestId('filmstrip-scroll')).toHaveCount(0);
});

test('the loupe toolbar enters fullscreen', async ({ page }) => {
  await openLoupe(page);

  await page.getByRole('button', { name: /^Fullscreen/ }).click();

  await expect(page.getByRole('button', { name: /^Exit fullscreen/ })).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Loupe toolbar' })).toHaveCount(0);
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
  await expect(image).toHaveAttribute('style', /scale\([\d.]+\) translate\(/);
  const first = await image.getAttribute('style');

  await page.keyboard.press('z');
  await expect(image).toHaveAttribute('style', /scale\([\d.]+\) translate\(/);
  expect(await image.getAttribute('style')).not.toBe(first);

  await page.keyboard.press('z');
  await expect(image).not.toHaveAttribute('style', /scale\(/);
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
