import { expect, test, type Page, type Route } from '@playwright/test';
import { installMocks } from './helpers';
import {
  dayHeader,
  datedAssets,
  monthlyShots,
  scrollGridTo,
  smallPages,
  tileTop,
  tileTopAt,
  type PageLog
} from './timeline';
import { touchDown, touchMoveTo, touchUp } from './touch';
import { open } from './touchShell';

const MONTHS = 30;
const ASSETS = datedAssets(monthlyShots(MONTHS, 10));
const JULY = 15;

test.use({ timezoneId: 'UTC' });

type Gate = { release: () => void; held: () => number };

async function holdEarlier(page: Page): Promise<Gate> {
  let open = false;
  const waiting: Array<() => void> = [];
  let held = 0;
  await page.route('**/api/search/window', async (route: Route) => {
    const body = route.request().postDataJSON() as { after?: number };
    if (body.after === 0 && !open) {
      held += 1;
      await new Promise<void>((resolve) => waiting.push(resolve));
    }
    await route.fallback();
  });
  return {
    release: () => {
      open = true;
      for (const resume of waiting.splice(0)) resume();
    },
    held: () => held
  };
}

async function railPoint(page: Page, month: number): Promise<{ x: number; y: number }> {
  const rail = await page.getByRole('slider', { name: 'Timeline' }).boundingBox();
  const track = await page.getByTestId('date-rail-track').boundingBox();
  if (!rail || !track) throw new Error('no rail');
  return { x: rail.x + rail.width / 2, y: track.y + ((month + 0.5) / MONTHS) * track.height };
}

async function openTimeline(page: Page): Promise<PageLog> {
  await page.setViewportSize({ width: 1440, height: 900 });
  await installMocks(page, { assets: ASSETS });
  const log = await smallPages(page, 30);
  await page.goto('/photos');
  await expect(page.getByRole('slider', { name: 'Timeline' })).toBeVisible();
  return log;
}

async function jumpToJuly(page: Page, log: PageLog): Promise<void> {
  const point = await railPoint(page, JULY);
  await page.mouse.move(point.x, point.y);
  await page.mouse.down();
  await page.mouse.up();
  await expect(dayHeader(page, 'Sunday 20 July 2025')).toBeVisible();
  expect(log.metadata.find((body) => body.size === 1)?.filter).toMatchObject({
    takenBefore: '2025-07-31T23:59:59.999Z'
  });
  expect(log.windows.at(-1)).toMatchObject({
    assetId: ASSETS[JULY * 10]?.id,
    before: 100,
    after: 400
  });
}

test('the rail shows years, a hover bubble and the visible range', async ({ page }) => {
  await openTimeline(page);
  const rail = page.getByRole('slider', { name: 'Timeline' });
  await expect(rail).toHaveAttribute('aria-valuetext', 'October 2026');
  await expect(rail).toContainText('2026');
  await expect(rail).toContainText('2025');
  await expect(page.getByTestId('date-rail-mark')).toBeVisible();

  const point = await railPoint(page, JULY);
  await page.mouse.move(point.x, point.y);
  const bubble = page.getByTestId('date-rail-bubble');
  await expect(bubble).toHaveText(/July 2025\s*10 photos/);
  const bubbleBox = await bubble.boundingBox();
  const railBox = await rail.boundingBox();
  expect((bubbleBox?.x ?? 0) + (bubbleBox?.width ?? 0)).toBeLessThan(railBox?.x ?? 0);

  await page.mouse.move(10, 400);
  await expect(bubble).toHaveCount(0);
});

test('clicking an unloaded month jumps there and scrolling up keeps the top photo still', async ({
  page
}) => {
  const log = await openTimeline(page);
  await jumpToJuly(page, log);
  const rail = page.getByRole('slider', { name: 'Timeline' });
  await expect(rail).toHaveAttribute('aria-valuetext', 'July 2025');
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toHaveCount(0);

  const gate = await holdEarlier(page);
  await scrollGridTo(page, 60);
  await expect.poll(gate.held).toBe(1);
  await expect(
    page.getByRole('status').filter({ hasText: 'Loading earlier photos…' })
  ).toBeVisible();
  const scroller = await page.getByTestId('browse-scroller').boundingBox();
  const probe = { x: (scroller?.x ?? 0) + 120, y: (scroller?.y ?? 0) + 200 };
  const before = await tileTopAt(page, probe.x, probe.y);

  gate.release();
  await expect
    .poll(() => page.getByTestId('browse-scroller').evaluate((node) => node.scrollTop))
    .toBeGreaterThan(1000);
  await expect.poll(() => tileTop(page, before.name)).toBeCloseTo(before.top, 0);
  expect(log.windows.at(-1)).toMatchObject({ assetId: ASSETS[JULY * 10 - 30]?.id, after: 0 });
});

test('a keyboard step on the rail moves one month', async ({ page }) => {
  await openTimeline(page);
  const rail = page.getByRole('slider', { name: 'Timeline' });
  await rail.focus();
  await page.keyboard.press('ArrowDown');
  await expect(rail).toHaveAttribute('aria-valuetext', 'September 2026');
  await expect(dayHeader(page, 'Sunday 20 September')).toBeVisible();
  await page.keyboard.press('PageDown');
  await expect(rail).toHaveAttribute('aria-valuetext', 'December 2025');
});

