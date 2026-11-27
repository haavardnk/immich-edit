import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks, numberedAssets } from './helpers';
import { openTileLoupe } from './touch';
import { backButton } from './touchShell';

const PLACES = [
  { city: 'Oslo', latitude: 59.9139, longitude: 10.7522 },
  { city: 'Bergen', latitude: 60.39, longitude: 5.32 }
];

const PHOTOS = numberedAssets(2).map((asset, index) => ({
  ...asset,
  exifInfo: PLACES[index]!
}));

const MARKERS = PHOTOS.map((photo, index) => ({
  id: photo.id,
  lat: PLACES[index]!.latitude,
  lon: PLACES[index]!.longitude
}));

async function touching(page: Page): Promise<boolean> {
  return (await page.locator('html').getAttribute('data-layout')) === 'touch';
}

async function press(page: Page, target: Locator): Promise<void> {
  await ((await touching(page)) ? target.tap() : target.click());
}

function infoButton(page: Page): Locator {
  return page
    .getByRole('navigation', { name: 'Loupe toolbar' })
    .getByRole('button', { name: /^Info/ });
}

async function recordPageMotion(page: Page): Promise<() => Promise<string[]>> {
  await page.evaluate(() => {
    const seen: string[] = [];
    Reflect.set(window, '__pageMotion', seen);
    new MutationObserver(() => {
      const motion = document.documentElement.dataset.pageMotion;
      if (motion && seen.at(-1) !== motion) seen.push(motion);
    }).observe(document.documentElement, { attributeFilter: ['data-page-motion'] });
  });
  return () => page.evaluate(() => [...(Reflect.get(window, '__pageMotion') as string[])]);
}

interface ReturnFrame {
  moving: boolean;
  pageGroup: boolean;
  loupe: boolean;
  editor: boolean;
  info: string | null;
}

async function recordReturn(page: Page): Promise<() => Promise<ReturnFrame[]>> {
  await page.evaluate(() => {
    const frames: ReturnFrame[] = [];
    Reflect.set(window, '__returnFrames', frames);
    const started = performance.now();
    const sample = (): void => {
      const info = document.querySelector('[data-sheet-panel], aside[aria-label="Info"]');
      const box = info?.getBoundingClientRect();
      const motion = document
        .getAnimations()
        .filter((animation) => (animation as CSSAnimation).animationName?.startsWith('page-'));
      frames.push({
        moving: motion.length > 0,
        pageGroup: motion.some((animation) =>
          (animation.effect as KeyframeEffect | null)?.pseudoElement?.includes('(page)')
        ),
        loupe: document.querySelector('[data-loupe-root]') !== null,
        editor: document.querySelector('nav[aria-label="Editor toolbar"]') !== null,
        info: box ? `${Math.round(box.x)},${Math.round(box.y)} ${Math.round(box.width)}` : null
      });
      if (performance.now() - started < 2500) requestAnimationFrame(sample);
    };
    requestAnimationFrame(sample);
  });
  return async () => {
    await page.waitForFunction(
      () => (Reflect.get(window, '__returnFrames') as ReturnFrame[]).length > 0
    );
    await page.waitForTimeout(2600);
    return page.evaluate(() => Reflect.get(window, '__returnFrames') as ReturnFrame[]);
  };
}

async function openBergenInfo(page: Page): Promise<void> {
  await installMocks(page, { assets: PHOTOS, mapMarkers: MARKERS });
  await page.goto('/photos');
  await openTileLoupe(page, page.locator('[role="group"][data-name="IMG_0002.ARW"]'));
  await press(page, infoButton(page));
  await expect(infoButton(page)).toHaveAttribute('aria-pressed', 'true');
}

async function goBack(page: Page): Promise<void> {
  const back = backButton(page);
  if (await back.isVisible()) await press(page, back);
  else await page.goBack();
}

for (const tag of [[], ['@touch']]) {
  test.describe(`the info panel map ${tag.length ? 'on touch' : 'on desktop'}`, { tag }, () => {
    test('pushes the map and Back returns to the photo with its info open', async ({ page }) => {
      await openBergenInfo(page);
      const motions = await recordPageMotion(page);
      await press(page, page.getByRole('link', { name: 'Open Bergen on the map' }));

      await expect(page).toHaveURL(/\/map\?at=60\.39000(,|%2C)5\.32000/);
      await expect(page.locator('[data-loupe-root]')).toHaveCount(0);
      await expect.poll(motions).toEqual([expect.stringMatching(/^(push|rise)$/)]);
      await expect(page.locator('html')).not.toHaveAttribute('data-page-motion');

      const frames = await recordReturn(page);
      await goBack(page);
      await expect(page).toHaveURL('/photos');
      await expect(infoButton(page)).toHaveAttribute('aria-pressed', 'true');
      await expect(page.getByRole('link', { name: 'Open Bergen on the map' })).toBeVisible();
      const [forward, back] = await motions();
      expect([forward, back]).toEqual(forward === 'push' ? ['push', 'pop'] : ['rise', 'sink']);

      const seen = await frames();
      const moving = seen.filter((frame) => frame.moving);
      expect(moving.length).toBeGreaterThan(0);
      expect(moving.every((frame) => frame.loupe && !frame.pageGroup)).toBe(true);
      const boxes = new Set(seen.flatMap((frame) => (frame.info ? [frame.info] : [])));
      expect([...boxes]).toHaveLength(1);
    });

    test('Back from the editor info map shows the editor with its info at rest', async ({
      page
    }) => {
      await installMocks(page, { assets: PHOTOS, mapMarkers: MARKERS });
      await page.goto('/photos');
      await openTileLoupe(page, page.locator('[role="group"][data-name="IMG_0002.ARW"]'));
      await press(page, page.getByRole('button', { name: 'Edit', exact: true }));
      const toolbar = page.getByRole('navigation', { name: 'Editor toolbar' });
      await press(page, toolbar.getByRole('button', { name: 'More editor actions' }));
      await press(page, page.getByRole('button', { name: 'Info', exact: true }));
      await press(page, page.getByRole('link', { name: 'Open Bergen on the map' }));
      await expect(page).toHaveURL(/\/map\?/);

      await page.route(`**/api/assets/${PHOTOS[1]!.id}`, async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 800));
        await route.fallback();
      });

      const frames = await recordReturn(page);
      await goBack(page);
      await expect(page).toHaveURL(/\/assets\//);
      await expect(page.getByRole('link', { name: 'Open Bergen on the map' })).toBeVisible();
      const seen = await frames();
      const editing = seen.filter((frame) => frame.editor);
      expect(editing.length).toBeGreaterThan(0);
      expect(editing.every((frame) => frame.info !== null)).toBe(true);
      const boxes = new Set(seen.flatMap((frame) => (frame.info ? [frame.info] : [])));
      expect([...boxes]).toHaveLength(1);
    });
  });
}
