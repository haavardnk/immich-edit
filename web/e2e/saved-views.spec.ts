import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks, numberedAssets, openFilters, type MockSavedView } from './helpers';
import { longPress } from './touch';
import { open } from './touchShell';

const ASSETS = numberedAssets(6);

function sidebar(page: Page): Locator {
  return page.getByRole('complementary', { name: 'Library' });
}

function savedList(scope: Locator | Page): Locator {
  return scope.getByRole('list', { name: 'Saved views' });
}

function sizeButton(page: Page, size: string): Locator {
  return page.getByRole('button', { name: `Thumbnail size ${size}`, exact: true });
}

async function pickSort(page: Page, name: string): Promise<void> {
  await page.getByRole('button', { name: /^Sort:/ }).click();
  await page.locator('[data-popover-content]').getByRole('radio', { name, exact: true }).click();
  await page.keyboard.press('Escape');
}

test.describe('saved views on desktop', () => {
  test('saving the view keeps filters, sort and size and reopens them', async ({ page }) => {
    const sorts: unknown[] = [];
    await installMocks(page, {
      assets: ASSETS,
      onMetadata: (body) => {
        const sort = body.sort;
        sorts.push(
          typeof sort === 'object' && sort !== null && 'direction' in sort ? sort.direction : null
        );
      }
    });
    await page.goto('/photos?favorite=1&country=Norway');

    await sizeButton(page, 'L').click();
    await pickSort(page, 'Oldest first');
    const panel = await openFilters(page);
    await panel.getByRole('button', { name: 'Save view', exact: true }).click();
    const name = panel.getByRole('textbox', { name: 'View name' });
    await expect(name).toBeFocused();
    await expect(name).toHaveValue('Favorites · Norway');
    await name.fill('Norway favourites');
    await name.press('Enter');

    await expect(page.getByText('Saved view “Norway favourites”')).toBeVisible();
    await expect(panel.getByRole('textbox', { name: 'View name' })).toHaveCount(0);
    const row = savedList(sidebar(page)).getByRole('link', { name: 'Norway favourites' });
    await expect(row).toHaveAttribute('aria-current', 'page');

    await sidebar(page).getByRole('link', { name: 'Photos', exact: true }).click();
    await expect(page).toHaveURL(/\/photos$/);
    await expect(row).not.toHaveAttribute('aria-current', 'page');
    await sizeButton(page, 'S').click();
    await pickSort(page, 'Newest first');

    sorts.length = 0;
    await row.click();
    await expect(page).toHaveURL(/\/photos\?favorite=1&country=Norway$/);
    await expect(row).toHaveAttribute('aria-current', 'page');
    await expect(sizeButton(page, 'L')).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByRole('button', { name: /^Sort:/ })).toHaveAccessibleName(
      'Sort: date taken, oldest first'
    );
    await expect.poll(() => sorts.at(-1)).toBe('asc');
  });

  test('a saved view can be renamed and a deletion undone, keeping focus on the list', async ({
    page
  }) => {
    const at = '2026-01-01T00:00:00Z';
    const views: MockSavedView[] = [
      { id: 'view-1', name: 'To edit', url: '/photos?rating=3', created_at: at, updated_at: at },
      { id: 'view-2', name: 'Zoo', url: '/photos?favorite=1', created_at: at, updated_at: at }
    ];
    await installMocks(page, { assets: ASSETS, savedViews: views });
    await page.goto('/photos');

    const list = savedList(sidebar(page));
    await list.getByRole('link', { name: 'To edit' }).click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Rename' }).click();
    const field = list.getByRole('textbox', { name: 'Rename To edit' });
    await expect(field).toBeFocused();
    await expect(field).toHaveValue('To edit');
    await field.press('Escape');
    await expect(field).toHaveCount(0);
    await expect(list.getByRole('link', { name: 'To edit' })).toBeFocused();

    await list.getByRole('link', { name: 'To edit' }).click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Rename' }).click();
    await field.fill('Three stars');
    await field.press('Enter');
    await expect(list.getByRole('link', { name: 'Three stars' })).toBeFocused();

    await list.getByRole('link', { name: 'Three stars' }).click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Delete' }).click();
    await expect(page.getByText('Deleted “Three stars”')).toBeVisible();
    await expect(list.getByRole('link', { name: 'Three stars' })).toHaveCount(0);
    await expect(list.getByRole('link', { name: 'Zoo' })).toBeFocused();
    await page.getByRole('button', { name: 'Undo' }).click();
    await expect(list.getByRole('link', { name: 'Three stars' })).toBeVisible();
    await page.reload();
    await expect(savedList(sidebar(page)).getByRole('link', { name: 'Three stars' })).toBeVisible();
  });

  test('the save form stays in the panel and Escape only closes the form', async ({ page }) => {
    await installMocks(page, { assets: ASSETS });
    await page.goto('/photos');

    const panel = await openFilters(page);
    const toggle = panel.getByRole('button', { name: 'Save view', exact: true });
    await toggle.click();
    const name = panel.getByRole('textbox', { name: 'View name' });
    await expect(name).toHaveValue('Photos');
    await name.fill('   ');
    await expect(panel.getByRole('button', { name: 'Save', exact: true })).toBeDisabled();
    await name.press('Escape');
    await expect(name).toHaveCount(0);
    await expect(panel).toBeVisible();
    await expect(toggle).toBeFocused();
  });
});

