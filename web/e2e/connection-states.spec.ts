import { expect, test, type Page } from '@playwright/test';
import { installMocks, json, SESSION_USER } from './helpers';

const IMMICH_DOWN = json({ code: 'upstream_unavailable', message: 'immich unavailable' }, 502);

function countdown(page: Page) {
  return page.getByTestId('reconnect-countdown');
}

test('a down immich-edit server counts down to its next retry', async ({ page }) => {
  await page.route('**/api/**', (route) =>
    route.fulfill({ status: 502, contentType: 'text/html', body: 'Bad Gateway' })
  );
  await page.goto('/photos');

  await expect(
    page.getByRole('heading', { name: "Can't reach the immich-edit server." })
  ).toBeVisible();
  await expect(page.getByText('It may still be starting up.')).toBeVisible();
  await expect(countdown(page)).toHaveText(/^Retrying (in \d+ s|…)$/);
  await expect(countdown(page)).toHaveText('Retrying in 4 s', { timeout: 10_000 });
});

test('going offline waits for the network and reconnects when it returns', async ({
  page,
  context
}) => {
  let down = true;
  await installMocks(page);
  await page.route('**/api/**', (route) =>
    down
      ? route.fulfill({ status: 502, contentType: 'text/html', body: 'Bad Gateway' })
      : route.fallback()
  );
  await page.goto('/photos');
  await expect(
    page.getByRole('heading', { name: "Can't reach the immich-edit server." })
  ).toBeVisible();

  await context.setOffline(true);
  await expect(page.getByRole('heading', { name: "You're offline." })).toBeVisible();
  await expect(countdown(page)).toHaveCount(0);

  down = false;
  await context.setOffline(false);
  await expect(page.getByRole('heading', { name: 'Photos', level: 1 })).toBeVisible();
});

test('an unreachable Immich gets its own page and the app returns with it', async ({ page }) => {
  let immichUp = false;
  await installMocks(page);
  await page.route('**/api/immich/ping', (route) =>
    immichUp ? route.fulfill({ status: 204 }) : route.fulfill(IMMICH_DOWN)
  );
  await page.goto('/photos');

  await expect(page.getByRole('heading', { name: "Can't reach Immich." })).toBeVisible();
  await expect(page.getByText("immich-edit is running, but Immich doesn't answer.")).toBeVisible();
  await expect(countdown(page)).toBeVisible();

  await page.getByRole('button', { name: 'Change server…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Change Immich server' });
  const confirm = dialog.getByRole('textbox', { name: 'Confirm hostname' });
  await expect(confirm).toHaveAttribute('placeholder', 'Type the new server’s hostname to confirm');
  await dialog
    .getByRole('textbox', { name: 'Immich server URL' })
    .fill('https://photos.example.net');
  await expect(confirm).toHaveAttribute('placeholder', 'Type photos.example.net to confirm');
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toHaveCount(0);

  immichUp = true;
  await page.getByRole('button', { name: 'Retry now' }).click();
  await expect(page.getByRole('heading', { name: "Can't reach Immich." })).toHaveCount(0);
  await expect(page).toHaveURL('/photos');
});

test('only admins can change the server from the Immich page', async ({ page }) => {
  await installMocks(page);
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ ...SESSION_USER, is_admin: false }))
  );
  await page.route('**/api/immich/ping', (route) => route.fulfill(IMMICH_DOWN));
  await page.goto('/photos');

  await expect(page.getByRole('heading', { name: "Can't reach Immich." })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Retry now' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Change server…' })).toHaveCount(0);
});

test('sign in waits for Immich instead of offering a form that cannot work', async ({ page }) => {
  let degraded = true;
  await installMocks(page);
  await page.route('**/api/auth/me', (route) =>
    route.fulfill(json({ code: 'unauthorized', message: 'unauthorized' }, 401))
  );
  await page.route('**/api/auth/providers', (route) =>
    route.fulfill(
      json({
        oauth: false,
        password_login: true,
        auto_launch: false,
        button_text: 'Login with OAuth',
        degraded
      })
    )
  );
  await page.goto('/login');

  await expect(page.getByRole('heading', { name: "Can't reach Immich." })).toBeVisible();
  await expect(page.getByLabel('Immich email')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Change server…' })).toHaveCount(0);

  degraded = false;
  await page.getByRole('button', { name: 'Retry now' }).click();
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
  await expect(page.getByLabel('Immich email')).toBeVisible();
});
