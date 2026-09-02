import { expect, test, type Page } from '@playwright/test';

import { installMocks } from './helpers';

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
