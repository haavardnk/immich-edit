import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

const ASSETS = [
  ASSET_SUMMARY,
  {
    ...ASSET_SUMMARY,
    id: '00000000-0000-0000-0000-000000000002',
    originalFileName: 'IMG_0002.ARW'
  },
  { ...ASSET_SUMMARY, id: '00000000-0000-0000-0000-000000000003', originalFileName: 'IMG_0003.ARW' }
];

async function openCompare(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await page.keyboard.press('c');
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
}

test('shift+arrow swaps only the focused compare pane', async ({ page }) => {
  await openCompare(page);

  await page.keyboard.press('Shift+ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
});

test('escape returns to a single pane before closing', async ({ page }) => {
  await openCompare(page);

  await page.keyboard.press('Escape');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByRole('button', { name: /^Back/ })).toBeHidden();
});

test('the filmstrip ring follows the focused pane', async ({ page }) => {
  await openCompare(page);
  const first = page.getByRole('button', { name: 'IMG_0001.ARW' });
  const second = page.getByRole('button', { name: 'IMG_0002.ARW' });

  await expect(first).toHaveAttribute('aria-pressed', 'true');
  await expect(second).toHaveAttribute('aria-pressed', 'true');
  await expect(first).toHaveClass(/ring-primary\/60/);
  await expect(second).toHaveClass(/ring-primary\/40/);

  await page.keyboard.press('ArrowRight');
  await expect(second).toHaveClass(/ring-primary\/60/);
  await expect(first).toHaveClass(/ring-primary\/40/);
});

test('pane borders stay outside the images', async ({ page }) => {
  await openCompare(page);
  const pane = page.getByRole('button', { name: 'Zoom in' }).first();
  const image = page.getByRole('img', { name: 'IMG_0001.ARW' });
  const paneBox = await pane.boundingBox();
  const imageBox = await image.boundingBox();
  if (!paneBox || !imageBox) throw new Error('compare pane geometry is unavailable');
  const border = await pane.evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      top: Number.parseFloat(style.borderTopWidth),
      right: Number.parseFloat(style.borderRightWidth),
      bottom: Number.parseFloat(style.borderBottomWidth),
      left: Number.parseFloat(style.borderLeftWidth)
    };
  });

  expect(border).toEqual({ top: 2, right: 2, bottom: 2, left: 2 });
  expect(imageBox.x).toBeGreaterThanOrEqual(paneBox.x + border.left);
  expect(imageBox.y).toBeGreaterThanOrEqual(paneBox.y + border.top);
  expect(imageBox.x + imageBox.width).toBeLessThanOrEqual(paneBox.x + paneBox.width - border.right);
  expect(imageBox.y + imageBox.height).toBeLessThanOrEqual(
    paneBox.y + paneBox.height - border.bottom
  );
});

test('auto-advance swaps the rated pane for the next photo', async ({ page }) => {
  await openCompare(page);
  await page.getByRole('button', { name: 'More loupe actions' }).click();
  await page.getByRole('button', { name: 'Auto-advance', exact: true }).click();

  await page.keyboard.press('3');
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
});

test('zoom syncs across panes until sync is turned off', async ({ page }) => {
  await openCompare(page);
  const first = page.getByRole('img', { name: 'IMG_0001.ARW' });
  const second = page.getByRole('img', { name: 'IMG_0002.ARW' });

  await page.keyboard.press('z');
  await expect(first).toHaveAttribute('style', /scale\(2\.5\)/);
  await expect(second).toHaveAttribute('style', /scale\(2\.5\)/);

  await page.keyboard.press('y');
  await page.keyboard.press('z');
  await expect(first).not.toHaveAttribute('style', /scale\(2\.5\)/);
  await expect(second).toHaveAttribute('style', /scale\(2\.5\)/);
});

test('a swapped pane keeps the zoom of the photo it replaced', async ({ page }) => {
  await openCompare(page);

  await page.keyboard.press('z');
  await page.keyboard.press('Shift+ArrowRight');
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toHaveAttribute(
    'style',
    /scale\(2\.5\)/
  );
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toHaveAttribute(
    'style',
    /scale\(2\.5\)/
  );
});

test('clicking an unfocused pane selects it without zooming', async ({ page }) => {
  await openCompare(page);
  const second = page.getByRole('img', { name: 'IMG_0002.ARW' });

  await second.click({ position: { x: 10, y: 10 } });
  await expect(page.getByRole('button', { name: 'IMG_0002.ARW' })).toHaveClass(/ring-primary\/60/);
  await expect(second).not.toHaveAttribute('style', /scale\(2\.5\)/);

  await second.click({ position: { x: 10, y: 10 } });
  await expect(second).toHaveAttribute('style', /scale\(2\.5\)/);
});

test('modifier-clicking the filmstrip adds a pane', async ({ page }) => {
  await openCompare(page);

  await page.getByRole('button', { name: 'IMG_0003.ARW' }).click({ modifiers: ['ControlOrMeta'] });
  for (const name of ['IMG_0001.ARW', 'IMG_0002.ARW', 'IMG_0003.ARW']) {
    await expect(page.getByRole('img', { name })).toBeVisible();
  }
});
