import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets, openFilters } from './helpers';

test('active filters stay visible as removable chips', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filterAt = (index: number) =>
    bodies.at(index)?.filter as { filename?: string; favorite?: boolean } | undefined;
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await openFilters(page, 'Library', 'File');
  await page.getByRole('checkbox', { name: 'Favorites only' }).click();
  await page.getByLabel('Filename').fill('portrait');
  await page.getByRole('button', { name: 'Close filter panel' }).click();

  const chips = page.getByRole('list', { name: 'Active filters' });
  await expect(chips.getByRole('listitem')).toHaveText(['Favorites', 'Name: portrait']);
  await expect.poll(() => filterAt(-1)?.filename).toBe('portrait');

  await chips.getByRole('button', { name: 'Remove Favorites' }).click();

  await expect(chips.getByRole('listitem')).toHaveText(['Name: portrait']);
  await expect.poll(() => filterAt(-1)?.favorite).toBeUndefined();
  expect(filterAt(-1)?.filename).toBe('portrait');
});

test('the not-in-album filter is offered outside albums only', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filterAt = (index: number) =>
    bodies.at(index)?.filter as { notInAlbum?: boolean; scope?: unknown } | undefined;
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Library');
  await panel.getByRole('checkbox', { name: 'Not in any album' }).click();
  await expect.poll(() => filterAt(-1)?.notInAlbum).toBe(true);
  await expect(page).toHaveURL(/[?&]noalbum=1/);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Not in any album']
  );

  await page.goto('/albums/album-1?noalbum=1');
  await expect.poll(() => filterAt(-1)?.scope).toEqual({ album: 'album-1' });
  expect(filterAt(-1)?.notInAlbum).toBeUndefined();
  await expect(page).not.toHaveURL(/noalbum/);
  await expect(panel.getByRole('checkbox', { name: 'Favorites only' })).toBeVisible();
  await expect(panel.getByRole('checkbox', { name: 'Not in any album' })).toHaveCount(0);
});

for (const structuredSearch of [false, true]) {
  test(`no people needs structured search, untagged does not (${structuredSearch})`, async ({
    page
  }) => {
    const bodies: Array<Record<string, unknown>> = [];
    const filterAt = (index: number) =>
      bodies.at(index)?.filter as { untagged?: boolean; noPeople?: boolean } | undefined;
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos');

    const panel = await openFilters(page, 'Library');
    const untagged = panel.getByRole('checkbox', { name: 'Untagged' });
    const noPeople = panel.getByRole('checkbox', { name: 'No people' });
    await expect(panel.getByRole('checkbox', { name: 'Not in any album' })).toBeVisible();
    await untagged.click();
    await expect.poll(() => filterAt(-1)).toMatchObject({ untagged: true });
    if (!structuredSearch) {
      await expect(noPeople).toHaveCount(0);
      return;
    }
    await noPeople.click();
    await expect.poll(() => filterAt(-1)).toMatchObject({ untagged: true, noPeople: true });
    await expect(page).toHaveURL(/untagged=1&nopeople=1/);

    await page.goto('/tags/tag-1?untagged=1&nopeople=1');
    await expect(noPeople).toBeChecked();
    await expect(untagged).toHaveCount(0);
    await expect(page).not.toHaveURL(/untagged/);
    expect(filterAt(-1)?.untagged).toBeUndefined();
  });

  test(`the file size range needs structured search (${structuredSearch})`, async ({ page }) => {
    const bodies: Array<Record<string, unknown>> = [];
    const filterAt = (index: number) =>
      bodies.at(index)?.filter as { minSize?: number; maxSize?: number } | undefined;
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos?minsize=2');

    const panel = await openFilters(page, 'File');
    await expect(panel.getByLabel('Filename')).toBeVisible();
    const minSize = panel.getByLabel('Min size (MB)');
    if (!structuredSearch) {
      await expect(minSize).toHaveCount(0);
      await expect(page).not.toHaveURL(/minsize/);
      expect(filterAt(-1)?.minSize).toBeUndefined();
      return;
    }
    await expect(minSize).toHaveValue('2');
    await panel.getByLabel('Max size (MB)').fill('25');
    await expect
      .poll(() => filterAt(-1))
      .toMatchObject({ minSize: 2_000_000, maxSize: 25_000_000 });
    await expect(page).toHaveURL(/minsize=2&maxsize=25/);
    const chips = page.getByRole('list', { name: 'Active filters' });
    await expect(chips.getByRole('listitem')).toHaveText(['≥ 2 MB', '≤ 25 MB']);

    await chips.getByRole('button', { name: 'Remove ≥ 2 MB' }).click();
    await expect(minSize).toHaveValue('');
    await expect.poll(() => filterAt(-1)?.minSize).toBeUndefined();
  });
}

