import { expect, test, type Locator, type Page } from '@playwright/test';
import { ASSET_ID, gotoAsset, installMocks, json } from './helpers';

const PACKAGE_TINY_CONTROL_HEIGHT = 36;
const PACKAGE_SMALL_CONTROL_HEIGHT = 40;
const EDITOR_COMPACT_CONTROL_HEIGHT = 28;

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

async function expectSegmentedControl(
  page: Page,
  control: Locator,
  activeItem: Locator
): Promise<void> {
  expect((await metricsOf(control)).height).toBe(EDITOR_COMPACT_CONTROL_HEIGHT);
  const activeMetrics = await metricsOf(activeItem);
  expect(activeMetrics.height).toBe(24);
  expect(activeMetrics.background).toBe(await resolvedColor(page, '--color-primary'));
}

test('editor and loupe use the same black image canvas', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const canvas = await resolvedColor(page, '--color-image-canvas');
  expect((await metricsOf(page.getByRole('application'))).background).toBe(canvas);

  await page.goto('/search?q=IMG');
  await expect(page.locator(`a[href^="/assets/${ASSET_ID}?"]`)).toBeVisible();
  await page.getByLabel('Quick review').first().click();

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
  await expectPrimaryFocusRing(page, page.getByLabel('Filename suffix'));
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

test('sliders use the primary violet fill without replacing color gradients', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const fill = await resolvedColor(page, '--color-slider-fill');
  const primary = await resolvedColor(page, '--color-primary');
  expect(fill).toBe(primary);

  const exposure = page.getByRole('slider', { name: 'Exposure', exact: true });
  const exposureBackground = await exposure.evaluate(
    (element) => getComputedStyle(element).backgroundImage
  );
  expect(exposureBackground).toContain(fill);

  const temperature = page.getByRole('slider', { name: 'Temperature', exact: true });
  const temperatureBackground = await temperature.evaluate(
    (element) => getComputedStyle(element).backgroundImage
  );
  expect(temperatureBackground).not.toContain(fill);
  expect(temperatureBackground).toContain(await resolvedColor(page, '--color-temperature-cool'));
  expect(temperatureBackground).toContain(await resolvedColor(page, '--color-temperature-warm'));
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

test('the shell uses subtle panel separators', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  const library = page.getByRole('complementary', { name: 'Library' });
  expect(await library.evaluate((element) => getComputedStyle(element).borderRightWidth)).toBe(
    '0px'
  );
  const sidebarGap = await page.getByRole('link', { name: /^Photos/ }).evaluate((link) => {
    const linkBounds = link.getBoundingClientRect();
    const sidebarBounds = link.closest('aside')?.getBoundingClientRect();
    return sidebarBounds ? Math.round(sidebarBounds.right - linkBounds.right) : 0;
  });
  expect(sidebarGap).toBe(20);
  const sidebarOrder = await Promise.all(
    [
      page.getByRole('button', { name: /^People/ }),
      library.locator('.mx-4.my-3.h-px.bg-hairline'),
      page.getByRole('link', { name: /^Favorites/ }),
      page.getByRole('button', { name: /^Albums/ })
    ].map((element) => element.evaluate((node) => node.getBoundingClientRect().top))
  );
  expect(sidebarOrder).toEqual([...sidebarOrder].sort((a, b) => a - b));

  await gotoAsset(page);
  const controls = page.getByRole('complementary', { name: 'Editor controls' });
  expect(await controls.evaluate((element) => getComputedStyle(element).borderLeftWidth)).toBe(
    '1px'
  );
});

test('export is anchored at the bottom of the editor tool rail', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const rail = page.getByRole('tablist', { name: 'Editor tools' });
  const geometry = page.getByRole('tab', { name: 'Geometry' });
  const exportTab = page.getByRole('tab', { name: 'Export', exact: true });
  const positions = await Promise.all(
    [rail, geometry, exportTab].map((locator) =>
      locator.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        return { top: Math.round(bounds.top), bottom: Math.round(bounds.bottom) };
      })
    )
  );

  expect(positions[0].bottom - positions[2].bottom).toBeLessThanOrEqual(8);
  expect(positions[2].top - positions[1].bottom).toBeGreaterThan(100);

  await geometry.focus();
  await page.keyboard.press('ArrowDown');
  await expect(exportTab).toBeFocused();
  await expect(exportTab).toHaveAttribute('aria-selected', 'true');
});

