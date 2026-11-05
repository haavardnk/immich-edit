import { expect, test, type Locator, type Page } from '@playwright/test';
import { numberedAssets } from './helpers';
import { ALBUMS, PEOPLE, TAGS, mockLibrary } from './touchShell';

const LONG = 'Lofoten winter expedition along the Atlantic coast and the fjords '.repeat(4).trim();
const LONG_PERSON = 'Ingrid Margrethe Kristoffersen-Halvorsen';
const LONG_FOLDERS = [`/photos/${LONG}/${LONG}`, '/photos/2026/Bergen'];

async function expectTooltipTwice(page: Page, target: Locator, text: string | RegExp) {
  const tooltip = page.locator('.tooltip-content', { hasText: text });
  for (let pass = 0; pass < 2; pass++) {
    await target.hover();
    await expect(tooltip).toBeVisible();
    await page.mouse.move(1, 1);
    await expect(tooltip).toBeHidden();
  }
}

async function mockLongNames(page: Page): Promise<void> {
  await page.addInitScript(
    (query) =>
      localStorage.setItem(
        'immich-edit:recentSearches',
        JSON.stringify({ items: [query, 'fjord'] })
      ),
    LONG
  );
  await mockLibrary(page, {
    assets: numberedAssets(5),
    people: PEOPLE.map((person, index) =>
      index === 0 ? { ...person, name: LONG_PERSON } : person
    ),
    albums: ALBUMS.map((album, index) => (index === 0 ? { ...album, albumName: LONG } : album)),
    tags: TAGS.map((tag, index) => (index === 0 ? { ...tag, name: LONG, value: LONG } : tag))
  });
  await page.route('**/api/folders/paths', (route) => route.fulfill({ json: LONG_FOLDERS }));
}

test('library, album, folder, tag and search triggers show styled tooltips', async ({ page }) => {
  await mockLongNames(page);
  const main = page.locator('main');

  await page.goto('/library');
  await expectTooltipTwice(
    page,
    main.getByRole('link', { name: 'See all people' }),
    'See all people'
  );
  await expectTooltipTwice(
    page,
    main.getByRole('list', { name: 'People' }).getByRole('link', { name: LONG_PERSON }),
    LONG_PERSON
  );
  await expectTooltipTwice(
    page,
    main.getByRole('list', { name: 'Recently edited' }).getByRole('link').first(),
    /^(IMG_\d{4}\.ARW|Open recent edit)$/
  );

  await page.goto('/albums');
  await expectTooltipTwice(
    page,
    main.getByRole('list', { name: 'Albums' }).getByRole('link', { name: /^Lofoten winter/ }),
    LONG
  );
  await expectTooltipTwice(
    page,
    main.getByRole('button', { name: /^Sort albums/ }),
    /^Sort albums:/
  );

  const deep = `/folders?path=${encodeURIComponent(`/photos/${LONG}/${LONG}`)}`;
  await page.goto(deep);
  await expectTooltipTwice(
    page,
    main.getByRole('navigation', { name: 'Folder path' }).getByRole('link').nth(2),
    LONG
  );

  await page.goto(`/folders?path=${encodeURIComponent('/photos')}`);
  await expectTooltipTwice(
    page,
    main.getByRole('list', { name: 'Subfolders' }).getByRole('link', { name: /^Lofoten/ }),
    LONG
  );

  await page.goto('/tags');
  await expectTooltipTwice(page, main.getByRole('link', { name: /^Lofoten/ }), LONG);

  await page.goto('/search');
  const recent = main.getByRole('region', { name: 'Recent' });
  await expectTooltipTwice(page, recent.getByRole('button', { name: LONG, exact: true }), LONG);
  await expectTooltipTwice(
    page,
    recent.getByRole('button', { name: 'Remove fjord from recent searches' }),
    'Remove fjord from recent searches'
  );
});

test('names that fit show no tooltip', async ({ page }) => {
  await page.clock.install();
  await mockLongNames(page);
  const main = page.locator('main');
  const tooltips = page.locator('.tooltip-content');

  async function expectNoTooltip(target: Locator): Promise<void> {
    await target.hover();
    await page.clock.runFor(1500);
    await expect(tooltips).toHaveCount(0);
    await page.mouse.move(1, 1);
  }

  await page.goto('/library');
  await expectNoTooltip(
    main.getByRole('list', { name: 'People' }).getByRole('link', { name: 'Lars', exact: true })
  );

  await page.goto('/albums');
  const albums = main.getByRole('list', { name: 'Albums' });
  await expectNoTooltip(albums.getByRole('link', { name: /^Family/ }));
  await albums.getByRole('link', { name: /^Lofoten winter/ }).hover();
  await page.clock.runFor(1500);
  await expect(page.locator('.tooltip-content', { hasText: LONG })).toBeVisible();
  await page.mouse.move(1, 1);

  await page.goto(`/folders?path=${encodeURIComponent('/photos/2026')}`);
  await expectNoTooltip(
    main.getByRole('navigation', { name: 'Folder path' }).getByRole('link', { name: '2026' })
  );
  await expectNoTooltip(
    main.getByRole('list', { name: 'Subfolders' }).getByRole('link', { name: /^Bergen/ })
  );

  await page.goto('/search');
  await expectNoTooltip(
    main.getByRole('region', { name: 'Recent' }).getByRole('button', { name: 'fjord', exact: true })
  );
});
