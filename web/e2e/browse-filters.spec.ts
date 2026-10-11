import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets } from './helpers';

test('active filters stay visible as removable chips', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filterAt = (index: number) =>
    bodies.at(index)?.filter as { filename?: string; favorite?: boolean } | undefined;
  await installMocks(page, { onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('checkbox', { name: 'Favorites only' }).click();
  await page.getByLabel('Filename').fill('portrait');
  await page.getByRole('button', { name: 'Close filters' }).click();

  const chips = page.getByRole('list', { name: 'Active filters' });
  await expect(chips.getByRole('listitem')).toHaveText(['Favorites', 'Name: portrait']);
  await expect.poll(() => filterAt(-1)?.filename).toBe('portrait');

  await chips.getByRole('button', { name: 'Remove Favorites' }).click();

  await expect(chips.getByRole('listitem')).toHaveText(['Name: portrait']);
  await expect.poll(() => filterAt(-1)?.favorite).toBeUndefined();
  expect(filterAt(-1)?.filename).toBe('portrait');
});

test('the taken-after calendar sets the date filter', async ({ page }) => {
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

  await page.getByRole('button', { name: 'Filters' }).click();
  await page.getByRole('button', { name: 'Open calendar' }).first().click();
  await page.getByRole('button', { name: dayLabel, exact: true }).click();

  await expect(page.locator('[data-popover-content]')).toBeVisible();
  await expect
    .poll(() => (bodies.at(-1)?.filter as { takenAfter?: string } | undefined)?.takenAfter)
    .toBe(new Date(iso).toISOString());
  await page.getByRole('button', { name: 'Close filters' }).click();
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    [`After ${iso}`]
  );
});

for (const structuredSearch of [false, true]) {
  test(`rating operator follows structured search (${structuredSearch})`, async ({ page }) => {
    const bodies: Array<Record<string, unknown>> = [];
    await installMocks(page, { structuredSearch, onMetadata: (body) => void bodies.push(body) });
    await page.goto('/photos');

    await page.getByRole('button', { name: 'Filters' }).click();
    const rating = page
      .locator('[data-popover-content]')
      .getByRole('radiogroup', { name: 'Rating', exact: true });
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
    await page.getByRole('button', { name: 'Close filters' }).click();
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

  await page.getByRole('button', { name: 'Filters' }).click();
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

    await page.getByRole('button', { name: 'Filters' }).click();
    await page.getByRole('button', { name: 'Rejected', exact: true }).click();
    await page.getByRole('option', { name: choice, exact: true }).click();

    await expect(
      page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')
    ).toHaveText([choice === 'Hide' ? 'No rejected' : 'Rejected only']);
    await expect.poll(() => rejectedAt(-1)).toBe(sent);
  });
}
