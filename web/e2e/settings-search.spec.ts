import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, NEUTRAL_RECORD, installMocks } from './helpers';
import { open } from './touchShell';

const MODIFIED_RECORD = {
  ...NEUTRAL_RECORD,
  hash: 'hash-modified',
  manifest: { schema_version: 4, ops: { exposure: { ev: 0.35 }, vibrance: { amount: 12 } } }
};

function tray(page: Page): Locator {
  return page.getByRole('region', { name: 'Adjustments' });
}

function findField(page: Page): Locator {
  return page.getByRole('textbox', { name: 'Search settings', exact: true });
}

async function openTouchEditor(page: Page, editRecord = NEUTRAL_RECORD): Promise<void> {
  await open(page, `/assets/${ASSET_ID}`, { editRecord });
  await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

test.describe('settings search on touch', { tag: ['@touch'] }, () => {
  test('search matches and jumps to the row', async ({ page }) => {
    await openTouchEditor(page);
    await page.getByRole('button', { name: 'Search settings', exact: true }).tap();
    await expect(findField(page)).toBeFocused();

    await findField(page).fill('aqua sat');
    const result = page.getByRole('button', { name: 'Aqua saturation, Color › Mix › Aqua' });
    await expect(result).toBeVisible();
    await expect(result.locator('mark')).toHaveText('Aqua sat');
    await result.tap();

    await expect(findField(page)).toHaveCount(0);
    await expect(
      page.getByRole('tablist', { name: 'Develop groups' }).getByRole('tab', { name: 'Color' })
    ).toHaveAttribute('aria-selected', 'true');
    await expect(
      page.getByRole('tablist', { name: 'Color views' }).getByRole('tab', { name: 'Mix' })
    ).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('radio', { name: 'Aqua' })).toBeChecked();
    await expect(page.locator('[data-setting="hsl_aqua_sat"]')).toHaveAttribute('data-flash', '');
  });
});

test.describe('settings search on a phone', { tag: ['@touch', '@phone'] }, () => {
  test('Modified lists changed settings as live rows under their paths', async ({ page }) => {
    await openTouchEditor(page, MODIFIED_RECORD);
    const top = (await tray(page).boundingBox())?.y;
    await page.getByRole('button', { name: 'Search settings', exact: true }).tap();
    await expect(findField(page)).toBeFocused();
    await expect.poll(async () => (await tray(page).boundingBox())?.y).toBe(top);
    await expect(tray(page)).toHaveAttribute('data-detent', 'compact');
    await expect(page.getByRole('tablist', { name: 'Editor modes' })).toHaveCount(0);
    await page.getByRole('radio', { name: 'Modified · 2' }).tap();

    const tone = page.getByRole('region', { name: 'Light › Tone' });
    await expect(tone.getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await expect(tone.getByRole('slider', { name: 'Contrast' })).toHaveCount(0);
    await expect(
      page.getByRole('region', { name: 'Color › Basic' }).getByRole('slider', { name: 'Vibrance' })
    ).toBeVisible();

    await page.getByRole('button', { name: 'Done' }).tap();
    await expect(findField(page)).toHaveCount(0);
    await expect(page.getByRole('tablist', { name: 'Editor modes' })).toBeVisible();
  });
});

test.describe('settings filter on desktop', () => {
  function filter(page: Page): Locator {
    return page.getByRole('textbox', { name: 'Search settings', exact: true });
  }

  test('narrows the inspector to matching panels and rows', async ({ page }) => {
    await installMocks(page);
    await page.goto(`/assets/${ASSET_ID}`);
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();

    await filter(page).fill('vib');
    await expect(page.getByRole('slider', { name: 'Vibrance' })).toBeVisible();
    await expect(page.getByRole('slider', { name: 'Exposure' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'HSL', exact: true })).toHaveCount(0);

    await filter(page).fill('aqua');
    await expect(page.getByRole('radio', { name: 'Aqua' })).toBeChecked();
    await expect(page.getByRole('slider', { name: 'Saturation' })).toBeVisible();

    await filter(page).fill('zzz');
    await expect(page.getByText('No matching settings')).toBeVisible();

    await filter(page).press('Escape');
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  });
});
