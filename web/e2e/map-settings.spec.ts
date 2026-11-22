import { expect, test } from '@playwright/test';
import { installMocks, settingsGroup } from './helpers';

test('admins pick a custom map style and turn place search off', async ({ page }) => {
  const saved: unknown[] = [];
  await installMocks(page, { onMapSettings: (body) => saved.push(body) });
  await page.goto('/settings/administration#map-settings');
  const group = settingsGroup(page, 'Map');
  await expect(group.getByRole('radio', { name: 'Follow Immich' })).toBeChecked();
  const search = group.locator('[data-setting-anchor="place-search"]');
  await expect(search.getByText('Place search', { exact: true })).toBeVisible();
  const hint = (await search.getByText(/^Searches go to this Photon server/).boundingBox())!;
  expect(hint.width).toBeGreaterThan(240);
  await group.getByRole('radio', { name: 'Custom' }).click();
  await expect(group.getByRole('button', { name: 'Save' })).toBeDisabled();
  await group
    .getByRole('textbox', { name: 'Dark style URL' })
    .fill('https://tiles.openfreemap.org/styles/dark');
  await group.getByRole('textbox', { name: 'Photon server URL' }).fill('');
  await group.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Map settings saved')).toBeVisible();
  expect(saved).toEqual([
    {
      styleLight: 'https://tiles.openfreemap.org/styles/dark',
      styleDark: 'https://tiles.openfreemap.org/styles/dark',
      geocoderUrl: ''
    }
  ]);
  await expect(group.getByRole('button', { name: 'Save' })).toHaveCount(0);
});
