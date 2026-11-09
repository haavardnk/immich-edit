import { expect, test, type Locator, type Page } from '@playwright/test';
import { loupeToolbar, tile } from './browse';
import { numberedAssets } from './helpers';
import { longPress } from './touch';
import { open, tapTile } from './touchShell';

const ASSETS = numberedAssets(6);
const [, SECOND] = ASSETS;

async function openPhotos(page: Page): Promise<void> {
  await open(page, '/photos', { assets: ASSETS, metadataPushConsented: true });
  await expect(tile(page, 'IMG_0002.ARW')).toBeVisible();
}

function cullBar(page: Page): Locator {
  return page.getByRole('navigation', { name: 'Photo actions' });
}

async function openLoupe(page: Page): Promise<void> {
  await openPhotos(page);
  await tapTile(page, 'IMG_0002.ARW');
  await expect(loupeToolbar(page).getByRole('heading')).toHaveText('IMG_0002.ARW');
}

async function expectTapTarget(target: Locator): Promise<void> {
  const box = await target.boundingBox();
  expect(box?.width ?? 0).toBeGreaterThanOrEqual(44);
  expect(box?.height ?? 0).toBeGreaterThanOrEqual(44);
}

test.describe('the touch loupe on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('a tap on a tile opens the loupe and the cull bar rates, favourites and rejects', async ({
    page
  }) => {
    await openLoupe(page);
    await expect(page).toHaveURL(/\/photos$/);
    await expect(page.getByRole('button', { name: 'Quick review' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Auto-advance/ })).toHaveCount(0);
    await expect(page.getByTestId('filmstrip-scroll')).toHaveCount(0);

    const bar = cullBar(page);
    await expect(bar).toHaveAttribute('data-density', 'stacked');
    for (const name of ['Favorite', 'Reject', 'Colour label', 'Tags', 'Edit']) {
      await expectTapTarget(bar.getByRole('button', { name, exact: true }));
    }

    await bar.getByRole('radio', { name: '3 stars' }).tap();
    await expect(bar.getByRole('radio', { name: '3 stars' })).toHaveAttribute(
      'aria-checked',
      'true'
    );
    await bar.getByRole('button', { name: 'Favorite', exact: true }).tap();
    await expect(bar.getByRole('button', { name: 'Unfavorite', exact: true })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    await bar.getByRole('button', { name: 'Reject', exact: true }).tap();
    await expect(bar.getByRole('button', { name: 'Unreject', exact: true })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  test('the more menu turns on auto-advance, so the next rating steps to the next photo', async ({
    page
  }) => {
    await openLoupe(page);
    await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Menu' });
    for (const name of ['Toggle zoom', 'Show filmstrip', 'Fullscreen', 'Compare']) {
      await expect(sheet.getByRole('button', { name, exact: true })).toBeVisible();
    }
    await sheet.getByRole('button', { name: 'Auto-advance', exact: true }).tap();
    await expect(sheet).toBeHidden();

    await cullBar(page).getByRole('radio', { name: '4 stars' }).tap();
    await expect(page.getByTestId('rating-hud')).toContainText('4 stars · next photo');
    await expect(loupeToolbar(page).getByRole('heading')).toHaveText('IMG_0003.ARW');
  });

  test('the more menu shows the filmstrip and Back returns to the grid', async ({ page }) => {
    await openLoupe(page);
    await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Menu' });
    await sheet.getByRole('button', { name: 'Show filmstrip', exact: true }).tap();
    await expect(sheet).toBeHidden();
    await expect(page.getByTestId('filmstrip-scroll')).toBeVisible();

    await loupeToolbar(page).getByRole('button', { name: /^Back/ }).tap();
    await expect(loupeToolbar(page)).toHaveCount(0);
    await expect(tile(page, 'IMG_0002.ARW')).toBeInViewport();
  });

  test('a narrow filmstrip thumb shows its whole rating', async ({ page }) => {
    await open(page, '/photos', {
      assets: numberedAssets(4, {
        exifInfo: { exifImageWidth: 2000, exifImageHeight: 6000, rating: 3 }
      })
    });
    await tapTile(page, 'IMG_0002.ARW');
    await loupeToolbar(page).getByRole('button', { name: 'More loupe actions' }).tap();
    await page
      .getByRole('dialog', { name: 'Menu' })
      .getByRole('button', { name: 'Show filmstrip', exact: true })
      .tap();

    const thumb = page
      .getByTestId('filmstrip-scroll')
      .getByRole('button', { name: 'IMG_0003.ARW' });
    const badge = thumb.getByRole('img', { name: '3 stars' });
    await expect(badge).toBeVisible();
    const [outer, inner] = await Promise.all([thumb.boundingBox(), badge.boundingBox()]);
    expect((inner?.x ?? 0) + (inner?.width ?? 0)).toBeLessThanOrEqual(
      (outer?.x ?? 0) + (outer?.width ?? 0)
    );
  });

  test('the photo sits between the top bar and the cull bar', async ({ page }) => {
    await openLoupe(page);
    const pane = page.getByRole('button', { name: 'Zoom in' });
    const [bar, photo, cull] = await Promise.all([
      loupeToolbar(page).boundingBox(),
      pane.boundingBox(),
      cullBar(page).boundingBox()
    ]);
    const viewport = page.viewportSize();
    expect(photo?.y ?? 0).toBeGreaterThanOrEqual((bar?.y ?? 0) + (bar?.height ?? 0) - 1);
    expect((photo?.y ?? 0) + (photo?.height ?? 0)).toBeLessThanOrEqual((cull?.y ?? 0) + 1);
    expect((cull?.y ?? 0) + (cull?.height ?? 0)).toBeCloseTo(viewport?.height ?? 0, 0);
  });

  test('Edit opens the photo in the editor', async ({ page }) => {
    await openLoupe(page);
    await cullBar(page).getByRole('button', { name: 'Edit', exact: true }).tap();
    await expect(page).toHaveURL(new RegExp(`/assets/${SECOND?.id}\\?from=`));
  });
});

test.describe('the touch loupe on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('lays the cull bar out in one row above a visible filmstrip', async ({ page }) => {
    await openLoupe(page);
    const bar = cullBar(page);
    await expect(bar).toHaveAttribute('data-density', 'row');
    await expect(
      loupeToolbar(page).getByRole('button', { name: 'Edit', exact: true })
    ).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Edit', exact: true })).toHaveCount(0);
    await expect(bar.getByRole('button', { name: /^Collapse filmstrip/ })).toHaveAttribute(
      'aria-pressed',
      'true'
    );

    const strip = page.getByTestId('filmstrip-scroll');
    await expect(strip).toBeVisible();
    const [barBox, stripBox] = await Promise.all([bar.boundingBox(), strip.boundingBox()]);
    expect(Math.abs((barBox?.y ?? 0) + (barBox?.height ?? 0) - (stripBox?.y ?? 0))).toBeLessThan(4);
    const actionTops = await Promise.all(
      ['Favorite', 'Reject', 'Colour label', 'Tags'].map(async (name) => {
        const box = await bar.getByRole('button', { name, exact: true }).boundingBox();
        return Math.round(box?.y ?? -1);
      })
    );
    expect(new Set(actionTops).size).toBe(1);

    const thumb = strip.getByRole('button', { name: 'IMG_0004.ARW', exact: true });
    const box = await thumb.boundingBox();
    if (!box) throw new Error('no filmstrip thumbnail');
    await longPress(page, box.x + box.width / 2, box.y + box.height / 2);
    const menu = page.getByRole('dialog', { name: 'Actions' });
    await expect(menu.getByRole('button', { name: 'Open in editor' })).toBeVisible();
    await menu.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await expect(menu).toBeHidden();
  });
});
