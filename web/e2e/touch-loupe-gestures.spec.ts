import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_EXIF, numberedAssets } from './helpers';
import {
  drag,
  pinch,
  tapAt,
  tapCentre,
  touchDown,
  touchMoveTo,
  touchUp,
  twoFingerTap,
  type TouchPoint
} from './touch';
import { open } from './touchShell';

const ASSETS = numberedAssets(6, { exifInfo: ASSET_EXIF });

function topBar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Loupe toolbar' });
}

function heading(page: Page): Locator {
  return topBar(page).getByRole('heading');
}

function pane(page: Page): Locator {
  return page.getByRole('button', { name: /^Zoom (in|out)$/ });
}

function track(page: Page): Locator {
  return page.getByTestId('loupe-track');
}

async function openLoupe(page: Page, name: string): Promise<void> {
  await open(page, '/photos', { assets: ASSETS });
  const tile = page.locator(`[role="group"][data-name="${name}"]`);
  await expect(tile).toBeVisible();
  await tapCentre(page, tile);
  await expect(heading(page)).toHaveText(name);
}

async function paneBox(page: Page): Promise<{ middle: TouchPoint; width: number }> {
  const box = await pane(page).boundingBox();
  if (!box) throw new Error('no loupe pane');
  return { middle: { x: box.x + box.width / 2, y: box.y + box.height / 2 }, width: box.width };
}

async function centre(page: Page): Promise<TouchPoint> {
  return (await paneBox(page)).middle;
}

function shifted(point: TouchPoint, dx: number, dy: number): TouchPoint {
  return { x: point.x + dx, y: point.y + dy };
}

function trackShift(page: Page): Promise<number> {
  return track(page).evaluate((element) => {
    const match = /translate3d\((-?[\d.]+)px/.exec(element.style.transform);
    return match ? Number(match[1]) : 0;
  });
}

async function expectSettled(page: Page): Promise<void> {
  await expect.poll(() => track(page).evaluate((element) => element.style.transform)).toBe('');
}

test.describe('loupe touch gestures', { tag: '@touch' }, () => {
  test('a swipe left steps to the next photo and a swipe right steps back', async ({ page }) => {
    await openLoupe(page, 'IMG_0002.ARW');
    const { middle, width } = await paneBox(page);
    const reach = Math.round(width * 0.4);

    await touchDown(page, middle);
    await touchMoveTo(page, shifted(middle, -40, 0));
    await touchMoveTo(page, shifted(middle, -reach, 0));
    await expect.poll(() => trackShift(page)).toBeCloseTo(-reach, 0);
    await expect(page.getByTestId('loupe-neighbour')).toHaveCount(2);
    await touchUp(page);
    await expect(heading(page)).toHaveText('IMG_0003.ARW');
    await expectSettled(page);

    await drag(page, middle, shifted(middle, reach, 0));
    await expect(heading(page)).toHaveText('IMG_0002.ARW');
  });

  test('the first photo rubber-bands and stays put', async ({ page }) => {
    await openLoupe(page, 'IMG_0001.ARW');
    const middle = await centre(page);

    await touchDown(page, middle);
    await touchMoveTo(page, shifted(middle, 40, 0));
    await touchMoveTo(page, shifted(middle, 200, 0));
    await expect.poll(() => trackShift(page)).toBeGreaterThan(0);
    expect(await trackShift(page)).toBeLessThan(150);
    await touchUp(page);

    await expectSettled(page);
    await expect(heading(page)).toHaveText('IMG_0001.ARW');
  });

  test('a pinch zooms in and a later swipe pans instead of stepping', async ({ page }) => {
    await openLoupe(page, 'IMG_0002.ARW');
    const middle = await centre(page);

    await pinch(page, middle, 80, 320);
    await expect(pane(page)).toHaveAccessibleName('Zoom out');
    const image = page.getByRole('img', { name: 'IMG_0002.ARW' });
    const before = await image.getAttribute('style');

    await drag(page, middle, shifted(middle, -160, 0));
    await expect(image).not.toHaveAttribute('style', before ?? '');
    await expect(pane(page)).toHaveAccessibleName('Zoom out');
    await expect(heading(page)).toHaveText('IMG_0002.ARW');
    await expectSettled(page);
  });

  test('a double tap zooms in and another fits again', async ({ page }) => {
    await openLoupe(page, 'IMG_0002.ARW');
    const middle = await centre(page);

    await tapAt(page, middle.x, middle.y);
    await tapAt(page, middle.x, middle.y);
    await expect(pane(page)).toHaveAccessibleName('Zoom out');

    await tapAt(page, middle.x, middle.y);
    await tapAt(page, middle.x, middle.y);
    await expect(pane(page)).toHaveAccessibleName('Zoom in');
  });

  test('a swipe up opens the info and a swipe down closes the loupe', async ({ page }) => {
    await openLoupe(page, 'IMG_0002.ARW');
    const middle = await centre(page);
    const info = topBar(page).getByRole('button', { name: /^Info/ });

    await drag(page, middle, shifted(middle, 0, -160));
    await expect(info).toHaveAttribute('aria-pressed', 'true');
    await page.getByRole('button', { name: 'Close info' }).tap();
    await expect(info).toHaveAttribute('aria-pressed', 'false');
    await expect(page.locator('body')).not.toHaveCSS('pointer-events', 'none');

    const settled = await centre(page);
    await drag(page, settled, shifted(settled, 0, 300));
    await expect(topBar(page)).toHaveCount(0);
    await expect(page.locator('[role="group"][data-name="IMG_0002.ARW"]')).toBeInViewport();
  });

  test('a two-finger tap hides the chrome and another shows it', async ({ page }) => {
    await openLoupe(page, 'IMG_0002.ARW');

    await twoFingerTap(page, await centre(page));
    await expect(topBar(page)).toHaveCount(0);
    await expect(page.getByRole('navigation', { name: 'Photo actions' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Exit fullscreen/ })).toBeVisible();

    await twoFingerTap(page, await centre(page));
    await expect(topBar(page)).toBeVisible();
    await expect(page.getByRole('navigation', { name: 'Photo actions' })).toBeVisible();
  });
});
