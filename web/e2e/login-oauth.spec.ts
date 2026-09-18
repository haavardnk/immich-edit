import { expect, test, type Page } from '@playwright/test';

const IDP_URL = 'http://idp.local/authorize?client_id=immich';

interface ProviderOverrides {
  oauth?: boolean;
  password_login?: boolean;
  auto_launch?: boolean;
  button_text?: string;
}

async function stubApi(page: Page, overrides: ProviderOverrides = {}): Promise<string[]> {
  const started: string[] = [];
  await page.route('http://idp.local/**', (route) =>
    route.fulfill({ status: 200, contentType: 'text/html', body: '<h1>identity provider</h1>' })
  );
  await page.route('**/api/**', async (route) => {
    const url = new URL(route.request().url());
    const body = (value: unknown, status = 200) =>
      route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(value) });
    switch (url.pathname) {
      case '/api/setup/status':
        return body({ configured: true });
      case '/api/auth/me':
        return body({ code: 'unauthorized', message: 'unauthorized' }, 401);
      case '/api/auth/providers':
        return body({
          oauth: true,
          password_login: true,
          auto_launch: false,
          button_text: 'Sign in with Keycloak',
          degraded: false,
          ...overrides
        });
      case '/api/auth/oauth/start':
        started.push(route.request().postData() ?? '');
        return body({ url: IDP_URL });
      case '/api/auth/oauth/callback':
        return body({ code: 'unauthorized', message: 'invalid code' }, 401);
      default:
        return body({});
    }
  });
  return started;
}

test('the login page offers the provider button from the server', async ({ page }) => {
  await stubApi(page);
  await page.goto('/login');
  await expect(page.getByRole('button', { name: 'Sign in with Keycloak' })).toBeVisible();
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();
});

test('choosing the provider hands off to the identity provider', async ({ page }) => {
  const started = await stubApi(page);
  await page.goto('/login?next=%2Falbums');
  await page.getByRole('button', { name: 'Sign in with Keycloak' }).click();
  await expect(page).toHaveURL(/idp\.local/);
  expect(started).toHaveLength(1);
  const sent = JSON.parse(started[0]);
  expect(sent.redirect_uri).toMatch(/\/login$/);
  expect(sent.next).toBe('/albums');
});

test('auto launch hands off without a click', async ({ page }) => {
  await stubApi(page, { auto_launch: true });
  await page.goto('/login');
  await expect(page).toHaveURL(/idp\.local/);
});

test('auto launch can be suppressed to reach the password form', async ({ page }) => {
  await stubApi(page, { auto_launch: true });
  await page.goto('/login?autoLaunch=0');
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();
});

test('a declined sign-in explains itself and keeps the password form', async ({ page }) => {
  await stubApi(page);
  await page.goto('/login?error=access_denied&error_description=Nope');
  await expect(page.getByText(/access_denied: Nope/)).toBeVisible();
  await expect(page.getByLabel('Immich email', { exact: true })).toBeVisible();
});

test('a provider-only server still allows an API key', async ({ page }) => {
  await stubApi(page, { password_login: false });
  await page.goto('/login');
  await expect(page.getByLabel('Immich email', { exact: true })).toBeHidden();
  await page.getByRole('button', { name: /use an immich api key/i }).click();
  await expect(page.getByLabel('Immich API key')).toBeVisible();
});
