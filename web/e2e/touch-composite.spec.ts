import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID } from './helpers';
import { drag, tapAt } from './touch';
import { open } from './touchShell';

async function openView(page: Page, group: string, view: string): Promise<Locator> {
  await open(page, `/assets/${ASSET_ID}`);
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  await page
    .getByRole('tablist', { name: 'Develop groups' })
    .getByRole('tab', { name: group })
    .tap();
  await page
    .getByRole('tablist', { name: `${group} views` })
    .getByRole('tab', { name: view, exact: true })
    .tap();
  const region = page.getByRole('region', { name: view, exact: true });
  await expect(region).toBeVisible();
  return region;
}

async function boxOf(
  target: Locator
): Promise<{ x: number; y: number; width: number; height: number }> {
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (!box) throw new Error('no box');
  return box;
}

test.describe('touch composite controls', { tag: '@touch' }, () => {
  test('the curve adds, moves, types and deletes a point', async ({ page }) => {
    const view = await openView(page, 'Light', 'Curve');
    const plot = view.getByRole('application', { name: 'Curves – RGB' });
    const deletePoint = view.getByRole('button', { name: 'Delete point' });
    await expect(deletePoint).toBeDisabled();

    const box = await boxOf(plot);
    const x = box.x + box.width * 0.5;
    const y = box.y + box.height * 0.5;
    await tapAt(page, x, y);
    await expect(plot.locator('circle')).toHaveCount(4);
    const out = view.getByRole('button', { name: 'Edit Out value' });
    const start = Number(await out.textContent());

    await drag(page, { x: x + 15, y: y + 10 }, { x: x + 15, y: y - 40 });
    await expect.poll(async () => Number(await out.textContent())).toBeGreaterThan(start + 30);
    await expect(plot.locator('circle')).toHaveCount(4);

    await out.tap();
    const keypad = page.getByRole('region', { name: 'Out keypad' });
    for (const key of ['1', '2', '8', 'Done']) {
      await keypad.getByRole('button', { name: key, exact: true }).tap();
    }
    await expect(keypad).toHaveCount(0);
    await expect(out).toHaveText('128');

    await deletePoint.tap();
    await expect(plot.locator('circle')).toHaveCount(2);
    await expect(view.getByText('Tap the curve to add a point')).toBeVisible();
  });

  test('a Mix band shows its modified dot', async ({ page }) => {
    const view = await openView(page, 'Color', 'Mix');
    const orange = view.getByRole('radio', { name: 'Orange' });
    await orange.tap();
    await expect(view.getByText('Orange', { exact: true })).toBeVisible();
    const box = await boxOf(view.getByRole('slider', { name: 'Saturation', exact: true }));
    const y = box.y + box.height / 2;
    await drag(page, { x: box.x + box.width / 2, y }, { x: box.x + box.width / 2 + 40, y });
    await expect(orange.locator('[data-modified-indicator]')).toHaveCount(1);
    await expect(
      view.getByRole('radio', { name: 'Red' }).locator('[data-modified-indicator]')
    ).toHaveCount(0);
  });

  test('grading shows one wheel that drags relative and resets on double-tap', async ({ page }) => {
    const view = await openView(page, 'Color', 'Grading');
    const shadows = view.getByRole('radio', { name: 'Shadows' });
    await shadows.tap();
    await expect(view.getByRole('slider', { name: 'Hue and saturation wheel' })).toHaveCount(1);
    const reset = view.getByRole('button', { name: 'Reset Shadows' });
    await expect(reset).toBeDisabled();

    const box = await boxOf(view.getByRole('slider', { name: 'Hue and saturation wheel' }));
    const centre = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    const start = { x: centre.x + box.width * 0.3, y: centre.y + box.height * 0.3 };
    await drag(page, start, { x: start.x + box.width * 0.25, y: start.y });
    await expect(view.getByText(/Saturation \d+/)).toHaveText(/Saturation [3-5]\d/);
    await expect(view.getByText(/Hue \d+°/)).toHaveText('Hue 0°');
    await expect(shadows.locator('[data-modified-indicator]')).toHaveCount(1);
    await expect(reset).toBeEnabled();

    await tapAt(page, centre.x, centre.y);
    await tapAt(page, centre.x, centre.y);
    await expect(view.getByText(/Saturation \d+/)).toHaveText('Saturation 0');
    await expect(shadows.locator('[data-modified-indicator]')).toHaveCount(0);
    await expect(view.getByRole('slider', { name: 'Luminance', exact: true })).toBeVisible();
    await expect(view.getByRole('slider', { name: 'Blending' })).toBeVisible();
  });
});
