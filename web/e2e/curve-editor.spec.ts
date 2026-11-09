import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';

async function openCurves(page: Page): Promise<Locator> {
  await installMocks(page);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  await page.getByRole('button', { name: 'Basic', exact: true }).click();
  await page.getByRole('button', { name: 'Curves', exact: true }).click();
  const plot = page.getByRole('application', { name: 'Curves – RGB' });
  await expect(plot).toBeVisible();
  return plot;
}

function outValue(page: Page): Locator {
  return page.getByRole('button', { name: 'Edit Out value' });
}

async function output(page: Page): Promise<number> {
  return Number(await outValue(page).first().textContent());
}

test.describe('wide curve on desktop', () => {
  test('adds, drags finer with Alt and removes a point at 1.5:1', async ({ page }) => {
    const plot = await openCurves(page);
    const box = await plot.boundingBox();
    if (!box) throw new Error('no plot');
    expect(box.width / box.height).toBeCloseTo(1.5, 1);

    const x = box.x + box.width * 0.5;
    const y = box.y + box.height * 0.5;
    await page.mouse.click(x, y);
    await expect(outValue(page)).toBeVisible();
    await expect(page.getByRole('button', { name: /^Undo/ })).toBeEnabled();
    const start = await output(page);

    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y - 60, { steps: 6 });
    await expect(page.locator('[data-curve-readout]')).toHaveText(/In \d+ · Out \d+/);
    await page.mouse.up();
    await expect(page.locator('[data-curve-readout]')).toHaveCount(0);
    const coarse = (await output(page)) - start;
    expect(coarse).toBeGreaterThan(60);

    await page.mouse.move(x, y - 60);
    await page.keyboard.down('Alt');
    await page.mouse.down();
    await page.mouse.move(x, y - 120, { steps: 6 });
    await page.mouse.up();
    await page.keyboard.up('Alt');
    const fine = (await output(page)) - start - coarse;
    expect(fine).toBeGreaterThan(coarse * 0.15);
    expect(fine).toBeLessThan(coarse * 0.35);

    await outValue(page).click();
    await page.getByRole('textbox', { name: 'Out value' }).fill('200');
    await page.keyboard.press('Enter');
    await expect(outValue(page)).toHaveText('200');

    const point = await plot.locator('circle').nth(2).boundingBox();
    if (!point) throw new Error('no point');
    await page.mouse.dblclick(point.x + point.width / 2, point.y + point.height / 2);
    await expect(outValue(page)).toHaveCount(0);
    await expect(page.getByText('Click the curve to add a point')).toBeVisible();
    await expect(plot.locator('circle')).toHaveCount(2);
  });

  test('the pop-out edits the same curve and channel and Escape closes it', async ({ page }) => {
    await openCurves(page);
    await page.getByRole('button', { name: 'Pop out curve editor' }).click();
    const popout = page.getByRole('dialog', { name: 'Curves' });
    await expect(popout).toBeVisible();
    const large = popout.getByRole('application', { name: 'Curves – RGB' });
    const box = await large.boundingBox();
    if (!box) throw new Error('no plot');
    expect(box.width).toBeGreaterThan(500);

    await popout.getByRole('radio', { name: 'Edit Red curve' }).click();
    await expect(page.getByRole('radio', { name: 'Edit Red curve' }).first()).toBeChecked();
    const red = popout.getByRole('application', { name: 'Curves – Red' });
    const redBox = await red.boundingBox();
    if (!redBox) throw new Error('no plot');
    await page.mouse.click(redBox.x + redBox.width * 0.5, redBox.y + redBox.height * 0.3);
    await expect(popout.getByRole('button', { name: 'Edit Out value' })).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(popout).toHaveCount(0);
    await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}`));
    const inspector = page.getByRole('application', { name: 'Curves – Red' });
    await expect(inspector.locator('circle')).toHaveCount(3);
  });
});
