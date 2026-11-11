import { expect, test, type Locator, type Page } from '@playwright/test';
import {
  ASSET_ID,
  NEUTRAL_RECORD,
  gotoAsset,
  installMocks,
  json,
  type CopyRecord
} from './helpers';
import { exposureManifest } from './snapshotMocks';
import { longPressUntil } from './touch';
import { mockLibrary } from './touchShell';

const COPY_1 = `${ASSET_ID}_1`;
const COPY_2 = `${ASSET_ID}_2`;

const HISTORIES: Record<string, string[]> = {
  [ASSET_ID]: ['Portra soft', 'Contrast', 'Auto'],
  [COPY_1]: [],
  [COPY_2]: ['Mono', 'Auto']
};

async function mockVersions(page: Page): Promise<CopyRecord[]> {
  const copies: CopyRecord[] = [
    { id: COPY_1, source_asset_id: ASSET_ID, name: null, created_at: '2024-01-01T00:00:00Z' },
    { id: COPY_2, source_asset_id: ASSET_ID, name: 'B&W', created_at: '2024-01-02T00:00:00Z' }
  ];
  await page.route(/\/api\/assets\/[^/]+\/copies$/, (route) =>
    route.request().method() === 'GET' ? route.fulfill(json(copies)) : route.fallback()
  );
  await page.route(/\/api\/copies\/[^/]+$/, (route) => {
    const id = new URL(route.request().url()).pathname.split('/').at(-1);
    const index = copies.findIndex((copy) => copy.id === id);
    if (route.request().method() !== 'PATCH' || index < 0) return route.fallback();
    const body = route.request().postDataJSON() as { name: string | null };
    copies[index] = { ...copies[index], name: body.name };
    return route.fulfill(json(copies[index]));
  });
  await page.route(/\/api\/assets\/[^/]+\/edits\/history$/, (route) => {
    const id = new URL(route.request().url()).pathname.split('/')[3] ?? '';
    const actions = HISTORIES[id] ?? [];
    return route.fulfill(
      json(
        actions.map((action, index) => ({
          id: actions.length - index,
          manifest_hash: `${id}-${index}`,
          deleted: false,
          manifest: null,
          created_at: '2024-01-03T00:00:00Z',
          action,
          undone: false
        }))
      )
    );
  });
  return copies;
}

function pill(page: Page): Locator {
  return page.locator('[data-photo-pill]');
}

function held(page: Page): Locator {
  return page.getByTestId('version-hold');
}

function link(scope: Locator, name: string): Locator {
  return scope.getByRole('link', { name: new RegExp(`^${name}`) });
}

function thumb(scope: Locator, name: string): Locator {
  return link(scope, name).locator('[data-version-thumb]');
}

test.describe('versions on desktop', () => {
  test('the More menu shows a thumbnail per version and holding one shows it', async ({ page }) => {
    await installMocks(page, {
      edits: [{ id: COPY_2, hash: 'h2', updated_at: '2024-01-02T00:00:00Z' }]
    });
    await mockVersions(page);
    await gotoAsset(page);

    await page.getByRole('button', { name: 'More editor actions' }).click();
    const versions = page.getByRole('navigation', { name: 'Versions' });
    await expect(versions.getByRole('link')).toHaveCount(3);
    await expect(link(versions, 'Original')).toHaveAttribute('aria-current', 'page');
    await expect(thumb(versions, 'Original').locator('img')).toHaveAttribute(
      'src',
      new RegExp(`/api/assets/${ASSET_ID}/thumb\\?size=thumbnail`)
    );
    await expect(thumb(versions, 'Copy 1').locator('img')).toHaveAttribute(
      'src',
      new RegExp(`/api/assets/${ASSET_ID}/thumb\\?size=thumbnail`)
    );
    await expect(thumb(versions, 'B&W').locator('img')).toHaveAttribute(
      'src',
      new RegExp(`/api/assets/${COPY_2}/edited-thumb\\?h=h2&`)
    );
    for (const img of await versions.locator('[data-version-thumb] img').all()) {
      await expect.poll(() => img.evaluate((el: HTMLImageElement) => el.naturalWidth)).toBe(1);
    }

    const box = await thumb(versions, 'B&W').boundingBox();
    if (!box) throw new Error('no thumbnail');
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await expect(pill(page)).toHaveText('Holding B&W · release to return to Original');
    await expect(held(page)).toHaveAttribute(
      'src',
      new RegExp(`/api/assets/${COPY_2}/preview\\?max=1600&clip=false$`)
    );
    await expect(held(page)).not.toHaveClass(/opacity-0/);
    await expect(page.getByTestId('preview-image')).toBeAttached();

    await page.mouse.up();
    await expect(held(page)).toHaveCount(0);
    await expect(pill(page)).toHaveCount(0);
    await expect(versions).toBeVisible();
    await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}$`));

    const own = await thumb(versions, 'Original').boundingBox();
    if (!own) throw new Error('no thumbnail');
    await page.mouse.move(own.x + own.width / 2, own.y + own.height / 2);
    await page.mouse.down();
    await page.waitForTimeout(500);
    await expect(pill(page)).toHaveCount(0);
    await expect(held(page)).toHaveCount(0);
    await page.mouse.up();
  });

  test('holding Space on a focused version shows it until release', async ({ page }) => {
    await installMocks(page);
    await mockVersions(page);
    await gotoAsset(page);

    await page.getByRole('button', { name: 'More editor actions' }).click();
    const versions = page.getByRole('navigation', { name: 'Versions' });
    await link(versions, 'B&W').focus();
    await page.keyboard.down(' ');
    await expect(pill(page)).toHaveText('Holding B&W · release to return to Original');
    await expect(held(page)).toBeVisible();

    await page.keyboard.up(' ');
    await expect(held(page)).toHaveCount(0);
    await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}$`));

    await page.keyboard.down(' ');
    await expect(held(page)).toBeVisible();
    await link(versions, 'Copy 1').focus();
    await expect(held(page)).toHaveCount(0);
    await page.keyboard.up(' ');
    await expect(versions).toBeVisible();
  });
});

