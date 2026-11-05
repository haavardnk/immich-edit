import { expect, test, type Page } from '@playwright/test';
import { installMocks, numberedAssets } from './helpers';
import { pinch } from './touch';
import { open } from './touchShell';

const ASSETS = numberedAssets(160);

function hud(page: Page) {
  return page.getByTestId('grid-size-hud');
}

async function tileAt(page: Page, x: number, y: number): Promise<string> {
  const read = (): Promise<string> =>
    page.evaluate(
      ([px, py]) => {
        const tile = document.elementFromPoint(px, py)?.closest('[role="group"]');
        return tile instanceof HTMLElement ? tile.title : '';
      },
      [x, y] as const
    );
  await expect.poll(read).not.toBe('');
  return read();
}

async function tileBox(page: Page, name: string): Promise<{ top: number; height: number }> {
  const box = await page.locator(`[role="group"][title="${name}"]`).boundingBox();
  return { top: box?.y ?? Number.NaN, height: box?.height ?? Number.NaN };
}

async function scrollGrid(page: Page, top: number): Promise<void> {
  await page.evaluate((value) => {
    let node = document.querySelector('[role="group"]')?.parentElement ?? null;
    while (node && !['auto', 'scroll'].includes(getComputedStyle(node).overflowY)) {
      node = node.parentElement;
    }
    node?.scrollTo({ top: value });
  }, top);
}

async function anchorRatio(page: Page, name: string, y: number): Promise<number> {
  const box = await tileBox(page, name);
  return (y - box.top) / box.height;
}

function sizeButton(page: Page, size: string) {
  return page.getByRole('button', { name: `Thumbnail size ${size}`, exact: true });
}

async function wheelBurst(page: Page, x: number, y: number, count: number): Promise<void> {
  await page.evaluate(
    ([px, py, n]) => {
      const target = document.elementFromPoint(px, py);
      for (let index = 0; index < n; index += 1) {
        target?.dispatchEvent(
          new WheelEvent('wheel', {
            deltaY: -100,
            ctrlKey: true,
            clientX: px,
            clientY: py,
            bubbles: true,
            cancelable: true
          })
        );
      }
    },
    [x, y, count] as const
  );
}

async function notchUntil(page: Page, deltaY: number, size: string): Promise<void> {
  await page.keyboard.down('Control');
  await expect(async () => {
    await page.mouse.wheel(0, deltaY);
    await expect(sizeButton(page, size)).toHaveAttribute('aria-pressed', 'true', {
      timeout: 1000
    });
  }).toPass();
  await page.keyboard.up('Control');
}

