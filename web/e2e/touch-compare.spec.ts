import { expect, test, type Locator, type Page } from '@playwright/test';
import { numberedAssets } from './helpers';
import { longPress, tapCentre } from './touch';
import { open } from './touchShell';

const ASSETS = numberedAssets(6);
const PANE_IMAGE = /^IMG_\d{4}\.ARW$/;

async function openLoupe(page: Page): Promise<void> {
  await open(page, '/photos', { assets: ASSETS });
  const tile = page.locator('[role="group"][data-name="IMG_0002.ARW"]');
  await expect(tile).toBeVisible();
  await tapCentre(page, tile);
  await expect(topBar(page)).toBeVisible();
}

function topBar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Loupe toolbar' });
}

function modes(page: Page): Locator {
  return topBar(page).getByRole('radiogroup', { name: 'View mode' });
}

function panes(page: Page): Locator {
  return page.getByRole('img', { name: PANE_IMAGE });
}

function rail(page: Page): Locator {
  return page.getByRole('toolbar', { name: 'Pane actions' });
}

function menu(page: Page): Locator {
  return page.getByRole('dialog', { name: 'Menu' });
}

async function openMenu(page: Page): Promise<void> {
  await topBar(page).getByRole('button', { name: 'More loupe actions' }).tap();
  await expect(menu(page)).toBeVisible();
}

async function longPressThumb(page: Page, name: string): Promise<void> {
  const thumb = page.getByTestId('filmstrip-scroll').getByRole('button', { name, exact: true });
  await expect(thumb).toBeVisible();
  await thumb.tap({ trial: true });
  const box = await thumb.boundingBox();
  if (!box) throw new Error('no filmstrip thumbnail');
  await longPress(page, box.x + box.width / 2, box.y + box.height / 2);
}

async function boxes(locator: Locator): Promise<{ x: number; y: number; height: number }[]> {
  const all = await locator.all();
  const found = await Promise.all(all.map((item) => item.boundingBox()));
  return found.map((box) => ({ x: box?.x ?? 0, y: box?.y ?? 0, height: box?.height ?? 0 }));
}

test.describe('compare and survey on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('compare holds two panes and survey is not offered', async ({ page }) => {
    await openLoupe(page);
    await expect(modes(page)).toHaveCount(0);
    await openMenu(page);
    await expect(menu(page).getByRole('button', { name: 'Survey' })).toHaveCount(0);
    await menu(page).getByRole('button', { name: 'Compare' }).tap();

    await expect(panes(page)).toHaveCount(2);
    await expect(topBar(page).getByRole('heading')).toHaveCount(0);
    await expect(modes(page).getByRole('radio')).toHaveCount(2);
    await expect(modes(page).getByRole('radio', { name: 'Compare' })).toBeChecked();
    await expect(modes(page).getByRole('radio', { name: 'Survey' })).toHaveCount(0);
    const [top, bottom] = await boxes(page.locator('div.group', { has: panes(page) }));
    expect(Math.abs((top?.x ?? 0) - (bottom?.x ?? -1))).toBeLessThan(2);
    expect(bottom?.y ?? 0).toBeGreaterThanOrEqual((top?.y ?? 0) + (top?.height ?? 0) - 1);
    await expect(rail(page)).toContainText('IMG_0002');

    await openMenu(page);
    await expect(menu(page).getByRole('button', { name: 'Survey' })).toHaveCount(0);
    await menu(page).getByRole('button', { name: 'Show filmstrip' }).tap();
    await expect(menu(page)).toHaveCount(0);
    await longPressThumb(page, 'IMG_0005.ARW');
    const add = page.getByRole('dialog', { name: 'Actions' }).getByRole('button', {
      name: /^Add to compare/
    });
    await expect(add).toBeDisabled();
    await expect(add).toContainText('2 panes max');
  });

  test('the bulk sheet compares the first two picks and offers no survey', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    await page.getByRole('button', { name: 'Select photos' }).tap();
    for (const name of ['IMG_0001.ARW', 'IMG_0002.ARW', 'IMG_0003.ARW']) {
      await tapCentre(page, page.locator(`[role="group"][data-name="${name}"]`));
    }
    await expect(page.locator('header[data-app-bar]')).toContainText('3 selected');
    await page
      .getByRole('toolbar', { name: 'Selection actions' })
      .getByRole('button', { name: 'More', exact: true })
      .tap();

    const sheet = page.getByRole('dialog', { name: 'More actions' });
    await expect(sheet.getByRole('button', { name: /^Survey/ })).toHaveCount(0);
    const compareRow = sheet.getByRole('button', { name: /^Compare/ });
    await expect(compareRow).toContainText('first 2');
    await compareRow.tap();
    await expect(panes(page)).toHaveCount(2);
  });

  test('Drop on the pane rail returns to a single photo', async ({ page }) => {
    await openLoupe(page);
    await topBar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    await page.getByRole('dialog', { name: 'Menu' }).getByRole('button', { name: 'Compare' }).tap();
    await expect(panes(page)).toHaveCount(2);

    await rail(page).getByRole('button', { name: 'Drop', exact: true }).tap();
    await expect(rail(page)).toHaveCount(0);
    await expect(modes(page)).toHaveCount(0);
    await expect(topBar(page).getByRole('heading')).toHaveText('IMG_0003.ARW');
  });
});

