import { expect, test, type Page } from '@playwright/test';
import { DEFAULT_THRESHOLDS } from '../src/lib/gestures/thresholds';
import { numberedAssets } from './helpers';
import { datedAssets, dayShots } from './timeline';
import {
  longPressUntil,
  tapCentre,
  touchDown,
  touchMoveTo,
  touchUp,
  type TouchPoint
} from './touch';
import { expectBarDockedUnderGrid, open, tabBar } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 10, 'Bergen'),
  ...dayShots('2026-10-30', 10, 'Ålesund')
]);

test.use({ timezoneId: 'UTC' });

async function openPhotos(page: Page, from?: string): Promise<void> {
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await open(page, from ?? '/photos', { assets: ASSETS });
  if (from) await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

function entryIndex(page: Page): Promise<number> {
  return page.evaluate(() => navigation.currentEntry?.index ?? -1);
}

async function settled(page: Page): Promise<void> {
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve)));
}

function entryId(page: Page): Promise<string> {
  return page.evaluate(() => navigation.currentEntry?.id ?? '');
}

function tile(page: Page, name: string) {
  return page.locator(`[data-name="${name}"]`);
}

async function centre(page: Page, name: string): Promise<TouchPoint> {
  const box = await tile(page, name).boundingBox();
  if (!box) throw new Error(`no tile ${name}`);
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function holdPastLongPress(page: Page, point: TouchPoint): Promise<void> {
  await page.evaluate((ms) => {
    window.addEventListener(
      'pointerdown',
      () => setTimeout(() => Object.assign(window, { longPressElapsed: true }), ms),
      { once: true }
    );
  }, DEFAULT_THRESHOLDS.longPressMs);
  await touchDown(page, point);
  await expect.poll(() => page.evaluate(() => Reflect.get(window, 'longPressElapsed'))).toBe(true);
}

async function tapTile(page: Page, name: string): Promise<void> {
  await tapCentre(page, tile(page, name));
}

function appBar(page: Page) {
  return page.locator('header[data-app-bar]');
}

function bulkBar(page: Page) {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

async function selectedNames(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[role="group"][data-selected]')]
      .map((node) => node.getAttribute('data-name') ?? '')
      .sort()
  );
}

async function rowGap(page: Page): Promise<TouchPoint> {
  const gap = await page.evaluate(() => {
    const boxes = [...document.querySelectorAll<HTMLElement>('[role="group"][data-name]')].map(
      (node) => node.getBoundingClientRect()
    );
    for (const left of boxes) {
      const right = boxes.find(
        (box) =>
          Math.abs(box.top - left.top) < 1 && box.left > left.right && box.left - left.right < 20
      );
      if (right) return { x: (left.right + right.left) / 2, y: left.top + left.height / 2 };
    }
    return null;
  });
  if (!gap) throw new Error('no gap between tiles');
  return gap;
}

async function selectTwo(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Select photos' }).tap();
  await tapTile(page, 'IMG_0001.ARW');
  await tapTile(page, 'IMG_0002.ARW');
  await expect(appBar(page)).toContainText('2 selected');
}

test.describe('touch selection', () => {
  test('@touch a long press picks the photo and enters select mode', async ({ page }) => {
    await openPhotos(page);
    await longPressUntil(page, tile(page, 'IMG_0003.ARW'), appBar(page).getByText('1 selected'));

    await expect(page).toHaveURL(/\/photos$/);
    await expect(tile(page, 'IMG_0003.ARW')).toHaveAttribute('data-selected', 'true');
    await expect(bulkBar(page)).toBeVisible();
    await expect(
      tile(page, 'IMG_0004.ARW').getByRole('button', { name: 'Select', exact: true })
    ).toHaveCSS('opacity', '1');
  });

  test('@touch dragging on after the long press selects the range in between', async ({ page }) => {
    await openPhotos(page);
    await touchDown(page, await centre(page, 'IMG_0001.ARW'));
    await expect(appBar(page)).toContainText('1 selected');
    await touchMoveTo(page, await centre(page, 'IMG_0005.ARW'));
    await expect(page.getByText('+4 in range')).toBeVisible();
    await expect(tile(page, 'IMG_0003.ARW')).toHaveAttribute('data-range-preview', 'true');
    await expect(appBar(page)).toContainText('5 selected');
    await touchUp(page);

    await expect(appBar(page)).toContainText('5 selected');
    expect(await selectedNames(page)).toEqual([
      'IMG_0001.ARW',
      'IMG_0002.ARW',
      'IMG_0003.ARW',
      'IMG_0004.ARW',
      'IMG_0005.ARW'
    ]);
    await expect(page.getByText(/in range/)).toHaveCount(0);
  });

  test('@touch a long press in a gap between photos leaves the touch to scrolling', async ({
    page
  }) => {
    await openPhotos(page);
    const gap = await rowGap(page);
    await page.evaluate(() => {
      const seen: boolean[] = [];
      Object.assign(window, { touchMovesPrevented: seen });
      window.addEventListener('touchmove', (event) => seen.push(event.defaultPrevented), {
        passive: true
      });
    });

    await holdPastLongPress(page, gap);
    await touchMoveTo(page, { x: gap.x, y: gap.y - 40 });
    await touchMoveTo(page, { x: gap.x, y: gap.y - 80 });
    await touchUp(page);

    const prevented = await page.evaluate(
      () => Reflect.get(window, 'touchMovesPrevented') as boolean[]
    );
    expect(prevented.length).toBeGreaterThan(0);
    expect(prevented).not.toContain(true);
    await expect(bulkBar(page)).toHaveCount(0);
    expect(await selectedNames(page)).toEqual([]);
  });

  test('@touch a tap outside select mode opens the loupe instead of picking it', async ({
    page
  }) => {
    await openPhotos(page);
    const check = tile(page, 'IMG_0002.ARW').getByRole('button', { name: 'Select', exact: true });
    await expect(check).toHaveCount(0);

    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(check).toHaveCSS('opacity', '1');
    await appBar(page).getByRole('button', { name: 'Exit selection' }).tap();
    await expect(check).toHaveCount(0);

    await tapTile(page, 'IMG_0002.ARW');
    await expect(
      page.getByRole('navigation', { name: 'Loupe toolbar' }).getByRole('heading')
    ).toHaveText('IMG_0002.ARW');
    await expect(page).toHaveURL(/\/photos$/);
    expect(await selectedNames(page)).toEqual([]);
  });

  test('@touch Select enters select mode, taps toggle and the X leaves it', async ({ page }) => {
    await openPhotos(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(appBar(page)).toContainText('0 selected');
    await expect(bulkBar(page)).toBeVisible();

    await tapTile(page, 'IMG_0002.ARW');
    await expect(appBar(page)).toContainText('1 selected');
    await expect(appBar(page).getByRole('button', { name: 'Clear selection' })).toBeVisible();
    await expect(page).toHaveURL(/\/photos$/);
    await tapTile(page, 'IMG_0002.ARW');
    await expect(appBar(page)).toContainText('0 selected');

    await appBar(page).getByRole('button', { name: 'Exit selection' }).tap();
    await expect(bulkBar(page)).toHaveCount(0);
    await expect(page.getByText(/^\d+ selected$/)).toHaveCount(0);
  });

  test('@touch @touch-smoke Back leaves select mode and stays on the page', async ({ page }) => {
    await openPhotos(page, '/albums');
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(bulkBar(page)).toBeVisible();

    await page.goBack();
    await expect(bulkBar(page)).toHaveCount(0);
    await expect(page.getByText(/^\d+ selected$/)).toHaveCount(0);
    await expect(page).toHaveURL(/\/photos$/);
    await expect(page.getByRole('button', { name: 'Select photos' })).toBeVisible();

    await page.goBack();
    await expect(page).toHaveURL(/\/albums$/);
  });

  test('@touch Back drops the picked photos with select mode', async ({ page }) => {
    await openPhotos(page, '/albums');
    await selectTwo(page);

    await page.goBack();
    await expect(bulkBar(page)).toHaveCount(0);
    expect(await selectedNames(page)).toEqual([]);
    await expect(page).toHaveURL(/\/photos$/);
  });

  test('@touch the X and Back stop select all and leave select mode', async ({ page }) => {
    const paged = numberedAssets(4);
    const held = new Map<number, () => void>();
    await open(page, '/photos', {
      assets: paged,
      searchPages: paged.map((asset) => [asset]),
      onMetadata: async (body) => {
        if (typeof body.cursor !== 'string' || !body.sort) return;
        const pageNumber = Number(body.cursor);
        await new Promise<void>((resolve) => held.set(pageNumber, resolve));
      }
    });
    const select = page.getByRole('button', { name: 'Select photos' });
    const selectAll = appBar(page).getByRole('button', { name: 'Select all' });

    await select.tap();
    await tapTile(page, 'IMG_0001.ARW');
    await selectAll.tap();
    await expect.poll(() => held.has(2)).toBe(true);
    await appBar(page).getByRole('button', { name: 'Clear selection' }).tap();
    await expect(bulkBar(page)).toHaveCount(0);
    held.get(2)?.();
    await expect(tile(page, 'IMG_0002.ARW')).toBeVisible();
    expect(await selectedNames(page)).toEqual([]);

    await select.tap();
    await tapTile(page, 'IMG_0001.ARW');
    await selectAll.tap();
    await expect.poll(() => held.has(3)).toBe(true);
    await page.goBack();
    await expect(bulkBar(page)).toHaveCount(0);
    held.get(3)?.();
    await expect(tile(page, 'IMG_0003.ARW')).toBeVisible();
    expect(await selectedNames(page)).toEqual([]);
    await expect(page).toHaveURL(/\/photos$/);
  });

  test('@touch the X leaves no extra Back step behind', async ({ page }) => {
    await openPhotos(page, '/albums');
    const before = await entryIndex(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect.poll(() => entryIndex(page)).toBe(before + 1);

    await appBar(page).getByRole('button', { name: 'Exit selection' }).tap();
    await expect(bulkBar(page)).toHaveCount(0);
    await expect.poll(() => entryIndex(page)).toBe(before);
    await expect(page).toHaveURL(/\/photos$/);

    await page.goBack();
    await expect(page).toHaveURL(/\/albums$/);
  });

  test('@touch a filter removed while selecting stays removed', async ({ page }) => {
    await openPhotos(page);
    await page.goto('/photos?city=Bergen');
    await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
    const before = await entryIndex(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(bulkBar(page)).toBeVisible();
    await expect.poll(() => entryIndex(page)).toBe(before + 1);

    await page.getByRole('button', { name: /^Remove / }).tap();
    await expect(page).toHaveURL(/\/photos$/);
    await expect(bulkBar(page)).toHaveCount(0);
    await settled(page);
    await expect.poll(() => entryIndex(page)).toBe(before + 2);
    await expect(page).toHaveURL(/\/photos$/);
    await expect(page.getByRole('button', { name: /^Remove / })).toHaveCount(0);
  });

  test('@touch @tablet leaving the page from select mode stays on the new page', async ({
    page
  }) => {
    await openPhotos(page, '/albums');
    const before = await entryIndex(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(bulkBar(page)).toBeVisible();
    await expect.poll(() => entryIndex(page)).toBe(before + 1);

    await page
      .getByRole('complementary', { name: 'Library' })
      .getByRole('link', { name: 'Favorites', exact: true })
      .tap();
    await expect(page).toHaveURL(/\/favorites$/);
    await expect(page.getByRole('heading', { name: 'Favorites' })).toBeVisible();
    await settled(page);
    await expect.poll(() => entryIndex(page)).toBe(before + 1);
    await expect(page).toHaveURL(/\/favorites$/);

    await page.goBack();
    await expect(page).toHaveURL(/\/photos$/);
    await expect(bulkBar(page)).toHaveCount(0);
    expect(await entryIndex(page)).toBe(before);

    await page.goBack();
    await expect(page).toHaveURL(/\/albums$/);
  });

  test('@touch Back leaves select mode entered on an entry left from before a reload', async ({
    page
  }) => {
    await openPhotos(page);
    await page.goto('/photos?city=Bergen');
    await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
    const before = await entryIndex(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(bulkBar(page)).toBeVisible();
    await expect.poll(() => entryIndex(page)).toBe(before + 1);
    await page.getByRole('button', { name: /^Remove / }).tap();
    await expect(page).toHaveURL(/\/photos$/);
    await expect.poll(() => entryIndex(page)).toBe(before + 2);

    await page.reload();
    await expect(page.getByTestId('day-header').first()).toBeVisible();
    await page.goBack();
    await expect(page).toHaveURL(/\/photos\?city=Bergen$/);
    await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
    const stale = await entryId(page);
    await page.getByRole('button', { name: 'Select photos' }).tap();
    await expect(bulkBar(page)).toBeVisible();
    await expect.poll(() => entryId(page)).not.toBe(stale);
    expect(await entryIndex(page)).toBe(before + 1);

    await page.goBack();
    await expect(bulkBar(page)).toHaveCount(0);
    await expect(page).toHaveURL(/\/photos\?city=Bergen$/);
    await expect.poll(() => entryIndex(page)).toBe(before);
  });

  test('@touch @phone the bulk bar takes the tab bar place with the phone actions', async ({
    page
  }) => {
    await openPhotos(page);
    await expect(tabBar(page)).toBeVisible();
    await selectTwo(page);

    await expect(tabBar(page)).toHaveCount(0);
    const bar = await bulkBar(page).boundingBox();
    const viewport = page.viewportSize();
    expect((bar?.y ?? 0) + (bar?.height ?? 0)).toBeCloseTo(viewport?.height ?? 0, 0);
    for (const name of ['Rate', 'Label', 'Reject', 'Album', 'Export', 'More']) {
      await expect(bulkBar(page).getByRole('button', { name, exact: true })).toBeInViewport();
    }
    await expect(bulkBar(page).getByRole('button', { name: 'Tags', exact: true })).toBeHidden();

    await appBar(page).getByRole('button', { name: 'Clear selection' }).tap();
    await expect(tabBar(page)).toBeVisible();
  });

  test('@touch @tablet the bulk bar docks under the grid column with Tags', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);

    await expectBarDockedUnderGrid(page);
    await expect(bulkBar(page).getByRole('button', { name: 'Tags', exact: true })).toBeVisible();
  });

  test('@touch Rate opens a picker that rates every selected photo', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'Rate', exact: true }).tap();
    await page.getByRole('radio', { name: '3 stars' }).last().tap();

    await expect(page.getByText('Updated 2 assets')).toBeVisible();
    await expect(page.getByRole('img', { name: '3 stars' })).toHaveCount(2);
    await expect(appBar(page)).toContainText('2 selected');
  });

  test('@touch No rating clears the rating of every selected photo', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);
    const rate = bulkBar(page).getByRole('button', { name: 'Rate', exact: true });
    await rate.tap();
    await page.getByRole('radio', { name: '3 stars' }).last().tap();
    await expect(page.getByRole('img', { name: '3 stars' })).toHaveCount(2);

    await rate.tap();
    await page.getByRole('button', { name: 'No rating', exact: true }).tap();

    await expect(page.getByRole('img', { name: '3 stars' })).toHaveCount(0);
    await expect(appBar(page)).toContainText('2 selected');
  });

  test('@touch Album opens the album sheet for the selection', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'Album', exact: true }).tap();
    const sheet = page.getByRole('dialog', { name: 'Add to album' });
    await expect(sheet).toBeVisible();
    await expect(
      sheet.getByText('Pick albums, then add the 2 selected photos to them')
    ).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Add to albums' })).toBeDisabled();
  });

  test('@touch More lists the remaining actions and keeps the selection on Cancel', async ({
    page
  }) => {
    await openPhotos(page);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();

    const sheet = page.getByRole('dialog', { name: 'More actions' });
    await expect(sheet).toBeVisible();
    await expect(sheet).not.toContainText('2 photos');
    for (const name of [
      'Apply preset',
      'Reset edits',
      'Compare',
      'Survey',
      'Favorite',
      'Create virtual copy',
      'Move to trash'
    ]) {
      await expect(sheet.getByRole('button', { name, exact: true })).toBeEnabled();
    }
    const paste = sheet.getByRole('button', { name: /^Paste edits/ });
    await expect(paste).toBeDisabled();
    await expect(paste).toContainText('Nothing copied');

    await sheet.getByRole('button', { name: 'Cancel' }).tap();
    await expect(sheet).toHaveCount(0);
    await expect(appBar(page)).toContainText('2 selected');
  });

  test('@touch @phone Tags in More opens the tag sheet', async ({ page }) => {
    await openPhotos(page);
    await selectTwo(page);
    await bulkBar(page).getByRole('button', { name: 'More', exact: true }).tap();
    await page
      .getByRole('dialog', { name: 'More actions' })
      .getByRole('button', { name: 'Tags', exact: true })
      .tap();

    const sheet = page.getByRole('dialog', { name: 'Tags' });
    await expect(sheet).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Add to selected' })).toBeVisible();
  });
});
