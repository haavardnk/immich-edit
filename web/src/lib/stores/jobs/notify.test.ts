import type { Job } from '#lib/api/jobs.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const toasts = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts }));

const stored = new Map<string, string>();

const failed: Job = {
  id: 'job-1',
  kind: 'export_immich',
  status: 'completed',
  target: null,
  params: null,
  total: 48,
  completed: 45,
  failed: 3,
  cancelled_at: null,
  created_at: '2026-01-01T10:00:00Z',
  updated_at: '2026-01-01T10:05:00Z'
};

function stubBrowser(permission: NotificationPermission, answer: NotificationPermission) {
  const shown: { title: string; options: NotificationOptions }[] = [];
  const requestPermission = vi.fn(async () => answer);
  vi.stubGlobal('Notification', Object.assign(class {}, { permission, requestPermission }));
  vi.stubGlobal('document', { visibilityState: 'hidden' });
  vi.stubGlobal('navigator', {
    serviceWorker: {
      getRegistration: async () => ({
        showNotification: async (title: string, options: NotificationOptions) => {
          shown.push({ title, options });
        }
      })
    }
  });
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => stored.get(key) ?? null,
    setItem: (key: string, value: string) => stored.set(key, value)
  });
  return { shown, requestPermission };
}

async function freshStore(): Promise<(typeof import('./notify.svelte'))['jobNotifications']> {
  vi.resetModules();
  return (await import('./notify.svelte')).jobNotifications;
}

beforeEach(() => {
  stored.clear();
  toasts.push.mockClear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('job notifications', () => {
  it('asks for permission only when turned on, and remembers the choice', async () => {
    const browser = stubBrowser('default', 'granted');
    const store = await freshStore();
    expect(store.supported).toBe(true);
    expect(store.enabled).toBe(false);
    expect(browser.requestPermission).not.toHaveBeenCalled();

    await store.toggle();

    expect(browser.requestPermission).toHaveBeenCalledOnce();
    expect(store.enabled).toBe(true);
    expect(stored.get('immich-edit:jobNotifications')).toBe('{"enabled":true}');
  });

  it('says how to unblock when the browser refuses', async () => {
    stubBrowser('default', 'denied');
    const store = await freshStore();

    await store.toggle();

    expect(store.enabled).toBe(false);
    expect(store.blocked).toBe(true);
    expect(toasts.push).toHaveBeenCalledWith(
      'warn',
      'Notifications are blocked for this site',
      expect.anything()
    );
  });

  it('notifies about a job that ends while the tab is hidden', async () => {
    const browser = stubBrowser('granted', 'granted');
    stored.set('immich-edit:jobNotifications', '{"enabled":true}');
    const store = await freshStore();

    store.notify(failed);
    store.notify({ ...failed, id: 'job-2', status: 'cancelled' });
    await vi.waitFor(() => expect(browser.shown).toHaveLength(1));

    expect(browser.shown[0]).toEqual({
      title: 'Export to Immich finished with errors',
      options: { body: '3 of 48 failed', tag: 'job-job-1', icon: '/icon-192.png' }
    });
  });

  it('stays quiet while the tab is visible or the user has not opted in', async () => {
    const browser = stubBrowser('granted', 'granted');
    const quiet = await freshStore();
    quiet.notify(failed);

    stored.set('immich-edit:jobNotifications', '{"enabled":true}');
    vi.stubGlobal('document', { visibilityState: 'visible' });
    const visible = await freshStore();
    visible.notify(failed);
    await Promise.resolve();

    expect(browser.shown).toEqual([]);
  });
});
