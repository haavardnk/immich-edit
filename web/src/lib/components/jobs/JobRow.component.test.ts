import type { Job, JobItem } from '#lib/api/jobs.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import JobRow from './JobRow.svelte';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

function job(patch: Partial<Job>): Job {
  return {
    id: 'job-1',
    kind: 'download_zip',
    status: 'completed',
    target: null,
    params: null,
    total: 3,
    completed: 2,
    failed: 1,
    cancelled_at: null,
    created_at: '2026-01-01T10:00:00Z',
    updated_at: '2026-01-01T10:05:00Z',
    ...patch
  };
}

const failedItem: JobItem = {
  id: 'i1',
  job_id: 'job-1',
  asset_id: 'asset-0001',
  position: 0,
  variant: 0,
  status: 'failed',
  error: 'Render failed',
  result: null,
  idempotency_key: null,
  attempts: 1,
  created_at: '2026-01-01T10:00:00Z',
  updated_at: '2026-01-01T10:00:00Z'
};

const Host: Component<{ component: Component<{ job: Job }>; props: { job: Job } }> = ComponentHost;

let target: HTMLDivElement;
let row: Record<string, unknown> | null = null;

function render(value: Job): void {
  row = mount(Host, { target, props: { component: JobRow, props: { job: value } } });
  flushSync();
}

function names(role: string): string[] {
  return [...target.querySelectorAll(role === 'link' ? 'a' : 'button')].map(
    (el) => el.getAttribute('aria-label') ?? el.textContent?.trim() ?? ''
  );
}

function button(name: string): HTMLButtonElement {
  const found = [...target.querySelectorAll('button')].find(
    (el) => (el.getAttribute('aria-label') ?? el.textContent?.trim()) === name
  );
  if (!found) throw new Error(`no button ${name}: ${names('button').join(', ')}`);
  return found;
}

beforeEach(() => {
  resetLayout();
  target = document.createElement('div');
  document.body.appendChild(target);
  jobs.expanded = null;
  jobs.items = {};
});

afterEach(() => {
  if (row) void unmount(row);
  row = null;
  target.remove();
  jobs.expanded = null;
  jobs.items = {};
  vi.restoreAllMocks();
});

describe('JobRow', () => {
  it('expands its details from anywhere on the row', () => {
    const load = vi.spyOn(jobs, 'loadItems').mockResolvedValue();
    render(job({}));
    const header = button('Download ZIP details');
    expect(header.getAttribute('aria-expanded')).toBe('false');

    header.click();
    flushSync();

    expect(jobs.expanded).toBe('job-1');
    expect(header.getAttribute('aria-expanded')).toBe('true');
    expect(load).toHaveBeenCalledWith('job-1');
  });

  it('retries failed photos and copies their errors from the desktop details', async () => {
    const retry = vi.spyOn(jobs, 'retry').mockResolvedValue();
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
    jobs.expanded = 'job-1';
    jobs.items = { 'job-1': [failedItem] };
    render(job({}));

    expect(names('button')).not.toContain('Retry 1');
    button('Retry 1 failed').click();
    button('Copy errors').click();
    await Promise.resolve();

    expect(retry).toHaveBeenCalledWith('job-1');
    expect(writeText).toHaveBeenCalledWith(
      'Download ZIP job-1: 1 of 3 failed\nasset-0001: Render failed'
    );
    vi.unstubAllGlobals();
  });

  it('puts Cancel, Save ZIP and Retry on the touch row itself', () => {
    resetLayout({ mode: 'touch' });
    const cancel = vi.spyOn(jobs, 'cancel').mockResolvedValue();
    const retry = vi.spyOn(jobs, 'retry').mockResolvedValue();
    render(job({ status: 'running' }));
    button('Cancel job').click();
    expect(cancel).toHaveBeenCalledWith('job-1');
    void unmount(row!);

    jobs.expanded = 'job-1';
    jobs.items = { 'job-1': [failedItem] };
    render(job({}));
    expect(target.querySelector('a')?.textContent?.trim()).toBe('Save ZIP');
    button('Retry 1 failed').click();
    expect(retry).toHaveBeenCalledWith('job-1');
    expect(target.textContent).toContain('Retry 1');
    expect(target.textContent).not.toContain('Retry 1 failed');
  });

  it('offers no retry for a cancelled job', () => {
    resetLayout({ mode: 'touch' });
    render(job({ status: 'cancelled' }));
    expect(names('button')).toEqual(['Download ZIP details']);
    expect(target.textContent).toMatch(/Cancelled\s*·\s*3 of 3 done/);
  });
});
