import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID } from './helpers';
import { drag } from './touch';
import { open } from './touchShell';

async function openEditor(page: Page): Promise<void> {
  await open(page, `/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

function exposureValue(page: Page): Locator {
  return page.getByRole('button', { name: 'Edit Exposure value' });
}

function keypad(page: Page): Locator {
  return page.getByRole('region', { name: 'Exposure keypad' });
}

function field(page: Page): Locator {
  return keypad(page).getByRole('textbox', { name: 'Exposure value', exact: true });
}

async function press(page: Page, ...keys: string[]): Promise<void> {
  for (const key of keys) await keypad(page).getByRole('button', { name: key, exact: true }).tap();
}

async function sampleHeights(page: Page): Promise<void> {
  await page.evaluate(() => {
    const tray = document.querySelector('section[aria-label="Adjustments"]');
    const photo = tray?.previousElementSibling;
    if (!tray || !photo) throw new Error('no tray');
    const samples: [number, number][] = [];
    Object.assign(window, { heightSamples: samples });
    const tick = (): void => {
      samples.push([tray.getBoundingClientRect().height, photo.getBoundingClientRect().height]);
      if (samples.length < 40) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}

async function distinctHeights(page: Page): Promise<[number, number]> {
  await page.waitForFunction(
    () => (window as unknown as { heightSamples: unknown[] }).heightSamples.length >= 40
  );
  return page.evaluate(() => {
    const samples = (window as unknown as { heightSamples: [number, number][] }).heightSamples;
    const count = (index: 0 | 1): number =>
      new Set(samples.map((sample) => Math.round(sample[index]))).size;
    return [count(0), count(1)];
  });
}

test.describe('value keypad on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the tray and photo resize once as the keypad opens and closes', async ({ page }) => {
    await openEditor(page);
    await sampleHeights(page);
    await exposureValue(page).tap();
    await expect(keypad(page)).toBeVisible();
    expect(await distinctHeights(page)).toEqual([2, 2]);

    await sampleHeights(page);
    await press(page, 'Done');
    await expect(keypad(page)).toHaveCount(0);
    expect(await distinctHeights(page)).toEqual([2, 2]);
  });
});

test.describe('value keypad on touch', { tag: '@touch' }, () => {
  test('keys, sign, steppers and reset render live and Done commits', async ({ page }) => {
    await openEditor(page);
    await exposureValue(page).tap();
    await expect(keypad(page)).toBeVisible();
    await expect(field(page)).toHaveValue('0');
    const arrangement = await page.locator('[data-arrangement]').getAttribute('data-arrangement');
    const groups = page.getByRole('tablist', { name: 'Develop groups' });
    await expect(groups).toBeVisible({ visible: arrangement === 'inspector' });
    const modes = page.getByRole('tablist', { name: 'Editor modes' });
    await expect(modes).toBeVisible({ visible: arrangement !== 'stack' });
    await expect(
      page.locator('div.group', { has: page.getByRole('slider', { name: 'Contrast' }) })
    ).toHaveCSS('opacity', '0.28');

    await press(page, '1', 'Decimal point', '5');
    await expect(field(page)).toHaveValue('1.5');
    await expect(exposureValue(page)).toHaveText('1.50');

    await press(page, 'Toggle sign');
    await expect(exposureValue(page)).toHaveText('-1.50');
    await press(page, 'Increase Exposure');
    await expect(field(page)).toHaveValue('-1.49');
    await expect(exposureValue(page)).toHaveText('-1.49');

    await press(page, 'Reset');
    await expect(exposureValue(page)).toHaveText('0.00');
    await press(page, '2', 'Done');
    await expect(keypad(page)).toHaveCount(0);
    await expect(exposureValue(page)).toHaveText('2.00');
    await expect(page.getByRole('button', { name: /^Undo/ })).toBeEnabled();
  });

  test('a swipe down cancels and a tap outside commits', async ({ page }) => {
    await openEditor(page);
    await exposureValue(page).tap();
    await press(page, '3');
    await expect(exposureValue(page)).toHaveText('3.00');
    const box = await keypad(page).boundingBox();
    if (!box) throw new Error('keypad has no box');
    const x = box.x + box.width / 2;
    await drag(page, { x, y: box.y + 20 }, { x, y: box.y + 140 });
    await expect(keypad(page)).toHaveCount(0);
    await expect(exposureValue(page)).toHaveText('0.00');

    await exposureValue(page).tap();
    await press(page, '4');
    await page.getByRole('tab', { name: 'Tone' }).tap();
    await expect(keypad(page)).toHaveCount(0);
    await expect(exposureValue(page)).toHaveText('4.00');
  });

  test('the edited row stays in view above the keypad', async ({ page }) => {
    await openEditor(page);
    await page.getByRole('button', { name: 'Edit Blacks value' }).tap();
    const pad = page.getByRole('region', { name: 'Blacks keypad' });
    await expect(pad).toBeVisible();
    const slider = page.getByRole('slider', { name: 'Blacks' });
    await expect
      .poll(async () => {
        const [row, keys] = await Promise.all([slider.boundingBox(), pad.boundingBox()]);
        return row && keys ? keys.y - (row.y + row.height) : -1;
      })
      .toBeGreaterThan(0);
    await expect(slider).toBeInViewport();
  });

  test('a hardware keyboard types arithmetic and hides the keys', async ({ page }) => {
    await openEditor(page);
    await exposureValue(page).tap();
    await page.keyboard.type('+1.5');
    await expect(keypad(page).getByRole('button', { name: '1', exact: true })).toHaveCount(0);
    await expect(exposureValue(page)).toHaveText('1.50');
    await page.keyboard.press('Enter');
    await expect(keypad(page)).toHaveCount(0);
    await expect(exposureValue(page)).toHaveText('1.50');
  });
});
