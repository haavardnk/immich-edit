import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks } from './helpers';
import type { MockAlbum } from './searchMocks';
import { longPressUntil } from './touch';

const OTHER = '00000000-0000-0000-0000-0000000000bb';

const LONG_DESCRIPTION = [
  'Winter light on the coast.',
  'Shot over three weeks.',
  'Mostly Lofoten and Senja.',
  'A few frames from Tromsø.'
].join('\n');

const ALBUMS: MockAlbum[] = [
  {
    id: 'album-norway',
    albumName: 'Norway 2026',
    assetIds: ['a', 'b'],
    updatedAt: '2026-11-01T00:00:00Z',
    startDate: '2026-10-12T00:00:00Z',
    endDate: '2026-11-03T00:00:00Z'
  },
  {
    id: 'album-family',
    albumName: 'Family',
    assetIds: ['a'],
    ownerId: OTHER,
    shared: true,
    updatedAt: '2026-10-01T00:00:00Z'
  },
  { id: 'album-empty', albumName: 'Archive', assetIds: [], updatedAt: '2026-08-01T00:00:00Z' }
];

async function open(page: Page, path: string): Promise<void> {
  await installMocks(page, { albums: ALBUMS, metadataPushConsented: true });
  await page.goto(path);
}

function card(page: Page, name: string): Locator {
  return page.getByRole('list', { name: 'Albums' }).getByRole('listitem').filter({ hasText: name });
}

function albumRequest(page: Page, method: string, path: string) {
  return page.waitForRequest(
    (req) => req.method() === method && new URL(req.url()).pathname === path
  );
}

function popover(page: Page): Locator {
  return page.locator('[data-popover-content]');
}

function menuItems(page: Page): Locator {
  return page.getByRole('menu').getByRole('menuitem');
}

function sheet(page: Page, name: string): Locator {
  return page.getByRole('dialog', { name });
}

function sidebar(page: Page): Locator {
  return page.getByRole('complementary', { name: 'Library' });
}

async function showSidebarAlbums(page: Page): Promise<void> {
  await sidebar(page).getByRole('button', { name: 'Show albums' }).click();
}

async function confirmDelete(page: Page, press: (target: Locator) => Promise<void>) {
  const dialog = page.getByRole('dialog', { name: 'Delete album' });
  await expect(dialog).toContainText('Immich deletes the album Archive. It has no photos.');
  await expect(dialog).toContainText('This cannot be undone.');
  const request = albumRequest(page, 'DELETE', '/api/albums/album-empty');
  await press(dialog.getByRole('button', { name: 'Delete album', exact: true }));
  await request;
  await expect(dialog).toBeHidden();
  await expect(page.getByRole('status').filter({ hasText: 'Deleted Archive' })).toBeVisible();
  await expect(card(page, 'Archive')).toHaveCount(0);
}

