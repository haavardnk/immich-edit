import { expect, test, type Locator, type Page } from '@playwright/test';
import { unfoldExport } from './exportSections';
import {
  ASSET_ID,
  DEFAULT_APPEARANCE,
  gotoAsset,
  installMocks,
  numberedAssets,
  openAccountMenu,
  openLoupeFromGrid
} from './helpers';

const PACKAGE_TINY_CONTROL_HEIGHT = 36;
const PACKAGE_SMALL_CONTROL_HEIGHT = 40;

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

async function expectFieldSurface(
  page: Page,
  metrics: Metrics,
  height = PACKAGE_TINY_CONTROL_HEIGHT
): Promise<void> {
  expect(metrics.height).toBe(height);
  expect(metrics.background).toBe(await resolvedColor(page, '--color-gray-800'));
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

async function expectNoRestingRing(surface: Locator): Promise<void> {
  await expect
    .poll(async () => {
      const lengths = (await metricsOf(surface)).boxShadow.match(/-?\d+(?:\.\d+)?px/g) ?? [];
      return lengths.every((length) => Number.parseFloat(length) === 0);
    })
    .toBe(true);
}

test('editor and loupe use the same black image canvas', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const canvas = await resolvedColor(page, '--color-image-canvas');
  expect((await metricsOf(page.getByRole('application'))).background).toBe(canvas);

  await page.goto('/search?q=IMG');
  await expect(page.locator(`[role="group"] > a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await openLoupeFromGrid(page);

  const loupeCanvases = page.locator('.bg-image-canvas');
  await expect(loupeCanvases).toHaveCount(2);
  for (const loupeCanvas of await loupeCanvases.all()) {
    expect((await metricsOf(loupeCanvas)).background).toBe(canvas);
  }
});

test('text inputs retain the package focus ring', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  await expectPrimaryFocusRing(page, page.getByLabel('Search your photos'));

  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();
  await unfoldExport(page.getByRole('tabpanel'));
  await expectPrimaryFocusRing(page, page.getByLabel('Filename', { exact: true }));
});

test('text fields and selects rest without a ring', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  const search = page.getByRole('textbox', { name: 'Search settings', exact: true });
  await expectNoRestingRing(containerOf(search));
  await expectPrimaryFocusRing(page, search);

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await unfoldExport(page.getByRole('tabpanel'));
  await expectNoRestingRing(page.getByRole('tabpanel').locator('[data-select-trigger]').first());

  await page.getByRole('button', { name: 'Export presets' }).click();
  const presetName = page.getByRole('textbox', { name: 'Export preset name' });
  await expect(presetName).toBeFocused();
  await presetName.blur();
  await expectNoRestingRing(containerOf(presetName));
  await expectPrimaryFocusRing(page, presetName);
});

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

test('auth fields use the shared borderless input', async ({ page }) => {
  await installAuthMocks(page);
  await page.goto('/login');

  await expect(page.getByText('Continue to your editing workspace.')).toHaveCount(0);
  const email = page.getByLabel('Immich email', { exact: true });
  const emailSurface = containerOf(email);
  await expect(email).toBeVisible();
  await expectFieldSurface(page, await metricsOf(emailSurface), PACKAGE_SMALL_CONTROL_HEIGHT);
  expect((await metricsOf(email)).fontSize).toBe('14px');
  await expectNoRestingRing(emailSurface);

  await expectPrimaryFocusRing(page, email);
  const focusedBackground = await resolvedBackground(page, 'bg-primary/10');
  await expect.poll(async () => (await metricsOf(emailSurface)).background).toBe(focusedBackground);

  const password = page.getByRole('textbox', { name: 'Immich password', exact: true });
  await expectFieldSurface(
    page,
    await metricsOf(containerOf(password)),
    PACKAGE_SMALL_CONTROL_HEIGHT
  );

  await password.fill('hunter2');
  await expect(password).toHaveAttribute('type', 'password');
  await page.getByRole('button', { name: 'Show password' }).click();
  await expect(password).toHaveAttribute('type', 'text');
});

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

test('light theme keeps the violet identity on a light page', async ({ page }) => {
  await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode: 'light' } });
  await page.goto('/photos');
  await expect(page.locator('html')).toHaveClass(/\blight\b/);

  const palettes = await page.evaluate(() => {
    const style = getComputedStyle(document.documentElement);
    const steps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
    return {
      primary: steps.map((step) => style.getPropertyValue(`--immich-ui-primary-${step}`).trim()),
      ladder: steps.map((step) => style.getPropertyValue(`--immich-ui-light-${step}`).trim()),
      gray: steps.map((step) => style.getPropertyValue(`--color-gray-${step}`).trim())
    };
  });
  expect(palettes.primary[5]).toBe('#7159b0');
  expect(palettes.ladder).toEqual(palettes.gray);
  expect(palettes.gray[0]).toBe('#f8f7f9');
  expect(palettes.gray[10]).toBe('#120f15');

  const primary = await resolvedColor(page, '--color-primary');
  expect(await resolvedColor(page, '--immich-ui-light')).toBe('rgb(255, 255, 255)');
  expect(await resolvedBackground(page, 'bg-primary')).toBe(primary);
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

for (const mode of ['dark', 'light'] as const) {
  test(`${mode} chrome keeps recess washes and elevation in scheme`, async ({ page }) => {
    await installMocks(page, { appearance: { ...DEFAULT_APPEARANCE, mode } });
    await gotoAsset(page);

    const body = page
      .locator('.bg-recess')
      .filter({ has: page.getByRole('slider', { name: 'Exposure', exact: true }) });
    const filmstrip = page.getByTestId('filmstrip-scroll').locator('../..');
    await expect(filmstrip).toBeVisible();
    const recess = await resolvedBackground(page, 'bg-recess');
    const blackWash = await resolvedBackground(page, 'bg-black/10');
    expect((await metricsOf(body)).background).toBe(recess);
    const filmstripShadow = (await metricsOf(filmstrip)).boxShadow;

    await page.goto('/photos');
    await page.locator('button[aria-label^="Sort:"]').click();
    const popoverShadow = (await metricsOf(page.locator('[data-popover-content]'))).boxShadow;

    if (mode === 'dark') {
      expect(recess).toBe(blackWash);
      expect(filmstripShadow).toContain('color(srgb 0 0 0 / 0.18) 0px -8px 24px 0px');
      expect(popoverShadow).toContain('color(srgb 0 0 0 / 0.5) 0px 18px 48px 0px');
      expect(popoverShadow).toContain('0px 0px 0px 1px');
    } else {
      expect(recess).not.toBe(blackWash);
      expect(filmstripShadow).toBe('none');
      expect(popoverShadow).toContain('color(srgb 0 0 0 / 0.12) 0px 18px 48px 0px');
      expect(popoverShadow).not.toContain('0px 0px 0px 1px');
    }
  });
}

test('the scope mode dropdown is slim and only shows while expanded', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const mode = page.getByRole('combobox', { name: 'Scope', exact: true });
  await expect(mode).toHaveText('Histogram');
  expect((await metricsOf(mode.locator('div').first())).height).toBe(20);

  await page.getByRole('button', { name: 'Scopes', exact: true }).click();
  await expect(mode).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Pin scopes' })).toHaveCount(0);
});

test('modified editor tools use a dot without duplicate counts', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const develop = page.getByRole('tab', { name: 'Develop' });
  const heading = page.getByRole('heading', { name: 'Develop', exact: true });
  const indicator = develop.locator('[data-modified-indicator]');
  await expect(indicator).toBeVisible();
  await expect(indicator).toHaveText('');
  await expect(heading.locator('..')).toHaveText('Develop');

  const exposure = page
    .locator('div.group', {
      has: page.getByRole('button', { name: 'Exposure', exact: true })
    })
    .getByRole('slider');
  await exposure.fill('1');

  await expect(indicator).toBeVisible();
  await expect(indicator).toHaveText('');
  await expect(heading.locator('..')).toHaveText('Develop');
  expect(await indicator.evaluate((element) => element.getBoundingClientRect().width)).toBe(6);
});

test('hovering a panel modified dot offers a reset for the whole panel', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const reset = page.getByRole('button', { name: 'Reset Basic panel' });
  await expect(reset).toHaveCount(0);

  const slider = (label: string): Locator =>
    page
      .locator('div.group', { has: page.getByRole('button', { name: label, exact: true }) })
      .getByRole('slider');
  await slider('Exposure').fill('1');
  await slider('Shadows').fill('20');

  const dot = reset.locator('span');
  const icon = reset.locator('svg');
  await expect(dot).toBeVisible();
  await expect(icon).toBeHidden();
  await reset.hover();
  await expect(dot).toBeHidden();
  await expect(icon).toBeVisible();
  expect((await metricsOf(reset)).height).toBe(24);

  const saved = page.waitForRequest(
    (request) => request.url().endsWith('/edits') && request.method() === 'PUT'
  );
  await reset.click();
  const body = (await saved).postDataJSON() as {
    manifest: { ops: Record<string, unknown> };
    action: string | null;
  };
  expect(body.action).toBe('Reset Basic');
  expect(body.manifest.ops.exposure).toBeUndefined();
  expect(body.manifest.ops.tone_regions).toBeUndefined();
  await expect(reset).toHaveCount(0);
  await expect(slider('Exposure')).toHaveValue('0');
  await expect(slider('Exposure')).toBeVisible();
});

test('browse and loupe controls name their shortcuts', async ({ page }) => {
  await installMocks(page, { assets: numberedAssets(2) });
  await page.goto('/photos');
  await expect(page.locator(`[role="group"] > a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();

  await page.getByRole('button', { name: 'Thumbnail size XL' }).hover();
  await expect(page.getByText('Thumbnail size XL (− / +)')).toBeVisible();

  await page.getByRole('button', { name: 'Select', exact: true }).first().click();
  await page.getByRole('button', { name: /^Favorite \(/ }).hover();
  await expect(page.getByText('Favorite (P)')).toBeVisible();
  await page
    .getByLabel('Selection actions')
    .getByRole('radio', { name: '3 stars', exact: true })
    .hover();
  await expect(page.getByText('3 stars (0 – 5)')).toBeVisible();
  await page.getByRole('button', { name: /^Reject \(/ }).hover();
  await expect(page.getByText('Reject (X)')).toBeVisible();
  await page.getByRole('button', { name: 'Clear selection' }).click();

  await openLoupeFromGrid(page);
  await page.getByRole('button', { name: 'Next', exact: true }).hover();
  await expect(page.getByText('Next (← / →)')).toBeVisible();
  await page.getByRole('button', { name: 'More loupe actions' }).click();
  await expect(page.getByRole('button', { name: 'Clipping overlay' })).toContainText('J');
});

test('tall mask picker stays inside the viewport', async ({ page }) => {
  await installMocks(page);
  await page.setViewportSize({ width: 1024, height: 340 });
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Masks' }).click();
  await page.getByRole('button', { name: 'New mask' }).click();

  const popover = page.locator('[data-popover-content]');
  await expect(popover).toBeVisible();
  const geometry = await popover.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return {
      top: bounds.top,
      bottom: bounds.bottom,
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight
    };
  });
  expect(geometry.top).toBeGreaterThanOrEqual(8);
  expect(geometry.bottom).toBeLessThanOrEqual(333);
  expect(geometry.scrollHeight).toBeGreaterThan(geometry.clientHeight);
});

test('the shortcuts dialog scrolls in one region', async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await installMocks(page);
  await page.goto('/photos');

  await (await openAccountMenu(page)).getByRole('button', { name: 'Keyboard shortcuts' }).click();
  const dialog = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
  const list = dialog.getByRole('region', { name: 'Keyboard shortcuts' });
  await expect(list).toBeVisible();

  const scrollers = await dialog.evaluate((root) =>
    [root, ...root.querySelectorAll('*')]
      .filter((element) => element.scrollHeight > element.clientHeight + 1)
      .filter((element) => /auto|scroll/.test(getComputedStyle(element).overflowY))
      .map((element) => element.getAttribute('aria-label'))
  );
  expect(scrollers).toEqual(['Keyboard shortcuts']);

  const dialogBox = await dialog.boundingBox();
  const listBox = await list.boundingBox();
  expect(listBox!.y + listBox!.height).toBeLessThanOrEqual(dialogBox!.y + dialogBox!.height);
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
