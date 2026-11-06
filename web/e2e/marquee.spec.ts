import { expect, test, type Page } from '@playwright/test';
import { installMocks } from './helpers';
import { datedAssets, dayShots, smallPages } from './timeline';

const ASSETS = datedAssets([
  ...dayShots('2026-10-31', 14, 'Bergen'),
  ...dayShots('2026-10-30', 12, 'Ålesund'),
  ...dayShots('2026-10-29', 12, 'Oslo')
]);

type Box = { x: number; y: number; width: number; height: number };
type Point = { x: number; y: number };

test.use({ timezoneId: 'UTC' });

async function openPhotos(page: Page): Promise<void> {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await installMocks(page, { assets: ASSETS });
  await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

function tile(page: Page, name: string) {
  return page.locator(`[data-name="${name}"]`);
}

async function boxOf(page: Page, name: string): Promise<Box> {
  const box = await tile(page, name).boundingBox();
  if (!box) throw new Error(`no tile ${name}`);
  return box;
}

async function gapStart(page: Page): Promise<Point> {
  const header = await page.getByTestId('day-header').first().boundingBox();
  if (!header) throw new Error('no day header');
  return { x: header.x + header.width * 0.55, y: header.y + header.height / 2 };
}

async function hitNames(page: Page, from: Point, to: Point): Promise<string[]> {
  return page.evaluate(
    ([a, b]) => {
      const left = Math.min(a.x, b.x);
      const right = Math.max(a.x, b.x);
      const top = Math.min(a.y, b.y);
      const bottom = Math.max(a.y, b.y);
      return [...document.querySelectorAll<HTMLElement>('[data-name]')]
        .filter((node) => {
          const box = node.getBoundingClientRect();
          return box.left <= right && box.right >= left && box.top <= bottom && box.bottom >= top;
        })
        .map((node) => node.getAttribute('data-name') ?? '')
        .sort();
    },
    [from, to] as const
  );
}

async function selectedNames(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[role="group"][data-selected]')]
      .map((node) => node.getAttribute('data-name') ?? '')
      .sort()
  );
}

async function pick(page: Page, name: string): Promise<void> {
  await tile(page, name).hover();
  await tile(page, name).getByRole('button', { name: 'Select', exact: true }).click();
}

function selectionBar(page: Page) {
  return page.getByRole('toolbar', { name: 'Selection actions' });
}

async function barCount(page: Page): Promise<number> {
  const text = await selectionBar(page)
    .getByText(/^\d+ selected$/)
    .textContent();
  const match = /^(\d+) selected$/.exec(text?.trim() ?? '');
  if (!match?.[1]) throw new Error(`no count in ${text ?? 'nothing'}`);
  return Number(match[1]);
}

async function marquee(page: Page, from: Point, to: Point): Promise<void> {
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 8 });
}

test('a drag from a gap box-selects the photos it touches and replaces the selection', async ({
  page
}) => {
  await openPhotos(page);
  await pick(page, 'IMG_0001.ARW');
  await expect(page.getByText('1 selected')).toBeVisible();

  const from = await gapStart(page);
  const corner = await boxOf(page, 'IMG_0004.ARW');
  const to = { x: corner.x + corner.width * 0.6, y: corner.y + corner.height * 1.4 };
  await marquee(page, from, to);

  await expect(page.getByTestId('marquee')).toBeVisible();
  const expected = await hitNames(page, from, to);
  expect(expected.length).toBeGreaterThan(1);
  expect(expected).not.toContain('IMG_0001.ARW');
  await expect.poll(() => selectedNames(page)).toEqual(expected);
  await expect(page.getByTestId('marquee-count')).toHaveText(String(expected.length));

  await page.mouse.up();
  await expect(page.getByTestId('marquee')).toHaveCount(0);
  await expect(page.getByText(`${expected.length} selected`)).toBeVisible();
  expect(await selectedNames(page)).toEqual(expected);
});

