import { expect, type Locator, type Page } from '@playwright/test';
import { installMocks, numberedAssets, type InstallOpts, type MockAssetSummary } from './helpers';
import { open } from './touchShell';

const NOW = new Date('2026-11-05T12:00:00Z');

export type Shot = { at: string; city?: string | null };

export function datedAssets(shots: readonly Shot[]): MockAssetSummary[] {
  return numberedAssets(shots.length).map((asset, index) => ({
    ...asset,
    fileCreatedAt: shots[index]?.at ?? asset.fileCreatedAt,
    exifInfo: {
      exifImageWidth: index % 3 === 1 ? 4000 : 6000,
      exifImageHeight: index % 5 === 2 ? 6000 : 4000,
      city: shots[index]?.city ?? null
    }
  }));
}

export function dayShots(day: string, count: number, city: string | null = null): Shot[] {
  return Array.from({ length: count }, (_, index) => ({
    at: `${day}T${String(20 - index).padStart(2, '0')}:00:00Z`,
    city
  }));
}

export function monthlyShots(months: number, perMonth: number): Shot[] {
  const shots: Shot[] = [];
  for (let month = 0; month < months; month += 1) {
    const at = new Date(Date.UTC(2026, 9 - month, 1));
    const key = at.toISOString().slice(0, 7);
    for (let photo = 0; photo < perMonth; photo += 1) {
      shots.push({ at: `${key}-${String(20 - photo).padStart(2, '0')}T12:00:00Z` });
    }
  }
  return shots;
}

export type PageLog = { windows: Record<string, unknown>[]; metadata: Record<string, unknown>[] };

export async function smallPages(page: Page, size: number): Promise<PageLog> {
  const log: PageLog = { windows: [], metadata: [] };
  const cap = (value: unknown): number => Math.min(size, typeof value === 'number' ? value : size);
  await page.route('**/api/search/metadata', async (route) => {
    const body = (route.request().postDataJSON() as Record<string, unknown>) ?? {};
    log.metadata.push(body);
    if (body.size === 1) return route.fallback();
    return route.fallback({ postData: JSON.stringify({ ...body, size }) });
  });
  await page.route('**/api/search/window', async (route) => {
    const body = (route.request().postDataJSON() as Record<string, unknown>) ?? {};
    log.windows.push(body);
    return route.fallback({
      postData: JSON.stringify({ ...body, before: cap(body.before), after: cap(body.after) })
    });
  });
  return log;
}

export async function openTimeline(page: Page, opts: InstallOpts, height = 900): Promise<void> {
  await page.setViewportSize({ width: 1440, height });
  await page.clock.setFixedTime(NOW);
  await installMocks(page, opts);
  await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

export async function openTouchTimeline(
  page: Page,
  opts: InstallOpts,
  from?: string
): Promise<void> {
  await page.clock.setFixedTime(NOW);
  await open(page, from ?? '/photos', opts);
  if (from) await page.goto('/photos');
  await expect(page.getByTestId('day-header').first()).toBeVisible();
}

export function dayHeader(page: Page, title: string): Locator {
  return page.getByTestId('day-header').filter({ has: page.getByRole('heading', { name: title }) });
}

export async function scrollGridTo(page: Page, top: number): Promise<void> {
  await page.getByTestId('browse-scroller').evaluate((node, value) => {
    node.scrollTop = value;
  }, top);
}

export async function tileTopAt(
  page: Page,
  x: number,
  y: number
): Promise<{ name: string; top: number }> {
  const read = () =>
    page.evaluate(
      ([px, py]) => {
        const tile = document.elementFromPoint(px, py)?.closest('[role="group"]');
        if (!(tile instanceof HTMLElement)) return { name: '', top: Number.NaN };
        return {
          name: tile.getAttribute('data-name') ?? '',
          top: tile.getBoundingClientRect().top
        };
      },
      [x, y] as const
    );
  await expect.poll(async () => (await read()).name).not.toBe('');
  return read();
}

export async function tileTop(page: Page, name: string): Promise<number> {
  return page.evaluate((title) => {
    const tile = document.querySelector(`[data-name="${title}"]`);
    return tile ? tile.getBoundingClientRect().top : Number.NaN;
  }, name);
}