test.describe('saved views on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('closing the panel from the app bar discards the open save form', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });

    const panel = await openFilters(page);
    await panel.getByRole('button', { name: 'Save view', exact: true }).tap();
    await expect(panel.getByRole('textbox', { name: 'View name' })).toBeFocused();
    const toggle = page.locator('main').getByRole('button', { name: /^View & filters/ });
    await toggle.tap();
    await expect(panel).toBeHidden();

    await toggle.tap();
    await expect(panel).toBeVisible();
    await expect(panel.getByRole('textbox', { name: 'View name' })).toHaveCount(0);
    await expect(panel.getByRole('button', { name: 'Save view', exact: true })).toHaveAttribute(
      'aria-expanded',
      'false'
    );
  });
});

test.describe('saved views on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('a view saved from the sheet opens from the library and can be deleted', async ({
    page
  }) => {
    await open(page, '/photos?favorite=1', { assets: ASSETS });

    await page
      .locator('main')
      .getByRole('button', { name: /^View & filters/ })
      .tap();
    const sheet = page.getByRole('dialog', { name: 'View & filters' });
    await sheet.getByRole('button', { name: 'Save view', exact: true }).tap();
    const name = sheet.getByRole('textbox', { name: 'View name' });
    await expect(name).toHaveValue('Favorites');
    await name.press('Escape');
    await expect(name).toHaveCount(0);
    await expect(sheet).toBeVisible();
    await expect(sheet.getByRole('button', { name: 'Save view', exact: true })).toBeFocused();

    await sheet.getByRole('button', { name: 'Save view', exact: true }).tap();
    await name.fill('Best');
    await sheet.getByRole('button', { name: 'Save', exact: true }).tap();
    await expect(page.getByText('Saved view “Best”')).toBeVisible();
    await sheet.getByRole('button', { name: 'Close view & filters' }).tap();
    await expect(sheet).toBeHidden();

    await page.goto('/library');
    const row = savedList(page).getByRole('link', { name: 'Best' });
    await expect(row).toBeVisible();
    expect((await row.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
    await row.tap();
    await expect(page).toHaveURL(/\/photos\?favorite=1$/);
    await expect(
      page.locator('main').getByRole('button', { name: 'View & filters (1 active)' })
    ).toBeVisible();

    await page.goto('/library');
    const box = await savedList(page).getByRole('link', { name: 'Best' }).boundingBox();
    await longPress(page, (box?.x ?? 0) + 40, (box?.y ?? 0) + (box?.height ?? 0) / 2);
    const actions = page.getByRole('dialog', { name: 'Actions' });
    await actions.getByRole('button', { name: 'Delete' }).tap();
    await expect(page.getByText('Deleted “Best”')).toBeVisible();
    await expect(actions).toBeHidden();
    await expect(page.locator('body')).not.toHaveCSS('pointer-events', 'none');
    await expect(savedList(page)).toHaveCount(0);
    await expect(page.getByRole('heading', { name: 'Saved views' })).toHaveCount(0);
  });

  test('a view renamed from the action sheet keeps focus on its row', async ({ page }) => {
    const at = '2026-01-01T00:00:00Z';
    await open(page, '/library', {
      assets: ASSETS,
      savedViews: [
        { id: 'view-1', name: 'To edit', url: '/photos?rating=3', created_at: at, updated_at: at }
      ]
    });

    const list = savedList(page);
    const box = await list.getByRole('link', { name: 'To edit' }).boundingBox();
    await longPress(page, (box?.x ?? 0) + 40, (box?.y ?? 0) + (box?.height ?? 0) / 2);
    const actions = page.getByRole('dialog', { name: 'Actions' });
    await actions.getByRole('button', { name: 'Rename' }).tap();
    await expect(actions).toBeHidden();
    const field = list.getByRole('textbox', { name: 'Rename To edit' });
    await expect(field).toBeFocused();
    await field.fill('Keepers');
    await field.press('Enter');
    await expect(list.getByRole('link', { name: 'Keepers' })).toBeFocused();
    await expect(page.getByRole('link', { name: 'To edit' })).toHaveCount(0);
  });
});
