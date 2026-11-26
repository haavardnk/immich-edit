import type { Job } from '#lib/api/jobs.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import JobToast from './JobToast.svelte';

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

const Host: Component<{ component: Component<{ job: Job }>; props: { job: Job } }> = ComponentHost;

let toast: Record<string, unknown> | null = null;
const target = document.body;

function render(job: Job): string[] {
  toast = mount(Host, { target, props: { component: JobToast, props: { job } } });
  flushSync();
  return [...target.querySelectorAll('button')].map(
    (el) => el.getAttribute('aria-label') ?? el.textContent?.trim() ?? ''
  );
}

afterEach(() => {
  if (toast) void unmount(toast);
  toast = null;
  vi.restoreAllMocks();
});

describe('JobToast', () => {
  it('retries the failed photos next to Details', () => {
    const retry = vi.spyOn(jobs, 'retry').mockResolvedValue();
    expect(render(failed)).toEqual(['Dismiss Export to Immich', 'Details', 'Retry 3 failed']);
    target.querySelector<HTMLButtonElement>('button[aria-label="Retry 3 failed"]')?.click();
    expect(retry).toHaveBeenCalledWith('job-1');
  });

  it('offers no retry after a cancel or a clean finish', () => {
    expect(render({ ...failed, status: 'cancelled' })).not.toContain('Retry 3 failed');
    void unmount(toast!);
    expect(render({ ...failed, failed: 0, completed: 48 })).toEqual(['Dismiss Export to Immich']);
  });
});
