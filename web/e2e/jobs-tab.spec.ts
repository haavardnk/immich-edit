import { expect, test, type Page } from '@playwright/test';
import { installMocks, jobsPanel, json } from './helpers';

const RUNNING = {
  id: 'job-t',
  kind: 'export_immich',
  status: 'running',
  target: {},
  params: {},
  total: 4,
  completed: 1,
  failed: 0,
  cancelled_at: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:01Z'
};

const FAILED = {
  ...RUNNING,
  status: 'completed',
  completed: 3,
  failed: 1,
  updated_at: '2024-01-01T00:00:09Z'
};

async function fakeNotifications(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const shown: { title: string; body?: string }[] = [];
    let permission = (sessionStorage.getItem('permission') ?? 'default') as NotificationPermission;
    let hidden = false;
    Object.assign(window, { shownNotifications: shown, hideTab: () => (hidden = true) });
    Object.defineProperty(document, 'visibilityState', {
      get: () => (hidden ? 'hidden' : 'visible')
    });
    class FakeNotification {
      static get permission(): NotificationPermission {
        return permission;
      }
      static async requestPermission(): Promise<NotificationPermission> {
        permission = 'granted';
        sessionStorage.setItem('permission', permission);
        return permission;
      }
      onclick: (() => void) | null = null;
      constructor(title: string, options?: NotificationOptions) {
        shown.push({ title, body: options?.body });
      }
      close(): void {}
    }
    Object.defineProperty(window, 'Notification', { value: FakeNotification });
    if (navigator.serviceWorker) {
      navigator.serviceWorker.getRegistration = async () => undefined;
    }
  });
}

async function mockEndingJob(page: Page): Promise<() => void> {
  let release: () => void = () => {};
  const ended = new Promise<void>((resolve) => {
    release = resolve;
  });
  let done = false;
  await page.route('**/api/jobs/job-t/events', async (route) => {
    await ended;
    done = true;
    return route.fulfill({
      status: 200,
      contentType: 'text/event-stream',
      body: `event: job\ndata: ${JSON.stringify(FAILED)}\n\n`
    });
  });
  await page.route('**/api/jobs/job-t', (route) => route.fulfill(json({ job: FAILED, items: [] })));
  await page.route('**/api/jobs', (route) =>
    route.request().method() === 'GET'
      ? route.fulfill(json([done ? FAILED : RUNNING]))
      : route.fallback()
  );
  return release;
}

function favicon(page: Page): Promise<string> {
  return page.evaluate(() =>
    decodeURIComponent(document.querySelector<HTMLLinkElement>('link[rel="icon"]')?.href ?? '')
  );
}

test('the tab shows job progress while a job runs', async ({ page }) => {
  await installMocks(page);
  const end = await mockEndingJob(page);
  await page.goto('/photos');

  await expect(page).toHaveTitle('(25%) immich-edit');
  await expect.poll(() => favicon(page)).toContain('stroke-dasharray="339.3 1357.2"');

  end();
  await expect(page).toHaveTitle('immich-edit');
  await expect.poll(() => favicon(page)).toContain('M64 182');
  expect(await favicon(page)).not.toContain('stroke-dasharray');
});

test('an opted-in tab notifies when a job ends in the background', async ({ page }) => {
  await fakeNotifications(page);
  await installMocks(page);
  const end = await mockEndingJob(page);
  await page.goto('/photos');
  await page.getByRole('button', { name: 'Jobs', exact: true }).click();

  const bell = jobsPanel(page).getByRole('button', { name: 'Notify when jobs finish' });
  await expect(bell).toHaveAttribute('aria-pressed', 'false');
  await bell.click();
  await expect(bell).toHaveAttribute('aria-pressed', 'true');

  await page.evaluate(() => (window as unknown as { hideTab: () => void }).hideTab());
  end();
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { shownNotifications: unknown[] }).shownNotifications
      )
    )
    .toEqual([{ title: 'Export to Immich finished with errors', body: '1 of 4 failed' }]);

  await page.reload();
  await page.getByRole('button', { name: 'Jobs', exact: true }).click();
  await expect(
    jobsPanel(page).getByRole('button', { name: 'Notify when jobs finish' })
  ).toHaveAttribute('aria-pressed', 'true');
});
