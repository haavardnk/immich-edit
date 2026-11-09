import { expect, test, type Locator, type Page } from '@playwright/test';
import { datedAssets, dayShots } from './timeline';
import { open } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 10, 'Bergen'),
  ...dayShots('2026-10-30', 10, 'Ålesund')
]);

test.use({ timezoneId: 'UTC' });

function toggle(page: Page): Locator {
  return page.locator('main').getByRole('button', { name: /^View & filters/ });
}

function surface(page: Page): Locator {
  const name = 'View & filters';
  return page.getByRole('dialog', { name }).or(page.getByRole('complementary', { name }));
}

async function openSurface(page: Page): Promise<Locator> {
  await toggle(page).tap();
  const panel = surface(page);
  await expect(panel).toBeVisible();
  return panel;
}

async function closeSurface(page: Page): Promise<void> {
  await surface(page)
    .getByRole('button', { name: /^Close (view & filters|filter panel)$/ })
    .tap();
  await expect(surface(page)).toBeHidden();
}

const DEFAULT_SIZE = /^\s*X?L$/;

test.describe('view options in the filter surface', { tag: '@touch' }, () => {
  test('the app bar has one View & filters button and Select, no separate view options', async ({
    page
  }) => {
    await open(page, '/photos', { assets: ASSETS });
    const header = page.locator('main header').first();

    await expect(header.getByRole('button', { name: 'View & filters', exact: true })).toBeVisible();
    await expect(header.getByRole('button', { name: 'Select photos' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'View options' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Thumbnail size/ })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Sort:/ })).toHaveCount(0);
  });

  test('the View block changes the thumbnail size and the sort', async ({ page }) => {
    await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
    await open(page, '/photos', { assets: ASSETS });
    await expect(page.getByTestId('day-header').first()).toContainText('Saturday 31 October');

    const panel = await openSurface(page);
    await expect(panel.getByRole('button', { name: 'View', exact: true })).toHaveCount(0);
    const sortBy = panel.getByRole('radiogroup', { name: 'Sort by' });
    await expect(sortBy.getByRole('radio', { checked: true })).toHaveText('Taken');

    const tileHeight = async (): Promise<number> =>
      (await page.locator('[data-asset-id]').first().boundingBox())?.height ?? 0;
    const before = await tileHeight();
    const sizes = panel.getByRole('radiogroup', { name: 'Thumbnail size' });
    await expect(sizes.getByRole('radio', { checked: true })).toHaveText(DEFAULT_SIZE);
    await sizes.getByRole('radio', { name: 'S', exact: true }).tap();
    await expect(sizes.getByRole('radio', { name: 'S', exact: true })).toBeChecked();
    await expect.poll(tileHeight).toBeLessThan(before);

    const order = panel.getByRole('radiogroup', { name: 'Order' });
    await order.getByRole('radio', { name: 'Oldest' }).tap();
    await expect(page.getByTestId('day-header').first()).toContainText('Friday 30 October');
    await expect(panel.getByRole('button', { name: 'Reset all filters' })).toBeDisabled();
    await closeSurface(page);

    const again = await openSurface(page);
    await expect(
      again.getByRole('radiogroup', { name: 'Order' }).getByRole('radio', { checked: true })
    ).toHaveText('Oldest');
    await expect(
      again
        .getByRole('radiogroup', { name: 'Thumbnail size' })
        .getByRole('radio', { checked: true })
    ).toHaveText('S');
  });

  test('the View block sits above the filters heading', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const panel = await openSurface(page);
    const view = panel.getByRole('heading', { name: 'View', exact: true });
    const filters = panel.getByRole('heading', { name: /^Filters/ });
    await expect(view).toBeVisible();
    const viewTop = (await view.boundingBox())?.y ?? 0;
    const filtersTop = (await filters.boundingBox())?.y ?? 0;
    expect(viewTop).toBeLessThan(filtersTop);
    await expect(panel.locator('[data-collapsible-trigger]').first()).toHaveAccessibleName(
      'Library'
    );
  });

  for (const { path, sorts, order } of [
    { path: '/albums/album-lofoten', sorts: true, order: ['Newest', 'Oldest'] },
    { path: '/people/person-ingrid', sorts: true, order: ['Newest', 'Oldest'] },
    { path: '/tags/tag-sky', sorts: true, order: ['Newest', 'Oldest'] },
    { path: '/folders?path=/photos/2026/Norway', sorts: true, order: ['Newest', 'Oldest'] },
    { path: '/search?q=IMG', sorts: false, order: [] },
    { path: '/edited', sorts: false, order: ['Newest', 'Oldest'] }
  ]) {
    test(`view options stay reachable on ${path}`, async ({ page }) => {
      await open(page, path);
      const panel = await openSurface(page);
      await expect(panel.getByRole('radiogroup', { name: 'Sort by' })).toHaveCount(sorts ? 1 : 0);
      await expect(panel.getByRole('radiogroup', { name: 'Order' }).getByRole('radio')).toHaveText(
        order
      );
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail size' })).toBeVisible();
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail info' })).toBeVisible();
    });
  }
});
