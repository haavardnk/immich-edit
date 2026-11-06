import { expect, test } from '@playwright/test';
import { seriousViolations } from './axeHelpers';
import { ASSET_SUMMARY, json, numberedAssets } from './helpers';
import { drag, tapAt } from './touch';
import {
  avatar,
  backButton,
  drawer,
  EDITS,
  expectTab,
  expectTargets,
  mockLibrary,
  open,
  openDrawer,
  tab,
  tabBar
} from './touchShell';

const RUNNING_JOB = {
  id: 'job-1',
  kind: 'export_immich',
  status: 'running',
  target: {},
  params: {},
  total: 4,
  completed: 1,
  failed: 0,
  cancelled_at: null,
  created_at: '2026-11-01T00:00:00Z',
  updated_at: '2026-11-01T00:00:01Z'
};

test.describe('touch browse shell on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the tab bar switches tabs and marks the current one', async ({ page }) => {
    await open(page, '/photos');

    await expect(page.locator('[aria-label="Global navigation"]')).toHaveCount(0);
    await expectTab(page, 'Photos');
    await expectTargets(tabBar(page).getByRole('link'), 44);
    await expect(page.getByRole('heading', { name: 'Photos', level: 1 })).toBeVisible();
    expect(await seriousViolations(page)).toEqual([]);

    await tab(page, 'Albums').tap();
    await expect(page).toHaveURL('/albums');
    await expectTab(page, 'Albums');

    await tab(page, 'Library').tap();
    await expect(page).toHaveURL('/library');
    await expectTab(page, 'Library');

    await page.getByRole('link', { name: /^Favorites/ }).tap();
    await expect(page).toHaveURL('/favorites');
    await expectTab(page, 'Library');

    await tab(page, 'Search').tap();
    await expect(page).toHaveURL('/search');
    await expectTab(page, 'Search');

    await tab(page, 'Photos').tap();
    await expect(page).toHaveURL('/photos');
    await expectTab(page, 'Photos');
  });

  test('the app bar shows the count, the filter badge and one View & filters sheet', async ({
    page
  }) => {
    await open(page, '/photos?make=SONY', {
      assets: [{ ...ASSET_SUMMARY, exifInfo: { make: 'SONY' } }]
    });

    const bar = page.locator('header[data-app-bar]');
    await expect(bar.getByText(/^\d+ photos?$/)).toBeVisible();
    await expect(bar.getByRole('button')).toHaveCount(3);
    await expect(bar.getByRole('button', { name: 'Select photos' })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'View options' })).toHaveCount(0);
    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText(['Make: SONY']);
    await expectTargets(page.getByRole('button', { name: 'Remove Make: SONY' }), 44);

    await bar.getByRole('button', { name: 'View & filters (1 active)' }).tap();
    const sheet = page.getByRole('dialog', { name: 'View & filters' });
    await expect(sheet).toBeVisible();
    const view = sheet.getByRole('button', { name: 'View', exact: true });
    await expect(view).toHaveAccessibleDescription('Date taken, newest · L · Badges');
    await view.tap();
    await expect(sheet.getByRole('radiogroup', { name: 'Sort by' })).toBeVisible();
    await sheet
      .getByRole('radiogroup', { name: 'Thumbnail size' })
      .getByRole('radio', { name: 'XL', exact: true })
      .tap();
    await expect(
      sheet.getByRole('radiogroup', { name: 'Thumbnail size' }).getByRole('radio', { name: 'XL' })
    ).toBeChecked();
    await sheet
      .getByRole('radiogroup', { name: 'Thumbnail info' })
      .getByRole('radio', { name: 'Name and date' })
      .tap();
    await expect(page.getByTestId('tile-name').first()).toBeVisible();
    await view.tap();
    await expect(view).toHaveAccessibleDescription('Date taken, newest · XL · Name and date');
    await sheet.getByRole('button', { name: 'Close view & filters' }).tap();
    await expect(sheet).toBeHidden();
  });

  test('the account drawer opens from the avatar and closes from its button, the scrim or a swipe', async ({
    page
  }) => {
    await open(page, '/photos');

    const panel = await openDrawer(page);
    await expect(panel.getByText('Test User', { exact: true })).toBeVisible();
    await expect(panel.getByText('user@example.com', { exact: true })).toBeVisible();
    await expect(panel.getByText('admin', { exact: true })).toBeVisible();
    await expect(panel.getByText('password', { exact: true })).toBeVisible();
    await expect(panel.getByRole('button', { name: 'Keyboard shortcuts' })).toHaveCount(0);
    await expectTargets(panel.getByRole('radiogroup', { name: 'Theme' }).getByRole('radio'), 44);
    expect(await seriousViolations(page)).toEqual([]);

    await panel.getByRole('button', { name: 'Close account menu' }).tap();
    await expect(panel).toBeHidden();
    await expect(page.getByRole('button', { name: 'Account', exact: true })).toBeFocused();

    await openDrawer(page);
    const viewport = page.viewportSize();
    await tapAt(page, (viewport?.width ?? 400) - 10, (viewport?.height ?? 800) / 2);
    await expect(panel).toBeHidden();

    await openDrawer(page);
    const box = await drawer(page).boundingBox();
    const y = (box?.y ?? 0) + (box?.height ?? 0) / 2;
    await drag(page, { x: 260, y }, { x: 20, y });
    await expect(drawer(page)).toBeHidden();
    await expect(page.getByRole('button', { name: 'Account', exact: true })).toBeVisible();
  });

  test('taps and closing sheets leave no tooltip open', async ({ page }) => {
    await page.clock.install();
    await open(page, '/photos');
    const tooltips = page.locator('.tooltip-content');
    const expectNoTooltip = async () => {
      await page.clock.runFor(1000);
      await expect(tooltips).toBeHidden();
      await expect(tooltips).toHaveCount(0);
    };

    const filters = page
      .locator('header[data-app-bar]')
      .getByRole('button', { name: 'View & filters', exact: true });
    const filterSheet = page.getByRole('dialog', { name: 'View & filters' });
    await filters.tap();
    await expect(filterSheet).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(filterSheet).toBeHidden();
    await expect(filters).toBeFocused();
    await expectNoTooltip();

    await filters.tap();
    await expect(filterSheet).toBeVisible();
    await filterSheet.getByRole('button', { name: 'Close view & filters' }).tap();
    await expect(filterSheet).toBeHidden();
    await expect(filters).toBeFocused();
    await expectNoTooltip();

    await tab(page, 'Library').tap();
    await expect(page).toHaveURL('/library');
    await expectNoTooltip();

    await page.getByRole('link', { name: 'See all people' }).tap();
    await expect(page).toHaveURL('/people');
    await expectNoTooltip();

    await backButton(page).tap();
    await expect(page).toHaveURL('/library');
    await expectNoTooltip();

    const panel = await openDrawer(page);
    await expectNoTooltip();
    await panel.getByRole('button', { name: 'Close account menu' }).tap();
    await expect(panel).toBeHidden();
    await expect(avatar(page)).toBeFocused();
    await expectNoTooltip();
  });

  test('the drawer rows open Settings, Jobs and the theme', async ({ page }) => {
    await open(page, '/photos');

    let panel = await openDrawer(page);
    const saved = page.waitForRequest(
      (request) => request.method() === 'PUT' && request.url().endsWith('/api/appearance')
    );
    await panel
      .getByRole('radiogroup', { name: 'Theme' })
      .getByRole('radio', { name: 'Light' })
      .tap();
    expect((await saved).postDataJSON()).toMatchObject({ mode: 'light' });
    await expect(
      panel.getByRole('radiogroup', { name: 'Theme' }).getByRole('radio', { name: 'Light' })
    ).toBeChecked();

    await panel.getByRole('button', { name: 'Jobs', exact: true }).tap();
    await expect(panel).toBeHidden();
    const jobsSheet = page.getByRole('dialog', { name: 'Jobs' });
    await expect(jobsSheet).toBeVisible();
    await jobsSheet.getByRole('button', { name: 'Close jobs' }).tap();
    await expect(jobsSheet).toBeHidden();

    panel = await openDrawer(page);
    await panel.getByRole('button', { name: 'Settings', exact: true }).tap();
    await expect(page).toHaveURL('/settings');
    await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
    await expect(backButton(page)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Account', exact: true })).toHaveCount(0);
    await expect(tabBar(page).locator('[aria-current="page"]')).toHaveCount(0);
  });

  test('a hardware key adds Keyboard shortcuts to the drawer', async ({ page }) => {
    await open(page, '/photos');
    await page.keyboard.press('Shift');

    const panel = await openDrawer(page);
    await panel.getByRole('button', { name: /^Keyboard shortcuts/ }).tap();
    await expect(page.getByRole('dialog', { name: 'Keyboard shortcuts' })).toBeVisible();
  });

  test('the library lists people, collections and recent edits', async ({ page }) => {
    await open(page, '/library');

    const people = page.getByRole('list', { name: 'People' });
    await expect(people.getByRole('link')).toHaveText(['Ingrid', 'Lars']);

    const collections = page.getByRole('list', { name: 'Collections' });
    await expect(collections.getByRole('link')).toHaveText([
      /^\s*Favorites/,
      /^\s*People\s*2/,
      /^\s*Tags\s*1/,
      /^\s*Folders\s*3/,
      /^\s*Edited\s*5/
    ]);
    await expectTargets(collections.getByRole('link'), 44);
    expect(await seriousViolations(page)).toEqual([]);

    const recent = page.getByRole('list', { name: 'Recently edited' });
    await expect(recent.getByRole('link')).toHaveCount(4);
    await expect(recent.getByRole('link').first()).toHaveAttribute(
      'href',
      new RegExp(`^/assets/${EDITS[4]?.id}`)
    );

    await page.getByRole('link', { name: 'See all people' }).tap();
    await expect(page).toHaveURL('/people');
    await expect(page.locator('header[data-app-bar]').getByText('2 people')).toBeVisible();
    await page.getByRole('link', { name: 'Lars' }).tap();
    await expect(page).toHaveURL('/people/person-lars');
    await expectTab(page, 'Library');

    await page.goto('/tags');
    await expect(page.getByRole('link', { name: 'Sky' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'reject' })).toHaveCount(0);
  });

  test('the albums tab shows two-column cards and sorts them', async ({ page }) => {
    await open(page, '/albums');

    const cards = page.getByRole('list', { name: 'Albums' }).getByRole('link');
    await expect(cards).toHaveText([
      /^\s*Family\s*1 photo\s*$/,
      /^\s*Lofoten winter\s*3 photos\s*$/
    ]);
    const [first, second] = await Promise.all([
      cards.nth(0).boundingBox(),
      cards.nth(1).boundingBox()
    ]);
    expect(Math.abs((first?.y ?? 0) - (second?.y ?? 1))).toBeLessThan(1);
    expect((second?.x ?? 0) > (first?.x ?? 0)).toBe(true);

    await page.getByRole('button', { name: /^Sort albums/ }).tap();
    const sheet = page.getByRole('dialog', { name: 'Sort albums' });
    await expect(sheet).toBeVisible();
    const sorts = sheet.getByRole('radiogroup', { name: 'Sort albums' });
    await expect(sorts.getByRole('radio', { name: 'Recently updated' })).toBeChecked();
    await sorts.getByRole('radio', { name: 'Photo count' }).tap();
    await expect(sorts.getByRole('radio', { name: 'Photo count' })).toBeChecked();
    await sheet.getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(sheet).toBeHidden();
    await expect(page.getByRole('button', { name: 'Sort albums: Photo count' })).toBeVisible();
    await expect(cards).toHaveText([/^\s*Lofoten winter/, /^\s*Family/]);

    await cards.first().tap();
    await expect(page).toHaveURL('/albums/album-lofoten');
    await expectTab(page, 'Albums');
  });

  test('the search tab focuses its field and keeps recent queries', async ({ page }) => {
    await open(page, '/photos');

    await tab(page, 'Search').tap();
    const field = page.getByRole('textbox', { name: 'Search your photos' });
    await expect(field).toBeFocused();
    await expect(page.getByRole('button', { name: 'Cancel', exact: true })).toBeVisible();

    await field.pressSequentially('IMG_0001');
    await expect(page).toHaveURL(/\/search\?q=IMG_0001/);
    await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();
    await expect(page.getByRole('button', { name: 'File name', exact: true })).toHaveAttribute(
      'aria-pressed',
      'true'
    );

    await page.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await expect(page).toHaveURL('/search');
    await expect(field).toHaveValue('');
    const recent = page.getByRole('region', { name: 'Recent' });
    await expect(recent.getByRole('button', { name: 'IMG_0001', exact: true })).toBeVisible();
    await expect(recent.getByRole('listitem')).toHaveCount(1);

    await recent.getByRole('button', { name: 'IMG_0001', exact: true }).tap();
    await expect(page).toHaveURL(/\/search\?q=IMG_0001/);
    await expect(field).toHaveValue('IMG_0001');

    await page.reload();
    await expect(field).toHaveValue('IMG_0001');
    await expect(field).not.toBeFocused();

    await page.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await recent
      .getByRole('button', { name: 'Remove IMG_0001 from recent searches', exact: true })
      .tap();
    await expect(recent).toHaveCount(0);

    await page.reload();
    await expect(page.getByRole('region', { name: 'Recent' })).toHaveCount(0);

    await field.pressSequentially('red ');
    await expect(page).toHaveURL(/\/search\?q=red$/);
    await field.pressSequentially('car');
    await expect(page).toHaveURL(/\/search\?q=red(\+|%20)car$/);
    await expect(field).toHaveValue('red car');
    await page.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await expect(page.getByRole('region', { name: 'Recent' }).getByRole('listitem')).toHaveText([
      /^\s*red car\s*$/
    ]);
  });

  test('folders drill down with a breadcrumb', async ({ page }) => {
    await open(page, '/folders');

    const subfolders = page.getByRole('list', { name: 'Subfolders' });
    await expect(subfolders.getByRole('link')).toHaveText([/^\s*photos\s*2 folders\s*$/]);
    await subfolders.getByRole('link', { name: /^photos/ }).tap();
    await expect(page).toHaveURL('/folders?path=%2Fphotos');

    await expect(subfolders.getByRole('link')).toHaveText([
      /^\s*2026\s*2 folders\s*$/,
      /^\s*2025\s*$/
    ]);
    await subfolders.getByRole('link', { name: /^2026/ }).tap();
    await subfolders.getByRole('link', { name: /^Norway/ }).tap();
    await expect(page).toHaveURL('/folders?path=%2Fphotos%2F2026%2FNorway');
    await expect(page.getByRole('heading', { name: 'Norway', level: 1 })).toBeVisible();
    await expect(page.getByRole('link', { name: ASSET_SUMMARY.originalFileName })).toBeVisible();

    const crumbs = page.getByRole('navigation', { name: 'Folder path' });
    await expect(crumbs.getByRole('link')).toHaveText(['Folders', 'photos', '2026', 'Norway']);
    const current = crumbs.getByRole('link', { name: 'Norway' });
    await expect(current).toHaveAttribute('aria-current', 'page');
    await expect(current).toBeInViewport();
    await expectTargets(crumbs.getByRole('link'), 44);

    await crumbs.getByRole('link', { name: 'photos' }).tap();
    await expect(page).toHaveURL('/folders?path=%2Fphotos');
    await expectTab(page, 'Library');
  });

  test('landscape hides the bars while scrolling down', async ({ page }) => {
    await page.setViewportSize({ width: 839, height: 412 });
    await open(page, '/photos', { assets: numberedAssets(80) });
    await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toBeVisible();
    await expect(page.locator('html')).not.toHaveAttribute('data-wide');

    const appBar = page.locator('header[data-app-bar]');
    const scrollBy = (delta: number) =>
      page.getByTestId('browse-scroller').evaluate(async (element, by) => {
        element.scrollTop += by;
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      }, delta);

    await scrollBy(20);
    await scrollBy(300);
    await expect(tabBar(page)).toHaveAttribute('inert', '');
    await expect(appBar).toHaveAttribute('inert', '');
    await expect(tabBar(page)).not.toBeInViewport();
    await expect(tab(page, 'Photos')).not.toBeFocused();

    await scrollBy(-40);
    await scrollBy(-60);
    await expect(tabBar(page)).not.toHaveAttribute('inert');
    await expect(appBar).not.toHaveAttribute('inert');
    await expect(tabBar(page)).toBeInViewport();
  });
});

