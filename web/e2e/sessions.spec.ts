import { expect, test } from '@playwright/test';
import { installMocks, json, settingsGroup } from './helpers';

const SESSIONS = [
  {
    id: 'session-current',
    current: true,
    created_at: '2024-01-01T00:00:00Z',
    last_seen_at: '2024-01-02T00:00:00Z',
    user_agent: 'Chrome on macOS',
    ip: '10.0.0.1'
  },
  {
    id: 'session-other',
    current: false,
    created_at: '2024-01-01T00:00:00Z',
    last_seen_at: '2024-01-02T00:00:00Z',
    user_agent: 'Firefox on Linux',
    ip: '10.0.0.2'
  }
];

test('revoking a session drops it from the signed-in devices list', async ({ page }) => {
  const revoked: string[] = [];
  await installMocks(page);
  await page.route('**/api/auth/sessions**', (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === '/api/auth/sessions') {
      const remaining = SESSIONS.filter((session) => !revoked.includes(session.id));
      return route.fulfill(json({ sessions: remaining }));
    }
    if (request.method() !== 'DELETE') return route.fallback();
    revoked.push(path.split('/').pop() ?? '');
    return route.fulfill(json({ ok: true }));
  });

  await page.goto('/settings/account#sessions');

  const devices = settingsGroup(page, 'Signed-in devices');
  const revoke = devices.getByRole('button', { name: 'Revoke session for Firefox on Linux' });
  await expect(devices.getByText('Firefox on Linux')).toBeVisible();
  await expect(devices.getByRole('button', { name: 'Revoke all other sessions' })).toBeVisible();
  await revoke.click();

  await expect(revoke).toHaveCount(0);
  await expect(devices.getByText('Firefox on Linux')).toHaveCount(0);
  await expect(devices.getByRole('button', { name: 'Revoke all other sessions' })).toHaveCount(0);
  await expect(page.getByText('Chrome on macOS')).toBeVisible();
  expect(revoked).toEqual(['session-other']);
});
