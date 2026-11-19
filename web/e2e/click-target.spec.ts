import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';
import { tapCentre } from './touch';

async function openGrid(page: Page): Promise<void> {
  await installMocks(page);
  await page.goto('/photos');
  await expect(tile(page)).toBeVisible();
}

function tile(page: Page) {
  return page.locator('[role="group"][data-name]').first();
}

async function expectLoupe(page: Page): Promise<void> {
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await expect(page).toHaveURL(/\/photos/);
}

async function expectEditor(page: Page): Promise<void> {
  await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}`));
}

test('a click opens the loupe and the hover button the editor', async ({ page }) => {
  await openGrid(page);
  await tile(page)
    .getByRole('link', { name: 'IMG_0001.ARW' })
    .click({ position: { x: 8, y: 8 } });
  await expectLoupe(page);

  await page.keyboard.press('Escape');
  await tile(page).hover();
  const edit = tile(page).getByRole('link', { name: 'Edit' });
  await edit.hover();
  await expect(page.locator('.tooltip-content', { hasText: 'Edit' })).toBeVisible();
  await edit.click();
  await expectEditor(page);
});

test('the D key opens the editor from a selected tile', async ({ page }) => {
  await openGrid(page);
  await tile(page).getByRole('button', { name: 'Select' }).click();
  await page.keyboard.press('d');
  await expectEditor(page);
});

test('settings offer no click setting', async ({ page }) => {
  await installMocks(page);
  await page.goto('/settings/preferences#appearance');
  await expect(page.getByRole('radiogroup', { name: 'Mode' })).toBeVisible();
  await expect(page.getByRole('radiogroup', { name: 'Click a photo' })).toHaveCount(0);
});

test.describe('click target on touch', { tag: '@touch' }, () => {
  test('a tap opens the loupe', async ({ page }) => {
    await openGrid(page);
    await tapCentre(page, tile(page));
    await expectLoupe(page);
  });
});
