import { expect, test } from '@playwright/test';
import { installMocks, numberedAssets, openFilters } from './helpers';

const PAGED_ASSETS = numberedAssets(3);

const CAMERA_ASSETS = [
  ['SONY', 'ILCE-7M3', 'FE 35mm F1.8'],
  ['FUJIFILM', 'X-T5', 'XF35mmF1.4 R'],
  ['SONY', 'ILCE-7M4', 'FE 85mm F1.8']
].map(([make, model, lensModel], index) => ({
  ...PAGED_ASSETS[index],
  exifInfo: { make, model, lensModel }
}));

type ExifFilter = Partial<
  Record<'make' | 'model' | 'lensModel' | 'country' | 'state' | 'city', (string | null)[]>
>;

test('one camera pick narrows the lens list on flat search', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await openFilters(page, 'Camera');
  const make = page.getByRole('combobox', { name: 'Make' });
  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();

  await expect(make).toHaveValue('SONY');
  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).make).toEqual(['SONY']);
  await expect(page.locator('div[title="IMG_0002.ARW"]')).toHaveCount(0);
  await page.getByRole('combobox', { name: 'Lens' }).click();
  await expect(page.getByRole('option')).toHaveText(['FE 35mm F1.8', 'FE 85mm F1.8']);
});

test('several cameras match together and a removed make prunes its models', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  await installMocks(page, {
    structuredSearch: true,
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await openFilters(page, 'Camera');
  const make = page.getByRole('combobox', { name: 'Make' });
  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();
  await page.getByRole('option', { name: 'FUJIFILM' }).click();
  await expect.poll(() => filter().make).toEqual(['SONY', 'FUJIFILM']);
  await page.keyboard.press('Escape');
  await expect(make).toHaveAttribute('aria-expanded', 'false');

  await page.getByRole('combobox', { name: 'Model' }).click();
  await expect(page.getByRole('option')).toHaveText(['ILCE-7M3', 'ILCE-7M4', 'X-T5']);
  await page.getByRole('option', { name: 'ILCE-7M3' }).click();
  await expect.poll(() => filter().model).toEqual(['ILCE-7M3']);
  await page.keyboard.press('Escape');

  await make.click();
  await page.getByRole('option', { name: 'SONY' }).click();
  await expect.poll(() => filter().make).toEqual(['FUJIFILM']);
  await expect.poll(() => filter().model).toBeUndefined();
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Make: FUJIFILM']
  );
});

test('an unknown lens matches photos without one and excludes named lenses', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  const bare = { ...numberedAssets(4)[3], exifInfo: { make: 'SONY', model: 'ILCE-7M3' } };
  await installMocks(page, {
    structuredSearch: true,
    assets: [...CAMERA_ASSETS, bare],
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  await openFilters(page, 'Camera');
  const lens = page.getByRole('combobox', { name: 'Lens' });
  await lens.click();
  await expect(page.getByRole('option')).toHaveText([
    'FE 35mm F1.8',
    'FE 85mm F1.8',
    'XF35mmF1.4 R',
    'Unknown'
  ]);
  await page.getByRole('option', { name: 'FE 35mm F1.8' }).click();
  await page.getByRole('option', { name: 'Unknown' }).click();
  await expect.poll(() => filter().lensModel).toEqual([null]);
  await expect(page).toHaveURL(/[?&]lens=(&|$)/);
  await expect(page.locator(`div[title="${bare.originalFileName}"]`)).toHaveCount(1);
  await expect(page.locator('div[title="IMG_0001.ARW"]')).toHaveCount(0);
  await page.getByRole('option', { name: 'FE 85mm F1.8' }).click();
  await expect.poll(() => filter().lensModel).toEqual(['FE 85mm F1.8']);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Lens: FE 85mm F1.8']
  );
});

const PEOPLE = [
  { id: 'person-ann', name: 'Ann', assetIds: [PAGED_ASSETS[0]!.id, PAGED_ASSETS[1]!.id] },
  { id: 'person-bo', name: 'Bo', assetIds: [PAGED_ASSETS[1]!.id] },
  { id: 'person-cy', name: 'Cy', isHidden: true, assetIds: [PAGED_ASSETS[2]!.id] },
  { id: 'person-unnamed', name: '', assetIds: [] }
];