test.describe('grid sizing on desktop', () => {
  test('Ctrl+wheel steps the size once per notch burst around the photo under the pointer', async ({
    page
  }) => {
    await installMocks(page, { assets: ASSETS });
    await page.goto('/photos');
    await expect(sizeButton(page, 'M')).toHaveAttribute('aria-pressed', 'true');
    await scrollGrid(page, 700);
    const x = 500;
    const y = 450;
    await page.mouse.move(x, y);
    const before = await tileAt(page, x, y);
    const ratio = await anchorRatio(page, before, y);
    const zoom = await page.evaluate(() => window.devicePixelRatio);

    await wheelBurst(page, x, y, 6);

    await expect(sizeButton(page, 'L')).toHaveAttribute('aria-pressed', 'true');
    await expect(hud(page)).toContainText('220 px');
    await expect.poll(() => anchorRatio(page, before, y)).toBeCloseTo(ratio, 1);

    await notchUntil(page, 100, 'M');
    await expect(hud(page)).toContainText('150 px');
    await notchUntil(page, 100, 'S');

    await expect(hud(page)).toContainText('110 px');
    await expect.poll(() => anchorRatio(page, before, y)).toBeCloseTo(ratio, 1);
    expect(await page.evaluate(() => window.devicePixelRatio)).toBe(zoom);
  });

  test('the header buttons and the keyboard show the size HUD for about a second', async ({
    page
  }) => {
    await page.clock.install();
    await installMocks(page, { assets: ASSETS });
    await page.goto('/photos');

    const xl = page.getByRole('button', { name: 'Thumbnail size XL' });
    const button = await xl.boundingBox();
    const x = (button?.x ?? 0) + (button?.width ?? 0) / 2;
    const y = (button?.y ?? 0) + (button?.height ?? 0) / 2;
    await page.mouse.click(x, y);

    await expect(hud(page)).toBeVisible();
    await expect(hud(page)).toContainText('320 px');
    await expect(hud(page)).toContainText('Ctrl scroll · pinch');
    const box = await hud(page).boundingBox();
    expect(box?.x ?? Number.NaN).toBeLessThan(x);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeGreaterThan(x);
    expect((box?.y ?? 0) + (box?.height ?? 0)).toBeLessThanOrEqual(y + 60);
    await expect(page.locator('[role="group"]').first()).toHaveCSS('height', /^(2|3)\d\d/);

    await page.clock.runFor(1000);
    await expect(hud(page)).toBeHidden();

    await page.evaluate(() => {
      if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    });
    await page.keyboard.press('-');
    await expect(hud(page)).toContainText('220 px');
    await expect(page.getByRole('button', { name: 'Thumbnail size L' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    await page.clock.runFor(1000);
    await expect(hud(page)).toBeHidden();
  });

  test('edited photos carry the Edited mark with badges and with name and date, not hover-only', async ({
    page
  }) => {
    const assets = numberedAssets(6);
    const edits = assets.slice(0, 2).map((asset, index) => ({
      id: asset.id,
      hash: `h${index}`,
      updated_at: '2026-11-01T00:00:00Z'
    }));
    await installMocks(page, { assets, edits });
    await page.goto('/photos');

    const marks = page.locator('[role="group"]').getByRole('img', { name: 'Edited', exact: true });
    await expect(marks).toHaveCount(2);
    await expect(
      page.locator('[role="group"][title="IMG_0001.ARW"]').getByRole('img', { name: 'Edited' })
    ).toBeVisible();

    await page.getByRole('button', { name: 'Thumbnail info: badges' }).click();
    await expect(marks).toHaveCount(2);
    await page.getByRole('button', { name: 'Thumbnail info: name and date' }).click();
    await expect(page.getByRole('button', { name: 'Thumbnail info: hover only' })).toBeVisible();
    await expect(marks).toHaveCount(0);
  });
});

test.describe('grid sizing on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('a two-finger pinch steps the row height around the focal point', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const first = page.locator('[role="group"]').first();
    await expect(first).toBeVisible();
    expect((await first.boundingBox())?.height ?? 0).toBeGreaterThan(100);

    await scrollGrid(page, 1500);
    const centre = { x: 196, y: 420 };
    const anchor = await tileAt(page, centre.x, centre.y);
    const ratio = await anchorRatio(page, anchor, centre.y);

    await pinch(page, centre, 300, 90);

    await expect(hud(page)).toContainText('56 px');
    await expect.poll(async () => (await first.boundingBox())?.height ?? 0).toBeLessThan(100);
    await expect.poll(() => anchorRatio(page, anchor, centre.y)).toBeCloseTo(ratio, 1);
    const box = await hud(page).boundingBox();
    expect((box?.x ?? 0) + (box?.width ?? 0) / 2).toBeCloseTo(
      (page.viewportSize()?.width ?? 0) / 2,
      -1
    );
    await expect(hud(page).locator('kbd')).toBeHidden();
    expect(await page.evaluate(() => window.visualViewport?.scale ?? 1)).toBe(1);

    await pinch(page, centre, 90, 300);

    await expect(hud(page)).toContainText('160 px');
    await expect.poll(async () => (await first.boundingBox())?.height ?? 0).toBeGreaterThan(100);
  });

  test('defaults to the phone row height and remembers a chosen size', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    await page.locator('header[data-app-bar]').getByRole('button', { name: 'View options' }).tap();
    const sizes = page
      .getByRole('dialog', { name: 'View options' })
      .getByRole('radiogroup', { name: 'Thumbnail size' });
    await expect(sizes.getByRole('radio', { name: 'L', exact: true })).toBeChecked();

    await sizes.getByRole('radio', { name: 'S', exact: true }).tap();
    await expect(hud(page)).toContainText('56 px');
    await page.reload();
    await expect(page.locator('[role="group"]').first()).toBeVisible();
    expect((await page.locator('[role="group"]').first().boundingBox())?.height ?? 0).toBeLessThan(
      100
    );
  });
});
