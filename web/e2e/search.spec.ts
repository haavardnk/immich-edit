import { expect, test } from '@playwright/test';
import { ASSET_ID, installMocks, numberedAssets, openFilters } from './helpers';

const TILE = `a[href^="/assets/${ASSET_ID}?"]`;

function filename(body: Record<string, unknown> | null): unknown {
  return (body?.filter as { filename?: string } | undefined)?.filename;
}

test('global search submits through its compound control', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  const search = page.getByRole('textbox', { name: 'Search your photos' });
  await search.fill('DSC00195');
  await search.press('Enter');

  await page.waitForURL(/\/search\?q=DSC00195/);
  await expect(page.locator(TILE).first()).toBeVisible();

  await page.goBack();
  await page.waitForURL(/\/photos$/);
  await expect(search).toHaveValue('');
});

test('filename-like query uses metadata search', async ({ page }) => {
  let smartHit = false;
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    onSmart: () => {
      smartHit = true;
    },
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=DSC00195');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(smartHit).toBe(false);
  expect(filename(metadataBody)).toBe('DSC00195');
});

test('plain query uses context search', async ({ page }) => {
  let smartBody: Record<string, unknown> | null = null;
  let metadataHit = false;
  await installMocks(page, {
    onSmart: (body) => {
      smartBody = body;
    },
    onMetadata: () => {
      metadataHit = true;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(smartBody?.['text']).toBe('red car');
  expect(metadataHit).toBe(false);
});

test('toggling to filename reruns as metadata search', async ({ page }) => {
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  await page.getByRole('button', { name: 'File name', exact: true }).click();
  await page.waitForURL(/mode=filename/);

  await expect.poll(() => filename(metadataBody)).toBe('red car');
});

test('editing the query keeps the picked mode and filters', async ({ page }) => {
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red&mode=filename&favorite=1');
  await expect.poll(() => filename(metadataBody)).toBe('red');

  await page.getByRole('textbox', { name: 'Search your photos' }).fill('red car');

  await page.waitForURL(/q=red\+car/);
  await expect(page).toHaveURL(/mode=filename/);
  await expect(page).toHaveURL(/favorite=1/);
  await expect(page.getByRole('button', { name: 'File name', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect.poll(() => filename(metadataBody)).toBe('red car');
});

test('refetching keeps the search header mounted', async ({ page }) => {
  const releases: Array<() => void> = [];
  await installMocks(page, {
    onMetadata: async (body) => {
      if (filename(body) !== 'red car') return;
      await new Promise<void>((resolve) => void releases.push(resolve));
    }
  });

  await page.goto('/search?q=red&mode=filename');
  await expect(page.locator(TILE).first()).toBeVisible();
  const modeButton = await page
    .getByRole('button', { name: 'File name', exact: true })
    .elementHandle();

  await page.getByRole('textbox', { name: 'Search your photos' }).fill('red car');

  await expect.poll(() => releases.length).toBe(1);
  await expect(page.getByText('Search: red car')).toBeVisible();
  await expect(page.getByText('Searching…')).toBeVisible();
  expect(await modeButton?.evaluate((el) => el.isConnected)).toBe(true);
  releases[0]?.();
  await expect(page.locator(TILE).first()).toBeVisible();
});

for (const [mode, label] of [
  ['description', 'Description'],
  ['path', 'Full path or folder'],
  ['ocr', 'OCR']
]) {
  test(`${label} mode searches the ${mode} field`, async ({ page }) => {
    const filters: Array<Record<string, unknown>> = [];
    await installMocks(page, {
      onMetadata: (body) => void filters.push(body.filter as Record<string, unknown>)
    });

    await page.goto('/search?q=red%20car');
    await page.getByRole('button', { name: label, exact: true }).click();

    await page.waitForURL(new RegExp(`mode=${mode}`));
    await expect(page.getByRole('button', { name: label, exact: true })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
    await expect.poll(() => filters.at(-1)?.[mode]).toBe('red car');
    expect(filters.at(-1)?.filename).toBeUndefined();
  });
}

test('modes the Immich server lacks are hidden', async ({ page }) => {
  let smartHit = false;
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    smartSearch: false,
    ocr: false,
    onSmart: () => {
      smartHit = true;
    },
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red%20car&mode=ocr');
  await expect(page.locator(TILE).first()).toBeVisible();

  await expect(page.getByRole('button', { name: 'File name', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await expect(page.getByRole('button', { name: 'Context', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'OCR', exact: true })).toHaveCount(0);
  expect(filename(metadataBody)).toBe('red car');
  expect(smartHit).toBe(false);
});

test('smart search hides rejects in the browser and matches exact ratings', async ({ page }) => {
  const [kept, rejected] = numberedAssets(2);
  if (!kept || !rejected) throw new Error('missing assets');
  const filters: Array<Record<string, unknown> | undefined> = [];
  await installMocks(page, {
    structuredSearch: true,
    onSmart: (body) => void filters.push(body.filter as Record<string, unknown> | undefined),
    assets: [
      kept,
      { ...rejected, tags: [{ id: 'r', name: 'reject', value: 'immich-edit/reject' }] }
    ]
  });
  const tile = (id: string) => page.locator(`a[href^="/assets/${id}?"]`);

  await page.goto('/search?q=red%20car&rating=4%2B');
  await expect(tile(rejected.id)).toBeVisible();
  await expect(page).not.toHaveURL(/rating/);
  expect(filters[0]?.rating).toBeUndefined();
  const panel = await openFilters(page, 'Library', 'Rating & label');
  await expect(panel.getByRole('button', { name: 'File', exact: true })).toHaveCount(0);
  await expect(panel.getByRole('button', { name: /^Rating match/ })).toHaveCount(0);
  await page.getByRole('button', { name: 'Rejected', exact: true }).click();
  await page.getByRole('option', { name: 'Hide', exact: true }).click();
  const rating = panel.getByRole('radiogroup', { name: 'Rating', exact: true });
  await rating.getByRole('radio', { name: '3 stars' }).click();

  await expect(page).toHaveURL(/rating=3(&|$)/);
  await expect.poll(() => filters.at(-1)?.rating).toEqual({ eq: 3 });
  await expect(tile(rejected.id)).toBeHidden();
  await expect(tile(kept.id)).toBeVisible();
});

test('smart failure falls back to filename search', async ({ page }) => {
  let metadataBody: Record<string, unknown> | null = null;
  await installMocks(page, {
    smartFails: true,
    onMetadata: (body) => {
      metadataBody = body;
    }
  });

  await page.goto('/search?q=red%20car');
  await expect(page.locator(TILE).first()).toBeVisible();

  expect(filename(metadataBody)).toBe('red car');

  const toast = page.getByText('Smart search unavailable, showing filename matches');
  await expect(toast).toBeVisible();
  await page.getByRole('button', { name: 'Close' }).click();
  await expect(toast).toBeHidden();
});
