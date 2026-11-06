import { expect, test, type Locator, type Page } from '@playwright/test';
import { numberedAssets } from './helpers';
import { mockLibrary, open } from './touchShell';

const MATCHES = ['Iceland', 'Finland', 'landscape', 'Lantern'];
const NAMES = [
  ...MATCHES,
  ...['Aurora', 'Bergen', 'Coast', 'Dawn', 'Fjord', 'Glacier', 'Harbour', 'Ice', 'Moss', 'Snow']
];
const TAGS = NAMES.map((name, index) => ({ id: `tag-${index}`, name, value: name }));
const COUNT = `4 of ${NAMES.length} tags`;

function tagLinks(scope: Locator): Locator {
  return scope.locator('a[href^="/tags/"]');
}

async function expectFiltered(scope: Locator, field: Locator): Promise<void> {
  await field.fill('lan');
  await expect(tagLinks(scope)).toHaveText(MATCHES);
  await expect(tagLinks(scope).locator('mark')).toHaveText(['lan', 'lan', 'lan', 'Lan']);
  await expect(scope.getByRole('status').filter({ hasText: COUNT })).toBeVisible();
}

async function expectCleared(page: Page, scope: Locator, field: Locator): Promise<void> {
  await field.press('Escape');
  await expect(field).toHaveValue('');
  await expect(tagLinks(scope)).toHaveCount(NAMES.length);
  await expect(scope.getByText(COUNT)).toHaveCount(0);
  await field.press('Escape');
  await expect(field).not.toBeFocused();
  await expect(page.locator('mark')).toHaveCount(0);
}

test('the sidebar tag list filters by name and Escape clears it', async ({ page }) => {
  await mockLibrary(page, { assets: numberedAssets(3), tags: TAGS });
  await page.goto('/photos');
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.first().click();
  await expect(page.getByText('1 selected')).toBeVisible();

  const sidebar = page.getByRole('complementary', { name: 'Library' });
  await sidebar.getByRole('button', { name: /^Tags/ }).click();
  const field = sidebar.getByRole('textbox', { name: 'Filter tags' });
  await expectFiltered(sidebar, field);
  await field.press('ArrowDown');
  await expect(tagLinks(sidebar).first()).toBeFocused();
  await field.focus();
  await expectCleared(page, sidebar, field);
  await expect(page.getByText('1 selected')).toBeVisible();

  await field.fill('zzz');
  await expect(sidebar.getByRole('status').filter({ hasText: 'No matching tags' })).toBeVisible();
});

test(
  'the tags page filters by name on a phone',
  { tag: ['@touch', '@phone'] },
  async ({ page }) => {
    await open(page, '/tags', { tags: TAGS });
    const main = page.locator('main');
    const field = main.getByRole('textbox', { name: 'Filter tags' });
    expect((await field.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
    await field.tap();
    await expectFiltered(main, field);
    await expectCleared(page, main, field);
  }
);
