import { expect, test } from '@playwright/test';

import { installMocks } from './helpers';

test('the shortcuts dialog opens from the keyboard and returns focus on close', async ({
  page
}) => {
  await installMocks(page);
  await page.goto('/photos');

  const trigger = page.getByRole('button', { name: 'Keyboard shortcuts' });
  await trigger.focus();
  await page.keyboard.press('Enter');

  const filter = page.getByRole('textbox', { name: 'Filter shortcuts' });
  await expect(filter).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(filter).toBeHidden();
  await expect(trigger).toBeFocused();
});

test('the sidebar section trigger toggles with the keyboard', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  const albums = page.getByRole('button', { name: /^Albums/ });
  await albums.focus();
  await expect(albums).toHaveAttribute('aria-expanded', 'false');

  await page.keyboard.press('Enter');
  await expect(albums).toHaveAttribute('aria-expanded', 'true');

  await page.keyboard.press('Enter');
  await expect(albums).toHaveAttribute('aria-expanded', 'false');
});
