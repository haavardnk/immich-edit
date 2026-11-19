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

test.describe('filter lists on touch', { tag: '@touch' }, () => {
  test('All people opens the whole list as a page and Back returns to the open section', async ({
    page
  }) => {
    const people = Array.from({ length: 9 }, (_, index) => ({
      id: `person-${index}`,
      name: `Person ${String(index + 1).padStart(2, '0')}`,
      assetIds: [ASSETS[0]!.id]
    }));
    await open(page, '/photos', { assets: ASSETS, people });
    const panel = await openSurface(page);
    await panel.getByRole('button', { name: 'People', exact: true }).tap();
    const rows = panel.getByRole('group', { name: 'People' }).getByRole('checkbox');
    await expect(rows).toHaveCount(5);

    await panel.getByRole('button', { name: 'All 9 people' }).tap();
    const list = panel.getByRole('region', { name: 'People' });
    await expect(list.getByRole('heading', { name: 'People' })).toBeInViewport();
    await expect(list.getByRole('checkbox')).toHaveCount(9);
    await expect(panel.getByRole('radiogroup', { name: 'Thumbnail size' })).toHaveCount(0);
    await list.getByRole('checkbox', { name: 'Person 09' }).tap();
    await expect(page).toHaveURL(/person=person-8/);

    await list.getByRole('button', { name: 'Back to filters' }).tap();
    await expect(list).toHaveCount(0);
    const section = panel.getByRole('button', { name: 'People', exact: true });
    await expect(section).toHaveAttribute('aria-expanded', 'true');
    await expect(section).toBeInViewport();
    await expect(rows.first()).toHaveAccessibleName('Person 09');
  });
});