test('the selected editor tool uses a muted primary tile', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const develop = page.getByRole('tab', { name: 'Develop' });
  const masks = page.getByRole('tab', { name: 'Masks' });
  const actionRow = page
    .getByRole('button', { name: 'Auto', exact: true })
    .locator('xpath=ancestor::div[.//button[@aria-label="Show modified only"]][1]');
  const styles = await Promise.all(
    [develop, masks].map((locator) =>
      locator.evaluate((element) => {
        const style = getComputedStyle(element);
        return {
          background: style.backgroundColor,
          color: style.color,
          width: element.getBoundingClientRect().width,
          height: element.getBoundingClientRect().height
        };
      })
    )
  );

  expect(styles[0].background).not.toBe(styles[1].background);
  expect(styles[0].color).not.toBe(styles[1].color);
  expect(styles[0].width).toBe(40);
  expect(styles[0].height).toBe(45);

  const positions = await Promise.all(
    [actionRow, develop].map((locator) =>
      locator.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        return { top: Math.round(bounds.top), height: Math.round(bounds.height) };
      })
    )
  );
  expect(positions[1]).toEqual(positions[0]);
});

test('upward tag picker uses neutral joined surfaces', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/tags', (route) =>
    route.fulfill(json([{ id: 'tag-1', value: 'Landscape' }]))
  );
  await gotoAsset(page);
  await page.getByRole('button', { name: 'Tags (T)' }).click();

  const search = page.getByPlaceholder('Search tags…');
  const input = containerOf(search);
  const content = page.locator('.editor-compact-combobox-content');
  const neutral = await resolvedColor(page, '--color-neutral-800');

  await expect(content).toBeVisible();
  expect((await metricsOf(input)).background).toBe(neutral);
  expect((await metricsOf(content)).background).toBe(neutral);

  const radii = await Promise.all(
    [input, content].map((element) =>
      element.evaluate((node) => {
        const style = getComputedStyle(node);
        return {
          topLeft: Number.parseFloat(style.borderTopLeftRadius),
          topRight: Number.parseFloat(style.borderTopRightRadius),
          bottomLeft: Number.parseFloat(style.borderBottomLeftRadius),
          bottomRight: Number.parseFloat(style.borderBottomRightRadius)
        };
      })
    )
  );
  expect(radii[0].topLeft).toBe(0);
  expect(radii[0].topRight).toBe(0);
  expect(radii[0].bottomLeft).toBeGreaterThan(0);
  expect(radii[0].bottomRight).toBeGreaterThan(0);
  expect(radii[1].topLeft).toBeGreaterThan(0);
  expect(radii[1].topRight).toBeGreaterThan(0);
  expect(radii[1].bottomLeft).toBe(0);
  expect(radii[1].bottomRight).toBe(0);
});

test('histogram has a slight inset without a frame', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const section = page.getByRole('button', { name: 'Histogram' }).locator('..');
  const histogram = page.getByText('No histogram data').locator('..');
  const geometry = await Promise.all(
    [section, histogram].map((element) =>
      element.evaluate((node) => {
        const bounds = node.getBoundingClientRect();
        const style = getComputedStyle(node);
        return {
          left: Math.round(bounds.left),
          right: Math.round(bounds.right),
          radius: Number.parseFloat(style.borderTopLeftRadius),
          shadow: style.boxShadow
        };
      })
    )
  );

  expect(geometry[1].left).toBe(geometry[0].left + 4);
  expect(geometry[1].right).toBe(geometry[0].right - 4);
  expect(geometry[1].radius).toBe(0);
  expect(geometry[1].shadow).toBe('none');
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

test('panel header actions match develop action buttons', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  const auto = page.getByRole('button', { name: 'Auto', exact: true });
  const reset = page.getByRole('button', { name: /Reset edits/ });
  const copy = page.getByRole('button', { name: 'Copy edits', exact: true });
  const paste = page.getByRole('button', { name: 'Paste edits', exact: true });
  const history = page.getByRole('button', { name: 'Edit history', exact: true });
  const filter = page.getByRole('button', { name: 'Show modified only', exact: true });
  const metrics = await Promise.all([auto, reset, copy, paste, history, filter].map(metricsOf));

  expect(metrics.map(({ height }) => height)).toEqual([32, 32, 32, 32, 32, 32]);
  expect(metrics[0].radius).toBeGreaterThan(0);
  expect(metrics[1].radius).toBeGreaterThan(0);
  expect(metrics.slice(2).map(({ radius }) => radius)).toEqual(Array(4).fill(metrics[2].radius));
  expect(metrics[2].radius).toBeGreaterThan(0);

  await page.getByRole('tab', { name: 'Masks' }).click();
  const maskOverlay = page.getByRole('button', { name: 'Toggle mask overlays' });
  const maskMetrics = await Promise.all(
    [maskOverlay, page.getByRole('button', { name: 'New mask' })].map(metricsOf)
  );

  expect(maskMetrics[0]).toEqual(metrics[5]);
  expect(maskMetrics[1]).toEqual(metrics[0]);

  const createMask = page.getByRole('button', { name: 'Create a mask' });
  await expect
    .soft(page.getByText('Masks limit an adjustment to part of the photo.'))
    .toHaveCount(0);
  expect
    .soft((await metricsOf(createMask)).background)
    .toBe(await resolvedColor(page, '--color-primary'));

  await createMask.click();
  await page.getByRole('button', { name: 'Radial gradient', exact: true }).click();
  await expect.soft(page.getByText('These sliders only affect the mask.')).toHaveCount(0);

  const labels = await page
    .getByRole('slider')
    .evaluateAll((sliders) => sliders.map((slider) => slider.getAttribute('aria-label')));
  expect.soft(labels.indexOf('Temperature')).toBeLessThan(labels.indexOf('Tint'));
  expect.soft(labels.indexOf('Tint')).toBeLessThan(labels.indexOf('Exposure'));
});

