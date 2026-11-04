import { expect, test, type Locator, type Page } from '@playwright/test';
import { installMocks, json } from './helpers';

const MAC_SAFARI =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15';
const IPHONE =
  'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';

type ManifestIcon = { src: string; sizes: string; type: string; purpose: string };

type ManifestScreenshot = { src: string; type: string; form_factor: string };

type Manifest = {
  id: string;
  scope: string;
  start_url: string;
  name: string;
  short_name: string;
  display: string;
  theme_color: string;
  background_color: string;
  launch_handler: { client_mode: string };
  icons: ManifestIcon[];
  screenshots: ManifestScreenshot[];
};

type PromptProbe = { prompted: boolean };

async function openInstall(page: Page): Promise<void> {
  await installMocks(page);
  await page.goto('/settings#install');
  await expect(page.getByRole('button', { name: /^Layout/ })).toBeVisible();
}

function installButton(page: Page): Locator {
  return page.getByRole('button', { name: 'Install app', exact: true });
}

function installGroup(page: Page): Locator {
  return page.getByRole('button', { name: /^Install app Open immich-edit/ });
}

function pngSize(body: Buffer): string {
  return `${body.readUInt32BE(16)}x${body.readUInt32BE(20)}`;
}

test('links a web app manifest with installable fields and icons', async ({ page }) => {
  await installMocks(page);
  await page.goto('/login');

  const href = await page
    .locator('link[rel="manifest"]')
    .evaluate((link) => (link instanceof HTMLLinkElement ? link.href : ''));
  const response = await page.request.get(href);
  expect(response.ok()).toBe(true);
  const manifest = (await response.json()) as Manifest;

  expect(manifest).toMatchObject({
    id: '/',
    scope: '/',
    start_url: '/',
    name: 'immich-edit',
    short_name: 'immich-edit',
    display: 'standalone',
    theme_color: '#000000',
    background_color: '#000000',
    launch_handler: { client_mode: 'focus-existing' }
  });
  expect(manifest.icons.map((icon) => `${icon.sizes} ${icon.purpose}`)).toEqual([
    '192x192 any',
    '512x512 any',
    '512x512 maskable'
  ]);
  for (const icon of manifest.icons) {
    const image = await page.request.get(new URL(icon.src, href).href);
    expect(image.headers()['content-type']).toContain('image/png');
    expect(pngSize(await image.body())).toBe(icon.sizes);
  }
  expect(manifest.screenshots.map((shot) => shot.form_factor)).toEqual(['wide', 'narrow']);
  for (const shot of manifest.screenshots) {
    const image = await page.request.get(new URL(shot.src, href).href);
    expect(image.headers()['content-type']).toContain(shot.type);
  }

  const touchIcon = await page
    .locator('link[rel="apple-touch-icon"]')
    .evaluate((link) => (link instanceof HTMLLinkElement ? link.href : ''));
  expect(pngSize(await (await page.request.get(touchIcon)).body())).toBe('180x180');
  await expect(page.locator('meta[name="apple-mobile-web-app-capable"]')).toHaveAttribute(
    'content',
    'yes'
  );
  await expect(page.locator('meta[name="apple-mobile-web-app-status-bar-style"]')).toHaveAttribute(
    'content',
    'black-translucent'
  );
});

test.describe('install entry', () => {
  test('uses the browser prompt when the browser offers one', async ({ page }) => {
    await openInstall(page);

    const prevented = await page.evaluate(() => {
      const probe: PromptProbe = { prompted: false };
      Object.assign(window, { installProbe: probe });
      const event = new Event('beforeinstallprompt', { cancelable: true });
      Object.assign(event, {
        prompt: () => {
          probe.prompted = true;
          return Promise.resolve();
        },
        userChoice: Promise.resolve({ outcome: 'accepted' })
      });
      window.dispatchEvent(event);
      return event.defaultPrevented;
    });
    expect(prevented).toBe(true);

    await expect(
      page.getByText('Not installed. Your browser installs it in one step.')
    ).toBeVisible();
    await installButton(page).click();

    await expect(
      page.getByText('Installed. Open immich-edit from your Home Screen, Dock or app list.')
    ).toBeVisible();
    await expect(installButton(page)).toBeHidden();
    const prompted = await page.evaluate(
      () => (window as unknown as { installProbe: PromptProbe }).installProbe.prompted
    );
    expect(prompted).toBe(true);
  });

  test.describe('in Safari on a Mac', () => {
    test.use({ userAgent: MAC_SAFARI });

    test('opens the Add to Dock steps', async ({ page }) => {
      await openInstall(page);
      await installButton(page).click();

      const dialog = page.getByRole('dialog', { name: 'Add immich-edit to your Dock' });
      await expect(dialog).toBeVisible();
      await expect(dialog.getByText('File › Add to Dock')).toBeVisible();
      await expect(
        dialog.getByText('The installed app has its own sign-in.', { exact: false })
      ).toBeVisible();

      await dialog.getByRole('button', { name: 'close', exact: true }).click();
      await expect(dialog).toBeHidden();
    });
  });

  test('explains HTTPS when the page is not a secure context', async ({ page }) => {
    await page.addInitScript(() =>
      Object.defineProperty(window, 'isSecureContext', { value: false })
    );
    await openInstall(page);

    await expect(installGroup(page)).toBeVisible();
    await expect(page.getByText('Installing needs HTTPS.', { exact: false })).toBeVisible();
    await expect(page.getByRole('link', { name: 'How to set up HTTPS' })).toHaveAttribute(
      'href',
      /\/deploy\/#https-for-the-installed-app$/
    );
    await expect(installButton(page)).toBeHidden();
  });

  test('is hidden in a standalone window', async ({ page }) => {
    await page.addInitScript(() => {
      const original = window.matchMedia.bind(window);
      window.matchMedia = (query: string) =>
        original(query.includes('display-mode: standalone') ? '(min-width: 0px)' : query);
    });
    await openInstall(page);

    await expect(installGroup(page)).toHaveCount(0);
  });

  test('is hidden in a Home Screen app', async ({ page }) => {
    await page.addInitScript(() =>
      Object.defineProperty(Navigator.prototype, 'standalone', { get: () => true })
    );
    await openInstall(page);

    await expect(installGroup(page)).toHaveCount(0);
  });
});