test.describe('album actions on the desktop', () => {
  test('the card more button lists actions by role', async ({ page }) => {
    await open(page, '/albums');
    const norway = card(page, 'Norway 2026');
    await norway.hover();
    await norway.getByRole('button', { name: 'Album actions' }).click();
    const menu = popover(page);
    await expect(menu.getByRole('button')).toHaveText([
      'Open album',
      'Rename',
      'Edit description',
      'Delete album'
    ]);
    await page.keyboard.press('Escape');
    await expect(menu).toBeHidden();

    const family = card(page, 'Family');
    await family.hover();
    await family.getByRole('button', { name: 'Album actions' }).click();
    await expect(popover(page).getByRole('button')).toHaveText(['Open album', 'Leave album']);
  });

  test('right-clicking a card or a sidebar album opens its menu', async ({ page }) => {
    await open(page, '/albums');
    await card(page, 'Norway 2026').getByRole('link').click({ button: 'right' });
    await expect(menuItems(page)).toHaveText([
      'Open album',
      'Rename',
      'Edit description',
      'Delete album'
    ]);
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);

    await showSidebarAlbums(page);
    await sidebar(page)
      .getByRole('link', { name: /^Family/ })
      .click({ button: 'right' });
    await expect(menuItems(page)).toHaveText(['Open album', 'Leave album']);
    await menuItems(page).filter({ hasText: 'Open album' }).click();
    await expect(page).toHaveURL('/albums/album-family');
  });

  test('the header menu renames the album', async ({ page }) => {
    await open(page, '/albums/album-norway');
    await expect(page.getByRole('heading', { name: 'Norway 2026', level: 1 })).toBeVisible();
    await page.locator('main header').getByRole('button', { name: 'Album actions' }).click();
    const menu = popover(page);
    await expect(menu.getByRole('button')).toHaveText([
      'Rename',
      'Edit description',
      'Delete album'
    ]);
    await menu.getByRole('button', { name: 'Rename' }).click();

    const dialog = page.getByRole('dialog', { name: 'Album details' });
    const name = dialog.getByRole('textbox', { name: 'Name' });
    await expect(name).toBeFocused();
    await expect(name).toHaveValue('Norway 2026');
    const save = dialog.getByRole('button', { name: 'Save', exact: true });
    await expect(save).toBeDisabled();
    await name.fill(' Norway 2027 ');
    const request = albumRequest(page, 'PATCH', '/api/albums/album-norway');
    await save.click();
    expect((await request).postDataJSON()).toEqual({ albumName: 'Norway 2027' });
    await expect(dialog).toBeHidden();
    await expect(page.getByRole('heading', { name: 'Norway 2027', level: 1 })).toBeVisible();

    await showSidebarAlbums(page);
    await expect(sidebar(page).getByRole('link', { name: /^Norway 2027/ })).toBeVisible();
  });

  test('the header menu edits the description shown under the title', async ({ page }) => {
    await open(page, '/albums/album-norway');
    await expect(page.getByText('12 Oct – 3 Nov 2026')).toBeVisible();
    await page.locator('main header').getByRole('button', { name: 'Album actions' }).click();
    await popover(page).getByRole('button', { name: 'Edit description' }).click();

    const dialog = page.getByRole('dialog', { name: 'Album details' });
    const description = dialog.getByRole('textbox', { name: 'Description' });
    await expect(description).toBeFocused();
    await expect(dialog).toContainText('0 / 2000');
    await description.fill(LONG_DESCRIPTION);
    await expect(dialog).toContainText(`${LONG_DESCRIPTION.length} / 2000`);
    const request = albumRequest(page, 'PATCH', '/api/albums/album-norway');
    await dialog.getByRole('button', { name: 'Save', exact: true }).click();
    expect((await request).postDataJSON()).toEqual({ description: LONG_DESCRIPTION });
    await expect(dialog).toBeHidden();

    const shown = page.locator('#album-description');
    await expect(shown).toContainText('Winter light on the coast.');
    const more = page.getByRole('button', { name: 'More', exact: true });
    await expect(more).toHaveAttribute('aria-expanded', 'false');
    const clamped = (await shown.boundingBox())?.height ?? 0;
    await more.click();
    const less = page.getByRole('button', { name: 'Less', exact: true });
    await expect(less).toHaveAttribute('aria-expanded', 'true');
    expect((await shown.boundingBox())?.height ?? 0).toBeGreaterThan(clamped);
  });

  test('deleting an album from its card asks first', async ({ page }) => {
    await open(page, '/albums');
    const archive = card(page, 'Archive');
    await archive.hover();
    await archive.getByRole('button', { name: 'Album actions' }).click();
    await popover(page).getByRole('button', { name: 'Delete album' }).click();
    await confirmDelete(page, (target) => target.click());
  });

  test('deleting an album from its page returns to the albums', async ({ page }) => {
    await open(page, '/albums/album-empty');
    await page.locator('main header').getByRole('button', { name: 'Album actions' }).click();
    await popover(page).getByRole('button', { name: 'Delete album' }).click();
    const dialog = page.getByRole('dialog', { name: 'Delete album' });
    const request = albumRequest(page, 'DELETE', '/api/albums/album-empty');
    await dialog.getByRole('button', { name: 'Delete album', exact: true }).click();
    await request;
    await expect(page).toHaveURL('/albums');
    await expect(card(page, 'Norway 2026')).toBeVisible();
    await expect(card(page, 'Archive')).toHaveCount(0);
  });

  test('leaving a shared album removes it from the list', async ({ page }) => {
    await open(page, '/albums/album-family');
    await expect(page.getByText('Shared by Anne Berg · you can view')).toBeVisible();
    await page.locator('main header').getByRole('button', { name: 'Album actions' }).click();
    const menu = popover(page);
    await expect(menu.getByRole('button')).toHaveText(['Leave album']);
    await menu.getByRole('button', { name: 'Leave album' }).click();

    const dialog = page.getByRole('dialog', { name: 'Leave album' });
    await expect(dialog).toContainText('You leave Family, shared by Anne Berg.');
    await expect(dialog).toContainText('Only Anne can invite you back.');
    const request = albumRequest(page, 'DELETE', '/api/albums/album-family/membership');
    await dialog.getByRole('button', { name: 'Leave album', exact: true }).click();
    await request;
    await expect(page).toHaveURL('/albums');
    await expect(page.getByRole('status').filter({ hasText: 'Left Family' })).toBeVisible();
    await expect(card(page, 'Family')).toHaveCount(0);
  });
});

test.describe('album actions on touch', { tag: '@touch' }, () => {
  test('long-pressing a card opens a titled sheet that deletes', async ({ page }) => {
    await open(page, '/albums');
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
    await expect(card(page, 'Archive').getByRole('button', { name: 'Album actions' })).toHaveCount(
      0
    );
    const actions = sheet(page, 'Archive');
    await longPressUntil(page, card(page, 'Archive').getByRole('link'), actions);
    await expect(actions).toContainText('0 photos · only you');
    await expect(actions.getByRole('button', { name: /album|Rename|description/ })).toHaveText([
      'Open album',
      'Rename',
      'Edit description',
      'Delete album'
    ]);
    await actions.getByRole('button', { name: 'Delete album' }).tap();
    await confirmDelete(page, (target) => target.tap());
  });
});

test.describe('album actions on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the app bar menu renames the album', async ({ page }) => {
    await open(page, '/albums/album-norway');
    await page.locator('header[data-app-bar]').getByRole('button', { name: 'Album actions' }).tap();
    await sheet(page, 'Norway 2026').getByRole('button', { name: 'Rename' }).tap();
    const dialog = page.getByRole('dialog', { name: 'Album details' });
    const name = dialog.getByRole('textbox', { name: 'Name' });
    await expect(name).toBeFocused();
    await name.fill('Norway 2027');
    const request = albumRequest(page, 'PATCH', '/api/albums/album-norway');
    await dialog.getByRole('button', { name: 'Save', exact: true }).tap();
    expect((await request).postDataJSON()).toEqual({ albumName: 'Norway 2027' });
    await expect(page.getByRole('heading', { name: 'Norway 2027', level: 1 })).toBeVisible();
  });
});

test.describe('album actions on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test('a viewer only sees open and leave', async ({ page }) => {
    await open(page, '/albums');
    const actions = sheet(page, 'Family');
    await longPressUntil(page, card(page, 'Family').getByRole('link'), actions);
    await expect(actions).toContainText('1 photo · shared by Anne Berg');
    await expect(actions.getByRole('button', { name: /album/ })).toHaveText([
      'Open album',
      'Leave album'
    ]);
  });
});
