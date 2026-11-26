import { expect, test, type Locator, type Page } from '@playwright/test';
import { apiRequest, installMocks, json, SESSION_USER } from './helpers';
import type { MockAlbum } from './searchMocks';
import { drawer, openDrawer } from './touchShell';

const OTHER = '00000000-0000-0000-0000-0000000000bb';

const ALBUMS: MockAlbum[] = [
  {
    id: 'album-norway',
    albumName: 'Norway 2026',
    assetIds: ['a', 'b'],
    updatedAt: '2026-11-01T00:00:00Z',
    endDate: '2025-06-01T00:00:00Z'
  },
  {
    id: 'album-family',
    albumName: 'Family',
    assetIds: ['a'],
    ownerId: OTHER,
    shared: true,
    updatedAt: '2026-10-01T00:00:00Z',
    endDate: '2026-07-14T00:00:00Z'
  },
  {
    id: 'album-portfolio',
    albumName: 'Portfolio picks',
    assetIds: ['a', 'b', 'c'],
    shared: true,
    updatedAt: '2026-09-01T00:00:00Z',
    endDate: '2026-01-10T00:00:00Z'
  },
  { id: 'album-empty', albumName: 'Archive', assetIds: [], updatedAt: '2026-08-01T00:00:00Z' }
];

async function openAlbums(page: Page, isAdmin = true, consented = true): Promise<void> {
  await installMocks(page, { albums: ALBUMS, metadataPushConsented: consented });
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ ...SESSION_USER, is_admin: isAdmin }))
  );
  await page.goto('/albums');
}

function cards(page: Page): Locator {
  return page.getByRole('list', { name: 'Albums' }).getByRole('link');
}

function chip(page: Page, name: string): Locator {
  return page
    .getByRole('group', { name: 'Show albums' })
    .getByRole('button', { name, exact: true });
}

async function expectAlbums(page: Page, names: string[]): Promise<void> {
  await expect(cards(page)).toHaveText(names.map((name) => new RegExp(`^\\s*${name}\\b`)));
}

async function narrowByOwner(page: Page, press: (target: Locator) => Promise<void>) {
  await expect(chip(page, 'All')).toHaveAttribute('aria-pressed', 'true');
  await expectAlbums(page, ['Norway 2026', 'Family', 'Portfolio picks', 'Archive']);

  const badge = (name: string): Locator =>
    cards(page).filter({ hasText: name }).getByRole('img', { name: 'Shared', exact: true });
  await expect(badge('Family')).toBeVisible();
  await expect(badge('Portfolio picks')).toBeVisible();
  await expect(badge('Norway 2026')).toHaveCount(0);
  await expect(cards(page).filter({ hasText: 'Family' })).toContainText(/1 photo\s*$/);
  await expect(page.getByRole('list', { name: 'Albums' })).not.toContainText(/shared/i);

  await press(chip(page, 'Mine'));
  await expect(chip(page, 'Mine')).toHaveAttribute('aria-pressed', 'true');
  await expectAlbums(page, ['Norway 2026', 'Portfolio picks', 'Archive']);
  await expect(page.getByText('3 albums', { exact: true })).toBeAttached();

  await press(chip(page, 'Shared with me'));
  await expectAlbums(page, ['Family']);
  await expect(page.getByText('1 album', { exact: true })).toBeAttached();

  await press(chip(page, 'All'));
  await expectAlbums(page, ['Norway 2026', 'Family', 'Portfolio picks', 'Archive']);
}

async function sortByNewestPhoto(page: Page, press: (target: Locator) => Promise<void>) {
  await press(page.getByRole('button', { name: /^Sort albums/ }));
  const sorts = page.getByRole('radiogroup', { name: 'Sort albums' });
  await expect(sorts.getByRole('radio')).toHaveText([
    'Recently updated',
    'Name',
    'Newest photo',
    'Photo count'
  ]);
  await press(sorts.getByRole('radio', { name: 'Newest photo' }));
  await expect(sorts.getByRole('radio', { name: 'Newest photo' })).toBeChecked();
  await expectAlbums(page, ['Family', 'Portfolio picks', 'Norway 2026', 'Archive']);
}

async function createAlbum(page: Page, press: (target: Locator) => Promise<void>) {
  await press(page.getByRole('button', { name: 'Create album', exact: true }));
  const dialog = page.getByRole('dialog', { name: 'Create album' });
  await expect(dialog).toBeVisible();
  const name = dialog.getByRole('textbox', { name: 'Name' });
  await expect(name).toBeFocused();
  const create = dialog.getByRole('button', { name: 'Create', exact: true });
  await expect(create).toBeDisabled();

  await name.fill('  Lofoten winter ');
  const request = apiRequest(page, 'POST', '/api/albums');
  await press(create);
  expect((await request).postDataJSON()).toEqual({ albumName: 'Lofoten winter' });
  await expect(dialog).toBeHidden();
  await expect(page).toHaveURL('/albums/album-5');
  await expect(page.getByRole('heading', { name: 'Lofoten winter', level: 1 })).toBeVisible();

  await page.goBack();
  await expect(page).toHaveURL('/albums');
  await expect(cards(page).filter({ hasText: 'Lofoten winter' })).toContainText('0 photos');
}