test.describe('touch browse shell on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('landscape puts the brand and search on top and the account and running jobs at the foot of a touch-sized sidebar', async ({
    page
  }) => {
    await mockLibrary(page, { assets: numberedAssets(40) });
    await page.route('**/api/jobs', (route) => route.fulfill(json([RUNNING_JOB])));
    await page.route('**/api/jobs/job-1/events', (route) =>
      route.fulfill({ status: 200, contentType: 'text/event-stream', body: '' })
    );
    await page.goto('/photos');
    await expect(page.locator('html')).toHaveAttribute('data-wide');

    const sidebar = page.getByRole('complementary', { name: 'Library' });
    await expect(sidebar).toBeVisible();
    expect((await page.locator('main').boundingBox())?.x).toBeCloseTo(280, 0);
    await expectTargets(sidebar.getByRole('link'), 44);
    await expect(tabBar(page)).toHaveCount(0);
    await expect(page.locator('[aria-label="Global navigation"]')).toHaveCount(0);

    const brand = page.getByRole('img', { name: 'immich-edit' });
    const search = page.getByRole('search');
    await expect(brand).toBeVisible();
    await expect(search).toBeVisible();
    const [brandBox, searchBox, sidebarBox, headerBox] = await Promise.all([
      brand.boundingBox(),
      search.boundingBox(),
      sidebar.boundingBox(),
      page.locator('main header').first().boundingBox()
    ]);
    expect(brandBox?.y ?? 0).toBeLessThan(searchBox?.y ?? 0);
    expect((searchBox?.y ?? 0) + (searchBox?.height ?? 0)).toBeLessThanOrEqual(sidebarBox?.y ?? 0);
    expect((searchBox?.x ?? 0) + (searchBox?.width ?? 0)).toBeLessThanOrEqual(280);
    expect(headerBox?.y).toBe(0);

    await expect(page.getByRole('button', { name: 'Jobs', exact: true })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Settings', exact: true })).toHaveCount(1);
    await expect(
      page.locator('main header').getByRole('button', { name: 'View & filters', exact: true })
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Thumbnail size XL' })).toHaveCount(0);
    await expect
      .poll(async () => (await page.locator('[role="group"]').first().boundingBox())?.height ?? 0)
      .toBeGreaterThan(130);

    await expect(page.getByText('Test User', { exact: true })).toBeVisible();
    await expect(page.getByText('photos.example.net', { exact: true })).toBeVisible();
    await expect(page.getByText(new URL(page.url()).host, { exact: true })).toHaveCount(0);
    const jobsCard = page.getByRole('button', { name: 'Jobs: Exporting 1 of 4' });
    await expect(jobsCard).toBeVisible();
    await expect(avatar(page)).toHaveCount(1);
    const [card, account] = await Promise.all([jobsCard.boundingBox(), avatar(page).boundingBox()]);
    expect((card?.y ?? 0) + (card?.height ?? 0)).toBeLessThanOrEqual(account?.y ?? 0);
    const viewportHeight = page.viewportSize()?.height ?? 0;
    expect((account?.y ?? 0) + (account?.height ?? 0)).toBeGreaterThan(viewportHeight - 70);

    await jobsCard.tap();
    const jobsDialog = page.getByRole('dialog', { name: 'Jobs' });
    await expect(jobsDialog).toBeVisible();
    await jobsDialog.getByRole('button', { name: 'Close jobs' }).tap();
    await expect(jobsDialog).toBeHidden();

    await openDrawer(page);
    await drawer(page).getByRole('button', { name: 'Close account menu' }).tap();
    await expect(drawer(page)).toBeHidden();

    await page.getByRole('button', { name: 'Settings', exact: true }).tap();
    await expect(page).toHaveURL('/settings');
    await expect(page.getByRole('button', { name: 'Close settings' })).toBeVisible();
  });

  test('landscape uses the sidebar shell, searches from it and renders the index pages', async ({
    page
  }) => {
    await open(page, '/library');

    await expect(page.locator('html')).toHaveAttribute('data-wide');
    await expect(page.locator('[aria-label="Global navigation"]')).toHaveCount(0);
    await expect(tabBar(page)).toHaveCount(0);
    await expect(page.getByRole('list', { name: 'Collections' }).getByRole('link')).toHaveCount(5);

    const field = page.getByRole('search').getByRole('textbox', { name: 'Search your photos' });
    await field.tap();
    await field.fill('fjord');
    await field.press('Enter');
    await expect(page).toHaveURL('/search?q=fjord');
    await expect(field).toHaveValue('fjord');

    await page.goto('/albums');
    await expect(page.getByRole('list', { name: 'Albums' }).getByRole('link')).toHaveCount(2);

    await page.getByRole('button', { name: /^People/ }).tap();
    await page.getByRole('link', { name: 'See all people' }).tap();
    await expect(page).toHaveURL('/people');
    await expect(page.getByRole('list', { name: 'People' }).getByRole('link')).toHaveText([
      'Ingrid',
      'Lars'
    ]);
  });

  test('portrait uses the tab bar and the drawer', async ({ page }) => {
    await page.setViewportSize({ width: 820, height: 1180 });
    await open(page, '/photos');

    await expect(page.locator('html')).not.toHaveAttribute('data-wide');
    await expectTab(page, 'Photos');
    await openDrawer(page);
    await drawer(page).getByRole('button', { name: 'Settings', exact: true }).tap();
    await expect(page).toHaveURL('/settings');

    await tab(page, 'Library').tap();
    await expect(page.getByRole('list', { name: 'Collections' }).getByRole('link')).toHaveCount(5);
  });

  test('a mouse hover after a tap still shows tooltips', async ({ page }) => {
    await page.setViewportSize({ width: 820, height: 1180 });
    await page.clock.install();
    await open(page, '/photos');
    const tooltips = page.locator('.tooltip-content');

    await tab(page, 'Library').tap();
    await expect(page).toHaveURL('/library');
    await page.clock.runFor(1000);
    await expect(tooltips).toHaveCount(0);

    const box = await avatar(page).boundingBox();
    const x = (box?.x ?? 0) + (box?.width ?? 0) / 2;
    const y = (box?.y ?? 0) + (box?.height ?? 0) / 2;
    await page.mouse.move(x, y + 60);
    await page.mouse.move(x, y, { steps: 4 });
    await page.clock.runFor(1000);
    await expect(tooltips.filter({ hasText: 'Account' })).toBeVisible();

    await page.mouse.move(x, y + 300);
    await expect(tooltips).toHaveCount(0);
    await avatar(page).tap();
    const panel = drawer(page);
    await expect(panel).toBeVisible();
    await panel.getByRole('button', { name: 'Close account menu' }).tap();
    await expect(panel).toBeHidden();
    await expect(avatar(page)).toBeFocused();
    await page.clock.runFor(1000);
    await expect(tooltips).toHaveCount(0);
  });
});

test.describe('touch browse shell on WebKit', { tag: '@touch-smoke' }, () => {
  test('the tab bar sits above the bottom inset and the drawer opens and closes', async ({
    page
  }) => {
    await open(page, '/photos');

    const bar = tabBar(page);
    await expect(bar).toBeVisible();
    const [box, height] = await Promise.all([
      bar.boundingBox(),
      page.evaluate(() => window.innerHeight)
    ]);
    expect((box?.y ?? Number.POSITIVE_INFINITY) + (box?.height ?? 0)).toBeLessThanOrEqual(height);
    await expectTab(page, 'Photos');

    await page.getByRole('button', { name: 'Account', exact: true }).tap();
    const panel = drawer(page);
    await expect(panel).toBeVisible();
    await panel.getByRole('button', { name: 'Close account menu' }).tap();
    await expect(panel).toBeHidden();
  });
});
