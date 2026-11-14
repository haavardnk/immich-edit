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

test('one camera pick drills into its models and lenses on flat search', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  await installMocks(page, {
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Camera');
  const makes = panel.getByRole('group', { name: 'Makes' });
  await expect(makes.getByRole('button')).toHaveText(['FUJIFILM', 'SONY']);
  await makes.getByRole('button', { name: 'SONY' }).click();

  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).make).toEqual(['SONY']);
  await expect(page.locator('[data-name="IMG_0002.ARW"]')).toHaveCount(0);
  const path = panel.getByRole('navigation', { name: 'Camera path' });
  await expect(path).toHaveText(/Makes.*SONY/);
  const models = panel.getByRole('group', { name: 'Models' });
  await expect(models.getByRole('button')).toHaveText(['ILCE-7M3', 'ILCE-7M4']);
  await models.getByRole('button', { name: 'ILCE-7M3' }).click();
  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).model).toEqual(['ILCE-7M3']);
  await expect(panel.getByRole('group', { name: 'Lenses' }).getByRole('button')).toHaveText([
    'FE 35mm F1.8'
  ]);
  await expect(
    panel.getByRole('button', { name: 'Camera', exact: true })
  ).toHaveAccessibleDescription('SONY · ILCE-7M3');

  await path.getByRole('button', { name: 'Makes' }).click();
  await makes.getByRole('button', { name: 'FUJIFILM' }).click();
  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).make).toEqual(['FUJIFILM']);
  await expect.poll(() => (bodies.at(-1)?.filter as ExifFilter).model).toBeUndefined();
});

