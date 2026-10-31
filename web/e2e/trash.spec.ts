import { expect, test, type Page } from '@playwright/test';
import { installMocks, numberedAssets, openFilters, type InstallOpts } from './helpers';

const ASSETS = numberedAssets(3);

async function openGrid(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, { assets: ASSETS, ...opts });
  await page.goto('/photos');
  await expect(tile(page, 'IMG_0003.ARW')).toBeVisible();
}

function tile(page: Page, name: string) {
  return page.getByRole('link', { name: new RegExp(name) });
}

async function selectFirstTwo(page: Page): Promise<void> {
  const select = page.getByRole('button', { name: 'Select', exact: true });
  await select.nth(0).click();
  await select.nth(0).click();
  await expect(page.getByText('2 selected')).toBeVisible();
}

test('the bulk bar moves photos to the Immich trash and undo brings them back', async ({
  page
}) => {
  await openGrid(page);
  await selectFirstTwo(page);

  await page
    .getByRole('toolbar', { name: 'Selection actions' })
    .getByRole('button', { name: 'Move to Immich trash' })
    .click();
  const dialog = page.getByRole('dialog', { name: 'Move to Immich trash' });
  await expect(dialog).toContainText('2 photos move to the Immich trash');
  await expect(dialog).toContainText('after 30 days');
  const trashed = page.waitForRequest(
    (req) => req.url().endsWith('/api/trash') && req.method() === 'POST'
  );
  await dialog.getByRole('button', { name: 'Move to trash' }).click();
  expect((await trashed).postDataJSON()).toEqual({
    ids: [ASSETS[0]?.id, ASSETS[1]?.id]
  });

  await expect(tile(page, 'IMG_0001.ARW')).toBeHidden();
  await expect(tile(page, 'IMG_0002.ARW')).toBeHidden();
  await expect(tile(page, 'IMG_0003.ARW')).toBeVisible();
  await expect(page.getByText('2 selected')).toBeHidden();

  await page.getByRole('button', { name: 'Undo' }).click();
  await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
  await expect(tile(page, 'IMG_0002.ARW')).toBeVisible();
});

test('the delete key asks first and cancel keeps the photo', async ({ page }) => {
  await openGrid(page);
  await page.getByRole('button', { name: 'Select', exact: true }).first().click();

  await page.keyboard.press('Delete');
  const dialog = page.getByRole('dialog', { name: 'Move to Immich trash' });
  await expect(dialog).toContainText('1 photo moves to the Immich trash');
  await dialog.getByRole('button', { name: 'Cancel' }).click();

  await expect(dialog).toBeHidden();
  await expect(tile(page, 'IMG_0001.ARW')).toBeVisible();
});

test('enter confirms the trash dialog without opening the photo behind it', async ({ page }) => {
  await openGrid(page);
  await selectFirstTwo(page);

  await page
    .getByRole('toolbar', { name: 'Selection actions' })
    .getByRole('button', { name: 'Move to Immich trash' })
    .click();
  const dialog = page.getByRole('dialog', { name: 'Move to Immich trash' });
  await expect(dialog.getByRole('button', { name: 'Move to trash' })).toBeFocused();
  const trashed = page.waitForRequest(
    (req) => req.url().endsWith('/api/trash') && req.method() === 'POST'
  );
  await page.keyboard.press('Enter');
  await trashed;

  await expect(dialog).toBeHidden();
  await expect(tile(page, 'IMG_0001.ARW')).toBeHidden();
  await expect(page).toHaveURL(/\/photos$/);
});

test('nothing is trashed while the Immich trash is off', async ({ page }) => {
  let calls = 0;
  page.on('request', (req) => {
    if (req.url().endsWith('/api/trash')) calls += 1;
  });
  await openGrid(page, { trash: false });
  await page.getByRole('button', { name: 'Select', exact: true }).first().click();

  await page.keyboard.press('Delete');

  await expect(page.getByText('The Immich trash is turned off')).toBeVisible();
  await expect(page.getByRole('dialog')).toBeHidden();
  expect(calls).toBe(0);
});

test('the loupe moves on to the next photo after trashing', async ({ page }) => {
  await openGrid(page);
  await page.getByLabel('Quick review').first().click();
  await expect(page.getByRole('img', { name: 'IMG_0001.ARW' })).toBeVisible();

  await page.getByRole('button', { name: 'More loupe actions' }).click();
  await page.getByRole('button', { name: 'Move to Immich trash' }).click();
  await page
    .getByRole('dialog', { name: 'Move to Immich trash' })
    .getByRole('button', { name: 'Move to trash' })
    .click();

  await expect(page.getByRole('img', { name: 'IMG_0002.ARW' })).toBeVisible();
});

test('trashing from the editor goes back to the grid without the photo', async ({ page }) => {
  await installMocks(page, { assets: ASSETS });
  await page.goto(`/assets/${ASSETS[0]?.id}?from=${encodeURIComponent('/photos')}`);
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();

  await page.getByRole('button', { name: 'More editor actions' }).click();
  await page.getByRole('button', { name: 'Move to Immich trash' }).click();
  await page
    .getByRole('dialog', { name: 'Move to Immich trash' })
    .getByRole('button', { name: 'Move to trash' })
    .click();

  await expect(page).toHaveURL(/\/photos$/);
  await expect(tile(page, 'IMG_0002.ARW')).toBeVisible();
  await expect(tile(page, 'IMG_0001.ARW')).toBeHidden();
});

test('a selected virtual copy is deleted, not trashed', async ({ page }) => {
  await openGrid(page);
  await page.evaluate(
    (id) => fetch(`/api/assets/${id}/copies`, { method: 'POST', body: '{}' }),
    ASSETS[0]?.id
  );
  await page.reload();
  await expect(tile(page, 'IMG_0001.ARW')).toHaveCount(2);
  await page.getByRole('button', { name: 'Select', exact: true }).nth(1).click();

  await page.keyboard.press('Backspace');
  const dialog = page.getByRole('dialog', { name: 'Delete virtual copy' });
  await expect(dialog).toContainText('1 virtual copy is deleted from immich-edit');
  const deleted = page.waitForRequest(
    (req) => req.url().endsWith(`/api/copies/${ASSETS[0]?.id}_1`) && req.method() === 'DELETE'
  );
  await dialog.getByRole('button', { name: 'Delete' }).click();
  await deleted;

  await expect(tile(page, 'IMG_0001.ARW')).toHaveCount(1);
});

test('a finished cull moves every reject to the trash', async ({ page }) => {
  const [kept, dropped] = ASSETS;
  const reject = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
  await installMocks(page, {
    assets: [kept, { ...dropped, tags: [reject] }].filter((a) => a !== undefined)
  });
  await page.goto('/photos');
  await openFilters(page, 'Library');
  await page.getByRole('combobox', { name: 'Rejected', exact: true }).click();
  await page.getByRole('option', { name: 'Only', exact: true }).click();
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Move rejected to trash' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move to Immich trash' });
  await expect(dialog).toContainText('1 photo moves to the Immich trash');
  const trashed = page.waitForRequest((req) => req.url().endsWith('/api/trash'));
  await dialog.getByRole('button', { name: 'Move to trash' }).click();

  expect((await trashed).postDataJSON()).toEqual({ ids: [dropped?.id] });
  await expect(tile(page, 'IMG_0002.ARW')).toBeHidden();
});