test('the loupe steps back across the near end of a window', async ({ page }) => {
  const log = await openTimeline(page);
  await jumpToJuly(page, log);
  const gate = await holdEarlier(page);
  await scrollGridTo(page, 0);
  await expect.poll(gate.held).toBe(1);

  const first = ASSETS[JULY * 10 - 30]?.originalFileName ?? '';
  const previous = ASSETS[JULY * 10 - 31]?.originalFileName ?? '';
  const tile = page.getByRole('group', { name: first });
  await tile.hover();
  await tile.getByRole('button', { name: 'Quick review' }).click();
  await expect(page.getByRole('img', { name: first })).toBeVisible();
  const rail = page.getByRole('navigation', { name: 'Photo actions' });
  await expect(rail).toBeVisible();
  await expect(rail.getByText(/^\d+ (of \d+|photos?)$/)).toHaveCount(0);

  await page.keyboard.press('ArrowLeft');
  await expect(page.getByRole('img', { name: first })).toBeVisible();
  gate.release();
  await expect(page.getByRole('img', { name: previous })).toBeVisible();
});

test('End and Home reach the true ends of a window and Select all stays offered', async ({
  page
}) => {
  const log = await openTimeline(page);
  await jumpToJuly(page, log);
  await page.getByRole('slider', { name: 'Timeline' }).blur();
  await page.keyboard.press('ArrowRight');
  const bar = page.getByRole('toolbar', { name: 'Selection actions' });
  await expect(bar).toContainText('1 selected');

  const last = ASSETS.at(-1)?.originalFileName ?? '';
  await page.keyboard.press('End');
  await expect(page.getByRole('group', { name: last })).toHaveAttribute('data-selected', 'true');
  expect(log.metadata.at(-1)).toMatchObject({ size: 1, sort: { direction: 'asc' } });
  await expect(page.getByRole('link', { name: 'IMG_0001.ARW' })).toHaveCount(0);
  await expect(bar.getByRole('button', { name: 'Select all' })).toBeVisible();

  await page.keyboard.press('Home');
  await expect(page.getByRole('group', { name: 'IMG_0001.ARW' })).toHaveAttribute(
    'data-selected',
    'true'
  );
  await expect(page.getByRole('slider', { name: 'Timeline' })).toHaveAttribute(
    'aria-valuetext',
    'October 2026'
  );
});

test('the editor round trip after a jump returns to the same photos without paging', async ({
  page
}) => {
  const log = await openTimeline(page);
  await jumpToJuly(page, log);
  await scrollGridTo(page, 1400);
  const inView = () =>
    page.getByTestId('browse-scroller').evaluate((el) => {
      const box = el.getBoundingClientRect();
      const tile = [...el.querySelectorAll<HTMLElement>('[role="group"]')].find((node) => {
        const top = node.getBoundingClientRect().top;
        return top >= box.top + 40 && top < box.bottom - 200;
      });
      return { name: tile?.title ?? '', top: tile?.getBoundingClientRect().top ?? 0 };
    });
  await expect.poll(async () => (await inView()).name).not.toBe('');
  const seen = await inView();
  await page.getByTestId('browse-scroller').evaluate((el, top) => {
    el.scrollTop += top - el.getBoundingClientRect().top;
  }, seen.top);
  await expect.poll(() => tileTop(page, seen.name)).toBeLessThan(seen.top);
  const before = { name: seen.name, top: await tileTop(page, seen.name) };
  const metadataBefore = log.metadata.length;

  await page
    .getByRole('link', { name: before.name })
    .evaluate((el) => (el as HTMLAnchorElement).click());
  await expect(page.getByRole('button', { name: /^Back/ })).toBeVisible();
  await page.getByRole('button', { name: /^Back/ }).click();
  await page.waitForURL('**/photos');

  await expect.poll(() => tileTop(page, before.name)).toBeCloseTo(before.top, 0);
  expect(log.metadata.slice(metadataBefore).filter((body) => body.size !== 1)).toEqual([]);
  expect(log.windows.at(-1)).toMatchObject({
    assetId: ASSETS.find((a) => a.originalFileName === before.name)?.id
  });
});

test.describe('touch scrubber', () => {
  test('@touch dragging the thumb shows the month and jumps on release', async ({ page }) => {
    await open(page, '/photos', { assets: ASSETS });
    const log = await smallPages(page, 30);
    await page.reload();
    const thumb = page.getByTestId('date-thumb');
    await expect(thumb).toHaveCSS('opacity', '0');

    await scrollGridTo(page, 300);
    await expect(thumb).toHaveCSS('opacity', '1');
    const box = await thumb.boundingBox();
    const track = await page.getByTestId('date-thumb-track').boundingBox();
    if (!box || !track) throw new Error('no thumb');
    const start = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    const target = { x: start.x, y: track.y + ((JULY + 0.5) / MONTHS) * track.height };

    await touchDown(page, start);
    for (let step = 1; step <= 8; step += 1) {
      await touchMoveTo(page, { x: start.x, y: start.y + ((target.y - start.y) * step) / 8 });
    }
    await expect(page.getByTestId('date-thumb-bubble')).toHaveText('July 2025');
    await expect(page.getByTestId('date-thumb-track')).toContainText('2025');
    await touchUp(page);

    await expect(page.getByTestId('date-thumb-bubble')).toHaveCount(0);
    await expect(dayHeader(page, 'Sunday 20 July 2025')).toBeVisible();
    expect(log.windows.at(-1)).toMatchObject({ assetId: ASSETS[JULY * 10]?.id });
  });

  test('@touch the thumb hides a moment after scrolling stops', async ({ page }) => {
    await page.clock.install();
    await open(page, '/photos', { assets: ASSETS });
    const thumb = page.getByTestId('date-thumb');
    await scrollGridTo(page, 300);
    await expect(thumb).toHaveCSS('opacity', '1');
    await page.clock.runFor(1000);
    await expect(thumb).toHaveCSS('opacity', '1');
    await page.clock.runFor(700);
    await expect(thumb).toHaveCSS('opacity', '0');
  });
});