test('several cameras match together and a removed make clears its models', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const filter = () => bodies.at(-1)?.filter as ExifFilter;
  await installMocks(page, {
    structuredSearch: true,
    assets: CAMERA_ASSETS,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Camera');
  const makes = panel.getByRole('group', { name: 'Makes' });
  await makes.getByRole('button', { name: 'SONY' }).click();
  await panel
    .getByRole('navigation', { name: 'Camera path' })
    .getByRole('button', { name: 'Makes' })
    .click();
  await makes.getByRole('button', { name: 'FUJIFILM' }).click();
  await expect.poll(() => filter().make).toEqual(['SONY', 'FUJIFILM']);
  await expect(makes.getByRole('button', { name: 'SONY' })).toHaveAttribute('aria-pressed', 'true');
  await expect(panel.getByRole('navigation', { name: 'Camera path' })).toHaveCount(0);

  await page.goto('/photos?make=SONY&make=FUJIFILM&model=ILCE-7M3');
  const models = panel.getByRole('group', { name: 'Models' });
  await expect(models.getByRole('button')).toHaveText(['ILCE-7M3', 'ILCE-7M4', 'X-T5']);
  await expect(models.getByRole('button', { name: 'ILCE-7M3' })).toHaveAttribute(
    'aria-pressed',
    'true'
  );
  await panel
    .getByRole('navigation', { name: 'Camera path' })
    .getByRole('button', { name: 'Makes' })
    .click();
  await makes.getByRole('button', { name: 'SONY' }).click();
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
  await page.goto('/photos?make=SONY&model=ILCE-7M3');

  const panel = await openFilters(page, 'Camera');
  const lenses = panel.getByRole('group', { name: 'Lenses' });
  await expect(lenses.getByRole('button')).toHaveText(['FE 35mm F1.8', 'Unknown']);
  await lenses.getByRole('button', { name: 'FE 35mm F1.8' }).click();
  await lenses.getByRole('button', { name: 'Unknown lens' }).click();
  await expect.poll(() => filter().lensModel).toEqual([null]);
  await expect(page).toHaveURL(/[?&]lens=(&|$)/);
  await expect(page.locator(`[data-name="${bare.originalFileName}"]`)).toHaveCount(1);
  await expect(page.locator('[data-name="IMG_0001.ARW"]')).toHaveCount(0);
  await lenses.getByRole('button', { name: 'FE 35mm F1.8' }).click();
  await expect.poll(() => filter().lensModel).toEqual(['FE 35mm F1.8']);
  await expect(page.getByRole('list', { name: 'Active filters' }).getByRole('listitem')).toHaveText(
    ['Make: SONY', 'Model: ILCE-7M3', 'Lens: FE 35mm F1.8']
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
    page.locator(`[data-name="${PAGED_ASSETS[index]!.originalFileName}"]`);
  await installMocks(page, {
    structuredSearch: true,
    assets: PAGED_ASSETS,
    people: PEOPLE,
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  const group = panel.getByRole('group', { name: 'People' });
  await expect(group.getByRole('button')).toHaveText(['Ann', 'Bo', 'No people']);
  await expect(panel.getByRole('radiogroup', { name: 'People match' })).toHaveCount(0);
  await group.getByRole('button', { name: 'Ann' }).click();
  await group.getByRole('button', { name: 'Bo' }).click();
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
  const group = panel.getByRole('group', { name: 'People' });
  await expect(group.getByRole('button')).toHaveText(['Bo']);
  await group.getByRole('button', { name: 'Bo' }).click();
  await expect.poll(people).toEqual({ all: ['person-bo'] });
  await expect(panel.getByRole('radiogroup', { name: 'People match' })).toHaveCount(0);
});

test('albums filter leaves out the page album', async ({ page }) => {
  const bodies: Array<Record<string, unknown>> = [];
  const albums = () => (bodies.at(-1)?.filter as { albums?: unknown }).albums;
  const tile = (index: number) =>
    page.locator(`[data-name="${PAGED_ASSETS[index]!.originalFileName}"]`);
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
  const group = panel.getByRole('group', { name: 'Albums' });
  await expect(group.getByRole('button')).toHaveText(['Bergen', 'Summer', 'Not in any album']);
  await group.getByRole('button', { name: 'Summer' }).click();
  await expect.poll(albums).toEqual({ all: ['album-1'] });
  await expect(page).toHaveURL(/album=album-1$/);
  await expect(tile(0)).toHaveCount(1);
  await expect(tile(1)).toHaveCount(0);

  await page.goto('/albums/album-1');
  await expect(group.getByRole('button')).toHaveText(['Bergen']);
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
  const tile = (index: number) => page.locator(`[data-name="${assets[index]!.originalFileName}"]`);
  await installMocks(page, {
    structuredSearch: true,
    assets,
    tags: [rome, oslo, reject],
    onMetadata: (body) => void bodies.push(body)
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  const group = panel.getByRole('group', { name: 'Tags' });
  await expect(group.getByRole('button')).toHaveText(['Trips/Oslo', 'Trips/Rome', 'Untagged']);
  await group.getByRole('button', { name: 'Trips/Rome' }).click();
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
  await expect(group.getByRole('button')).toHaveText(['Trips/Oslo']);
});

test('one search narrows albums, people and tags and keeps the exclusions', async ({ page }) => {
  await installMocks(page, {
    structuredSearch: true,
    assets: PAGED_ASSETS,
    people: PEOPLE,
    albums: [
      { id: 'album-1', albumName: 'Norway 2026', assetIds: [] },
      { id: 'album-2', albumName: 'Summer', assetIds: [] }
    ],
    tags: [
      { id: 'tag-nordic', name: 'Nordic', value: 'Light/Nordic' },
      { id: 'tag-rome', name: 'Rome', value: 'Trips/Rome' }
    ]
  });
  await page.goto('/photos');

  const panel = await openFilters(page, 'Albums, people & tags');
  const search = panel.getByRole('textbox', { name: 'Search albums, people and tags' });
  await search.fill('nor');
  await expect(panel.getByRole('group', { name: 'People' }).getByRole('button')).toHaveText([
    'No people'
  ]);
  await expect(panel.getByRole('group', { name: 'Albums' }).getByRole('button')).toHaveText([
    'Norway 2026',
    'Not in any album'
  ]);
  await expect(panel.getByRole('group', { name: 'Tags' }).getByRole('button')).toHaveText([
    'Light/Nordic',
    'Untagged'
  ]);
  await expect(panel.getByRole('status').filter({ hasText: 'albums' })).toHaveText('1 of 2 albums');
  await expect(panel.getByRole('status').filter({ hasText: 'people' })).toHaveText(
    'No matching people'
  );
  await expect(panel.locator('mark', { hasText: /^Nor$/ })).toHaveCount(2);

  await search.press('Escape');
  await expect(search).toHaveValue('');
  await expect(panel.getByRole('group', { name: 'People' }).getByRole('button')).toHaveText([
    'Ann',
    'Bo',
    'No people'
  ]);
});

test('a long camera list folds behind a more chip', async ({ page }) => {
  const assets = numberedAssets(12).map((asset, index) => ({
    ...asset,
    exifInfo: { make: `Make ${String(index + 1).padStart(2, '0')}` }
  }));
  await installMocks(page, { structuredSearch: true, assets });
  await page.goto('/photos?make=Make%2012&make=Make%2011');

  const panel = await openFilters(page, 'Camera');
  const makes = panel.getByRole('group', { name: 'Makes' });
  await expect(makes.getByRole('button')).toHaveText([
    ...Array.from({ length: 8 }, (_, index) => `Make 0${index + 1}`),
    'Make 11',
    'Make 12',
    '+2 more'
  ]);
  await makes.getByRole('button', { name: '+2 more' }).click();
  await expect(makes.getByRole('button')).toHaveCount(12);
});
