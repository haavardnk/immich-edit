import { expect, test, type Page } from '@playwright/test';
import { installMocks } from './helpers';

function rootState(page: Page): Promise<{ dark: boolean; light: boolean; primary: string }> {
  return page.evaluate(() => {
    const root = document.documentElement;
    return {
      dark: root.classList.contains('dark'),
      light: root.classList.contains('light'),
      primary: getComputedStyle(root).getPropertyValue('--immich-ui-primary-500').trim()
    };
  });
}

async function emulateScheme(page: Page, colorScheme: 'light' | 'dark'): Promise<void> {
  await page.emulateMedia({ colorScheme });
  await page.waitForFunction(
    (dark) => matchMedia('(prefers-color-scheme: dark)').matches === dark,
    colorScheme === 'dark'
  );
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)))
  );
}

test('the default dark theme survives an OS switch to light', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await installMocks(page);
  await page.goto('/photos');
  await expect(page.getByRole('link', { name: 'Photos' })).toBeVisible();

  await emulateScheme(page, 'light');

  expect(await rootState(page)).toEqual({
    dark: true,
    light: false,
    primary: '#b7a1fb'
  });
});
