import { expect, test, type Page } from '@playwright/test';
import {
  gotoAsset,
  installMocks,
  json,
  NEUTRAL_RECORD,
  SESSION_USER,
  type PreviewRequest
} from './helpers';

type Status = Record<string, unknown>;

const savedDenoise = {
  ...NEUTRAL_RECORD,
  manifest: { schema_version: 4, ops: { ai_denoise: { strength: 75 } } },
  hash: 'hash-denoise'
};

async function routeDenoise(page: Page, next: () => Status, onStart?: () => void): Promise<void> {
  await page.route('**/api/assets/*/denoise', (route) => {
    if (route.request().method() === 'POST') onStart?.();
    return route.fulfill(json(next()));
  });
}

function savedOp(saves: Array<Record<string, unknown>>, fragment: string): boolean {
  return saves.some((s) => JSON.stringify(s).includes(fragment));
}

async function openDetail(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Detail', exact: true }).click();
}

test('turning on AI denoise bakes once, reports progress, re-renders when ready and resets when off', async ({
  page
}) => {
  const saves: Array<Record<string, unknown>> = [];
  const previews: PreviewRequest[] = [];
  const statuses: Status[] = [
    { state: 'queued' },
    { state: 'running', done: 6, total: 12 },
    { state: 'ready' }
  ];
  let starts = 0;
  await installMocks(page, {
    previewMeta: { is_raw: true },
    onSave: (body) => saves.push(body),
    onPreview: (req) => previews.push(req)
  });
  await routeDenoise(
    page,
    () => (statuses.length > 1 ? (statuses.shift() ?? { state: 'ready' }) : statuses[0]!),
    () => starts++
  );
  await gotoAsset(page);
  await openDetail(page);

  const toggle = page.getByRole('checkbox', { name: 'AI Denoise' });
  const strength = page.getByRole('slider', { name: 'Strength' });
  const detail = page
    .getByRole('group', { name: 'AI Denoise' })
    .getByRole('slider', { name: 'Detail' });
  await expect(toggle).toBeEnabled();
  await expect(strength).toBeDisabled();
  await expect(detail).toBeDisabled();

  await toggle.check();
  await expect(strength).toBeEnabled();
  await expect.poll(() => savedOp(saves, '"ai_denoise":{"strength":75,')).toBe(true);
  await expect.poll(() => starts).toBe(1);
  await expect(
    page.getByRole('status').filter({ hasText: 'Denoising at full resolution…' })
  ).toBeVisible();
  const progress = page.getByRole('progressbar', { name: 'AI Denoise progress' });
  await expect(progress).toHaveAttribute('aria-valuenow', '50');
  const before = previews.length;
  await expect(progress).toBeHidden({ timeout: 10_000 });
  await expect.poll(() => previews.length).toBeGreaterThan(before);
  expect(starts).toBe(1);

  await strength.fill('40');
  await expect.poll(() => savedOp(saves, '"ai_denoise":{"strength":40,')).toBe(true);

  await detail.fill('30');
  await expect.poll(() => savedOp(saves, '"ai_denoise":{"strength":40,"detail":30}')).toBe(true);

  await toggle.uncheck();
  await expect(detail).toBeDisabled();
  await expect(detail).toHaveValue('0');
  expect(starts).toBe(1);
});

for (const { who, isAdmin, text, links } of [
  {
    who: 'points admins to the models settings',
    isAdmin: true,
    text: 'The AI Denoise model is not installed.',
    links: 1
  },
  {
    who: 'asks non-admins to contact an administrator',
    isAdmin: false,
    text: 'The AI Denoise model is not installed. Ask an administrator to install it.',
    links: 0
  }
]) {
  test(`a missing model ${who}`, async ({ page }) => {
    await installMocks(page, { previewMeta: { is_raw: true } });
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(json({ ...SESSION_USER, is_admin: isAdmin }))
    );
    await routeDenoise(page, () => ({ state: 'model_missing' }));
    await gotoAsset(page);
    await openDetail(page);

    await page.getByRole('checkbox', { name: 'AI Denoise' }).check();
    await expect(page.getByText(text, { exact: true })).toBeVisible();
    const link = page.getByRole('link', { name: 'Models settings' });
    await expect(link).toHaveCount(links);
    if (isAdmin) await expect(link).toHaveAttribute('href', '/settings#models');
  });
}

test('a failed bake can be retried', async ({ page }) => {
  let failing = true;
  let starts = 0;
  await installMocks(page, { previewMeta: { is_raw: true } });
  await routeDenoise(
    page,
    () =>
      failing
        ? { state: 'failed', error: 'Denoising failed. Check the server log.' }
        : { state: 'ready' },
    () => starts++
  );
  await gotoAsset(page);
  await openDetail(page);

  await page.getByRole('checkbox', { name: 'AI Denoise' }).check();
  const alert = page.getByRole('alert');
  await expect(alert).toContainText('AI Denoise failed');
  await expect(alert).toContainText('Denoising failed. Check the server log.');
  failing = false;
  await page.getByRole('button', { name: 'Try again' }).click();
  await expect(page.getByText(/AI Denoise failed/)).toBeHidden();
  await expect.poll(() => starts).toBe(2);
});

test('AI denoise is disabled for non-raw assets', async ({ page }) => {
  let starts = 0;
  await installMocks(page, { previewMeta: { is_raw: false }, editRecord: savedDenoise });
  await routeDenoise(
    page,
    () => ({ state: 'unsupported' }),
    () => starts++
  );
  await gotoAsset(page);
  await openDetail(page);

  await expect(page.getByRole('checkbox', { name: 'AI Denoise' })).toBeDisabled();
  await expect(page.getByRole('slider', { name: 'Strength' })).toBeDisabled();
  await page.getByText('AI Denoise', { exact: true }).hover();
  await expect(
    page.locator('.tooltip-content', { hasText: 'Only available for raw files' })
  ).toBeVisible();
  expect(starts).toBe(0);
});

test('a saved AI denoise edit bakes once when the raw photo opens', async ({ page }) => {
  let starts = 0;
  await installMocks(page, { previewMeta: { is_raw: true }, editRecord: savedDenoise });
  await routeDenoise(
    page,
    () => ({ state: 'running', done: 1, total: 4 }),
    () => starts++
  );
  await gotoAsset(page);
  await openDetail(page);

  await expect(page.getByRole('checkbox', { name: 'AI Denoise' })).toBeChecked();
  await expect(page.getByRole('progressbar', { name: 'AI Denoise progress' })).toBeVisible();
  expect(starts).toBe(1);
});
