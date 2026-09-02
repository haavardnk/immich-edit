import { expect, test, type Locator, type Page } from '@playwright/test';

import { installMocks } from './helpers';

const PACKAGE_TINY_CONTROL_HEIGHT = 36;

type Metrics = {
  height: number;
  background: string;
  boxShadow: string;
  color: string;
  fontSize: string;
  fontWeight: string;
  radius: number;
};

async function metricsOf(locator: Locator): Promise<Metrics> {
  return locator.evaluate((el) => {
    const style = getComputedStyle(el);
    return {
      height: Math.round(el.getBoundingClientRect().height),
      background: style.backgroundColor,
      boxShadow: style.boxShadow,
      color: style.color,
      fontSize: style.fontSize,
      fontWeight: style.fontWeight,
      radius: Number.parseFloat(style.borderTopLeftRadius)
    };
  });
}

function containerOf(input: Locator): Locator {
  return input.locator('..');
}

async function resolvedColor(page: Page, variable: string): Promise<string> {
  return page.evaluate((name) => {
    const probe = document.createElement('span');
    probe.style.color = `var(${name})`;
    document.body.append(probe);
    const color = getComputedStyle(probe).color;
    probe.remove();
    return color;
  }, variable);
}

async function resolvedBackground(page: Page, className: string): Promise<string> {
  return page.evaluate((name) => {
    const probe = document.createElement('span');
    probe.className = name;
    document.body.append(probe);
    const background = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return background;
  }, className);
}

async function expectPrimaryFocusRing(page: Page, input: Locator): Promise<void> {
  await input.focus();
  const primary = await resolvedColor(page, '--color-primary');
  await expect
    .poll(async () => {
      const shadow = (await metricsOf(containerOf(input))).boxShadow;
      const lengths = shadow.match(/-?\d+(?:\.\d+)?px/g) ?? [];
      return shadow.includes(primary) && lengths.some((length) => Number.parseFloat(length) > 0);
    })
    .toBe(true);
}

async function expectNoRestingRing(input: Locator): Promise<void> {
  await input.blur();
  const ringColor = await containerOf(input).evaluate((element) => {
    const raw = getComputedStyle(element).getPropertyValue('--tw-ring-color').trim();
    const probe = document.createElement('span');
    probe.style.color = raw;
    document.body.append(probe);
    const color = getComputedStyle(probe).color;
    probe.remove();
    return color;
  });
  expect(ringColor).toBe('rgba(0, 0, 0, 0)');
}

function milliseconds(duration: string): number {
  const value = Number.parseFloat(duration);
  return duration.endsWith('ms') ? value : value * 1000;
}

async function installAuthMocks(page: Page): Promise<void> {
  await page.route('**/api/**', (route) => {
    const p = new URL(route.request().url()).pathname;
    if (p === '/api/setup/status')
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ configured: true })
      });
    if (p === '/api/auth/me')
      return route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'unauthorized', message: 'unauthorized' })
      });
    return route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });
  });
}

test('dark theme uses the immich-edit violet identity', async ({ page }) => {
  await installAuthMocks(page);
  await page.goto('/login');

  const palettes = await page.evaluate(() => {
    const style = getComputedStyle(document.documentElement);
    const steps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
    return {
      primary: steps.map((step) => style.getPropertyValue(`--immich-ui-primary-${step}`).trim()),
      gray: steps.map((step) => style.getPropertyValue(`--color-gray-${step}`).trim())
    };
  });
  expect(palettes.primary).toEqual([
    '#221533',
    '#2f1d46',
    '#3c2559',
    '#4d306f',
    '#7046a0',
    '#b7a1fb',
    '#c4b5fd',
    '#d0c5fd',
    '#ddd6fe',
    '#ede9fe',
    '#f5f3ff'
  ]);
  expect(palettes.gray).toEqual([
    '#f8f7f9',
    '#ebe8ee',
    '#d5d0d9',
    '#b7b0bd',
    '#93899b',
    '#6e6275',
    '#504459',
    '#392f41',
    '#29212f',
    '#1c171f',
    '#120f15'
  ]);

  const primary = await resolvedColor(page, '--color-primary');
  expect(primary).toBe(await resolvedColor(page, '--immich-ui-primary-500'));
  expect(await resolvedBackground(page, 'bg-primary')).toBe(primary);
  expect(
    await page
      .getByRole('button', { name: 'Sign in' })
      .evaluate((element) => getComputedStyle(element).backgroundColor)
  ).toBe(primary);

  const wordmark = page.getByRole('img', { name: 'immich-edit' });
  expect(
    await wordmark
      .locator('svg path')
      .first()
      .evaluate((element) => getComputedStyle(element).stroke)
  ).toBe(primary);
  expect(
    await wordmark
      .locator('.text-logo-primary')
      .evaluate((element) => getComputedStyle(element).color)
  ).toBe(primary);
});