test('editor segmented controls share geometry and selection states', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);

  await page.getByRole('button', { name: 'Color Grading', exact: true }).click();
  const gradingTabs = page
    .getByRole('tablist')
    .filter({ has: page.getByRole('tab', { name: '3-Way', exact: true }) });
  const threeWay = gradingTabs.getByRole('tab', { name: '3-Way', exact: true });
  await expectSegmentedControl(page, gradingTabs, threeWay);
  await threeWay.focus();
  await threeWay.press('ArrowRight');
  await expect(gradingTabs.getByRole('tab', { name: 'Global', exact: true })).toBeFocused();
  await expect(threeWay).toHaveAttribute('aria-selected', 'true');

  await page.getByRole('button', { name: 'Camera Profile', exact: true }).click();
  await page.getByRole('button', { name: 'Profile options', exact: true }).click();
  const illuminant = page.getByRole('radiogroup', { name: 'Illuminant' });
  const automatic = illuminant.getByRole('radio', { name: 'Auto', exact: true });
  await expectSegmentedControl(page, illuminant, automatic);
  await automatic.focus();
  await automatic.press('ArrowRight');
  await expect(illuminant.getByRole('radio', { name: 'Illum 1' })).toBeChecked();

  await page.getByRole('tab', { name: 'Retouch', exact: true }).click();
  const retouch = page.getByRole('radiogroup', { name: 'Retouch tool' });
  const heal = retouch.getByRole('radio', { name: 'Heal', exact: true });
  await expectSegmentedControl(page, retouch, heal);
  await heal.focus();
  await heal.press('ArrowRight');
  await expect(retouch.getByRole('radio', { name: 'Clone', exact: true })).toBeChecked();

  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  const destination = page.getByRole('radiogroup', { name: 'Export destination' });
  const download = destination.getByRole('radio', { name: 'Download', exact: true });
  await expectSegmentedControl(page, destination, download);
  await download.focus();
  await download.press('ArrowRight');
  await expect(destination.getByRole('radio', { name: 'To Immich', exact: true })).toBeChecked();

  await page.getByRole('tab', { name: 'Masks', exact: true }).click();
  await page.getByRole('button', { name: 'New mask' }).click();
  await page.getByRole('button', { name: 'Brush', exact: true }).click();
  const paint = page.getByRole('button', { name: 'Paint', exact: true });
  await expect(paint.locator('..')).toHaveCSS('height', '20px');
});

