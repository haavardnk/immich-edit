import { expect, test, type Locator, type Page } from '@playwright/test';
import { apiRequest, installMocks, type InstallOpts } from './helpers';
import { tagTree } from './tagMocks';
import { longPressUntil } from './touch';
import { open as openTouch } from './touchShell';

async function open(page: Page, path: string, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, {
    tags: tagTree(),
    metadataPushConsented: true,
    tagRename: true,
    ...opts
  });
  await page.goto(path);
}

function tagLink(scope: Page | Locator, name: string): Locator {
  return scope.getByRole('link', { name: new RegExp(`^${name}( \\d+)?$`) });
}

function row(page: Page, name: string): Locator {
  return page.locator('main li').filter({ has: tagLink(page, name) });
}

async function rowMenu(page: Page, name: string): Promise<Locator> {
  const target = row(page, name);
  await target.hover();
  await target.getByRole('button', { name: 'Tag actions' }).click();
  return page.locator('[data-popover-content]');
}

const LIST_ITEMS = ['Open tag', 'Rename', 'Change color', 'New sub-tag', 'Delete tag'];

test.describe('tag actions on the desktop', () => {
  test('the row, right-click and page menus list the tag actions', async ({ page }) => {
    await open(page, '/tags');
    const menu = await rowMenu(page, 'Travel');
    await expect(menu.getByRole('button')).toHaveText(LIST_ITEMS);
    await page.keyboard.press('Escape');

    await row(page, 'Film').click({ button: 'right' });
    await expect(page.getByRole('menu').getByRole('menuitem')).toHaveText(LIST_ITEMS);
    await page.keyboard.press('Escape');

    await page.goto('/tags/norway');
    await page.locator('main header').getByRole('button', { name: 'Tag actions' }).click();
    await expect(page.locator('[data-popover-content]').getByRole('button')).toHaveText(
      LIST_ITEMS.slice(1)
    );
  });

  test('renaming checks siblings and renames the whole branch', async ({ page }) => {
    await open(page, '/tags');
    await page.getByRole('button', { name: 'Expand Travel' }).click();
    await (await rowMenu(page, 'Norway')).getByRole('button', { name: 'Rename' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tag details' });
    await expect(dialog).toContainText('Travel / Norway');
    await expect(dialog).toContainText('Its 2 sub-tags move with it');
    const name = dialog.getByRole('textbox', { name: 'Name' });
    await expect(name).toBeFocused();
    await name.fill('germany');
    await expect(dialog).toContainText('Travel / Germany already exists');
    await expect(dialog.getByRole('button', { name: 'Save' })).toBeDisabled();
    await name.fill('Norge');
    const request = apiRequest(page, 'PATCH', '/api/tags/norway');
    await dialog.getByRole('button', { name: 'Save' }).click();
    expect((await request).postDataJSON()).toEqual({ name: 'Norge' });
    await expect(dialog).toBeHidden();
    await expect(tagLink(page, 'Norge')).toBeVisible();
    await page.getByRole('button', { name: 'Expand Norge' }).click();
    await page.getByRole('link', { name: /^Lofoten/ }).click();
    await expect(page.locator('main header').getByRole('link', { name: 'Norge' })).toBeVisible();
  });

  test('an older Immich keeps the name and still changes the color', async ({ page }) => {
    await open(page, '/tags', { tagRename: false });
    await (await rowMenu(page, 'Film')).getByRole('button', { name: 'Rename' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tag details' });
    await expect(dialog.getByRole('textbox', { name: 'Name' })).toHaveAttribute('readonly', '');
    await expect(dialog).toContainText('Renaming tags needs Immich 3.3 or later');
    await dialog.getByRole('radio', { name: 'Red' }).click();
    const request = apiRequest(page, 'PATCH', '/api/tags/film');
    await dialog.getByRole('button', { name: 'Save' }).click();
    expect((await request).postDataJSON()).toEqual({ color: '#e5484d' });
    await expect(row(page, 'Film').locator('[style*="color"]')).toHaveCSS(
      'color',
      'rgb(229, 72, 77)'
    );
  });

  test('Change color focuses the swatches and None clears the color', async ({ page }) => {
    await open(page, '/tags');
    await (await rowMenu(page, 'Travel')).getByRole('button', { name: 'Change color' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tag details' });
    await expect(dialog.getByRole('radio', { name: 'Blue' })).toBeFocused();
    await dialog.getByRole('radio', { name: 'No color' }).click();
    const request = apiRequest(page, 'PATCH', '/api/tags/travel');
    await dialog.getByRole('button', { name: 'Save' }).click();
    expect((await request).postDataJSON()).toEqual({ color: null });
  });

  test('Create tag places the new tag inside the chosen parent', async ({ page }) => {
    await open(page, '/tags');
    await page.locator('main header').getByRole('button', { name: 'Create tag' }).click();
    const dialog = page.getByRole('dialog', { name: 'Create tag' });
    await dialog.getByRole('textbox', { name: 'Name' }).fill('Svalbard');
    await dialog.getByRole('combobox', { name: 'Inside' }).click();
    await page.getByRole('option', { name: 'Travel / Norway', exact: true }).click();
    await expect(dialog).toContainText('Creates Travel / Norway / Svalbard');
    const request = apiRequest(page, 'POST', '/api/tags');
    await dialog.getByRole('button', { name: 'Create' }).click();
    expect((await request).postDataJSON()).toEqual({ name: 'Svalbard', parentId: 'norway' });
    await expect(dialog).toBeHidden();
    await expect(row(page, 'Svalbard')).toContainText('0');
  });

  test('New sub-tag starts inside the tag and refuses a sibling name', async ({ page }) => {
    await open(page, '/tags');
    await (await rowMenu(page, 'Travel')).getByRole('button', { name: 'New sub-tag' }).click();
    const dialog = page.getByRole('dialog', { name: 'Create tag' });
    await dialog.getByRole('textbox', { name: 'Name' }).fill('Norway');
    await expect(dialog).toContainText('Travel / Norway already exists');
    await expect(dialog.getByRole('button', { name: 'Create' })).toBeDisabled();
    await dialog.getByRole('textbox', { name: 'Name' }).fill('Sweden');
    await expect(dialog).toContainText('Creates Travel / Sweden');

    const inside = await dialog.getByRole('combobox', { name: 'Inside' }).boundingBox();
    const clear = dialog.getByRole('button', { name: 'Clear Travel' });
    const cross = await clear.boundingBox();
    expect(
      cross &&
        inside &&
        cross.y >= inside.y &&
        cross.y + cross.height <= inside.y + inside.height + 1
    ).toBe(true);
    await clear.click();
    await expect(dialog.getByRole('combobox', { name: 'Inside' })).toHaveAttribute(
      'placeholder',
      'Top level'
    );
    await expect(dialog).toContainText('Creates Sweden');
    await expect(dialog.getByRole('option', { name: 'Top level' })).toHaveCount(0);
  });

  test('deleting a tag from its page names the branch and returns to Tags', async ({ page }) => {
    await open(page, '/tags/norway');
    await page.locator('main header').getByRole('button', { name: 'Tag actions' }).click();
    await page
      .locator('[data-popover-content]')
      .getByRole('button', { name: 'Delete tag' })
      .click();
    const dialog = page.getByRole('dialog', { name: 'Delete tag' });
    await expect(dialog).toContainText(
      'Immich deletes the tag Travel / Norway and its 2 sub-tags.'
    );
    await expect(dialog).toContainText(
      'They come off 640 photos. The photos stay in your library with their other tags.'
    );
    const request = apiRequest(page, 'DELETE', '/api/tags/norway');
    await dialog.getByRole('button', { name: 'Delete tag', exact: true }).click();
    await request;
    await expect(page).toHaveURL('/tags');
    await expect(page.getByRole('status').filter({ hasText: 'Deleted Norway' })).toBeVisible();
    await expect(tagLink(page, 'Norway')).toHaveCount(0);
    await expect(tagLink(page, 'Germany')).toBeVisible();
  });

  test('deleting a parent from the sidebar leaves the page of its sub-tag', async ({ page }) => {
    await open(page, '/tags/lofoten');
    const sidebar = page.getByRole('complementary', { name: 'Library' });
    await sidebar.getByRole('button', { name: 'Show tags' }).click();
    await tagLink(sidebar, 'Norway').click({ button: 'right' });
    await page.getByRole('menu').getByRole('menuitem', { name: 'Delete tag' }).click();
    const request = apiRequest(page, 'DELETE', '/api/tags/norway');
    await page
      .getByRole('dialog', { name: 'Delete tag' })
      .getByRole('button', { name: 'Delete tag', exact: true })
      .click();
    await request;
    await expect(page).toHaveURL('/tags');
    await expect(tagLink(sidebar, 'Norway')).toHaveCount(0);
  });

  test('deleting a tag from the list stays on Tags', async ({ page }) => {
    await open(page, '/tags');
    await (await rowMenu(page, 'Film')).getByRole('button', { name: 'Delete tag' }).click();
    const dialog = page.getByRole('dialog', { name: 'Delete tag' });
    await expect(dialog).toContainText(
      'It comes off 220 photos. The photos stay in your library with their other tags.'
    );
    await dialog.getByRole('button', { name: 'Delete tag', exact: true }).click();
    await expect(tagLink(page, 'Film')).toHaveCount(0);
    await expect(page).toHaveURL('/tags');
  });

  test('a tag write asks for metadata consent first', async ({ page }) => {
    await open(page, '/tags', { metadataPushConsented: false });
    await page.locator('main header').getByRole('button', { name: 'Create tag' }).click();
    await expect(page.getByRole('dialog', { name: /Immich/ })).toBeVisible();
    await expect(page.getByRole('dialog', { name: 'Create tag' })).toHaveCount(0);
  });
});

test.describe('tag actions on touch', () => {
  test(
    'a long press opens the tag sheet with its caption',
    { tag: ['@touch', '@phone'] },
    async ({ page }) => {
      await openTouch(page, '/tags', { tags: tagTree(), metadataPushConsented: true });
      const sheet = page.getByRole('dialog', { name: 'Travel' });
      await longPressUntil(page, tagLink(page, 'Travel'), sheet);
      await expect(sheet).toContainText('1290 photos · 5 sub-tags');
      await sheet.getByRole('button', { name: 'Delete tag' }).tap();
      const confirm = page.getByRole('dialog', { name: 'Delete tag' });
      await expect(confirm).toContainText('and its 5 sub-tags.');
      await confirm.getByRole('button', { name: 'Cancel' }).tap();
      await expect(confirm).toBeHidden();
    }
  );

  test('Create tag works from the app bar on touch', { tag: ['@touch'] }, async ({ page }) => {
    await openTouch(page, '/tags', { tags: tagTree(), metadataPushConsented: true });
    await page.getByRole('button', { name: 'Create tag' }).tap();
    const dialog = page.getByRole('dialog', { name: 'Create tag' });
    await dialog.getByRole('textbox', { name: 'Name' }).fill('Archive');
    await expect(dialog).toContainText('Creates Archive');
    const request = apiRequest(page, 'POST', '/api/tags');
    await dialog.getByRole('button', { name: 'Create' }).tap();
    expect((await request).postDataJSON()).toEqual({ name: 'Archive' });
    await expect(tagLink(page, 'Archive')).toBeVisible();
  });

  test('the tag page menu leaves out Open tag', { tag: ['@touch'] }, async ({ page }) => {
    await openTouch(page, '/tags/norway', {
      tags: tagTree(),
      metadataPushConsented: true,
      tagRename: true
    });
    await page.getByRole('button', { name: 'Tag actions' }).tap();
    const sheet = page.getByRole('dialog', { name: 'Norway' });
    await expect(sheet.getByRole('button')).toContainText(['Rename', 'Change color']);
    await expect(sheet.getByRole('button', { name: 'Open tag' })).toHaveCount(0);
    await sheet.getByRole('button', { name: 'Change color' }).tap();
    const details = page.getByRole('dialog', { name: 'Tag details' });
    await details.getByRole('radio', { name: 'Green' }).tap();
    const request = apiRequest(page, 'PATCH', '/api/tags/norway');
    await details.getByRole('button', { name: 'Save' }).tap();
    expect((await request).postDataJSON()).toEqual({ color: '#46a758' });
  });
});