test('people match all, any or none of the picked people', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const people = () => (bodies.at(-1)?.filter as { people?: unknown }).people;
  const tile = (index: number) =>
    page.locator(`div[title="${PAGED_ASSETS[index]!.originalFileName}"]`);
  await installMocks(page, {
    structuredSearch: true,
    assets: PAGED_ASSETS,
    people: PEOPLE,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  const picker = panel.getByRole('combobox', { name: 'People' });
  await picker.click();
  await expect(page.getByRole('option')).toHaveText(['Ann', 'Bo']);
  await page.getByRole('option', { name: 'Ann' }).click();
  await page.getByRole('option', { name: 'Bo' }).click();
  await page.keyboard.press('Escape');
  await expect.poll(people).toEqual({ all: ['person-ann', 'person-bo'] });
  await expect(page).toHaveURL(/person=person-ann&person=person-bo$/);
  await expect(tile(1)).toHaveCount(1);
  await expect(tile(0)).toHaveCount(0);

  const match = panel.getByRole('radiogroup', { name: 'People match' });
  await match.getByRole('radio', { name: 'Any of' }).click();
  await expect.poll(people).toEqual({ any: ['person-ann', 'person-bo'] });
  await expect(tile(0)).toHaveCount(1);
  await match.getByRole('radio', { name: 'None of' }).click();
  await expect.poll(people).toEqual({ none: ['person-ann', 'person-bo'] });
  await expect(page).toHaveURL(/peoplematch=none/);
  await expect(tile(2)).toHaveCount(1);
  await expect(tile(1)).toHaveCount(0);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Without people: Ann +1']
  );

  await page.reload();
  await expect(
    page.getByRole('radiogroup', { name: 'People match' }).getByRole('radio', { name: 'None of' })
  ).toBeChecked();
});