test('dense panel fields and selects share package sizing and states', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Export', exact: true }).click();
  await page.getByRole('radio', { name: 'To Immich' }).click();

  const suffix = page.getByLabel('Filename suffix');
  await expect(suffix).toBeVisible();
  const neutral = await resolvedColor(page, '--color-neutral-800');
  const neutralHover = await resolvedColor(page, '--color-neutral-700');
  const suffixMetrics = await metricsOf(containerOf(suffix));
  expect(suffixMetrics.height).toBe(EDITOR_COMPACT_CONTROL_HEIGHT);
  expect(suffixMetrics.background).toBe(neutral);
  expect(
    (suffixMetrics.boxShadow.match(/-?\d+(?:\.\d+)?px/g) ?? []).every(
      (length) => Number.parseFloat(length) === 0
    )
  ).toBe(true);
  await containerOf(suffix).hover();
  await expect
    .poll(async () => (await metricsOf(containerOf(suffix))).background)
    .toBe(neutralHover);
  await expectPrimaryFocusRing(page, suffix);

  const panel = page.getByRole('tabpanel');
  const labelMetrics = await Promise.all(
    ['Format', 'Color space', 'Quality', 'Filename suffix', 'Albums', 'Tags'].map((label) =>
      metricsOf(panel.getByText(label, { exact: true }))
    )
  );
  const labelText = labelMetrics.map(({ color, fontSize, fontWeight }) => ({
    color,
    fontSize,
    fontWeight
  }));
  expect(labelText).toEqual(Array(6).fill(labelText[0]));
  expect(labelText[0].fontSize).toBe('11px');
  expect(labelText[0].fontWeight).toBe('500');

  for (const label of ['Add album…', 'Add tag…']) {
    const field = containerOf(page.getByLabel(label));
    const metrics = await metricsOf(field);
    expect(metrics.height).toBe(EDITOR_COMPACT_CONTROL_HEIGHT);
    expect(metrics.background).toBe(neutral);
    await field.hover();
    await expect.poll(async () => (await metricsOf(field)).background).toBe(neutralHover);
  }

  const trigger = page.getByRole('button', { name: 'Format' });
  await expect(trigger).toBeVisible();
  const triggerMetrics = await metricsOf(trigger.locator('div').first());
  expect(triggerMetrics.height).toBe(EDITOR_COMPACT_CONTROL_HEIGHT);
  expect(triggerMetrics.background).toBe(neutral);
  await trigger.hover();
  await expect
    .poll(async () => (await metricsOf(trigger.locator('div').first())).background)
    .toBe(neutralHover);

  const before = (await trigger.textContent())?.trim();
  await trigger.click();

  const menu = page.getByRole('listbox');
  await expect(menu).toBeVisible();
  expect((await metricsOf(menu)).background).toBe(await resolvedColor(page, '--color-neutral-800'));

  const option = page.locator('[data-select-item]').first();
  const optionMetrics = await metricsOf(option);
  expect(optionMetrics.height).toBe(PACKAGE_TINY_CONTROL_HEIGHT);
  expect(optionMetrics.fontSize).toBe('12px');

  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await expect(menu).toBeHidden();
  await expect(trigger).toBeFocused();
  expect((await trigger.textContent())?.trim()).not.toBe(before);
});

test('bulk export shows the neutral suffix field only for Immich', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Select', exact: true }).click();
  await page.getByRole('button', { name: 'Edit and export selected' }).click();
  await page.getByRole('tab', { name: 'Export' }).click();

  const dialog = page.getByRole('dialog', { name: 'Edit and export selected' });
  const suffix = dialog.getByLabel('Filename suffix');
  await expect(suffix).toBeHidden();

  await dialog.getByRole('radio', { name: 'To Immich' }).click();
  await expect(suffix).toBeVisible();
  const suffixMetrics = await metricsOf(containerOf(suffix));
  expect(suffixMetrics.background).toBe(await resolvedColor(page, '--color-neutral-800'));
  expect(
    (suffixMetrics.boxShadow.match(/-?\d+(?:\.\d+)?px/g) ?? []).every(
      (length) => Number.parseFloat(length) === 0
    )
  ).toBe(true);
  await expectPrimaryFocusRing(page, suffix);

  await dialog.getByRole('radio', { name: 'Download ZIP' }).click();
  await expect(suffix).toBeHidden();
});

test('crop ratio controls share the neutral grey surface', async ({ page }) => {
  await installMocks(page);
  await gotoAsset(page);
  await page.getByRole('tab', { name: 'Geometry' }).click();

  const aspect = page.getByRole('button', { name: 'Aspect Ratio' });
  const orientation = page.getByRole('button', { name: /^Switch to/ });
  const neutral = await resolvedColor(page, '--color-neutral-800');

  expect((await metricsOf(aspect.locator('div').first())).background).toBe(neutral);
  expect((await metricsOf(orientation)).background).toBe(neutral);

  const rotateLeft = page.getByRole('button', { name: 'Rotate left 90°' });
  const rotateMetrics = await metricsOf(rotateLeft);
  expect(rotateMetrics.height).toBe(EDITOR_COMPACT_CONTROL_HEIGHT);
  expect(rotateMetrics.background).toBe(await resolvedColor(page, '--color-ghost'));
  await rotateLeft.hover();
  await expect
    .poll(async () => (await metricsOf(rotateLeft)).background)
    .toBe(await resolvedBackground(page, 'bg-white/10'));
});

test('browse filter popover uses a neutral dark surface', async ({ page }) => {
  await installMocks(page);
  await page.goto('/photos');

  await page.getByRole('button', { name: 'Filters' }).click();
  const popover = page.locator('[data-popover-content]');
  await expect(popover).toBeVisible();
  expect((await metricsOf(popover)).background).toBe(
    await resolvedColor(page, '--color-neutral-900')
  );
  expect(
    await page
      .getByLabel('Visibility')
      .locator('div')
      .first()
      .evaluate((element) => getComputedStyle(element).backgroundColor)
  ).toBe(await resolvedColor(page, '--color-gray-800'));
});

test('tall mask picker stays inside the viewport', async ({ page }) => {
  await installMocks(page);
  await page.setViewportSize({ width: 1024, height: 420 });
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
  expect(geometry.bottom).toBeLessThanOrEqual(413);
  expect(geometry.scrollHeight).toBeGreaterThan(geometry.clientHeight);
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
