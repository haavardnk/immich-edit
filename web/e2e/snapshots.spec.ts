import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks } from './helpers';
import { exposureManifest, mockSnapshots, type SnapshotMock } from './snapshotMocks';
import { mockLibrary } from './touchShell';

const WARM = { name: 'Warm v1', exposure: 0.7, minutesAgo: 125 };

function snapshots(page: Page): Locator {
  return page.getByRole('region', { name: 'Snapshots' });
}

function row(scope: Locator, name: string): Locator {
  return scope.getByRole('button', { name: `Restore ${name}`, exact: true });
}

function toast(page: Page, message: string): Locator {
  return page.getByRole('status').filter({ hasText: message });
}

async function openEditor(page: Page): Promise<SnapshotMock> {
  await installMocks(page);
  const mock = await mockSnapshots(page, [WARM]);
  await gotoAsset(page);
  return mock;
}

async function openHistory(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: 'Edit history' }).click();
  const list = snapshots(page);
  await expect(list).toBeVisible();
  return list;
}

test.describe('snapshots on desktop', () => {
  test('the history popover lists, names, updates and deletes snapshots', async ({ page }) => {
    const mock = await openEditor(page);
    const list = await openHistory(page);

    await expect(row(list, 'Warm v1')).toContainText('2 h ago');
    await expect(
      page.locator('[data-popover-content]').getByText('History', { exact: true })
    ).toBeVisible();

    await list.getByRole('button', { name: 'New snapshot' }).click();
    const field = list.getByRole('textbox', { name: 'Snapshot name' });
    await expect(field).toBeFocused();
    await expect(field).toHaveValue('Snapshot 1');
    expect(await field.evaluate((input: HTMLInputElement) => input.selectionEnd)).toBe(10);
    await page.keyboard.type('Before grading');
    await page.keyboard.press('Enter');

    await expect(row(list, 'Before grading')).toBeFocused();
    expect(mock.items.map((item) => item.name)).toEqual(['Before grading', 'Warm v1']);

    await row(list, 'Before grading').hover();
    const rename = list.getByRole('button', { name: 'Rename Before grading' });
    await rename.hover();
    await expect(page.locator('.tooltip-content', { hasText: 'Rename' })).toBeVisible();
    await rename.click();
    await field.fill('Graded');
    await field.press('Enter');
    await expect(row(list, 'Graded')).toBeVisible();
    expect(mock.updates.at(-1)).toEqual({ id: 'snap-2', body: { name: 'Graded' } });

    await row(list, 'Graded').hover();
    await list.getByRole('button', { name: 'Update Graded with current edits' }).click();
    await expect(toast(page, 'Updated Graded')).toBeVisible();
    expect(Object.keys(mock.updates.at(-1)?.body ?? {})).toEqual(['manifest']);

    await row(list, 'Graded').hover();
    await list.getByRole('button', { name: 'Delete Graded' }).click();
    await list.getByRole('button', { name: 'Confirm delete Graded' }).click();
    await expect(row(list, 'Graded')).toHaveCount(0);
    expect(mock.items.map((item) => item.name)).toEqual(['Warm v1']);
  });

  test('restoring keeps the popover open, marks the row and offers undo', async ({ page }) => {
    const saves: unknown[] = [];
    await installMocks(page, { onSave: (body) => saves.push(body.manifest) });
    const mock = await mockSnapshots(page, [WARM]);
    await gotoAsset(page);
    const list = await openHistory(page);

    await row(list, 'Warm v1').click();

    await expect(toast(page, 'Restored Warm v1')).toBeVisible();
    await expect(row(list, 'Warm v1')).toContainText('Restored');
    await expect(list).toBeVisible();
    expect(mock.restores).toEqual([{ id: 'snap-1', ifMatch: 'hash-neutral' }]);

    await toast(page, 'Restored Warm v1').getByRole('button', { name: 'Undo' }).click();
    await expect.poll(() => saves.length).toBe(1);
    expect(saves[0]).not.toEqual(exposureManifest(WARM.exposure));
    await expect(snapshots(page)).toBeHidden();
    const reopened = await openHistory(page);
    await expect(row(reopened, 'Warm v1')).not.toContainText('Restored');
  });

  test('the shortcut saves the next numbered snapshot', async ({ page }) => {
    const mock = await openEditor(page);

    await page.keyboard.press('ControlOrMeta+Shift+S');

    await expect(toast(page, 'Saved Snapshot 1')).toBeVisible();
    expect(mock.items.map((item) => item.name)).toEqual(['Snapshot 1', 'Warm v1']);
    const list = await openHistory(page);
    await expect(row(list, 'Snapshot 1')).toContainText('now');
  });
});

test.describe('snapshots on touch', { tag: ['@touch'] }, () => {
  test('More opens the snapshots sheet, names a new snapshot and restores', async ({ page }) => {
    await mockLibrary(page);
    const mock = await mockSnapshots(page, [WARM]);
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.locator('html')).toHaveAttribute('data-layout', 'touch');

    await page.getByRole('button', { name: 'More editor actions' }).tap();
    const entry = page.getByRole('button', { name: 'Snapshots', exact: true });
    await expect(entry).toContainText('1');
    await entry.tap();

    const sheet = page.getByRole('dialog', { name: 'Snapshots' });
    await expect(sheet).toContainText('IMG_0001.ARW · 1 snapshot');
    await expect(row(sheet, 'Warm v1').locator('img')).toHaveCount(1);

    await sheet.getByRole('button', { name: 'New snapshot' }).tap();
    const field = sheet.getByRole('textbox', { name: 'Snapshot name' });
    await expect(field).toHaveValue('Snapshot 1');
    await field.fill('Mine');
    await field.press('Enter');
    await expect(row(sheet, 'Mine')).toBeVisible();
    await expect(sheet).toContainText('2 snapshots');
    expect(mock.items[0]?.name).toBe('Mine');

    await row(sheet, 'Warm v1').tap();
    await expect(toast(page, 'Restored Warm v1')).toBeVisible();
    await expect(sheet).toBeVisible();
    await expect(row(sheet, 'Warm v1')).toContainText('Restored');

    await toast(page, 'Restored Warm v1').getByRole('button', { name: 'Undo' }).tap();
    await expect(row(sheet, 'Warm v1')).not.toContainText('Restored');
    await expect(sheet).toBeVisible();

    await sheet.getByRole('button', { name: 'Actions for Mine' }).tap();
    await page.getByRole('button', { name: 'Delete', exact: true }).tap();
    await page.getByRole('button', { name: 'Confirm delete', exact: true }).tap();
    await expect(row(sheet, 'Mine')).toHaveCount(0);
    expect(mock.items.map((item) => item.name)).toEqual(['Warm v1']);
  });
});
