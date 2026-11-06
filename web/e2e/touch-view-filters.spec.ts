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

  test('the View section changes the thumbnail size and the sort', async ({ page }) => {
    await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
    await open(page, '/photos', { assets: ASSETS });
    await expect(page.getByTestId('day-header').first()).toContainText('Saturday 31 October');

    const panel = await openSurface(page);
    const view = panel.getByRole('button', { name: 'View', exact: true });
    await expect(view).toHaveAccessibleDescription(/^Date taken, newest · X?L · Badges$/);
    await view.tap();
    await expect(view).toHaveAttribute('aria-expanded', 'true');

    const sizes = panel.getByRole('radiogroup', { name: 'Thumbnail size' });
    await expect(sizes.getByRole('radio', { checked: true })).toHaveText(DEFAULT_SIZE);
    await sizes.getByRole('radio', { name: 'S', exact: true }).tap();
    await expect(sizes.getByRole('radio', { name: 'S', exact: true })).toBeChecked();
    await expect
      .poll(async () => (await page.locator('[role="group"]').first().boundingBox())?.height ?? 0)
      .toBeLessThan(100);

    await panel
      .getByRole('radiogroup', { name: 'Order' })
      .getByRole('radio', { name: 'Oldest first' })
      .tap();
    await expect(page.getByTestId('day-header').first()).toContainText('Friday 30 October');

    await view.tap();
    await expect(view).toHaveAccessibleDescription('Date taken, oldest · S · Badges');
    await closeSurface(page);

    await openSurface(page);
    await expect(surface(page).getByRole('button', { name: 'View', exact: true })).toHaveAttribute(
      'aria-expanded',
      'false'
    );
  });

  test('the View section is the first section and keeps its open state', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const panel = await openSurface(page);
    const sections = panel.locator('[data-collapsible-trigger]');
    await expect(sections.first()).toHaveAccessibleName('View');
    await panel.getByRole('button', { name: 'View', exact: true }).tap();
    await closeSurface(page);

    await page.reload();
    const again = await openSurface(page);
    await expect(again.getByRole('radiogroup', { name: 'Thumbnail size' })).toBeVisible();
  });

  for (const { path, summary } of [
    { path: '/albums/album-lofoten', summary: /^Date taken, (newest|oldest) · X?L · Badges$/ },
    { path: '/people/person-ingrid', summary: /^Date taken, (newest|oldest) · X?L · Badges$/ },
    { path: '/tags/tag-sky', summary: /^Date taken, (newest|oldest) · X?L · Badges$/ },
    {
      path: '/folders?path=/photos/2026/Norway',
      summary: /^Date taken, (newest|oldest) · X?L · Badges$/
    },
    { path: '/search?q=IMG', summary: /^X?L · Badges$/ },
    { path: '/edited', summary: /^(Newest|Oldest) edit · X?L · Badges$/ }
  ]) {
    test(`view options stay reachable on ${path}`, async ({ page }) => {
      await open(page, path);
      const panel = await openSurface(page);
      const view = panel.getByRole('button', { name: 'View', exact: true });
      await expect(view).toHaveAccessibleDescription(summary);
      await view.tap();
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail size' })).toBeVisible();
      await expect(panel.getByRole('radiogroup', { name: 'Thumbnail info' })).toBeVisible();
    });
  }
});
