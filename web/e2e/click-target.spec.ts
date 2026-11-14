import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, installMocks } from './helpers';
import { tapCentre } from './touch';

type Target = 'editor' | 'loupe';

async function openGrid(page: Page, clickTarget: Target = 'editor'): Promise<string[]> {
  const saved: string[] = [];
  await installMocks(page, {
    clickTarget,
    onPreferences: (patch) => saved.push(String(patch.click_target))
  });
  await page.goto('/photos');
  await expect(tile(page)).toBeVisible();
  return saved;
}

function tile(page: Page) {
  return page.locator('[role="group"][data-name]').first();
}

function tileLink(page: Page) {
  return tile(page).getByRole('link', { name: 'IMG_0001.ARW' });
}

async function expectLoupe(page: Page): Promise<void> {
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await expect(page).toHaveURL(/\/photos/);
}

async function expectEditor(page: Page): Promise<void> {
  await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}`));
}

test('a click opens the editor and the hover button the loupe by default', async ({ page }) => {
  await openGrid(page);
  await tile(page).hover();
  await expect(tile(page).getByRole('link', { name: 'Edit' })).toHaveCount(0);
  await tile(page).getByRole('button', { name: 'Quick review' }).click();
  await expectLoupe(page);

  await page.keyboard.press('Escape');
  await tileLink(page).click({ position: { x: 8, y: 8 } });
  await expectEditor(page);
});

test('with the loupe setting a click opens the loupe and the hover button the editor', async ({
  page
}) => {
  await openGrid(page, 'loupe');
  await tileLink(page).click({ position: { x: 8, y: 8 } });
  await expectLoupe(page);

  await page.keyboard.press('Escape');
  await tile(page).hover();
  await expect(tile(page).getByRole('button', { name: 'Quick review' })).toHaveCount(0);
  const edit = tile(page).getByRole('link', { name: 'Edit' });
  await edit.hover();
  await expect(page.locator('.tooltip-content', { hasText: 'Edit' })).toBeVisible();
  await edit.click();
  await expectEditor(page);
});

test('the keys keep their targets whatever a click opens', async ({ page }) => {
  await openGrid(page, 'loupe');
  await tile(page).getByRole('button', { name: 'Select' }).click();
  await page.keyboard.press('d');
  await expectEditor(page);
});

test('the setting is chosen in Appearance and kept for the account', async ({ page }) => {
  const saved = await openGrid(page);
  await page.goto('/settings#appearance');
  const choice = page.getByRole('radiogroup', { name: 'Click a photo' });
  await expect(choice.getByRole('radio', { name: 'Open editor' })).toBeChecked();

  await choice.getByRole('radio', { name: 'Open loupe' }).click();
  await expect.poll(() => saved).toEqual(['loupe']);

  await page.goto('/photos');
  await tileLink(page).click({ position: { x: 8, y: 8 } });
  await expectLoupe(page);
});

test.describe('click target on touch', { tag: '@touch' }, () => {
  test('a tap opens the loupe whatever the setting says', async ({ page }) => {
    await openGrid(page, 'editor');
    await tapCentre(page, tile(page));
    await expectLoupe(page);
  });

  test('settings offer no click setting', async ({ page }) => {
    await installMocks(page);
    await page.goto('/settings#appearance');
    await expect(page.getByRole('radiogroup', { name: 'Mode' })).toBeVisible();
    await expect(page.getByRole('radiogroup', { name: 'Click a photo' })).toHaveCount(0);
  });
});