test.describe('compare on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the centred segmented control lays compare side by side', async ({ page }) => {
    await openLoupe(page);
    const group = modes(page);
    await expect(group).toBeVisible();
    const [groupBox, viewport] = [await group.boundingBox(), page.viewportSize()];
    const centre = (groupBox?.x ?? 0) + (groupBox?.width ?? 0) / 2;
    expect(Math.abs(centre - (viewport?.width ?? 0) / 2)).toBeLessThan(4);

    await openMenu(page);
    await expect(menu(page).getByRole('button', { name: 'Auto-advance' })).toBeVisible();
    await expect(menu(page).getByRole('button', { name: 'Compare' })).toHaveCount(0);
    await menu(page).getByRole('button', { name: 'Cancel', exact: true }).tap();

    await group.getByRole('radio', { name: 'Compare' }).tap();
    await expect(panes(page)).toHaveCount(2);
    const [left, right] = await boxes(page.locator('div.group', { has: panes(page) }));
    expect(Math.abs((left?.y ?? 0) - (right?.y ?? -1))).toBeLessThan(2);
    expect(right?.x ?? 0).toBeGreaterThan(left?.x ?? 0);
    const bar = page.getByRole('navigation', { name: 'Photo actions' });
    for (const name of ['Sync', 'Promote', 'Drop']) {
      await expect(bar.getByRole('button', { name, exact: true })).toBeVisible();
    }

    await longPressThumb(page, 'IMG_0005.ARW');
    const sheet = page.getByRole('dialog', { name: 'Actions' });
    await expect(sheet.getByRole('button', { name: /^Add to compare/ })).toContainText('3 panes');
  });

  test('survey shows four panes in portrait and every member again in landscape', async ({
    page
  }) => {
    await openLoupe(page);
    await modes(page).getByRole('radio', { name: 'Survey' }).tap();
    await expect(panes(page)).toHaveCount(5);

    await page.setViewportSize({ width: 820, height: 1180 });
    await expect(page.locator('html')).not.toHaveAttribute('data-wide', /.*/);
    await expect(panes(page)).toHaveCount(4);
    await expect(modes(page).getByRole('radio', { name: 'Survey' })).toBeChecked();

    await page.setViewportSize({ width: 1180, height: 820 });
    await expect(panes(page)).toHaveCount(5);
  });

  test('portrait offers survey from the menu and caps it at four panes', async ({ page }) => {
    await page.setViewportSize({ width: 820, height: 1180 });
    await openLoupe(page);
    await openMenu(page);
    await menu(page).getByRole('button', { name: 'Survey' }).tap();
    await expect(panes(page)).toHaveCount(4);
    await expect(modes(page).getByRole('radio')).toHaveCount(3);
  });
});