async function expectHomeScreenSheet(page: Page): Promise<void> {
  await openInstall(page);
  await installButton(page).click();

  const sheet = page.getByRole('dialog', { name: 'Add immich-edit to your Home Screen' });
  await expect(sheet).toBeVisible();
  await expect(sheet.locator('[data-sheet-handle]')).toBeAttached();
  await expect(sheet.getByText('Add to Home Screen', { exact: true })).toBeVisible();
  await expect(sheet.getByText('Open as Web App', { exact: true })).toBeVisible();
}

test.describe('install sheet on a phone', { tag: ['@touch', '@phone'] }, () => {
  test.use({ userAgent: IPHONE });

  test('an iPhone gets the Add to Home Screen steps in a sheet', async ({ page }) => {
    await expectHomeScreenSheet(page);
  });
});

test.describe('install sheet on a tablet', { tag: ['@touch', '@tablet'] }, () => {
  test.use({ userAgent: MAC_SAFARI });

  test('an iPad asking for the desktop site gets the Home Screen steps', async ({ page }) => {
    await expectHomeScreenSheet(page);
  });
});

test.describe('install sheet in WebKit', { tag: '@touch-smoke' }, () => {
  test('Safari on an iPhone gets the Add to Home Screen steps', async ({ page }) => {
    await expectHomeScreenSheet(page);
  });
});

test.describe('service worker', () => {
  test.use({ serviceWorkers: 'allow' });

  async function waitForControl(page: Page): Promise<void> {
    await page.goto('/settings');
    const scope = await page.evaluate(async () => (await navigator.serviceWorker.ready).scope);
    expect(scope).toBe(new URL('/', page.url()).href);
    await page.reload();
    await expect(page.getByRole('button', { name: /^This device/ })).toBeVisible();
    expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
  }

  test('registers on the built app and leaves /api to the network', async ({ page }) => {
    await installMocks(page);
    let probes = 0;
    await page.route('**/api/pwa-probe', (route) => {
      probes++;
      return route.fulfill(json({ ok: true }));
    });

    await waitForControl(page);

    const status = await page.evaluate(async () => (await fetch('/api/pwa-probe')).status);
    expect(status).toBe(200);
    expect(probes).toBe(1);

    const cached = await page.evaluate(async () => {
      const names = await caches.keys();
      const paths: string[] = [];
      for (const name of names) {
        const requests = await (await caches.open(name)).keys();
        paths.push(...requests.map((request) => new URL(request.url).pathname));
      }
      return { names, paths };
    });
    expect(cached.names).toEqual([expect.stringMatching(/^immich-edit-/)]);
    expect(cached.paths).toContain('/200.html');
    expect(cached.paths).toContain('/manifest.webmanifest');
    expect(cached.paths.filter((path) => path.startsWith('/api'))).toEqual([]);
  });

  test('opens the app shell offline and shows the server as unreachable', async ({
    page,
    context
  }) => {
    await installMocks(page);
    await waitForControl(page);

    await page.unrouteAll({ behavior: 'ignoreErrors' });
    await context.setOffline(true);

    for (const path of ['/photos', '/assets/offline-deep-link']) {
      await page.goto(path);
      await expect(page.getByText("Can't reach the immich-edit server.")).toBeVisible();
      expect(new URL(page.url()).pathname).toBe(path);
    }
  });
});
