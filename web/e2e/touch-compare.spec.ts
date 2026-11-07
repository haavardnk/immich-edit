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

async function boxes(locator: Locator): Promise<{ x: number; y: number; height: number }[]> {
  const all = await locator.all();
  const found = await Promise.all(all.map((item) => item.boundingBox()));
  return found.map((box) => ({ x: box?.x ?? 0, y: box?.y ?? 0, height: box?.height ?? 0 }));
}

test.describe('compare and survey on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('survey holds four panes and Compare from the segmented control stacks two', async ({
    page
  }) => {
    await openLoupe(page);
    await expect(modes(page)).toHaveCount(0);
    await topBar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    await page.getByRole('dialog', { name: 'Menu' }).getByRole('button', { name: 'Survey' }).tap();

    await expect(panes(page)).toHaveCount(4);
    await expect(topBar(page).getByRole('heading')).toHaveCount(0);
    await expect(modes(page).getByRole('radio', { name: 'Survey' })).toBeChecked();
    await expect(rail(page)).toContainText('IMG_0002');
    await expect(rail(page)).not.toContainText('of 4');

    await modes(page).getByRole('radio', { name: 'Compare' }).tap();
    await expect(modes(page).getByRole('radio', { name: 'Compare' })).toBeChecked();
    await expect(panes(page)).toHaveCount(2);
    const [top, bottom] = await boxes(page.locator('div.group', { has: panes(page) }));
    expect(Math.abs((top?.x ?? 0) - (bottom?.x ?? -1))).toBeLessThan(2);
    expect(bottom?.y ?? 0).toBeGreaterThanOrEqual((top?.y ?? 0) + (top?.height ?? 0) - 1);
    await expect(rail(page)).toContainText('IMG_0002');
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

    await topBar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    const menu = page.getByRole('dialog', { name: 'Menu' });
    await expect(menu.getByRole('button', { name: 'Auto-advance' })).toBeVisible();
    await expect(menu.getByRole('button', { name: 'Compare' })).toHaveCount(0);
    await menu.getByRole('button', { name: 'Cancel', exact: true }).tap();

    await group.getByRole('radio', { name: 'Compare' }).tap();
    await expect(panes(page)).toHaveCount(2);
    const [left, right] = await boxes(page.locator('div.group', { has: panes(page) }));
    expect(Math.abs((left?.y ?? 0) - (right?.y ?? -1))).toBeLessThan(2);
    expect(right?.x ?? 0).toBeGreaterThan(left?.x ?? 0);
    const bar = page.getByRole('navigation', { name: 'Photo actions' });
    for (const name of ['Sync', 'Promote', 'Drop']) {
      await expect(bar.getByRole('button', { name, exact: true })).toBeVisible();
    }

    const thumb = page
      .getByTestId('filmstrip-scroll')
      .getByRole('button', { name: 'IMG_0005.ARW', exact: true });
    const box = await thumb.boundingBox();
    if (!box) throw new Error('no filmstrip thumbnail');
    await longPress(page, box.x + box.width / 2, box.y + box.height / 2);
    const sheet = page.getByRole('dialog', { name: 'Actions' });
    await expect(sheet.getByRole('button', { name: /^Add to compare/ })).toContainText('3 panes');
  });
});
