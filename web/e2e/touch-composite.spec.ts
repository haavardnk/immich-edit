import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID } from './helpers';
import { drag } from './touch';
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
});
