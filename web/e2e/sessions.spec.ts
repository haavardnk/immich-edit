import { expect, test } from '@playwright/test';
import { installMocks, json } from './helpers';

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

  await page.goto('/settings#sessions');

  const revoke = page.getByRole('button', { name: 'Revoke session for Firefox on Linux' });
  await expect(page.getByText('2 active sessions')).toBeVisible();
  await revoke.click();

  await expect(page.getByText('1 active session', { exact: true })).toBeVisible();
  await expect(revoke).toHaveCount(0);
  await expect(page.getByText('Chrome on macOS')).toBeVisible();
  expect(revoked).toEqual(['session-other']);
});
