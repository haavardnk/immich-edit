import { expect, test, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, json, type InstallOpts } from './helpers';
import { unfoldExport } from './exportSections';
import { mockLibrary } from './touchShell';

async function openImmichExport(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, opts);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await unfoldExport(page.getByRole('tabpanel'));
}

test('replace original warns, drops stacking and leaves for the grid', async ({ page }) => {
  await openImmichExport(page);
  const bodies: Record<string, unknown>[] = [];
  await page.route('**/api/assets/*/export/immich', (route) => {
    bodies.push(route.request().postDataJSON() as Record<string, unknown>);
    return route.fulfill(
      json({
        asset_id: '00000000-0000-0000-0000-0000000000aa',
        filename: 'IMG_0001.jpg',
        status: 'created',
        warnings: [],
        original_trashed: true
      })
    );
  });
  const panel = page.getByRole('tabpanel');
  const replace = panel.getByRole('checkbox', { name: 'Replace original' });

  await replace.click();
  await expect(
    panel.getByText(/the original goes to the Immich trash and is deleted/)
  ).toBeVisible();
  await expect(panel.getByText(/for good after 30 days/)).toBeVisible();
  await expect(panel.getByRole('checkbox', { name: 'Stack with original' })).toBeDisabled();
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveCount(0);
  await expect(panel.getByText('IMG_0001.jpg', { exact: true })).toBeVisible();
  await replace.click();
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue('{name}_edit');
  await replace.click();

  await panel.getByRole('button', { name: /Upload JPEG to Immich/ }).click();
  await expect(page).toHaveURL(/\/photos$/);
  await expect(
    page.getByText('Uploaded IMG_0001.jpg to Immich; the original moved to the Immich trash')
  ).toBeVisible();
  expect(bodies).toEqual([
    expect.objectContaining({
      replace_original: true,
      stack_with_original: false,
      filename_template: '{name}'
    })
  ]);
});

test('replace original is unavailable while the Immich trash is off', async ({ page }) => {
  await openImmichExport(page, { trash: false });

  await expect(
    page.getByRole('tabpanel').getByRole('checkbox', { name: 'Replace original' })
  ).toBeDisabled();
});

for (const device of ['phone', 'tablet'] as const) {
  test(
    `the touch sheet names a replacing upload after the original on a ${device}`,
    { tag: ['@touch', `@${device}`] },
    async ({ page }) => {
      await mockLibrary(page);
      await page.goto(`/assets/${ASSET_ID}`);
      await page.getByRole('button', { name: 'Share', exact: true }).tap();
      const sheet = page.getByRole('dialog', { name: 'Export', exact: true });
      await sheet.getByRole('radio', { name: 'Immich' }).tap();
      const name = sheet.getByRole('region', { name: 'Name' }).getByRole('button');
      await expect(name).toContainText('{name}_edit');
      await sheet.getByRole('checkbox', { name: 'Replace original' }).tap();
      await expect(name).toContainText('Original name');
      await name.tap();
      await expect(sheet.getByText('IMG_0001.jpg', { exact: true })).toBeVisible();
    }
  );
}
