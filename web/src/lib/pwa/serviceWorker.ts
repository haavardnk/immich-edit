import { dev } from '$app/env';
import { reportClientError } from '#lib/api/clientLog.ts';
import { shouldReport, UpdateThrottle, watchRegistration } from '#lib/pwa/registration.ts';
import { UpdatePrompt } from '#lib/pwa/updatePrompt.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const SCRIPT = '/service-worker.js';
const NAVIGATION_CHECK_MS = 60_000;

let registration: ServiceWorkerRegistration | null = null;
let updates: UpdatePrompt | null = null;
const throttle = new UpdateThrottle(NAVIGATION_CHECK_MS);

function hasController(): boolean {
  return navigator.serviceWorker.controller !== null;
}

function announce(accept: () => void, dismissed: () => void): void {
  toasts.push('info', 'immich-edit was updated', {
    ttlMs: 0,
    pinned: true,
    action: { label: 'Reload', run: accept },
    description: 'Reload to use the new version.',
    onDismiss: dismissed
  });
}

function check(throttled: boolean): void {
  const reg = registration;
  if (!reg) return;
  if (reg.waiting && hasController()) updates?.available();
  const now = Date.now();
  if (throttled && !throttle.due(now)) return;
  throttle.mark(now);
  reg.update().catch(() => undefined);
}

export function checkForUpdate(): void {
  check(true);
}

function checkWhenVisible(): void {
  if (document.visibilityState === 'visible') check(false);
}

function controllerChanged(): void {
  void updates?.controllerChanged();
}

export function installServiceWorker(busy: () => boolean): () => void {
  if (dev || !('serviceWorker' in navigator) || !isSecureContext) return () => undefined;
  const container = navigator.serviceWorker;
  const prompt = new UpdatePrompt({
    announce,
    waiting: () => registration?.waiting ?? null,
    busy,
    reload: () => location.reload(),
    sleep: (ms) => new Promise((resolve) => setTimeout(resolve, ms))
  });
  updates = prompt;
  container.addEventListener('controllerchange', controllerChanged);
  document.addEventListener('visibilitychange', checkWhenVisible);
  container
    .register(SCRIPT, { type: 'module' })
    .then((reg) => {
      registration = reg;
      throttle.mark(Date.now());
      watchRegistration(reg, hasController, () => prompt.available());
    })
    .catch((err: unknown) => {
      if (shouldReport(err)) reportClientError('exception', err, 'warn');
    });
  return () => {
    container.removeEventListener('controllerchange', controllerChanged);
    document.removeEventListener('visibilitychange', checkWhenVisible);
    updates = null;
    registration = null;
  };
}
