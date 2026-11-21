import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks } from './helpers';
import { tagTree } from './tagMocks';
import { appBar, open as openTouch } from './touchShell';

async function open(page: Page, path: string): Promise<void> {
  await installMocks(page, { tags: tagTree() });
  await page.goto(path);
}

function tree(page: Page): Locator {
  return page.locator('main li');
}

function names(rows: Locator): Promise<string[]> {
  return rows.evaluateAll((items) =>
    items.map((item) => item.querySelector('[data-truncate]')?.textContent?.trim() ?? '')
  );
}

function tagLink(scope: Page | Locator, name: string): Locator {
  return scope.getByRole('link', { name: new RegExp(`^${name}( \\d+)?$`) });
}

function row(page: Page, name: string): Locator {
  return tree(page).filter({ has: tagLink(page, name) });
}

test.describe('tag tree on the desktop', () => {
  test('lists top-level tags by name with counts, colors and no managed tags', async ({ page }) => {
    await open(page, '/tags');
    await expect(page.getByRole('heading', { name: 'Tags' })).toBeVisible();
    expect(await names(tree(page))).toEqual(['Berries', 'Family', 'Film', 'Travel']);
    await expect(row(page, 'Travel')).toContainText('1290');
    await expect(row(page, 'Travel')).toContainText('5');
    await expect(page.locator('main').getByText('Sub-tags', { exact: true })).toBeVisible();
    await expect(row(page, 'Travel').locator('[style*="color"]')).toHaveCSS(
      'color',
      'rgb(62, 142, 208)'
    );
    await expect(page.getByText('reject')).toHaveCount(0);
  });

  test('branches unfold one at a time and stay open after a reload', async ({ page }) => {
    await open(page, '/tags');
    await page.getByRole('button', { name: 'Expand Travel' }).click();
    expect(await names(tree(page))).toEqual([
      'Berries',
      'Family',
      'Film',
      'Travel',
      'Germany',
      'Norway'
    ]);
    await page.getByRole('button', { name: 'Expand Norway' }).click();
    await expect(tagLink(page, 'Lofoten')).toBeVisible();
    await page.reload();
    await expect(tagLink(page, 'Lofoten')).toBeVisible();
    await page.getByRole('button', { name: 'Collapse Travel' }).click();
    await expect(tagLink(page, 'Norway')).toHaveCount(0);
  });

  test('Expand all and Collapse all fold every branch', async ({ page }) => {
    await open(page, '/tags');
    await page.getByRole('button', { name: 'Expand all' }).click();
    await expect(tree(page)).toHaveCount(9);
    await page.getByRole('button', { name: 'Collapse all' }).click();
    await expect(tree(page)).toHaveCount(4);
  });

  test('sorting by photo count orders siblings within their parent', async ({ page }) => {
    await open(page, '/tags');
    await page.getByRole('button', { name: 'Expand all' }).click();
    await page.getByRole('button', { name: /^Sort tags/ }).click();
    await page.getByRole('radio', { name: 'Photo count' }).click();
    await page.keyboard.press('Escape');
    expect(await names(tree(page))).toEqual([
      'Travel',
      'Norway',
      'Lofoten',
      'Bergen',
      'Germany',
      'Berlin',
      'Family',
      'Film',
      'Berries'
    ]);
  });

  test('filtering keeps matches under their dimmed, open parents', async ({ page }) => {
    await open(page, '/tags');
    await page.getByRole('textbox', { name: 'Filter tags' }).fill('ber');
    expect(await names(tree(page))).toEqual([
      'Berries',
      'Travel',
      'Germany',
      'Berlin',
      'Norway',
      'Bergen'
    ]);
    await expect(page.getByRole('status').filter({ hasText: '3 of 9 tags' })).toBeVisible();
    await expect(row(page, 'Travel').getByRole('link')).toHaveClass(/text-dark\/45/);
    await expect(row(page, 'Bergen').locator('mark')).toHaveText('Ber');
    await expect(page.getByRole('button', { name: 'Collapse Travel' })).toHaveCount(0);
    await page.getByRole('textbox', { name: 'Filter tags' }).press('Escape');
    await expect(tree(page)).toHaveCount(4);
  });

  test('a cold tag page names the tag, its path and its sub-tags', async ({ page }) => {
    await open(page, '/tags/norway');
    await expect(page.getByRole('heading', { name: 'Norway' })).toBeVisible();
    const header = page.locator('main header').first();
    await expect(header.getByRole('link', { name: 'Tags' })).toHaveAttribute('href', '/tags');
    await expect(header.getByRole('link', { name: 'Travel' })).toHaveAttribute(
      'href',
      '/tags/travel'
    );
    const subtags = page.getByRole('navigation', { name: 'Sub-tags' });
    await expect(subtags.getByRole('link')).toHaveText([/Bergen\s+54/, /Lofoten\s+186/]);
    await subtags.getByRole('link', { name: /Lofoten/ }).click();
    await expect(page).toHaveURL('/tags/lofoten');
    await expect(page.getByRole('heading', { name: 'Lofoten' })).toBeVisible();
    await expect(page.getByRole('navigation', { name: 'Sub-tags' })).toHaveCount(0);
  });

  test('the sidebar shows the same tree and reveals the open tag', async ({ page }) => {
    await open(page, '/tags/bergen');
    const sidebar = page.getByRole('complementary', { name: 'Library' });
    await sidebar.getByRole('button', { name: 'Show tags' }).click();
    await expect(tagLink(sidebar, 'Bergen')).toHaveAttribute('aria-current', 'page');
    await expect(sidebar.getByRole('button', { name: 'Collapse Norway' })).toBeVisible();
  });
});

test.describe('tag tree on touch', () => {
  test(
    'the phone tag page shows the path under the title',
    { tag: ['@touch', '@phone'] },
    async ({ page }) => {
      await openTouch(page, '/tags/norway', { tags: tagTree() });
      await expect(appBar(page).getByRole('heading', { name: 'Norway' })).toBeVisible();
      await expect(appBar(page)).toContainText('Travel ·');
      await expect(
        page.getByRole('navigation', { name: 'Sub-tags' }).getByRole('link')
      ).toHaveCount(2);
    }
  );

  test('tree rows are tall enough to tap', { tag: ['@touch'] }, async ({ page }) => {
    await openTouch(page, '/tags', { tags: tagTree() });
    await page.getByRole('button', { name: 'Expand Travel' }).tap();
    if ((page.viewportSize()?.width ?? 0) < 768) {
      await expect(page.locator('main').getByText('Sub-tags', { exact: true })).toHaveCount(0);
    }
    for (const item of await tree(page).all()) {
      expect((await item.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
    }
    await tagLink(page, 'Norway').tap();
    await expect(page).toHaveURL('/tags/norway');
  });
});