test('flat search matches all of the picked people and leaves out the page person', async ({
  page
}) => {
  const bodies: Array<Record<string, unknown>> = [];
  const people = () => (bodies.at(-1)?.filter as { people?: unknown }).people;
  await installMocks(page, {
    assets: PAGED_ASSETS,
    people: PEOPLE,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/people/person-ann?person=person-bo&peoplematch=any');
  await expect.poll(() => bodies.length).toBeGreaterThan(0);
  await expect.poll(people).toBeUndefined();

  const panel = await openFilters(page, 'Albums, people & tags');
  await panel.getByRole('combobox', { name: 'People' }).click();
  await expect(page.getByRole('option')).toHaveText(['Bo']);
  await page.getByRole('option', { name: 'Bo' }).click();
  await page.keyboard.press('Escape');
  await expect.poll(people).toEqual({ all: ['person-bo'] });
  await expect(panel.getByRole('radiogroup', { name: 'People match' })).toHaveCount(0);
});

test('albums filter leaves out the page album', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const albums = () => (bodies.at(-1)?.filter as { albums?: unknown }).albums;
  const tile = (index: number) =>
    page.locator(`div[title="${PAGED_ASSETS[index]!.originalFileName}"]`);
  await installMocks(page, {
    structuredSearch: true,
    assets: PAGED_ASSETS,
    albums: [
      { id: 'album-1', albumName: 'Summer', assetIds: [PAGED_ASSETS[0]!.id] },
      { id: 'album-2', albumName: 'Bergen', assetIds: [PAGED_ASSETS[1]!.id] }
    ],
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  await panel.getByRole('combobox', { name: 'Albums' }).click();
  await expect(page.getByRole('option')).toHaveText(['Bergen', 'Summer']);
  await page.getByRole('option', { name: 'Summer' }).click();
  await page.keyboard.press('Escape');
  await expect.poll(albums).toEqual({ all: ['album-1'] });
  await expect(page).toHaveURL(/album=album-1$/);
  await expect(tile(0)).toHaveCount(1);
  await expect(tile(1)).toHaveCount(0);

  await page.goto('/albums/album-1');
  await panel.getByRole('combobox', { name: 'Albums' }).click();
  await expect(page.getByRole('option')).toHaveText(['Bergen']);
});

test('tags filter offers user tags and leaves out the page tag', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const tags = () => (bodies.at(-1)?.filter as { tags?: unknown }).tags;
  const rome = { id: 'tag-rome', name: 'Rome', value: 'Trips/Rome' };
  const oslo = { id: 'tag-oslo', name: 'Oslo', value: 'Trips/Oslo' };
  const reject = { id: 'tag-reject', name: 'reject', value: 'immich-edit/reject' };
  const assets = PAGED_ASSETS.slice(0, 3).map((asset, index) => ({
    ...asset,
    tags: [[rome], [oslo, reject], []][index]
  }));
  const tile = (index: number) => page.locator(`div[title="${assets[index]!.originalFileName}"]`);
  await installMocks(page, {
    structuredSearch: true,
    assets,
    tags: [rome, oslo, reject],
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  await panel.getByRole('combobox', { name: 'Tags' }).click();
  await expect(page.getByRole('option')).toHaveText(['Trips/Oslo', 'Trips/Rome']);
  await page.getByRole('option', { name: 'Trips/Rome' }).click();
  await page.keyboard.press('Escape');
  await expect.poll(tags).toEqual({ all: ['tag-rome'] });
  await expect(tile(0)).toHaveCount(1);
  await expect(tile(1)).toHaveCount(0);

  await panel
    .getByRole('radiogroup', { name: 'Tags match' })
    .getByRole('radio', { name: 'None of' })
    .click();
  await expect.poll(tags).toEqual({ none: ['tag-rome'] });
  await expect(page).toHaveURL(/tag=tag-rome&tagsmatch=none/);
  await expect(tile(0)).toHaveCount(0);
  await expect(tile(2)).toHaveCount(1);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Without tags: Trips/Rome']
  );

  await page.goto('/tags/tag-rome');
  await panel.getByRole('combobox', { name: 'Tags' }).click();
  await expect(page.getByRole('option')).toHaveText(['Trips/Oslo']);
});

test('a country narrows states and a state narrows cities', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  const assets = [
    ['Norway', 'Vestland', 'Bergen'],
    ['Norway', 'Oslo', 'Oslo'],
    ['Japan', 'Tokyo', 'Tokyo']
  ].map(([country, state, city], index) => ({
    ...PAGED_ASSETS[index],
    exifInfo: { country, state, city }
  }));
  await installMocks(page, { assets, onMetadata: (body) => void bodies.push(body) });
  await page.goto('/photos');

  await openFilters(page, 'Place');
  await page.getByRole('combobox', { name: 'Country' }).click();
  await page.getByRole('option', { name: 'Norway' }).click();
  await expect.poll(() => filter().country).toEqual(['Norway']);
  await expect(page.locator('div[title="IMG_0003.ARW"]')).toHaveCount(0);

  await page.getByRole('combobox', { name: 'State' }).click();
  await expect(page.getByRole('option')).toHaveText(['Oslo', 'Vestland']);
  await page.getByRole('option', { name: 'Vestland' }).click();
  await expect.poll(() => filter().state).toEqual(['Vestland']);

  await page.getByRole('combobox', { name: 'City' }).click();
  await expect(page.getByRole('option')).toHaveText(['Bergen']);
});

test('filter pickers join the list edge they open on', async ({ page }) => {
  await installMocks(page, { structuredSearch: true, assets: CAMERA_ASSETS });
  await page.setViewportSize({ width: 1280, height: 570 });
  await page.goto('/photos');
  await openFilters(page, 'Library', 'Camera');
  const sides: string[] = [];
  for (const name of ['Make', 'Lens']) {
    const picker = page.getByRole('combobox', { name });
    await picker.click();
    const side = (await page.locator('[data-combobox-content]').getAttribute('data-side')) ?? '';
    sides.push(side);
    const [joined, free] =
      side === 'top'
        ? ['border-top-left-radius', 'border-bottom-left-radius']
        : ['border-bottom-left-radius', 'border-top-left-radius'];
    const radius = (corner: string) =>
      picker.evaluate((el, key) => getComputedStyle(el).getPropertyValue(key), corner);
    await expect.poll(() => radius(joined)).toBe('0px');
    expect(await radius(free)).not.toBe('0px');
    await page.keyboard.press('Escape');
    await expect(picker).toHaveAttribute('aria-expanded', 'false');
  }
  expect(sides).toEqual(['bottom', 'top']);
});
