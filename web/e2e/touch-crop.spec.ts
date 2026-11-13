import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, makePng } from './helpers';
import { drag, touchDown, touchUp, type TouchPoint } from './touch';
import { open } from './touchShell';

type SaveBody = { action?: string | null };

async function openCrop(page: Page, saves: SaveBody[] = []): Promise<void> {
  await open(page, `/assets/${ASSET_ID}`, {
    previewBody: makePng(60, 40),
    onSave: (body) => saves.push(body as SaveBody)
  });
  await page
    .getByRole('tablist', { name: 'Editor modes' })
    .getByRole('tab', { name: 'Crop' })
    .tap();
  await expect(page.getByRole('button', { name: 'resize nw' })).toBeVisible();
}

function dial(page: Page): Locator {
  return page.getByRole('tablist', { name: 'Dial' });
}

function tools(page: Page): Locator {
  return page.getByRole('toolbar', { name: 'Crop tools' });
}

async function centreOf(target: Locator): Promise<TouchPoint> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function sampleWidths(page: Page, target: 'crop' | 'preview'): Promise<void> {
  await page.evaluate((target) => {
    const widths: number[] = [];
    (window as unknown as { widths: number[] }).widths = widths;
    const started = performance.now();
    const width = (): number | null => {
      if (target === 'preview') {
        const preview = document.querySelector('[data-testid="preview-image"]');
        return preview ? preview.getBoundingClientRect().width : null;
      }
      const nw = document.querySelector('[aria-label="resize nw"]');
      const se = document.querySelector('[aria-label="resize se"]');
      return nw && se ? se.getBoundingClientRect().right - nw.getBoundingClientRect().left : null;
    };
    const sample = () => {
      const w = width();
      if (w !== null) widths.push(w);
      if (performance.now() - started < 1000) requestAnimationFrame(sample);
    };
    requestAnimationFrame(sample);
  }, target);
}

async function sampledWidths(page: Page): Promise<number[]> {
  await page.waitForTimeout(1100);
  return page.evaluate(() => (window as unknown as { widths: number[] }).widths);
}

function expectEase(widths: number[], way: 'shrinks' | 'grows'): void {
  expect(widths.length).toBeGreaterThan(5);
  const first = widths[0] ?? 0;
  const last = widths.at(-1) ?? 0;
  const steps = widths.slice(1).map((w, i) => w - (widths[i] ?? w));
  if (way === 'shrinks') {
    expect(Math.max(...steps)).toBeLessThan(1);
    expect(last / first).toBeGreaterThan(0.8);
    expect(last / first).toBeLessThan(0.97);
  } else {
    expect(Math.min(...steps)).toBeGreaterThan(-1);
    expect(first / last).toBeGreaterThan(0.8);
    expect(first / last).toBeLessThan(0.97);
  }
}

async function pickRatio(page: Page, name: string): Promise<void> {
  await tools(page)
    .getByRole('button', { name: /^Aspect ratio:/ })
    .tap();
  const sheet = page.getByRole('dialog', { name: 'Aspect ratio' });
  await expect(sheet).toBeVisible();
  await sheet.getByRole('button', { name, exact: true }).tap();
  await expect(sheet).toHaveCount(0);
}

async function expectToolsFit(page: Page): Promise<void> {
  const buttons = tools(page).getByRole('button');
  await expect(buttons).toHaveCount(10);
  const names = await buttons.evaluateAll((all) =>
    all.map((button) => (button.getAttribute('aria-label') ?? '').replace(/:.*/, ':'))
  );
  expect(names).toEqual([
    'Aspect ratio:',
    'Rotate left 90°',
    'Rotate right 90°',
    'Flip horizontal',
    'Flip vertical',
    'Crop guide:',
    'Level line',
    'Guided upright',
    'Corners',
    'Reset geometry'
  ]);
  const bar = await tools(page).boundingBox();
  if (!bar) throw new Error('crop tools have no box');
  const boxes = await buttons.evaluateAll((all) =>
    all.map((button) => {
      const box = button.getBoundingClientRect();
      const label = button.lastElementChild as HTMLElement;
      return {
        x: box.x,
        y: box.y,
        right: box.right,
        clipped: label.scrollWidth > label.clientWidth
      };
    })
  );
  for (const box of boxes) {
    expect(box.clipped).toBe(false);
    expect(box.x).toBeGreaterThanOrEqual(bar.x - 0.5);
    expect(box.right).toBeLessThanOrEqual(bar.x + bar.width + 0.5);
  }
  expect(new Set(boxes.map((box) => Math.round(box.y))).size).toBe(2);
}