test('a box that touches no photo clears the selection', async ({ page }) => {
  await openPhotos(page);
  await pick(page, 'IMG_0001.ARW');
  await expect(selectionBar(page)).toBeVisible();

  const header = await page.getByTestId('day-header').first().boundingBox();
  if (!header) throw new Error('no day header');
  const y = header.y + header.height / 2;
  await marquee(
    page,
    { x: header.x + header.width * 0.3, y },
    { x: header.x + header.width * 0.5, y }
  );
  await expect(page.getByTestId('marquee-count')).toHaveText('0');
  await expect.poll(() => selectedNames(page)).toEqual([]);
  await page.mouse.up();
  await expect(selectionBar(page)).toHaveCount(0);
});

test('pressing a gap still takes focus away from the search field', async ({ page }) => {
  await openPhotos(page);
  const search = page.getByRole('textbox', { name: 'Search your photos' });
  await search.focus();
  await expect(search).toBeFocused();
  const from = await gapStart(page);
  await page.mouse.click(from.x, from.y);
  await expect(search).not.toBeFocused();
  await expect(page.getByTestId('marquee')).toHaveCount(0);
});

test('Shift adds the box to the selection and Escape puts the old one back', async ({ page }) => {
  await openPhotos(page);
  await pick(page, 'IMG_0001.ARW');

  const from = await gapStart(page);
  const corner = await boxOf(page, 'IMG_0004.ARW');
  const to = { x: corner.x + corner.width * 0.6, y: corner.y + corner.height * 1.4 };
  const boxed = await hitNames(page, from, to);

  await page.keyboard.down('Shift');
  await marquee(page, from, to);
  await expect.poll(() => selectedNames(page)).toEqual([...boxed, 'IMG_0001.ARW'].sort());
  await expect(page.getByTestId('marquee-count')).toHaveText(`+${boxed.length}`);
  await page.mouse.up();
  await page.keyboard.up('Shift');
  await expect(page.getByText(`${boxed.length + 1} selected`)).toBeVisible();

  await page.keyboard.press('Escape');
  await expect(page.getByText(/selected$/)).toHaveCount(0);
  await pick(page, 'IMG_0001.ARW');
  await marquee(page, from, to);
  await expect.poll(() => selectedNames(page)).toEqual(boxed);
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('marquee')).toHaveCount(0);
  expect(await selectedNames(page)).toEqual(['IMG_0001.ARW']);
  await page.mouse.up();
  expect(await selectedNames(page)).toEqual(['IMG_0001.ARW']);
});

test('a drag that starts on a photo never draws a box', async ({ page }) => {
  await openPhotos(page);
  const box = await boxOf(page, 'IMG_0002.ARW');
  await page.mouse.move(box.x + 10, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + 200, box.y + 200, { steps: 6 });
  await expect(page.getByTestId('marquee')).toHaveCount(0);
  await page.mouse.up();
  expect(await selectedNames(page)).toEqual([]);
});

test('holding the box at the bottom edge scrolls the grid and keeps selecting', async ({
  page
}) => {
  await openPhotos(page);
  const scroller = page.getByTestId('browse-scroller');
  const frame = await scroller.boundingBox();
  if (!frame) throw new Error('no scroller');
  const from = await gapStart(page);
  await marquee(page, from, { x: frame.x + frame.width - 40, y: frame.y + frame.height - 4 });

  await expect.poll(() => scroller.evaluate((node) => node.scrollTop)).toBeGreaterThan(300);
  const before = await barCount(page);
  await expect.poll(() => barCount(page)).toBeGreaterThan(before);
  await page.mouse.up();
  await expect(selectionBar(page).getByText(/^\d+ selected$/)).toBeVisible();
});

test('the bar says how much of the view is loaded until everything is', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.clock.setFixedTime(new Date('2026-11-05T12:00:00Z'));
  await installMocks(page, { assets: ASSETS });
  await smallPages(page, 12);
  await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
  await pick(page, 'IMG_0001.ARW');

  const bar = page.getByRole('toolbar', { name: 'Selection actions' });
  await expect(bar).toContainText(/\d+ of 38 loaded/);
  await bar.getByRole('button', { name: 'Select all' }).click();
  await expect(bar.getByText('38 selected')).toBeVisible();
  await expect(bar).not.toContainText('loaded');
});
