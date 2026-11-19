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

test.describe('wide curve on desktop', () => {
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
    await expect(inspector.locator('circle')).toHaveCount(4);
    await expect(page.getByRole('button', { name: 'Edit Out value' })).toBeVisible();
  });
});