test.describe('touch crop on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the dial straightens live, marks its chip and holds the original', async ({ page }) => {
    const saves: SaveBody[] = [];
    await openCrop(page, saves);
    const value = page.getByTestId('crop-dial-value');
    await expect(value).toHaveText('0.0°');
    const ruler = await centreOf(page.getByTestId('crop-dial'));
    await drag(page, ruler, { x: ruler.x - 80, y: ruler.y });
    await expect(value).not.toHaveText('0.0°');
    const levelled = await value.textContent();
    await expect.poll(() => saves.at(-1)?.action).toBe('Straighten');
    const chip = dial(page).getByRole('tab', { name: 'Straighten' });
    await expect(chip.locator('[data-modified-indicator]')).toHaveCount(1);

    await touchDown(page, await centreOf(chip));
    await expect(value).toHaveText('0.0°');
    await touchUp(page);
    await expect(value).toHaveText(levelled ?? '');

    await dial(page).getByRole('tab', { name: 'Vertical' }).tap();
    await expect(value).toHaveText('0');
  });

  test('a zoomed view keeps the crop box centred on the photo', async ({ page }) => {
    await openCrop(page);
    const ruler = await centreOf(page.getByTestId('crop-dial'));
    await drag(page, ruler, { x: ruler.x - 180, y: ruler.y });
    await expect(page.getByTestId('crop-dial-value')).not.toHaveText('0.0°');
    const width = page.viewportSize()?.width ?? 0;
    await expect
      .poll(async () => {
        const nw = await centreOf(page.getByRole('button', { name: 'resize nw' }));
        const se = await centreOf(page.getByRole('button', { name: 'resize se' }));
        return Math.round((nw.x + se.x) / 2 - width / 2);
      })
      .toBe(0);
  });

  test('entering and leaving crop ease the photo by the fill margin only', async ({ page }) => {
    await openCrop(page);
    await pickRatio(page, '1:1');
    const modes = page.getByRole('tablist', { name: 'Editor modes' });
    await modes.getByRole('tab', { name: 'Develop' }).tap();
    await expect(page.getByRole('button', { name: 'resize nw' })).toHaveCount(0);

    await sampleWidths(page, 'crop');
    await modes.getByRole('tab', { name: 'Crop' }).tap();
    expectEase(await sampledWidths(page), 'shrinks');

    await sampleWidths(page, 'preview');
    await modes.getByRole('tab', { name: 'Develop' }).tap();
    expectEase(await sampledWidths(page), 'grows');
  });

  test('corners shows the whole photo so every corner handle is reachable', async ({ page }) => {
    await openCrop(page);
    await pickRatio(page, '1:1');
    const ruler = await centreOf(page.getByTestId('crop-dial'));
    await drag(page, ruler, { x: ruler.x - 60, y: ruler.y });
    await expect(page.getByTestId('crop-dial-value')).not.toHaveText('0.0°');

    const box = page.getByRole('button', { name: 'resize nw' });
    const framedWidth = async () => {
      const nw = await centreOf(box);
      const se = await centreOf(page.getByRole('button', { name: 'resize se' }));
      return se.x - nw.x;
    };
    const settledWidth = async () => {
      let last = Number.NaN;
      await expect
        .poll(
          async () => {
            const now = await framedWidth();
            const still = Math.abs(now - last) < 0.5;
            last = now;
            return still;
          },
          { intervals: [300] }
        )
        .toBe(true);
      return last;
    };
    const framed = await settledWidth();

    await tools(page).getByRole('button', { name: 'Corners' }).tap();
    const stage = await page.locator('.editor-stage').boundingBox();
    if (!stage) throw new Error('stage has no box');
    await expect
      .poll(async () => {
        for (let i = 1; i <= 4; i++) {
          const at = await centreOf(page.getByRole('button', { name: `perspective corner ${i}` }));
          const inside =
            at.x >= stage.x &&
            at.x <= stage.x + stage.width &&
            at.y >= stage.y &&
            at.y <= stage.y + stage.height;
          if (!inside) return false;
        }
        return true;
      })
      .toBe(true);
    expect(await framedWidth()).toBeLessThan(framed);

    await tools(page).getByRole('button', { name: 'Corners' }).tap();
    await expect(page.getByRole('button', { name: 'perspective corner 1' })).toHaveCount(0);
    expect(await settledWidth()).toBeCloseTo(framed, 0);
  });

  test('the ratio sheet, the custom keypad and the readout agree', async ({ page }) => {
    await openCrop(page);
    const readout = page.locator('[data-photo-pill]');
    await expect(readout).toHaveText(/^Original 6000 × 4000 px$/);

    await pickRatio(page, '4:3');
    await expect(readout).toHaveText(/^4:3 \d+ × \d+ px$/);
    await expect(tools(page).getByRole('button', { name: 'Aspect ratio: 4:3' })).toBeVisible();

    await tools(page).getByRole('button', { name: 'Aspect ratio: 4:3' }).tap();
    await page
      .getByRole('dialog', { name: 'Aspect ratio' })
      .getByRole('radio', { name: 'Portrait' })
      .tap();
    await expect(readout).toHaveText(/^3:4 /);
    await page
      .getByRole('dialog', { name: 'Aspect ratio' })
      .getByRole('button', { name: 'Custom…' })
      .tap();

    const keypad = page.getByRole('region', { name: 'Custom ratio' });
    await expect(keypad).toBeVisible();
    await expect(keypad.getByRole('textbox', { name: 'Ratio width' })).toHaveValue('3');
    await keypad.getByRole('button', { name: 'Clear' }).tap();
    for (const key of ['2', '1', ':', '9']) {
      await keypad.getByRole('button', { name: key, exact: true }).tap();
    }
    await expect(keypad.getByRole('textbox', { name: 'Ratio height' })).toHaveValue('9');
    await keypad.getByRole('button', { name: 'Done' }).tap();
    await expect(keypad).toHaveCount(0);
    await expect(readout).toHaveText(/^21:9 \d+ × \d+ px$/);
  });

  test('the tool row flips, cycles the guide, resets, and swaps the dial for upright', async ({
    page
  }) => {
    await openCrop(page);
    await expectToolsFit(page);
    const readout = page.locator('[data-photo-pill]');
    await tools(page).getByRole('button', { name: 'Rotate right 90°' }).tap();
    await expect(readout).toHaveText(/^Original 4000 × 6000 px$/);
    await tools(page).getByRole('button', { name: 'Rotate left 90°' }).tap();
    await expect(readout).toHaveText(/^Original 6000 × 4000 px$/);
    const flipV = tools(page).getByRole('button', { name: 'Flip vertical' });
    await flipV.tap();
    await expect(flipV).toHaveAttribute('aria-pressed', 'true');
    const reset = tools(page).getByRole('button', { name: 'Reset geometry' });
    await reset.tap();
    await expect(reset).toBeDisabled();
    await expect(flipV).toHaveAttribute('aria-pressed', 'false');

    const guide = page.getByTestId('crop-guide');
    const guideButton = tools(page).getByRole('button', { name: /^Crop guide: / });
    await expect(guideButton).toHaveAccessibleName('Crop guide: Rule of thirds');
    for (const [mode, turn, name, label] of [
      ['golden', null, 'Golden ratio', 'Golden'],
      ['spiral', '0', 'Golden spiral', 'Spiral'],
      ['spiral', '1', 'Golden spiral, mirrored', 'Spiral ↔'],
      ['diagonal', null, 'Diagonals', 'Diagonals'],
      ['triangle', '0', 'Golden triangle', 'Triangle'],
      ['triangle', '1', 'Golden triangle, mirrored', 'Triangle ↔'],
      ['grid', null, 'Grid', 'Grid'],
      ['off', null, 'No guide', 'None'],
      ['thirds', null, 'Rule of thirds', 'Thirds']
    ] as const) {
      await guideButton.tap();
      await expect(guideButton).toHaveAccessibleName(`Crop guide: ${name}`);
      await expect(guideButton).toHaveText(label);
      await expect(guide).toHaveAttribute('data-mode', mode);
      if (turn === null) await expect(guide).not.toHaveAttribute('data-turn');
      else await expect(guide).toHaveAttribute('data-turn', turn);
    }

    await tools(page).getByRole('button', { name: 'Guided upright' }).tap();
    await expect(page.getByTestId('upright-surface')).toBeVisible();
    const tray = page.getByRole('region', { name: 'Guided upright' });
    await expect(tray).toContainText(/Vertical 0 of 2\s*·\s*Horizontal 0 of 2/);
    await expect(dial(page)).toHaveCount(0);
    await tray.getByRole('button', { name: 'Done drawing' }).tap();
    await expect(dial(page)).toBeVisible();
    await expect(page.getByTestId('upright-surface')).toHaveCount(0);
  });

  test('crop keeps the tray detent and height of the other modes', async ({ page }) => {
    await open(page, `/assets/${ASSET_ID}`, { previewBody: makePng(60, 40) });
    const tray = page.getByRole('region', { name: 'Adjustments' });
    const handle = page.getByRole('button', { name: 'Resize adjustments' });
    const modes = page.getByRole('tablist', { name: 'Editor modes' });
    const height = async () => (await tray.boundingBox())?.height;
    const enter = async (name: string) => {
      const tab = modes.getByRole('tab', { name });
      if ((await tab.getAttribute('aria-selected')) !== 'true') await tab.tap();
      await expect(tab).toHaveAttribute('aria-selected', 'true');
    };
    const settle = async (detent: string) => {
      for (let step = 0; step < 3; step++) {
        const at = await tray.getAttribute('data-detent');
        if (at === detent) break;
        await handle.click();
        await expect(tray).not.toHaveAttribute('data-detent', at ?? '');
      }
      await expect(tray).toHaveAttribute('data-detent', detent);
      await expect(tray).not.toHaveAttribute('data-snapping');
    };

    for (const detent of ['expanded', 'peek', 'compact']) {
      await enter('Develop');
      await settle(detent);
      const develop = await height();

      await enter('Crop');
      await expect(tray).toHaveAttribute('data-detent', detent);
      await expect(tray).not.toHaveAttribute('data-snapping');
      await expect.poll(height).toBe(develop);
      if (detent === 'peek') {
        await expect(dial(page).getByRole('tab', { name: 'Straighten' })).toBeInViewport();
      } else {
        await expect(dial(page)).toBeInViewport();
        await expect(tools(page)).toBeInViewport({ ratio: 1 });
        const bottom = async (target: Locator) => {
          const box = await target.boundingBox();
          return box ? box.y + box.height : 0;
        };
        expect(await bottom(tray)).toBeCloseTo(await bottom(tools(page)), -1.5);
      }
    }

    await settle('peek');
    for (const mode of ['Masks', 'Retouch', 'Presets', 'Develop']) {
      await enter(mode);
      await expect(tray).toHaveAttribute('data-detent', 'peek');
    }

    await settle('expanded');
    await enter('Develop');
    await expect(tray).toHaveAttribute('data-detent', 'expanded');
  });

  test('the crop tools stay reachable beside the photo in landscape', async ({ page }) => {
    const size = page.viewportSize();
    if (!size) throw new Error('no viewport');
    await page.setViewportSize({ width: size.height, height: size.width });
    await openCrop(page);
    await expect(page.locator('[data-arrangement="side"]')).toBeVisible();
    await expect(dial(page)).toBeVisible();
    await expectToolsFit(page);
    await pickRatio(page, '1:1');
    await expect(tools(page).getByRole('button', { name: 'Aspect ratio: 1:1' })).toBeVisible();
  });
});

test.describe('touch crop on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('the inspector holds the dial and the crop tools', async ({ page }) => {
    await openCrop(page);
    await expect(page.locator('[data-arrangement="inspector"]')).toBeVisible();
    await expect(dial(page)).toBeVisible();
    await expectToolsFit(page);
    await tools(page).getByRole('button', { name: 'Level line' }).tap();
    await expect(tools(page).getByRole('button', { name: 'Level line' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    await expect(page.getByTestId('straighten-surface')).toBeVisible();
  });
});