async function cancelWhileCreating(page: Page, press: (target: Locator) => Promise<void>) {
  let release = (): void => {};
  const held = new Promise<void>((resolve) => (release = resolve));
  await page.route('**/api/albums', async (route) => {
    if (route.request().method() === 'POST') await held;
    await route.fallback();
  });
  await press(page.getByRole('button', { name: 'Create album', exact: true }));
  const dialog = page.getByRole('dialog', { name: 'Create album' });
  await dialog.getByRole('textbox', { name: 'Name' }).fill('Lofoten winter');
  const created = page.waitForResponse(
    (res) => res.request().method() === 'POST' && new URL(res.url()).pathname === '/api/albums'
  );
  await press(dialog.getByRole('button', { name: 'Create', exact: true }));
  await press(dialog.getByRole('button', { name: 'Cancel', exact: true }));
  await expect(dialog).toBeHidden();
  release();
  await created;
  await expect(cards(page).filter({ hasText: 'Lofoten winter' })).toBeVisible();
  await expect(page).toHaveURL('/albums');

  await press(page.getByRole('button', { name: 'Create album', exact: true }));
  await expect(dialog.getByRole('textbox', { name: 'Name' })).toHaveValue('');
  await dialog.getByRole('textbox', { name: 'Name' }).fill('Senja');
  await expect(dialog.getByRole('button', { name: 'Create', exact: true })).toBeEnabled();
  await expect(page).toHaveURL('/albums');
}

function placeholderFits(input: Locator): Promise<boolean> {
  return input.evaluate((field: HTMLInputElement) => {
    const probe = document.createElement('span');
    probe.style.font = getComputedStyle(field).font;
    probe.style.position = 'absolute';
    probe.textContent = field.placeholder;
    document.body.append(probe);
    const width = probe.getBoundingClientRect().width;
    probe.remove();
    return field.clientWidth >= width;
  });
}

async function openManyAlbums(page: Page): Promise<void> {
  const many: MockAlbum[] = Array.from({ length: 13 }, (_, i) => ({
    id: `album-${i}`,
    albumName: `Album ${i}`,
    assetIds: ['a']
  }));
  await installMocks(page, { albums: many, metadataPushConsented: true });
  await page.goto('/albums');
  await expect(page.getByRole('textbox', { name: 'Filter albums' })).toBeVisible();
}

test.describe('albums page on the desktop', () => {
  const click = (target: Locator) => target.click();

  test('chips narrow albums by owner and shared albums carry a badge', async ({ page }) => {
    await openAlbums(page);
    await narrowByOwner(page, click);
  });

  test('sorts albums by their newest photo', async ({ page }) => {
    await openAlbums(page);
    await sortByNewestPhoto(page, click);
  });

  test('the plus button creates an album and opens it', async ({ page }) => {
    await openAlbums(page);
    await createAlbum(page, click);
    const sidebar = page.getByRole('complementary', { name: 'Library' });
    await sidebar.getByRole('button', { name: 'Show albums' }).click();
    await expect(sidebar.getByRole('link', { name: /^Lofoten winter/ })).toHaveAttribute(
      'href',
      '/albums/album-5'
    );
  });

  test('the title, count, chips and filter share one centred header row', async ({ page }) => {
    await openManyAlbums(page);
    const header = page.locator('main header');
    const parts = [
      header.getByRole('heading', { name: 'Albums', level: 1 }),
      header.getByText('13', { exact: true }),
      header.getByRole('group', { name: 'Show albums' }),
      header.getByRole('textbox', { name: 'Filter albums' }),
      header.getByRole('button', { name: 'Create album', exact: true })
    ];
    const row = await header.boundingBox();
    const middle = (row?.y ?? 0) + (row?.height ?? 0) / 2;
    for (const part of parts) {
      const box = await part.boundingBox();
      expect(Math.abs((box?.y ?? 0) + (box?.height ?? 0) / 2 - middle)).toBeLessThanOrEqual(1);
    }
    await expect(header.getByText('13 albums', { exact: true })).toBeAttached();
    const chips = await parts[2]!.boundingBox();
    const field = await parts[3]!.locator('..').boundingBox();
    expect(field?.height).toBe(chips?.height);
  });

  test('closing the dialog while the album is created stays on the albums page', async ({
    page
  }) => {
    await openAlbums(page);
    await cancelWhileCreating(page, click);
  });

  test('creating an album asks to sync metadata to Immich first', async ({ page }) => {
    await openAlbums(page, true, false);
    await page.getByRole('button', { name: 'Create album', exact: true }).click();
    const consent = page.getByRole('dialog', { name: 'Sync metadata to Immich' });
    await expect(consent).toBeVisible();
    await expect(page.getByRole('dialog', { name: 'Create album' })).toHaveCount(0);
    await consent.getByRole('button', { name: 'Sync to Immich', exact: true }).click();
    await expect(page.getByRole('dialog', { name: 'Create album' })).toBeVisible();
    await expect(
      page.getByRole('dialog', { name: 'Create album' }).getByRole('textbox', { name: 'Name' })
    ).toBeFocused();
  });

  test('account settings name the Immich server for a non-admin', async ({ page }) => {
    await installMocks(page);
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(json({ ...SESSION_USER, is_admin: false }))
    );
    await page.goto('/settings/account');
    await expect(
      page.getByText('photos.example.net · Immich v3.1.0', { exact: true })
    ).toBeVisible();
  });
});

