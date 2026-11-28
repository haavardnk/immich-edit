import { expect, test, type Locator, type Page } from '@playwright/test';
import { datedAssets, dayShots } from './timeline';
import { open } from './touchShell';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 10, 'Bergen'),
  ...dayShots('2026-10-30', 10, 'Ålesund')
]);

test.use({ timezoneId: 'UTC' });

function viewToggle(page: Page): Locator {
  return page.locator('main').getByRole('button', { name: 'View', exact: true });
}

function filtersToggle(page: Page): Locator {
  return page.locator('main').getByRole('button', { name: /^Filters/ });
}

function surface(page: Page, name: string): Locator {
  return page
    .getByRole('dialog', { name, exact: true })
    .or(page.getByRole('complementary', { name, exact: true }));
}

async function openView(page: Page): Promise<Locator> {
  await viewToggle(page).tap();
  const panel = surface(page, 'View');
  await expect(panel).toBeVisible();
  return panel;
}

async function openFiltersSurface(page: Page): Promise<Locator> {
  await filtersToggle(page).tap();
  const panel = surface(page, 'Filters');
  await expect(panel).toBeVisible();
  return panel;
}

async function closeView(page: Page): Promise<void> {
  await surface(page, 'View')
    .getByRole('button', { name: /^Close view( panel)?$/ })
    .tap();
  await expect(surface(page, 'View')).toBeHidden();
}

const DEFAULT_SIZE = /^\s*X?L$/;

test.describe('view and filters on touch', { tag: '@touch' }, () => {
  test('the app bar has separate View and Filters buttons beside Select', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const header = page.locator('main header').first();

    await expect(header.getByRole('button', { name: 'View', exact: true })).toBeVisible();
    await expect(header.getByRole('button', { name: 'Filters', exact: true })).toBeVisible();
    await expect(header.getByRole('button', { name: 'Select photos' })).toBeVisible();
    await expect(page.getByRole('button', { name: /^View & filters/ })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Thumbnail size/ })).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Sort:/ })).toHaveCount(0);
  });

  test('View changes the thumbnail size and the sort and holds no filters', async ({ page }) => {
    await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
    await open(page, '/photos', { assets: ASSETS });
    await expect(page.getByTestId('day-header').first()).toContainText('Saturday 31 October');

    const panel = await openView(page);
    await expect(viewToggle(page)).toHaveAttribute('aria-expanded', 'true');
    await expect(panel.locator('[data-collapsible-trigger]')).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'Reset all filters' })).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'Save view' })).toHaveCount(0);
    await expect(panel.getByRole('radiogroup', { name: 'Thumbnail info' })).toBeVisible();
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
    await closeView(page);
    await expect(viewToggle(page)).toBeFocused();

    const again = await openView(page);
    await expect(
      again.getByRole('radiogroup', { name: 'Order' }).getByRole('radio', { checked: true })
    ).toHaveText('Oldest');
    await expect(
      again
        .getByRole('radiogroup', { name: 'Thumbnail size' })
        .getByRole('radio', { checked: true })
    ).toHaveText('S');
  });

  test('Filters holds only the filters with Reset and Save view in its header', async ({
    page
  }) => {
    await open(page, '/photos', { assets: ASSETS });
    const panel = await openFiltersSurface(page);
    await expect(panel.getByRole('radiogroup', { name: 'Sort by' })).toHaveCount(0);
    await expect(panel.getByRole('radiogroup', { name: 'Thumbnail size' })).toHaveCount(0);
    await expect(panel.getByRole('radiogroup', { name: 'Thumbnail info' })).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'Reset all filters' })).toBeDisabled();
    await expect(panel.getByRole('button', { name: 'Save view' })).toBeVisible();
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
      const panel = await openView(page);
      await expect(panel.getByRole('radiogroup', { name: 'Sort by' })).toHaveCount(sorts ? 1 : 0);
      await expect(panel.getByRole('radiogroup', { name: 'Order' }).getByRole('radio')).toHaveText(
        order
      );
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail size' })).toBeVisible();
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail info' })).toBeVisible();
    });
  }
});

test.describe('the View sheet on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('Done closes the sheet in portrait and is gone in landscape', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const sheet = await openView(page);
    const done = sheet.getByRole('button', { name: 'Done', exact: true });
    await expect(done).toBeInViewport();
    await done.tap();
    await expect(sheet).toBeHidden();
    await expect(viewToggle(page)).toBeFocused();

    await page.setViewportSize({ width: 915, height: 412 });
    const rotated = await openView(page);
    await expect(rotated.getByRole('button', { name: 'Close view' })).toBeVisible();
    await expect(rotated.getByRole('button', { name: 'Done', exact: true })).toHaveCount(0);
  });
});

test.describe('view and filters share the tablet dock', { tag: ['@touch', '@tablet'] }, () => {
  test('opening one panel replaces the other in the dock', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const view = await openView(page);
    await expect(page.getByRole('complementary', { name: 'View', exact: true })).toBeVisible();

    const filters = await openFiltersSurface(page);
    await expect(page.getByRole('complementary', { name: 'Filters', exact: true })).toBeVisible();
    await expect(view).toBeHidden();
    await expect(viewToggle(page)).toHaveAttribute('aria-expanded', 'false');

    await viewToggle(page).tap();
    await expect(surface(page, 'View')).toBeVisible();
    await expect(filters).toBeHidden();
    await expect(filtersToggle(page)).toHaveAttribute('aria-expanded', 'false');
  });
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
    const panel = await openFiltersSurface(page);
    await panel.getByRole('button', { name: 'People', exact: true }).tap();
    const rows = panel.getByRole('group', { name: 'People' }).getByRole('checkbox');
    await expect(rows).toHaveCount(5);

    await panel.getByRole('button', { name: 'All 9 people' }).tap();
    const list = panel.getByRole('region', { name: 'People' });
    await expect(list.getByRole('heading', { name: 'People' })).toBeInViewport();
    await expect(list.getByRole('button', { name: 'Back to filters' })).toBeFocused();
    await expect(list.getByRole('checkbox')).toHaveCount(9);
    await list.getByRole('checkbox', { name: 'Person 09' }).tap();
    await expect(page).toHaveURL(/person=person-8/);

    await list.getByRole('button', { name: 'Back to filters' }).tap();
    await expect(list).toHaveCount(0);
    const section = panel.getByRole('button', { name: 'People', exact: true });
    await expect(section).toHaveAttribute('aria-expanded', 'true');
    await expect(section).toBeInViewport();
    await expect(rows.first()).toHaveAccessibleName('Person 09');
    await expect(panel.getByRole('button', { name: 'All 9 people' })).toBeFocused();
  });

  test('Back focuses the section when the whole list now fits', async ({ page }) => {
    const people = Array.from({ length: 6 }, (_, index) => ({
      id: `person-${index}`,
      name: `Person ${String(index + 1).padStart(2, '0')}`,
      assetIds: [ASSETS[0]!.id]
    }));
    await open(page, '/photos', { assets: ASSETS, people });
    const panel = await openFiltersSurface(page);
    await panel.getByRole('button', { name: 'People', exact: true }).tap();
    await panel.getByRole('button', { name: 'All 6 people' }).tap();
    const list = panel.getByRole('region', { name: 'People' });
    await list.getByRole('checkbox', { name: 'Person 06' }).tap();
    await expect(page).toHaveURL(/person=person-5/);

    await list.getByRole('button', { name: 'Back to filters' }).tap();
    await expect(panel.getByRole('button', { name: /^All 6 people/ })).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'People', exact: true })).toBeFocused();
  });
});