test('the top bar search keeps its native pill shape', async ({ page }) => {
  await installMocks(page);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/photos');

  const search = page.getByRole('textbox', { name: 'Search your photos' });
  await expect(search).toBeVisible();

  const searchSurface = containerOf(search);
  const container = await metricsOf(searchSurface);
  await expectNoRestingRing(search);
  expect(container.height).toBe(46);
  expect(
    await searchSurface.evaluate((element) => Math.round(element.getBoundingClientRect().width))
  ).toBe(896);
  expect(container.background).toBe(await resolvedBackground(page, 'bg-white/5'));
  expect(container.radius).toBe(12);

  const iconInset = await page
    .getByRole('button', { name: 'Search', exact: true })
    .locator('svg')
    .evaluate((icon) => {
      const iconBounds = icon.getBoundingClientRect();
      const containerBounds = icon.closest('form')?.getBoundingClientRect();
      return containerBounds ? Math.round(iconBounds.left - containerBounds.left) : 0;
    });
  expect(iconInset).toBe(16);

  const navigation = page.getByRole('navigation', { name: 'Global navigation' });
  const bar = navigation;
  const logo = page.getByRole('img', { name: 'immich-edit' });
  const logoMetrics = await logo.evaluate((element) => {
    const mark = element.querySelector('svg');
    const style = getComputedStyle(element);
    return {
      height: Math.round(element.getBoundingClientRect().height),
      markHeight: Math.round(mark?.getBoundingClientRect().height ?? 0),
      fontSize: Number.parseFloat(style.fontSize),
      gap: Number.parseFloat(style.columnGap)
    };
  });
  expect((await metricsOf(bar)).height).toBe(64);
  expect(logoMetrics).toEqual({ height: 32, markHeight: 32, fontSize: 20, gap: 8 });
  const centers = await Promise.all(
    [bar, logo, search].map((locator) =>
      locator.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        return bounds.top + bounds.height / 2;
      })
    )
  );
  expect(Math.abs(centers[0] - centers[1])).toBeLessThanOrEqual(1);
  expect(Math.abs(centers[0] - centers[2])).toBeLessThanOrEqual(1);
});

test('the shortcuts filter uses a neutral grey surface', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Keyboard shortcuts' }).click();
  const filter = page.getByRole('textbox', { name: 'Filter shortcuts' });
  await expect(filter).toBeFocused();

  const filterSurface = containerOf(filter);
  expect((await metricsOf(filterSurface)).height).toBe(PACKAGE_TINY_CONTROL_HEIGHT);
  await expectPrimaryFocusRing(page, filter);
  await expectNoRestingRing(filter);
  const focusedBackground = await resolvedColor(page, '--color-neutral-800');
  await expect
    .poll(async () => (await metricsOf(filterSurface)).background)
    .toBe(focusedBackground);

  await filter.fill('retouch');
  await expect(page.getByText('Retouch', { exact: false }).first()).toBeVisible();
});

test('reduced motion disables generic motion', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await installMocks(page);
  await page.goto('/photos');

  const motion = await page.evaluate(() => {
    const probe = document.createElement('div');
    probe.style.animation = 'spin 1s linear infinite';
    probe.style.scrollBehavior = 'smooth';
    probe.style.transition = 'opacity 1s';
    document.body.append(probe);
    const style = getComputedStyle(probe);
    const result = {
      animationDuration: style.animationDuration,
      animationIterationCount: style.animationIterationCount,
      scrollBehavior: style.scrollBehavior,
      transitionDuration: style.transitionDuration
    };
    probe.remove();
    return result;
  });

  expect(milliseconds(motion.animationDuration)).toBeLessThanOrEqual(0.01);
  expect(motion.animationIterationCount).toBe('1');
  expect(motion.scrollBehavior).toBe('auto');
  expect(milliseconds(motion.transitionDuration)).toBeLessThanOrEqual(0.01);
});
