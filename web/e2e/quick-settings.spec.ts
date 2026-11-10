import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, installMocks, type InstallOpts } from './helpers';
import { longPressUntil } from './touch';
import { open } from './touchShell';

function quick(page: Page): Locator {
  return page.locator('[data-panel="quick"]');
}

function basic(page: Page): Locator {
  return page.locator('[data-panel="basic"]');
}

function pinsSaved(saved: unknown[]): InstallOpts['onPreferences'] {
  return (patch) => {
    if ('pinned_settings' in patch) saved.push(patch.pinned_settings);
  };
}

async function openDesktop(page: Page, opts: InstallOpts = {}): Promise<void> {
  await installMocks(page, opts);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`/assets/${ASSET_ID}`);
  await expect(basic(page).getByRole('slider', { name: 'Exposure' })).toBeVisible();
}

async function rowMenu(page: Page, row: Locator): Promise<Locator> {
  await row.locator('button').first().click({ button: 'right' });
  const menu = page.getByRole('menu');
  await expect(menu).toBeVisible();
  return menu;
}

test.describe('quick section on desktop', () => {
  test('shows the default pins, and both rows edit one value', async ({ page }) => {
    await openDesktop(page, { pinnedSettings: null });
    const sliders = quick(page).getByRole('slider');
    await expect(sliders).toHaveCount(6);
    expect(
      await quick(page)
        .locator('[data-setting]')
        .evaluateAll((rows) => rows.map((row) => row.getAttribute('data-setting')))
    ).toEqual(['exposure', 'contrast', 'shadows', 'vibrance', 'clarity', 'sharpen_amount']);

    const pinned = quick(page).getByRole('slider', { name: 'Contrast' });
    await pinned.focus();
    await page.keyboard.press('ArrowRight');
    const home = basic(page).getByRole('slider', { name: 'Contrast' });
    await expect(home).not.toHaveValue('0');
    await expect(home).toHaveValue(await pinned.inputValue());
    await expect(quick(page).getByRole('button', { name: 'Reset Quick panel' })).toBeVisible();
  });

  test('the row menu pins, unpins and resets', async ({ page }) => {
    const saved: unknown[] = [];
    await openDesktop(page, { onPreferences: pinsSaved(saved) });
    await expect(quick(page)).toHaveCount(0);

    let menu = await rowMenu(page, basic(page).locator('[data-setting="highlights"]'));
    await expect(menu.getByRole('menuitem', { name: 'Reset Highlights' })).toBeDisabled();
    await expect(menu.getByRole('menuitem', { name: 'Adjust on photo' })).toHaveCount(0);
    await menu.getByRole('menuitem', { name: 'Pin to Quick' }).click();
    await expect(quick(page).getByRole('slider', { name: 'Highlights' })).toBeVisible();
    expect(saved).toEqual([['highlights']]);

    await quick(page).getByRole('slider', { name: 'Highlights' }).focus();
    await page.keyboard.press('ArrowLeft');
    menu = await rowMenu(page, quick(page).locator('[data-setting="highlights"]'));
    await menu.getByRole('menuitem', { name: 'Reset Highlights' }).click();
    await expect(basic(page).getByRole('slider', { name: 'Highlights' })).toHaveValue('0');

    menu = await rowMenu(page, quick(page).locator('[data-setting="highlights"]'));
    await menu.getByRole('menuitem', { name: 'Unpin' }).click();
    await expect(quick(page)).toHaveCount(0);
    expect(saved).toEqual([['highlights'], []]);
  });

  test('Adjust on photo arms the section target from its rows', async ({ page }) => {
    await openDesktop(page);
    await page.getByRole('button', { name: 'HSL', exact: true }).click();
    const menu = await rowMenu(page, page.locator('[data-setting="hsl_red_sat"]'));
    await menu.getByRole('menuitem', { name: 'Adjust on photo' }).click();
    await expect(page.getByRole('button', { name: 'Adjust HSL on the photo' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  test('Shift+Enter in the palette pins the highlighted setting', async ({ page }) => {
    const saved: unknown[] = [];
    await openDesktop(page, { onPreferences: pinsSaved(saved) });
    await page.keyboard.press('ControlOrMeta+k');
    const palette = page.getByRole('dialog', { name: 'Search settings and actions' });
    await palette.getByRole('combobox').fill('highlights');
    await expect(palette.getByRole('option').first()).toContainText('Highlights');
    await page.keyboard.press('Shift+Enter');
    await expect(palette.getByRole('option').first().getByLabel('Pinned')).toBeVisible();
    expect(saved).toEqual([['highlights']]);
    await page.keyboard.press('Escape');
    await expect(quick(page).getByRole('slider', { name: 'Highlights' })).toBeVisible();
  });

  test('a palette jump lands on the home row, not the pinned one', async ({ page }) => {
    await openDesktop(page, { pinnedSettings: ['exposure'] });
    await expect(quick(page).getByRole('slider', { name: 'Exposure' })).toBeVisible();
    await page.keyboard.press('ControlOrMeta+k');
    const palette = page.getByRole('dialog', { name: 'Search settings and actions' });
    await palette.getByRole('combobox').fill('exposure');
    await page.keyboard.press('Enter');
    await expect(basic(page).locator('[data-setting="exposure"]')).toHaveAttribute(
      'data-flash',
      ''
    );
    await expect(basic(page).getByRole('slider', { name: 'Exposure' })).toBeFocused();
  });
});

test.describe('quick group on touch', { tag: ['@touch'] }, () => {
  function groups(page: Page): Locator {
    return page.getByRole('tablist', { name: 'Develop groups' });
  }

  async function openTouch(page: Page, opts: InstallOpts = {}): Promise<void> {
    await open(page, `/assets/${ASSET_ID}`, opts);
    await expect(page.getByRole('slider', { name: 'Exposure' })).toBeVisible();
  }

  test('Quick is the first chip and lists the pins', async ({ page }) => {
    await openTouch(page, { pinnedSettings: null });
    await expect(groups(page).getByRole('tab').first()).toHaveText('Quick');
    await groups(page).getByRole('tab', { name: 'Quick' }).tap();
    const region = page.getByRole('region', { name: 'Quick' });
    await expect(region.getByRole('slider')).toHaveCount(6);
    await expect(page.getByText('6 pinned')).toBeVisible();
    await expect(
      page.getByText('Hold a label to pin or unpin').filter({ visible: true })
    ).toHaveCount(1);
  });

  test('holding a label pins the row', async ({ page }) => {
    const saved: unknown[] = [];
    await openTouch(page, { onPreferences: pinsSaved(saved) });
    const sheet = page.getByRole('dialog', { name: 'Highlights · Light › Tone' });
    await longPressUntil(page, page.getByRole('button', { name: 'Focus Highlights' }), sheet);
    await expect(sheet.getByRole('button', { name: 'Reset Highlights' })).toBeDisabled();
    await sheet.getByRole('button', { name: 'Pin to Quick' }).tap();
    await expect(sheet).toHaveCount(0);
    await expect(page.getByRole('region', { name: 'Focus Highlights' })).toHaveCount(0);
    expect(saved).toEqual([['highlights']]);

    await groups(page).getByRole('tab', { name: 'Quick' }).tap();
    await expect(
      page.getByRole('region', { name: 'Quick' }).getByRole('slider', { name: 'Highlights' })
    ).toBeVisible();
    await expect(page.getByText('1 pinned')).toBeVisible();
  });

  test('the keypad pins and unpins its setting', async ({ page }) => {
    const saved: unknown[] = [];
    await openTouch(page, { pinnedSettings: null, onPreferences: pinsSaved(saved) });
    await page.getByRole('button', { name: 'Edit Exposure value' }).tap();
    const pad = page.getByRole('region', { name: 'Exposure keypad' });
    await pad.getByRole('button', { name: 'Unpin Exposure' }).tap();
    await expect(pad.getByRole('button', { name: 'Pin Exposure to Quick' })).toBeVisible();
    expect(saved).toEqual([['contrast', 'shadows', 'vibrance', 'clarity', 'sharpen_amount']]);
    await pad.getByRole('button', { name: 'Done' }).tap();
    await expect(pad).toHaveCount(0);
  });
});
