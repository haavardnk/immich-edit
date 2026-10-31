import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { jobs } from './jobs.svelte';
import { toasts } from './toasts.svelte';
import { getJob, listJobs, type Job, type JobItem } from '#lib/api/jobs.ts';
import { ApiError } from '#lib/api/client.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';

vi.mock('#lib/stores/trash.svelte.ts', () => ({ trash: { removeTrashed: vi.fn() } }));

vi.mock('#lib/stores/editedThumbs.svelte.ts', () => ({
  editedThumbs: { refresh: vi.fn().mockResolvedValue(undefined) }
}));

vi.mock('#lib/api/jobs.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/api/jobs.ts')>()),
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

function trashedItem(): JobItem {
  return {
    asset_id: '11111111-1111-1111-1111-111111111111',
    result: { original_trashed: true }
  } as JobItem;
}

function giveUpStreams(): { onerror: (() => void) | null }[] {
  const streams: { onerror: (() => void) | null }[] = [];
  vi.stubGlobal(
    'EventSource',
    class {
      static CLOSED = 2;
      readyState = 0;
      onerror: (() => void) | null = null;
      constructor() {
        streams.push(this);
      }
      addEventListener(): void {}
      close(): void {}
    }
  );
  return streams;
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

  it('takes trashed originals of a replace export out of the grid', async () => {
    const replace = job({ kind: 'export_immich', params: { replace_original: true } });
    const item = (asset_id: string, original_trashed: boolean) =>
      ({ asset_id, result: { original_trashed } }) as JobItem;
    mockedGetJob.mockResolvedValue({
      job: replace,
      items: [
        item('11111111-1111-1111-1111-111111111111_1', true),
        item('b', false),
        item('33333333-3333-3333-3333-333333333333', true)
      ]
    });
    jobs.track(replace);
    jobs.receive({ ...replace, status: 'completed', completed: 3 });
    await vi.advanceTimersByTimeAsync(0);
    expect(trash.removeTrashed).toHaveBeenCalledWith(
      ['11111111-1111-1111-1111-111111111111', '33333333-3333-3333-3333-333333333333'],
      'Moved 2 originals to the Immich trash'
    );
  });

  it('takes trashed originals out of the grid when the drawer poll sees the end', async () => {
    const replace = job({ kind: 'export_immich', params: { replace_original: true } });
    const done = { ...replace, status: 'completed' as const, completed: 1 };
    mockedGetJob.mockResolvedValue({ job: done, items: [trashedItem()] });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);
    mockedListJobs.mockResolvedValue([done]);

    await jobs.load();
    await jobs.load();
    await vi.advanceTimersByTimeAsync(0);

    expect(trash.removeTrashed).toHaveBeenCalledOnce();
  });

  it('finishes a job once its details show the end', async () => {
    const replace = job({ kind: 'export_immich', params: { replace_original: true } });
    const done = { ...replace, status: 'completed' as const, completed: 1 };
    mockedGetJob.mockResolvedValue({ job: done, items: [trashedItem()] });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);

    await jobs.loadItems('job-1');
    await vi.advanceTimersByTimeAsync(0);

    expect(trash.removeTrashed).toHaveBeenCalledOnce();
  });

  it('keeps an ended job ended when a stale poll still says running', async () => {
    const replace = job({ kind: 'export_immich', params: { replace_original: true } });
    const done = { ...replace, status: 'completed' as const, completed: 1 };
    mockedGetJob.mockResolvedValue({ job: done, items: [trashedItem()] });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);
    jobs.receive(done);
    mockedListJobs.mockResolvedValue([replace]);

    await jobs.load();
    jobs.receive(done);
    await vi.advanceTimersByTimeAsync(0);

    expect(jobs.jobs[0]?.status).toBe('completed');
    expect(trash.removeTrashed).toHaveBeenCalledOnce();
  });

  it('polls a job whose event stream gave up until it ends', async () => {
    const streams = giveUpStreams();
    const replace = job({ kind: 'export_immich', params: { replace_original: true } });
    const done = { ...replace, status: 'completed' as const, completed: 1 };
    mockedGetJob
      .mockResolvedValueOnce({ job: replace, items: [] })
      .mockResolvedValue({ job: done, items: [trashedItem()] });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);
    for (let i = 0; i < 5; i++) streams[0]?.onerror?.();

    await vi.advanceTimersByTimeAsync(4000);
    expect(trash.removeTrashed).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(4000);

    expect(trash.removeTrashed).toHaveBeenCalledOnce();
    expect(jobs.jobs[0]?.status).toBe('completed');
  });

  it('keeps polling through repeated failures until the job ends', async () => {
    const streams = giveUpStreams();
    const replace = job({
      id: 'job-flaky',
      kind: 'export_immich',
      params: { replace_original: true }
    });
    const done = { ...replace, status: 'completed' as const, completed: 1 };
    mockedGetJob.mockReset();
    for (let i = 0; i < 6; i++) mockedGetJob.mockRejectedValueOnce(new Error('offline'));
    mockedGetJob.mockResolvedValue({ job: done, items: [trashedItem()] });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);
    for (let i = 0; i < 5; i++) streams[0]?.onerror?.();

    await vi.advanceTimersByTimeAsync(300000);

    expect(trash.removeTrashed).toHaveBeenCalledOnce();
  });

  it('stops polling a job the server no longer has', async () => {
    const streams = giveUpStreams();
    mockedGetJob.mockReset();
    mockedGetJob.mockRejectedValue(new ApiError(404, 'not_found', 'Job not found'));
    jobs.track(job({ id: 'job-gone' }));
    for (let i = 0; i < 5; i++) streams[0]?.onerror?.();

    await vi.advanceTimersByTimeAsync(300000);

    expect(mockedGetJob).toHaveBeenCalledOnce();
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
