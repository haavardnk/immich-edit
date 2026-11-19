import { expect, test, type Locator, type Page } from '@playwright/test';
import { editsToManifest } from '../src/lib/edits/manifest';
import { neutralEdits } from '../src/lib/types/edits';
import { ASSET_ID, gotoAsset, installMocks, openPresetMenu, openPresetStrip } from './helpers';
import { tapCentre } from './touch';
import { open } from './touchShell';

function preset(id: string, name: string, group_name: string | null, exposure = 0) {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return {
    id,
    name,
    group_name,
    manifest: editsToManifest(edits),
    favorite: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

const PRESETS = [
  preset('preset-warm', 'Warm', 'Film', 1),
  preset('preset-kodak', 'Kodak 400', 'Film', 0.5),
  preset('preset-mono', 'Mono', 'B&W')
];

type Saves = Array<Record<string, unknown>>;

async function openStrip(page: Page): Promise<{ strip: Locator; saves: Saves }> {
  const saves: Saves = [];
  await installMocks(page, {
    presets: PRESETS.map((p) => ({ ...p })),
    onSave: (body) => saves.push(body)
  });
  await gotoAsset(page);
  const strip = await openPresetStrip(page);
  await expect(tile(strip, 'Warm')).toBeEnabled();
  return { strip, saves };
}

function tile(strip: Locator, name: string): Locator {
  return strip.getByRole('button', { name: `Apply ${name}`, exact: true });
}

function pill(page: Page): Locator {
  return page.locator('[data-photo-pill]');
}

function exposure(page: Page): Locator {
  return page
    .locator('div.group', { has: page.getByRole('button', { name: 'Exposure', exact: true }) })
    .getByRole('slider');
}

test('the applied preset keeps its badge until the next edit', async ({ page }) => {
  const { strip, saves } = await openStrip(page);
  await tile(strip, 'Warm').click();
  await page.mouse.move(0, 0);

  await expect(tile(strip, 'Warm')).toHaveAttribute('aria-current', 'true');
  await expect(tile(strip, 'Warm').locator('[data-applied-badge]')).toBeVisible();
  await expect(tile(strip, 'Mono')).not.toHaveAttribute('aria-current');
  await expect(pill(page)).toHaveText('Warm applied · Amount 100% stays live until your next edit');
  await expect(strip.getByText('Blending Warm')).toBeVisible();
  await expect.poll(() => saves.map((body) => body.action)).toEqual(['Preset: Warm']);

  await exposure(page).fill('0.3');
  await expect(tile(strip, 'Warm')).not.toHaveAttribute('aria-current');
  await expect(pill(page)).toHaveCount(0);
  await expect(strip.getByText('Blending Warm')).toHaveCount(0);
});

test('Amount stays live after an apply and amends the same step', async ({ page }) => {
  const { strip, saves } = await openStrip(page);
  const amount = strip.getByRole('slider', { name: 'Preset amount' });
  await tile(strip, 'Warm').click();
  await page.mouse.move(0, 0);
  await expect(exposure(page)).toHaveValue('1');

  await amount.fill('65');
  await expect(exposure(page)).toHaveValue('0.65');
  await expect(pill(page)).toHaveText('Warm applied · Amount 65% stays live until your next edit');
  await expect
    .poll(() => saves.map((body) => body.action))
    .toEqual(['Preset: Warm', 'Preset: Warm (65%)']);

  await page.getByRole('button', { name: /^Undo/ }).click();
  await expect(exposure(page)).toHaveValue('0');
  await expect(page.getByRole('button', { name: /^Undo/ })).toBeDisabled();
  await expect(amount).toHaveValue('100');
  await expect(tile(strip, 'Warm')).not.toHaveAttribute('aria-current');
});

test('the next edit commits the blend and Amount returns to 100%', async ({ page }) => {
  const { strip, saves } = await openStrip(page);
  const amount = strip.getByRole('slider', { name: 'Preset amount' });
  await amount.fill('50');
  await tile(strip, 'Kodak 400').click();
  await page.mouse.move(0, 0);
  await expect(amount).toHaveValue('50');
  await expect(exposure(page)).toHaveValue('0.25');

  await exposure(page).fill('0.4');
  await expect(amount).toHaveValue('100');
  await tile(strip, 'Warm').click();
  await expect(exposure(page)).toHaveValue('1');
  await expect
    .poll(() => saves.map((body) => body.action))
    .toEqual(['Preset: Kodak 400 (50%)', expect.any(String), 'Preset: Warm']);
});

test('favourites get a star and their own filter', async ({ page }) => {
  const { strip } = await openStrip(page);
  const groups = strip.getByRole('navigation', { name: 'Preset groups' });
  await expect(groups.getByRole('button')).toHaveText(['All', 'B&W', 'Film']);
  await expect(strip.getByRole('img', { name: 'Favourite' })).toHaveCount(0);

  const menu = await openPresetMenu(page, 'Mono');
  await menu.getByRole('button', { name: 'Add to favourites' }).click();
  await expect(strip.getByRole('img', { name: 'Favourite' })).toHaveCount(1);
  await expect(groups.getByRole('button')).toHaveText(['Favourites', 'All', 'B&W', 'Film']);

  await groups.getByRole('button', { name: 'Favourites' }).click();
  await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText(['Mono']);

  const again = await openPresetMenu(page, 'Mono');
  await again.getByRole('button', { name: 'Remove from favourites' }).click();
  await expect(groups.getByRole('button')).toHaveText(['All', 'B&W', 'Film']);
  await expect(strip.getByRole('button', { name: /^Apply / })).toHaveCount(3);
});

test('search heads the group column and finds names in every group', async ({ page }) => {
  const { strip } = await openStrip(page);
  const search = strip.getByRole('textbox', { name: 'Search presets' });
  const groups = strip.getByRole('navigation', { name: 'Preset groups' });
  expect((await search.boundingBox())?.y ?? 0).toBeLessThan((await groups.boundingBox())?.y ?? 0);
  await groups.getByRole('button', { name: 'B&W' }).click();

  await search.fill('o');
  await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText(['Kodak 400', 'Mono']);
  await expect(groups.getByRole('button', { pressed: true })).toHaveCount(0);
  await search.fill('zzz');
  await expect(strip.getByText('No presets match.')).toBeVisible();

  await groups.getByRole('button', { name: 'Film' }).click();
  await expect(search).toHaveValue('');
  await expect(strip.getByRole('button', { name: /^Apply / })).toHaveText(['Warm', 'Kodak 400']);
});

test.describe('touch applied preset', { tag: '@touch' }, () => {
  test('the tray marks the applied preset and keeps Amount live', async ({ page }) => {
    const saves: Saves = [];
    await open(page, `/assets/${ASSET_ID}`, {
      presets: PRESETS.map((p) => ({ ...p, favorite: p.id === 'preset-kodak' })),
      onSave: (body) => saves.push(body)
    });
    await page
      .getByRole('tablist', { name: 'Editor modes' })
      .getByRole('tab', { name: 'Presets' })
      .tap();
    const strip = page.getByRole('region', { name: 'Presets' });
    await expect(tile(strip, 'Warm')).toBeEnabled();
    await expect(strip.getByRole('tablist', { name: 'Preset groups' }).getByRole('tab')).toHaveText(
      ['Favourites', 'All', 'B&W', 'Film']
    );
    await expect(strip.getByRole('img', { name: 'Favourite' })).toHaveCount(1);

    await tapCentre(page, tile(strip, 'Warm'));
    await expect(tile(strip, 'Warm')).toHaveAttribute('aria-current', 'true');
    await expect(tile(strip, 'Warm').locator('[data-applied-badge]')).toBeVisible();
    await expect(pill(page)).toHaveText(
      'Warm applied · Amount 100% stays live until your next edit'
    );
    await strip.getByRole('slider', { name: 'Preset amount' }).fill('40');
    await expect(pill(page)).toHaveText(
      'Warm applied · Amount 40% stays live until your next edit'
    );
    await expect
      .poll(() => saves.map((body) => body.action))
      .toEqual(['Preset: Warm', 'Preset: Warm (40%)']);
  });
});