test('the filter panel remembers its sections and resets them', async ({ page }) => {
  await installMocks(page, { structuredSearch: true });
  await page.goto('/photos');
  const toggle = page.getByRole('button', { name: /^Filters/ });

  const panel = await openFilters(page);
  await expect(toggle).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Close filter panel' })).toBeFocused();
  await expect(panel.getByRole('button', { name: 'Library', exact: true })).toHaveAttribute(
    'aria-expanded',
    'false'
  );
  const resetAll = panel.getByRole('button', { name: 'Reset all filters' });
  await expect(resetAll).toBeDisabled();
  await panel.getByRole('button', { name: 'Rating & label', exact: true }).click();
  await panel.getByRole('button', { name: 'File', exact: true }).click();
  await panel.getByLabel('Filename').fill('DSC');
  await panel.getByRole('radio', { name: '3 stars' }).click();
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');
  await expect(chips).toHaveText(['≥ 3 stars', 'Name: DSC']);
  const reset = panel.getByRole('button', { name: 'Reset Rating & label panel' });
  await reset.click();
  await expect(chips).toHaveText(['Name: DSC']);
  await expect(reset).toHaveCount(0);
  await resetAll.click();
  await expect(chips).toHaveCount(0);
  await expect(panel.getByLabel('Filename')).toHaveValue('');
  await expect(resetAll).toBeDisabled();

  await page.reload();
  await expect(panel.getByLabel('Filename')).toBeVisible();
  await expect(panel.getByRole('radiogroup', { name: 'Rating', exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Visibility', exact: true })).toHaveCount(0);

  await page.getByRole('button', { name: 'Close filter panel' }).click();
  await expect(panel).toHaveCount(0);
  await expect(toggle).toBeFocused();
  await page.keyboard.press('Shift+F');
  await expect(panel).toBeVisible();
  await expect(toggle).toHaveCount(0);
  await page.keyboard.press('Shift+F');
  await expect(panel).toHaveCount(0);
});

test('the date calendars set the taken and added filters', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');
  const now = new Date();
  const day = new Date(now.getFullYear(), now.getMonth(), 15);
  const iso = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-15`;
  const dayLabel = new Intl.DateTimeFormat('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric'
  }).format(day);

  const panel = await openFilters(page, 'Date');
  await panel.getByRole('button', { name: 'Open calendar' }).first().click();
  await page.getByRole('button', { name: dayLabel, exact: true }).click();

  await expect(panel.getByRole('group', { name: 'Taken after' })).toBeVisible();
  await expect
    .poll(() => (bodies.at(-1)?.filter as { takenAfter?: string } | undefined)?.takenAfter)
    .toBe(new Date(`${iso}T00:00:00`).toISOString());
  await panel.getByRole('button', { name: 'Open calendar' }).nth(2).click();
  await page.getByRole('button', { name: dayLabel, exact: true }).click();
  await expect
    .poll(() => (bodies.at(-1)?.filter as { addedAfter?: string } | undefined)?.addedAfter)
    .toBe(new Date(`${iso}T00:00:00`).toISOString());
  await page.getByRole('button', { name: 'Close filter panel' }).click();
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    [`After ${iso}`, `Added after ${iso}`]
  );
});

test('resetting filters clears a typed date', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  const panel = await openFilters(page, 'Date');
  const added = panel.getByRole('group', { name: 'Added after' });
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');

  await added.locator('[data-segment="month"]').click();
  await page.keyboard.type('10012026');
  await expect(chips).toHaveText(['Added after 2026-10-01']);
  await expect(added.getByRole('button', { name: 'Open calendar' })).toBeFocused();

  await page.getByRole('button', { name: 'Reset all filters' }).click();
  await expect(chips).toHaveCount(0);
  await expect(added.locator('[data-segment="month"]')).toHaveText('mm');
});

for (const structuredSearch of [false, true]) {
  test(`rating operator follows structured search (${structuredSearch})`, async ({ page }) => {
    const bodies: Array<Record<string, unknown>> = [];
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos');

    const panel = await openFilters(page, 'Rating & label');
    const rating = panel.getByRole('radiogroup', { name: 'Rating', exact: true });
    const operator = page.getByRole('button', { name: /^Rating match/ });
    await rating.getByRole('radio', { name: '3 stars' }).click();
    if (!structuredSearch) {
      await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { eq: 3 } });
      await expect(operator).toHaveCount(0);
      return;
    }
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { gte: 3 } });
    await operator.click();
    await expect(operator).toHaveAccessibleName('Rating match: at most');
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { lte: 3 } });
    await rating.getByRole('radio', { name: '2 stars' }).click();
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { lte: 2 } });
    await page.getByRole('button', { name: 'Close filter panel' }).click();
    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText(['≤ 2 stars']);
    await page.getByRole('button', { name: 'Filters (active)' }).click();
    await operator.click();
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { eq: 2 } });
  });
}

test('the count names photos hidden by the reject filter', async ({ page }) => {
  const [kept, dropped] = numberedAssets(2);
  if (!kept || !dropped) throw new Error('missing assets');
  const rejectedAsset = {
    ...dropped,
    tags: [{ id: 'r', name: 'reject', value: 'immich-edit/reject' }]
  };
  await installMocks(page, { assets: [kept, rejectedAsset] });
  await page.goto('/photos');

  await openFilters(page, 'Library');
  await page.getByRole('button', { name: 'Rejected', exact: true }).click();
  await page.getByRole('option', { name: 'Hide', exact: true }).click();

  await expect(page.getByText('2 photos, 1 hidden', { exact: true })).toBeVisible();
});

for (const [structuredSearch, choice, sent] of [
  [false, 'Hide', undefined],
  [true, 'Hide', 'hide'],
  [false, 'Only', 'only']
] as const) {
  test(`rejected ${choice} with structured search ${structuredSearch} sends ${sent}`, async ({
    page
  }) => {
    const bodies: Array<Record<string, unknown>> = [];
    const rejectedAt = (index: number) =>
      (bodies.at(index)?.filter as { rejected?: string } | undefined)?.rejected;
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos');

    await openFilters(page, 'Library');
    await page.getByRole('button', { name: 'Rejected', exact: true }).click();
    await page.getByRole('option', { name: choice, exact: true }).click();

    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText([choice === 'Hide' ? 'No rejected' : 'Rejected only']);
    await expect.poll(() => rejectedAt(-1)).toBe(sent);
  });
}
