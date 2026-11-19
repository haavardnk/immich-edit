import { expect, test, type Locator, type Page } from '@playwright/test';
import { gotoAsset, installMocks } from './helpers';

const REST_MS = 400;
const BURST_GAP_MS = 200;

function row(page: Page, label: string): Locator {
  return page.locator('div.group', {
    has: page.getByRole('button', { name: label, exact: true })
  });
}

function valueOf(page: Page, label: string): Locator {
  return page.getByRole('button', { name: `Edit ${label} value` });
}

async function numberOf(page: Page, label: string): Promise<number> {
  return Number((await valueOf(page, label).textContent())?.trim());
}

async function centreOf(target: Locator): Promise<{ x: number; y: number }> {
  const box = await target.boundingBox();
  if (!box) throw new Error('target has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function wheel(page: Page, deltaY: number): Promise<void> {
  await page.mouse.wheel(0, deltaY);
  await page.waitForTimeout(BURST_GAP_MS);
}

function scrollerTop(target: Locator): Promise<number> {
  return target.evaluate((element) => {
    let node = element.parentElement;
    while (
      node &&
      !(
        /(auto|scroll)/.test(getComputedStyle(node).overflowY) &&
        node.scrollHeight > node.clientHeight
      )
    ) {
      node = node.parentElement;
    }
    return node?.scrollTop ?? -1;
  });
}

test('the wheel adjusts a row only after the pointer rests on it', async ({ page }) => {
  const saves: Array<Record<string, unknown>> = [];
  await installMocks(page, { onSave: (body) => saves.push(body) });
  await gotoAsset(page);

  const exposure = page.getByRole('slider', { name: 'Exposure', exact: true });
  const point = await centreOf(exposure);
  await page.mouse.move(point.x, point.y);
  await wheel(page, -100);
  await expect(valueOf(page, 'Exposure')).toHaveText('0.00');

  await page.waitForTimeout(REST_MS);
  await wheel(page, -100);
  await expect(valueOf(page, 'Exposure')).toHaveText('0.05');

  await page.keyboard.down('Shift');
  await wheel(page, -100);
  await page.keyboard.up('Shift');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.55');

  await page.keyboard.down('Alt');
  await wheel(page, 100);
  await page.keyboard.up('Alt');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.54');

  await expect
    .poll(() => {
      const ops = (saves.at(-1)?.manifest as { ops?: { exposure?: { ev?: number } } })?.ops;
      return ops?.exposure?.ev;
    })
    .toBe(0.54);
});

test('a wheel that arrives with the pointer scrolls the panel instead', async ({ page }) => {
  await installMocks(page);
  await page.setViewportSize({ width: 1280, height: 560 });
  await gotoAsset(page);

  const contrast = page.getByRole('slider', { name: 'Contrast', exact: true });
  await contrast.hover();
  const before = await scrollerTop(contrast);
  await page.mouse.wheel(0, 120);
  await expect.poll(() => scrollerTop(contrast)).toBeGreaterThan(before);
  await expect(valueOf(page, 'Contrast')).toHaveText('0');
});

test('typed values accept arithmetic relative to the current value and clamp', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  async function type(label: string, text: string): Promise<void> {
    await valueOf(page, label).click();
    const field = page.getByRole('textbox', { name: `${label} value`, exact: true });
    await field.fill(text);
    await field.press('Enter');
  }

  await type('Exposure', '+0.5');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.50');
  await type('Exposure', 'x1.5');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.75');
  await type('Exposure', '+100');
  await expect(valueOf(page, 'Exposure')).toHaveText('5.00');
  await type('Exposure', '−0.3');
  await expect(valueOf(page, 'Exposure')).toHaveText('-0.30');
  await type('Highlights', '-38');
  await expect(valueOf(page, 'Highlights')).toHaveText('-38');

  await valueOf(page, 'Highlights').click();
  const field = page.getByRole('textbox', { name: 'Highlights value', exact: true });
  await field.fill('/2');
  await expect(valueOf(page, 'Highlights')).toHaveCount(0);
  await field.press('Escape');
  await expect(valueOf(page, 'Highlights')).toHaveText('-38');
});

test('dragging a label adjusts from the current value and keeps double-click reset', async ({
  page
}) => {
  await installMocks(page);
  await gotoAsset(page);

  const label = row(page, 'Contrast').getByRole('button', { name: 'Contrast', exact: true });
  const start = await centreOf(label);
  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x + 40, start.y, { steps: 8 });
  await page.mouse.up();
  const scrubbed = await numberOf(page, 'Contrast');
  expect(scrubbed).toBeGreaterThan(20);

  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x - 20, start.y, { steps: 4 });
  await page.mouse.up();
  const back = await numberOf(page, 'Contrast');
  expect(back).toBeLessThan(scrubbed);
  expect(back).toBeGreaterThan(0);

  await page.waitForTimeout(600);
  await label.dblclick();
  await expect(valueOf(page, 'Contrast')).toHaveText('0');
});

test('a track click jumps and a drag slows as the pointer leaves the track', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const slider = page.getByRole('slider', { name: 'Contrast', exact: true });
  const box = await slider.boundingBox();
  if (!box) throw new Error('contrast slider has no box');
  const y = box.y + box.height / 2;
  const centre = box.x + box.width / 2;

  await page.mouse.click(box.x + box.width * 0.75, y);
  const jumped = await numberOf(page, 'Contrast');
  expect(jumped).toBeGreaterThan(40);
  expect(jumped).toBeLessThan(60);
  await slider.dblclick();
  await expect(valueOf(page, 'Contrast')).toHaveText('0');

  await page.mouse.move(centre, y);
  await page.mouse.down();
  await page.mouse.move(centre + 40, y, { steps: 8 });
  await page.mouse.up();
  const near = await numberOf(page, 'Contrast');
  await slider.dblclick();
  await expect(valueOf(page, 'Contrast')).toHaveText('0');

  await page.mouse.move(centre, y);
  await page.mouse.down();
  await page.mouse.move(centre, y + 200, { steps: 8 });
  await expect(valueOf(page, 'Contrast')).toHaveText('0');
  await page.mouse.move(centre + 40, y + 200, { steps: 8 });
  await page.mouse.up();
  const far = await numberOf(page, 'Contrast');

  expect(far).toBeGreaterThan(0);
  expect(far).toBeLessThan(near / 2);
});

test('alt with an arrow key moves a slider by its fine step', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const slider = page.getByRole('slider', { name: 'Exposure', exact: true });
  await slider.focus();
  await slider.press('Alt+ArrowRight');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.01');
  await slider.press('ArrowRight');
  await expect(valueOf(page, 'Exposure')).toHaveText('0.06');
});