test.describe('albums page on touch', { tag: '@touch' }, () => {
  const tap = (target: Locator) => target.tap();

  test('chips narrow albums by owner and shared albums carry a badge', async ({ page }) => {
    await openAlbums(page);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
    await narrowByOwner(page, tap);
  });

  test('the sort sheet sorts albums by their newest photo', async ({ page }) => {
    await openAlbums(page);
    await sortByNewestPhoto(page, tap);
    await page.getByRole('button', { name: 'Done', exact: true }).tap();
    await expect(page.getByRole('button', { name: 'Sort albums: Newest photo' })).toBeVisible();
  });

  test('the plus button creates an album in a sheet and opens it', async ({ page }) => {
    await openAlbums(page);
    await createAlbum(page, tap);
  });

  test('closing the sheet while the album is created stays on the albums page', async ({
    page
  }) => {
    await openAlbums(page);
    await cancelWhileCreating(page, tap);
  });

  test('the filter field matches the chips and fits its placeholder', async ({ page }) => {
    await openManyAlbums(page);
    const pill = await chip(page, 'All').boundingBox();
    const input = page.getByRole('textbox', { name: 'Filter albums' });
    const field = await input.locator('..').boundingBox();
    expect(field?.height).toBe(pill?.height);
    expect((await input.boundingBox())?.height).toBeGreaterThanOrEqual(44);
    expect(await placeholderFits(input)).toBe(true);
  });

  test('the account drawer names the Immich server for a non-admin', async ({ page }) => {
    await openAlbums(page, false);
    const panel = await openDrawer(page);
    await expect(
      panel.getByText('photos.example.net · Immich v3.1.0', { exact: true })
    ).toBeVisible();
    await expect(drawer(page).getByText('admin', { exact: true })).toHaveCount(0);
  });
});

test.describe('albums page in phone portrait', { tag: ['@touch', '@phone'] }, () => {
  test('the filter takes its own row, flush with the grid edges', async ({ page }) => {
    await openManyAlbums(page);
    const input = page.getByRole('textbox', { name: 'Filter albums' });
    const field = await input.locator('..').boundingBox();
    const edges = await cards(page).evaluateAll((links) => {
      const boxes = links.map((link) => link.getBoundingClientRect());
      return [
        Math.min(...boxes.map((box) => box.left)),
        Math.max(...boxes.map((box) => box.right))
      ];
    });
    expect(Math.abs((field?.x ?? 0) - edges[0]!)).toBeLessThanOrEqual(1);
    expect(Math.abs((field?.x ?? 0) + (field?.width ?? 0) - edges[1]!)).toBeLessThanOrEqual(1);
    const pill = await chip(page, 'All').boundingBox();
    expect(field?.y ?? 0).toBeGreaterThan((pill?.y ?? 0) + (pill?.height ?? 0));
  });
});

test.describe('albums page in phone landscape', { tag: ['@touch', '@phone'] }, () => {
  test.use({ viewport: { width: 915, height: 412 } });

  test('the filter shares the chip row and fits its placeholder', async ({ page }) => {
    await openManyAlbums(page);
    const pill = await chip(page, 'All').boundingBox();
    const input = page.getByRole('textbox', { name: 'Filter albums' });
    const field = await input.locator('..').boundingBox();
    expect((field?.y ?? 0) + (field?.height ?? 0) / 2).toBeCloseTo(
      (pill?.y ?? 0) + (pill?.height ?? 0) / 2,
      0
    );
    expect(await placeholderFits(input)).toBe(true);
  });

  test('keeps the chips, sort and plus button', async ({ page }) => {
    await openAlbums(page);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');
    await page.getByRole('button', { name: 'Create album', exact: true }).tap();
    await expect(page.getByRole('dialog', { name: 'Create album' })).toBeVisible();
    await page.getByRole('button', { name: 'Cancel', exact: true }).tap();
    await narrowByOwner(page, (target) => target.tap());
  });
});
