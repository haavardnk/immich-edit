import { expect, test, type Locator, type Page } from '@playwright/test';
import { tile } from './browse';
import { ASSET_SUMMARY, installMocks, type MockAssetSummary } from './helpers';
import { open } from './touchShell';

const DEFAULT_ICONS = ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy'];

const PHOTO: MockAssetSummary = {
  ...ASSET_SUMMARY,
  exifInfo: {
    city: 'Bergen',
    state: 'Vestland',
    country: 'Norway',
    latitude: 60.39,
    longitude: 5.32
  },
  tags: [
    { id: 'tag-fjords', name: 'Fjords', value: 'Travel/Fjords' },
    { id: 'tag-red', name: 'red', value: 'immich-edit/label/red' }
  ]
};

const ALBUMS = [{ id: 'album-norway', albumName: 'Norway', assetIds: [ASSET_SUMMARY.id] }];

function popover(page: Page): Locator {
  return page.locator('[data-popover-content]');
}

async function openMenu(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: /^Thumbnail info: / }).click();
  await expect(popover(page)).toBeVisible();
  return popover(page);
}

function memberships(page: Page): string[] {
  const seen: string[] = [];
  page.on('request', (request) => {
    const path = new URL(request.url()).pathname;
    if (path.endsWith('/membership') && request.method() === 'POST') seen.push(path);
  });
  return seen;
}

test.use({ timezoneId: 'UTC' });

test('chosen icons appear on the tiles, load membership only when shown and save to the account', async ({
  page
}) => {
  const saved: unknown[] = [];
  await installMocks(page, {
    assets: [PHOTO],
    albums: ALBUMS,
    onPreferences: (body) => saved.push(body.tile_overlay)
  });
  const seen = memberships(page);
  await page.goto('/photos');
  const photo = tile(page, PHOTO.originalFileName);
  await expect(photo.getByRole('img', { name: 'Red label' })).toBeVisible();

  const menu = await openMenu(page);
  const icons = menu.getByRole('group', { name: 'Icons' });
  await expect(menu.getByText('6 of 11')).toBeVisible();
  expect(seen).toEqual([]);

  const albums = page.waitForRequest((r) => r.url().endsWith('/api/albums/membership'));
  await icons.getByRole('checkbox', { name: 'Albums' }).click();
  expect((await albums).postDataJSON()).toEqual({ ids: [ASSET_SUMMARY.id] });
  await expect(photo.getByRole('img', { name: '1 album' })).toBeVisible();
  await expect
    .poll(() => saved.at(-1))
    .toEqual({
      icons: [...DEFAULT_ICONS, 'albums'],
      details: ['filename', 'taken']
    });

  const tags = page.waitForRequest((r) => r.url().endsWith('/api/tags/membership'));
  await icons.getByRole('checkbox', { name: 'Tags' }).click();
  await tags;
  await expect(photo.getByRole('img', { name: '1 tag' })).toBeVisible();

  await icons.getByRole('checkbox', { name: 'File format' }).click();
  await icons.getByRole('checkbox', { name: 'Location' }).click();
  await expect(photo.getByRole('img', { name: 'ARW file' })).toBeVisible();
  await expect(photo.getByRole('img', { name: 'Location' })).toBeVisible();
  await expect(menu.getByText('10 of 11')).toBeVisible();
  expect(seen.toSorted()).toEqual(['/api/albums/membership', '/api/tags/membership']);
});

test('details stop at four and full mode says how many fit', async ({ page }) => {
  await installMocks(page, { assets: [PHOTO] });
  await page.goto('/photos');
  const menu = await openMenu(page);

  await menu.getByRole('checkbox', { name: 'Place' }).click();
  await menu.getByRole('checkbox', { name: 'Camera' }).click();
  await expect(menu.getByText('4 of 4')).toBeVisible();
  await expect(menu.getByRole('checkbox', { name: 'Lens' })).toBeDisabled();
  await expect(menu.getByText('Up to 4. Turn one off to choose another.')).toBeVisible();

  await menu.getByRole('radio', { name: 'Full' }).click();
  await expect(menu.getByRole('status')).toHaveText(
    'Showing 2 of 4 at size M. Larger thumbnails show more.'
  );
  await expect(
    tile(page, PHOTO.originalFileName).getByTestId('tile-info').locator('.truncate')
  ).toHaveText([PHOTO.originalFileName, new Date('2024-01-01T00:00:00Z').toLocaleDateString()]);

  await menu.getByRole('checkbox', { name: 'Taken' }).click();
  await expect(menu.getByRole('checkbox', { name: 'Lens' })).toBeEnabled();
  await expect(tile(page, PHOTO.originalFileName).getByText('Bergen, Vestland')).toBeVisible();
});

test('an account overlay loads on start and Reset restores the default', async ({ page }) => {
  const saved: unknown[] = [];
  await installMocks(page, {
    assets: [PHOTO],
    tileOverlay: { icons: ['format'], details: ['lens'] },
    onPreferences: (body) => saved.push(body.tile_overlay)
  });
  await page.goto('/photos');
  const photo = tile(page, PHOTO.originalFileName);
  await expect(photo.getByRole('img', { name: 'ARW file' })).toBeVisible();
  await expect(photo.getByRole('img', { name: 'Red label' })).toHaveCount(0);

  const menu = await openMenu(page);
  await menu.getByRole('button', { name: 'Reset thumbnail info' }).click();
  await expect(photo.getByRole('img', { name: 'Red label' })).toBeVisible();
  await expect(photo.getByRole('img', { name: 'ARW file' })).toHaveCount(0);
  await expect(menu.getByRole('button', { name: 'Reset thumbnail info' })).toBeDisabled();
  await expect.poll(() => saved).toEqual([null]);
});

test.describe('thumbnail info on touch', { tag: '@touch' }, () => {
  test('the View panel changes the tile icons inline and resets them', async ({ page }) => {
    await open(page, '/photos', { assets: [PHOTO] });
    await page.locator('main').getByRole('button', { name: 'View', exact: true }).tap();
    const surface = page
      .getByRole('dialog', { name: 'View', exact: true })
      .or(page.getByRole('complementary', { name: 'View', exact: true }));

    const info = surface.getByRole('region', { name: 'Thumbnail info' });
    await expect(info.getByRole('heading', { name: 'Thumbnail info' })).toBeVisible();
    const reset = info.getByRole('button', { name: 'Reset thumbnail info' });
    await expect(reset).toBeDisabled();
    await expect(info.getByText('cycles the modes')).toHaveCount(0);
    await info.getByRole('checkbox', { name: 'File format' }).tap();
    await expect(
      tile(page, PHOTO.originalFileName).getByRole('img', { name: 'ARW file' })
    ).toHaveCount(1);

    await reset.tap();
    await expect(
      tile(page, PHOTO.originalFileName).getByRole('img', { name: 'ARW file' })
    ).toHaveCount(0);
    await expect(reset).toBeDisabled();
  });
});
