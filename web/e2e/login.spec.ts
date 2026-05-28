import { expect, test } from '@playwright/test';

test('login page renders sign-in form', async ({ page }) => {
  await page.goto('/login');
  await expect(page.getByRole('heading', { name: 'immich-edit' })).toBeVisible();
  await expect(page.getByLabel('Access token')).toBeVisible();
  await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
});
