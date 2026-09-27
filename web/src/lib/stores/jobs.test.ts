import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { jobs } from './jobs.svelte';
import { toasts } from './toasts.svelte';
import { getJob, listJobs, type Job } from '$lib/api/jobs';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';

vi.mock('$lib/stores/editedThumbs.svelte', () => ({
  editedThumbs: { refresh: vi.fn().mockResolvedValue(undefined) }
}));

vi.mock('$lib/api/jobs', async (importOriginal) => ({
  ...(await importOriginal<typeof import('$lib/api/jobs')>()),
  listJobs: vi.fn(),
  getJob: vi.fn(),
  cancelJob: vi.fn(),
  clearJobs: vi.fn()
}));

const mockedListJobs = vi.mocked(listJobs);
const mockedGetJob = vi.mocked(getJob);

beforeEach(() => {
  vi.useFakeTimers();
  mockedListJobs.mockReset();
  toasts.items = [];
});

afterEach(() => {
  jobs.close();
  vi.useRealTimers();
});

describe('jobs polling', () => {
  it('backs off exponentially and toasts only the first failure', async () => {
    mockedListJobs.mockRejectedValue(new Error('backend down'));
    const pushed = vi.spyOn(toasts, 'push');

    jobs.toggle();
    await vi.advanceTimersByTimeAsync(0);
    expect(mockedListJobs).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(8000);
    expect(mockedListJobs).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(8000);
    expect(mockedListJobs).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(8000);
    expect(mockedListJobs).toHaveBeenCalledTimes(3);

    expect(pushed).toHaveBeenCalledOnce();
    pushed.mockRestore();
  });

  it('returns to the base interval once a poll succeeds', async () => {
    mockedListJobs.mockRejectedValueOnce(new Error('backend down')).mockResolvedValue([]);

    jobs.toggle();
    await vi.advanceTimersByTimeAsync(0);
    await vi.advanceTimersByTimeAsync(8000);
    expect(mockedListJobs).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(4000);
    expect(mockedListJobs).toHaveBeenCalledTimes(3);
  });

  it('stops polling when the drawer closes', async () => {
    mockedListJobs.mockResolvedValue([]);

    jobs.toggle();
    await vi.advanceTimersByTimeAsync(0);
    jobs.close();

    await vi.advanceTimersByTimeAsync(20000);
    expect(mockedListJobs).toHaveBeenCalledTimes(1);
  });
});

function job(overrides: Partial<Job>): Job {
  return {
    id: 'job-1',
    kind: 'paste_edits',
    status: 'running',
    target: {},
    params: {},
    total: 4,
    completed: 0,
    failed: 0,
    cancelled_at: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    ...overrides
  };
}

describe('tracked jobs', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'EventSource',
      class {
        addEventListener(): void {}
        close(): void {}
      }
    );
    jobs.jobs = [];
    jobs.watched = [];
    jobs.attention = false;
    jobs.expanded = null;
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('shows a queued job without opening the drawer', () => {
    jobs.track(job({}));
    expect(jobs.watched).toEqual(['job-1']);
    expect(jobs.open).toBe(false);
  });

  it('drops a clean finish after a moment', async () => {
    jobs.track(job({}));
    jobs.receive(job({ status: 'completed', completed: 4 }));
    expect(jobs.watched).toEqual(['job-1']);
    await vi.advanceTimersByTimeAsync(6000);
    expect(jobs.watched).toEqual([]);
    expect(jobs.attention).toBe(false);
    expect(editedThumbs.refresh).toHaveBeenCalled();
  });

  it('keeps a finish with failures and flags the jobs button', async () => {
    jobs.track(job({}));
    jobs.receive(job({ status: 'completed', completed: 3, failed: 1 }));
    await vi.advanceTimersByTimeAsync(10000);
    expect(jobs.watched).toEqual(['job-1']);
    expect(jobs.attention).toBe(true);
  });

  it('opens the drawer on the job and clears the flag from details', () => {
    mockedGetJob.mockResolvedValue({ job: job({ status: 'failed' }), items: [] });
    jobs.track(job({}));
    jobs.receive(job({ status: 'failed', failed: 4 }));
    jobs.showJob('job-1');
    expect(jobs.open).toBe(true);
    expect(jobs.expanded).toBe('job-1');
    expect(jobs.watched).toEqual([]);
    expect(jobs.attention).toBe(false);
  });
});
