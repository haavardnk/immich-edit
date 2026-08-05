import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

const ASSETS = [
  ASSET_SUMMARY,
  { ...ASSET_SUMMARY, id: '00000000-0000-0000-0000-000000000002', originalFileName: 'IMG_0002.ARW' },
  { ...ASSET_SUMMARY, id: '00000000-0000-0000-0000-000000000003', originalFileName: 'IMG_0003.ARW' }
];

const PANES = ['IMG_0001.ARW', 'IMG_0002.ARW', 'IMG_0003.ARW'];

async function openSurvey(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href="/assets/${ASSET_ID}"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await page.keyboard.press('n');
  for (const name of PANES) {
    await expect(page.getByRole('img', { name })).toBeVisible();
  }
}

test('backspace drops the focused survey pane', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('Backspace');
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('enter keeps only the focused survey pane', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeHidden();
});

test('leaving survey selects the survivors', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('Backspace');
  await page.keyboard.press('Escape');
  await page.keyboard.press('Escape');

  await expect(page.getByText('2 selected')).toBeVisible();
});

test('leaving an untouched survey selects nothing', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('Escape');
  await page.keyboard.press('Escape');

  await expect(page.getByText('selected')).toBeHidden();
});

test('zoom applies to every survey pane', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('z');
  for (const name of PANES) {
    await expect(page.getByRole('img', { name })).toHaveAttribute('style', /scale\(2\.5\)/);
  }
});
