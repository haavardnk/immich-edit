import { expect, test } from '@playwright/test';
import { ASSET_DETAIL, ASSET_ID, ASSET_SUMMARY, gotoAsset, installMocks, json } from './helpers';

const TAGS = [
  { id: 'tag-1', value: 'Landscape' },
  { id: 'tag-2', value: 'Portrait' }
];

const NEXT_ASSET = {
  ...ASSET_SUMMARY,
  id: '00000000-0000-0000-0000-000000000002',
  originalFileName: 'IMG_0002.ARW'
};

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

test('a popover closes on Escape and restores focus to its trigger', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/tags', (route) => route.fulfill(json(TAGS)));
  await gotoAsset(page);

  const trigger = page.getByRole('button', { name: 'Tags (T)' });
  await trigger.focus();
  await page.keyboard.press('Enter');

  const search = page.getByPlaceholder('Search tags…');
  await expect(search).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(search).toBeHidden();
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

test('the rating radiogroup keeps its arrow keys from the editor keymap', async ({ page }) => {
  await installMocks(page, { assets: [ASSET_SUMMARY, NEXT_ASSET] });
  await page.addInitScript(() => {
    localStorage.setItem('immich-edit:settings', JSON.stringify({ metadataPushConsented: true }));
  });
  await page.route('**/api/assets/*', async (route) => {
    if (route.request().method() !== 'PUT') return route.fallback();
    const body = route.request().postDataJSON() as { rating?: number | null };
    return route.fulfill(json({ ...ASSET_DETAIL, exifInfo: { rating: body.rating ?? null } }));
  });

  await page.goto('/photos');
  const link = page.locator(`a[href^="/assets/${ASSET_ID}?"]`).first();
  await link.evaluate((el) => (el as HTMLAnchorElement).click());
  await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));

  const group = page.getByRole('radiogroup', { name: 'Rating' });
  await group.focus();
  await expect(group).toBeFocused();

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('radio', { name: '1 star', exact: true })).toHaveAttribute(
    'aria-checked',
    'true'
  );

  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('radio', { name: '2 stars', exact: true })).toHaveAttribute(
    'aria-checked',
    'true'
  );

  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('radio', { name: '1 star', exact: true })).toHaveAttribute(
    'aria-checked',
    'true'
  );

  for (const star of ['1 star', '2 stars', '3 stars', '4 stars', '5 stars'])
    await expect(page.getByRole('radio', { name: star, exact: true })).toHaveAttribute(
      'tabindex',
      '-1'
    );

  await expect(page).toHaveURL(new RegExp(`/assets/${ASSET_ID}(?:\\?.*)?$`));
});
