import { expect, test, type Page } from '@playwright/test';
import { installMocks, json } from './helpers';

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
