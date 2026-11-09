import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets, openFilters } from './helpers';

test('active filters stay visible as removable chips', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filterAt = (index: number) =>
    bodies.at(index)?.filter as { filename?: string; favorite?: boolean } | undefined;
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await openFilters(page, 'Library', 'File');
  await page.getByRole('button', { name: 'Favorites only' }).click();
  await page.getByRole('textbox', { name: 'Name contains' }).fill('portrait');
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

  const panel = await openFilters(page, 'Library', 'Albums, people & tags');
  await panel.getByRole('button', { name: 'Not in any album' }).click();
  await expect.poll(() => filterAt(-1)?.notInAlbum).toBe(true);
  await expect(page).toHaveURL(/[?&]noalbum=1/);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Not in any album']
  );

  await page.goto('/albums/album-1?noalbum=1');
  await expect.poll(() => filterAt(-1)?.scope).toEqual({ album: 'album-1' });
  expect(filterAt(-1)?.notInAlbum).toBeUndefined();
  await expect(page).not.toHaveURL(/noalbum/);
  await expect(panel.getByRole('button', { name: 'Favorites only' })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Not in any album' })).toHaveCount(0);
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

    const panel = await openFilters(page, 'Albums, people & tags');
    const untagged = panel.getByRole('button', { name: 'Untagged' });
    const noPeople = panel.getByRole('button', { name: 'No people' });
    await expect(panel.getByRole('button', { name: 'Not in any album' })).toBeVisible();
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
    await expect(noPeople).toHaveAttribute('aria-pressed', 'true');
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
    await expect(panel.getByRole('textbox', { name: 'Name contains' })).toBeVisible();
    const minSize = panel.getByLabel('Min size (MB)');
    if (!structuredSearch) {
      await expect(minSize).toHaveCount(0);
      await expect(panel.getByRole('group', { name: 'File size' })).toHaveCount(0);
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
    await expect(chips.getByRole('listitem')).toHaveText(['2–25 MB']);
    const sizes = panel.getByRole('group', { name: 'File size' });
    await expect(sizes.getByRole('button', { name: 'Custom' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );

    await chips.getByRole('button', { name: 'Remove 2–25 MB' }).click();
    await expect(minSize).toHaveCount(0);
    await expect.poll(() => filterAt(-1)?.minSize).toBeUndefined();
    expect(filterAt(-1)?.maxSize).toBeUndefined();

    const preset = sizes.getByRole('button', { name: '1–10 MB' });
    await preset.click();
    await expect
      .poll(() => filterAt(-1))
      .toMatchObject({ minSize: 1_000_000, maxSize: 10_000_000 });
    await expect(chips.getByRole('listitem')).toHaveText(['1–10 MB']);
    await expect(minSize).toHaveCount(0);
    await preset.click();
    await expect(chips.getByRole('listitem')).toHaveCount(0);
    await expect(preset).toHaveAttribute('aria-pressed', 'false');
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
  const filename = panel.getByRole('textbox', { name: 'Name contains' });
  await filename.fill('DSC');
  const threeStars = panel.getByRole('button', { name: '3 stars' });
  await threeStars.click();
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');
  await expect(chips).toHaveText(['≥ 3 stars', 'Name: DSC']);
  await expect(panel.getByRole('heading', { name: 'Filters' })).toContainText('2');
  await expect(
    panel.getByRole('button', { name: 'Rating & label', exact: true })
  ).toHaveAccessibleDescription('≥ 3 stars');
  await expect(
    panel.getByRole('button', { name: 'Library', exact: true })
  ).toHaveAccessibleDescription('');
  await expect(panel.getByRole('button', { name: /^Reset .* panel$/ })).toHaveCount(0);
  await threeStars.click();
  await expect(chips).toHaveText(['Name: DSC']);
  await expect(threeStars).toHaveAttribute('aria-pressed', 'false');
  await resetAll.click();
  await expect(chips).toHaveCount(0);
  await expect(filename).toHaveValue('');
  await expect(resetAll).toBeDisabled();

  await page.reload();
  await expect(filename).toBeVisible();
  await expect(panel.getByRole('group', { name: 'Rating', exact: true })).toBeVisible();
  await expect(panel.getByRole('radiogroup', { name: 'Show', exact: true })).toHaveCount(0);

  await page.getByRole('button', { name: 'Close filter panel' }).click();
  await expect(panel).toHaveCount(0);
  await expect(toggle).toBeFocused();
  await page.keyboard.press('Shift+F');
  await expect(panel).toBeVisible();
  await expect(toggle).toHaveCount(0);
  await page.keyboard.press('Shift+F');
  await expect(panel).toHaveCount(0);
});

test('the date chips pick a year, a month, the added basis and a custom range', async ({
  page
}) => {
  const assets = numberedAssets(2).map((asset, index) => ({
    ...asset,
    fileCreatedAt: index === 0 ? '2025-10-03T12:00:00Z' : '2025-03-03T12:00:00Z'
  }));
  await installMocks(page, { assets });
  await page.goto('/photos');
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');

  const panel = await openFilters(page, 'Date');
  const trigger = panel.getByRole('button', { name: 'Date', exact: true });
  const years = panel.getByRole('group', { name: 'Year' });
  await expect(years.getByRole('button')).toHaveText(['2025', 'Custom']);
  await expect(panel.getByRole('group', { name: 'Month' })).toHaveCount(0);
  await years.getByRole('button', { name: '2025' }).click();
  await expect(page).toHaveURL(/after=2025-01-01&before=2025-12-31/);
  await expect(chips).toHaveText(['2025']);

  const months = panel.getByRole('group', { name: 'Month' });
  await expect(months.getByRole('button', { name: 'Oct 2025' })).toBeEnabled();
  await expect(months.getByRole('button', { name: 'Apr 2025' })).toBeDisabled();
  await months.getByRole('button', { name: 'Oct 2025' }).click();
  await expect(page).toHaveURL(/after=2025-10-01&before=2025-10-31/);
  await expect(trigger).toHaveAccessibleDescription('Oct 2025');
  await months.getByRole('button', { name: 'Oct 2025' }).click();
  await expect(chips).toHaveText(['2025']);

  await panel.getByRole('radio', { name: 'Added' }).click();
  await expect(page).toHaveURL(/addedafter=2025-01-01&addedbefore=2025-12-31/);
  await expect(page).not.toHaveURL(/[?&]after=/);
  await expect(chips).toHaveText(['Added 2025']);
  await expect(months.getByRole('button', { name: 'Apr 2025' })).toBeEnabled();
  await years.getByRole('button', { name: '2025' }).click();
  await expect(chips).toHaveCount(0);

  await years.getByRole('button', { name: 'Custom' }).click();
  const from = panel.getByRole('group', { name: 'From' });
  await from.locator('[data-segment="month"]').click();
  await page.keyboard.type('10242025');
  await expect(chips).toHaveText(['Added from 24 Oct 2025']);
  await expect(years.getByRole('button', { name: 'Custom' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
});

test('resetting filters clears a typed date', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  const panel = await openFilters(page, 'Date');
  const custom = panel.getByRole('group', { name: 'Year' }).getByRole('button', { name: 'Custom' });
  const from = panel.getByRole('group', { name: 'From' });
  const chips = page.getByRole('list', { name: 'Active filters' }).getByRole('listitem');

  await custom.click();
  await from.locator('[data-segment="month"]').click();
  await page.keyboard.type('10012026');
  await expect(chips).toHaveText(['From 1 Oct 2026']);
  await expect(from.getByRole('button', { name: 'Open calendar' })).toBeFocused();

  await page.getByRole('button', { name: 'Reset all filters' }).click();
  await expect(chips).toHaveCount(0);
  await expect(from).toHaveCount(0);
  await custom.click();
  await expect(from.locator('[data-segment="month"]')).toHaveText('mm');
});

for (const structuredSearch of [false, true]) {
  test(`rating operator follows structured search (${structuredSearch})`, async ({ page }) => {
    const bodies: Array<Record<string, unknown>> = [];
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos');

    const panel = await openFilters(page, 'Rating & label');
    const rating = panel.getByRole('group', { name: 'Rating', exact: true });
    const operator = panel.getByRole('radiogroup', { name: 'Rating match' });
    await rating.getByRole('button', { name: '3 stars' }).click();
    if (!structuredSearch) {
      await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { eq: 3 } });
      await expect(operator).toHaveCount(0);
      return;
    }
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { gte: 3 } });
    await operator.getByRole('radio', { name: 'At most' }).click();
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { lte: 3 } });
    await rating.getByRole('button', { name: '2 stars' }).click();
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { lte: 2 } });
    await page.getByRole('button', { name: 'Close filter panel' }).click();
    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText(['≤ 2 stars']);
    await page.getByRole('button', { name: 'Filters (1 active)' }).click();
    await operator.getByRole('radio', { name: 'Exactly' }).click();
    await expect.poll(() => bodies.at(-1)?.filter).toMatchObject({ rating: { eq: 2 } });
    await rating.getByRole('button', { name: '2 stars' }).click();
    await expect
      .poll(() => (bodies.at(-1)?.filter as { rating?: unknown } | undefined)?.rating)
      .toBeUndefined();
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

  const panel = await openFilters(page, 'Library');
  await panel
    .getByRole('radiogroup', { name: 'Rejected', exact: true })
    .getByRole('radio', { name: 'Hide', exact: true })
    .click();

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

    const panel = await openFilters(page, 'Library');
    await panel
      .getByRole('radiogroup', { name: 'Rejected', exact: true })
      .getByRole('radio', { name: choice, exact: true })
      .click();

    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText([choice === 'Hide' ? 'No rejected' : 'Rejected only']);
    await expect.poll(() => rejectedAt(-1)).toBe(sent);
  });
}
