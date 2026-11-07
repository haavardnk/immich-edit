import { expect, test, type Locator, type Page } from '@playwright/test';
import { numberedAssets } from './helpers';
import { openTileLoupe } from './touch';
import { open } from './touchShell';

type TopEdge = { position: string; top: number; background: string | null; colours: number };

const ASSETS = numberedAssets(3);

function topEdge(page: Page): Promise<TopEdge | null> {
  return page.evaluate(() => {
    const visible = (colour: string) =>
      colour !== 'transparent' && !/rgba\(.*,\s*0\)$/.test(colour) && !/\/\s*0\)$/.test(colour);
    const unblock = document.createElement('style');
    unblock.textContent = '* { pointer-events: auto !important; }';
    document.head.append(unblock);
    const hit = document.elementFromPoint(window.innerWidth / 2, 4);
    unblock.remove();
    const backgrounds: string[] = [];
    for (let element = hit; element; element = element.parentElement) {
      const rect = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      const sized = rect.width >= window.innerWidth * 0.9 && rect.height > 10;
      if (sized && visible(style.backgroundColor)) backgrounds.push(style.backgroundColor);
      if (sized && (style.position === 'fixed' || style.position === 'sticky')) {
        return {
          position: style.position,
          top: rect.top,
          background: backgrounds[0] ?? null,
          colours: new Set(backgrounds).size
        };
      }
    }
    return null;
  });
}

function background(target: Locator): Promise<string> {
  return target.evaluate((element) => getComputedStyle(element).backgroundColor);
}

test.describe(
  'the top edge gives iOS an opaque bar to tint the status bar with',
  { tag: ['@touch', '@phone'] },
  () => {
    for (const path of ['/photos', '/search', '/settings']) {
      test(
        `the app bar on ${path} is a sticky page-coloured surface`,
        { tag: '@touch-smoke' },
        async ({ page }) => {
          await open(page, path, { assets: ASSETS });
          const bar = page.locator('[data-app-bar]');
          await expect(bar).toBeVisible();

          expect(await topEdge(page)).toEqual({
            position: 'sticky',
            top: 0,
            background: await background(bar),
            colours: 1
          });
        }
      );
    }

    test('the loupe bars keep the app chrome colour and back the top edge', async ({ page }) => {
      await open(page, '/photos', { assets: ASSETS });
      const chrome = await background(page.locator('[data-app-bar]'));
      await openTileLoupe(page, page.locator('[role="group"][data-name="IMG_0002.ARW"]'));
      const toolbar = page.getByRole('navigation', { name: 'Loupe toolbar' });
      const cullBar = page.getByRole('navigation', { name: 'Photo actions' });
      await expect(toolbar.getByRole('heading')).toHaveText('IMG_0002.ARW');

      expect(await background(cullBar)).toBe(chrome);
      expect(await background(toolbar)).toBe(chrome);
      expect(await topEdge(page)).toEqual({
        position: 'fixed',
        top: 0,
        background: chrome,
        colours: 1
      });
    });
  }
);
