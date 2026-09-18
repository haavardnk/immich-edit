import { expect, test, type Page } from '@playwright/test';

const IDP_URL = 'http://idp.local/authorize?client_id=immich';

async function stubApi(page: Page): Promise<string[]> {
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
        return body({ configured: false });
      case '/api/auth/me':
        return body({ code: 'unauthorized', message: 'unauthorized' }, 401);
      case '/api/setup/providers':
        return body({
          oauth: true,
          password_login: true,
          auto_launch: false,
          button_text: 'Sign in with Authentik',
          degraded: false
        });
      case '/api/setup/oauth/start':
        started.push(route.request().postData() ?? '');
        return body({ url: IDP_URL });
      default:
        return body({});
    }
  });
  return started;
}

test('setup offers the provider once the immich url is known', async ({ page }) => {
  const started = await stubApi(page);
  await page.goto('/setup');
  await expect(page.getByRole('button', { name: 'Sign in with Authentik' })).toBeHidden();

  await page.getByLabel('Immich URL').fill('https://immich.example.com');
  await page.getByLabel('Immich URL').blur();

  const provider = page.getByRole('button', { name: 'Sign in with Authentik' });
  await expect(provider).toBeVisible();
  await provider.click();
  await expect(page).toHaveURL(/idp\.local/);

  expect(started).toHaveLength(1);
  const sent = JSON.parse(started[0]);
  expect(sent.immich_url).toBe('https://immich.example.com');
  expect(sent.redirect_uri).toMatch(/\/setup$/);
});

test('setup still accepts an admin password', async ({ page }) => {
  await stubApi(page);
  await page.goto('/setup');
  await expect(page.getByRole('button', { name: /connect and claim instance/i })).toBeVisible();
  await expect(page.getByLabel('Email', { exact: true })).toBeVisible();
});
