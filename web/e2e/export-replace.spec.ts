import { expect, test, type Page } from '@playwright/test';
import { gotoAsset, installMocks, json, type InstallOpts } from './helpers';

async function openImmichExport(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, opts);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
}

test('replace original warns, drops stacking and leaves for the grid', async ({ page }) => {
  await openImmichExport(page);
  const bodies: Record<string, unknown>[] = [];
  await page.route('**/api/assets/*/export/immich', (route) => {
    bodies.push(route.request().postDataJSON() as Record<string, unknown>);
    return route.fulfill(
      json({
        asset_id: '00000000-0000-0000-0000-0000000000aa',
        filename: 'IMG_0001_edit.jpg',
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

  await panel.getByRole('button', { name: /Upload JPEG to Immich/ }).click();
  await expect(page).toHaveURL(/\/photos$/);
  await expect(
    page.getByText('Uploaded IMG_0001_edit.jpg to Immich; the original moved to the Immich trash')
  ).toBeVisible();
  expect(bodies).toEqual([
    expect.objectContaining({ replace_original: true, stack_with_original: false })
  ]);
});

test('replace original is unavailable while the Immich trash is off', async ({ page }) => {
  await openImmichExport(page, { trash: false });

  await expect(
    page.getByRole('tabpanel').getByRole('checkbox', { name: 'Replace original' })
  ).toBeDisabled();
});