test.describe('versions on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('the versions sheet lists, renames, switches and holds versions', async ({ page }) => {
    await mockLibrary(page);
    const copies = await mockVersions(page);
    await page.goto(`/assets/${COPY_2}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    await page.getByRole('button', { name: 'More editor actions' }).tap();
    await expect(page.getByRole('navigation', { name: 'Versions' })).toHaveCount(0);
    const entry = page.getByRole('button', { name: 'Versions and copies', exact: true });
    await expect(entry).toContainText('2 copies');
    await entry.tap();

    const sheet = page.getByRole('dialog', { name: 'Versions and copies' });
    await expect(sheet).toContainText('IMG_0001.ARW · 3 versions');
    const current = sheet.locator('a[aria-current="page"]');
    await expect(current).toHaveCount(1);
    await expect(current).toContainText('B&W');
    await expect(current).toContainText('Current');
    await expect(link(sheet, 'Original')).toContainText('Portra soft · 3 steps');
    await expect(link(sheet, 'Copy 1')).toContainText('No edits');
    await expect(link(sheet, 'B&W')).toContainText('Copy 2 · Mono · 2 steps');
    await expect(thumb(sheet, 'Original').locator('svg')).toHaveCount(0);
    await expect(thumb(sheet, 'B&W').locator('svg')).toHaveCount(1);
    await expect(sheet.getByRole('button', { name: 'Create virtual copy' })).toContainText(
      'from Current'
    );

    await sheet.getByRole('button', { name: 'Actions for Copy 1' }).tap();
    await expect(page.getByRole('button', { name: 'Open in loupe', exact: true })).toBeVisible();
    await page.getByRole('button', { name: 'Rename', exact: true }).tap();
    const field = sheet.getByRole('textbox', { name: 'Version name' });
    await expect(field).toBeVisible();
    await field.fill('Square crop');
    await field.press('Enter');
    await expect(link(sheet, 'Square crop')).toContainText('Copy 1 · No edits');
    expect(copies[0]?.name).toBe('Square crop');

    await link(sheet, 'Original').tap();
    await page.waitForURL(new RegExp(`/assets/${ASSET_ID}$`));
    await expect(sheet).toBeVisible();
    await expect(sheet.locator('a[aria-current="page"]')).toContainText('Original');

    await longPressUntil(page, thumb(sheet, 'B&W'), pill(page));
    await expect(pill(page)).toHaveCount(0);
    await expect(held(page)).toHaveCount(0);
    await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}$`));
  });

  test('More copies edits, then pastes them and resets develop', async ({ page }) => {
    await mockLibrary(page, {
      editRecord: { ...NEUTRAL_RECORD, manifest: exposureManifest(0.5), hash: 'hash-exposure' }
    });
    await mockVersions(page);
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    const more = page.getByRole('button', { name: 'More editor actions' });
    const paste = page.getByRole('button', { name: 'Paste edits', exact: true });
    await more.tap();
    await expect(paste).toBeDisabled();
    await expect(paste).not.toContainText('from');
    await expect(page.getByRole('button', { name: 'Reset Develop', exact: true })).toBeEnabled();
    await page.getByRole('button', { name: 'Copy edits…', exact: true }).tap();

    const dialog = page.getByRole('dialog', { name: 'Copy settings' });
    await expect(dialog).toContainText('Then open another photo and use More › Paste edits');
    await expect(dialog).toContainText('Each paste is one history step.');
    await dialog.getByRole('button', { name: 'Copy', exact: true }).tap();
    await expect(dialog).toBeHidden();

    await more.tap();
    await expect(paste).toBeEnabled();
    await expect(paste).toContainText('from IMG_');
    await page.getByRole('button', { name: 'Reset Develop', exact: true }).tap();
    await expect(
      page.getByRole('status').filter({ hasText: 'Reset develop settings' })
    ).toBeVisible();
  });
});
