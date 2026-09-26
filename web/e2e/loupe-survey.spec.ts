import { expect, test, type Page } from '@playwright/test';
import { ASSET_EXIF, ASSET_ID, ASSET_SUMMARY, installMocks } from './helpers';

const ASSETS = [
  { ...ASSET_SUMMARY, exifInfo: ASSET_EXIF },
  {
    ...ASSET_SUMMARY,
    id: '00000000-0000-0000-0000-000000000002',
    originalFileName: 'IMG_0002.ARW',
    exifInfo: ASSET_EXIF
  },
  {
    ...ASSET_SUMMARY,
    id: '00000000-0000-0000-0000-000000000003',
    originalFileName: 'IMG_0003.ARW',
    exifInfo: ASSET_EXIF
  }
];

const PANES = ['IMG_0001.ARW', 'IMG_0002.ARW', 'IMG_0003.ARW'];

async function openGrid(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
}

async function selectTile(page: Page, name: string): Promise<void> {
  await page.locator(`div[title="${name}"]`).getByLabel('Select', { exact: true }).click();
}

async function openSurvey(page: Page): Promise<void> {
  await installMocks(page, { assets: ASSETS });
  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();
  await page.keyboard.press('n');
  for (const name of PANES) {
    await expect(page.getByRole('img', { name })).toBeVisible();
  }
}

test('keep only in the survey rail leaves the focused pane', async ({ page }) => {
  await openSurvey(page);

  await page.getByRole('button', { name: 'Keep only this photo' }).click();

  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeHidden();
});

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

test('g leaves survey straight for the grid with the survivors selected', async ({ page }) => {
  await openSurvey(page);

  await page.keyboard.press('Backspace');
  await page.keyboard.press('g');

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
    await expect(page.getByRole('img', { name })).toHaveAttribute('style', /scale\(/);
  }
});

test('modifier-clicking a survey member drops its pane', async ({ page }) => {
  await openSurvey(page);

  await page.getByRole('button', { name: 'IMG_0002.ARW' }).click({ modifiers: ['ControlOrMeta'] });
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();

  await page.getByRole('button', { name: 'IMG_0001.ARW' }).click({ modifiers: ['ControlOrMeta'] });
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeHidden();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('the bulk bar opens a survey of three selected photos', async ({ page }) => {
  await openGrid(page);
  for (const name of PANES) await selectTile(page, name);

  await page.getByLabel('Survey selected').click();

  for (const name of PANES) {
    await expect(page.getByRole('img', { name })).toBeVisible();
  }
});

test('c opens a compare of the grid selection', async ({ page }) => {
  await openGrid(page);
  await selectTile(page, 'IMG_0001.ARW');
  await selectTile(page, 'IMG_0003.ARW');

  await page.keyboard.press('c');

  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeHidden();
});

test('hovering a survey pane offers a drop button for that photo', async ({ page }) => {
  await openSurvey(page);
  const pane = page.locator('div.group', { has: page.getByRole('img', { name: 'IMG_0002.ARW' }) });
  const drop = pane.getByRole('button', { name: 'Drop from survey' });

  await expect(drop).toHaveCSS('opacity', '0');
  await pane.hover();
  await expect(drop).toHaveCSS('opacity', '1');
  await drop.click();

  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toHaveCount(0);
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'IMG_0003.ARW' })).toBeVisible();
});

test('the filmstrip menu removes a photo from the survey', async ({ page }) => {
  await openSurvey(page);
  const thumb = page
    .getByTestId('filmstrip-scroll')
    .getByRole('button', { name: 'IMG_0002.ARW', exact: true });

  await thumb.click({ button: 'right' });
  await page.getByRole('menuitem', { name: 'Remove from survey' }).click();

  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toHaveCount(0);
  await thumb.click({ button: 'right' });
  await expect(page.getByRole('menuitem', { name: 'Add to survey' })).toBeVisible();
});
