import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { jobs } from './jobs.svelte';
import { toasts } from './toasts.svelte';
import { getJob, listJobs, retryJob, type Job, type JobItem } from '#lib/api/jobs.ts';
import { ApiError } from '#lib/api/client.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import { downloadUrl } from '#lib/utils/download.ts';

vi.mock('#lib/stores/trash.svelte.ts', () => ({ trash: { removeTrashed: vi.fn() } }));

vi.mock('#lib/utils/download.ts', () => ({ downloadUrl: vi.fn() }));

vi.mock('#lib/stores/editedThumbs.svelte.ts', () => ({
  editedThumbs: { refresh: vi.fn().mockResolvedValue(undefined) }
}));

vi.mock('#lib/api/jobs.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/api/jobs.ts')>()),
  listJobs: vi.fn(),
  getJob: vi.fn(),
  cancelJob: vi.fn(),
  clearJobs: vi.fn(),
  retryJob: vi.fn()
}));

const mockedListJobs = vi.mocked(listJobs);
const mockedGetJob = vi.mocked(getJob);
const mockedRetryJob = vi.mocked(retryJob);

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

function item(status: JobItem['status']): JobItem {
  return { id: 'item-1', asset_id: 'asset-1', status } as JobItem;
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

  it('downloads a ZIP it was asked for once when it finishes', async () => {
    vi.mocked(downloadUrl).mockClear();
    const zip = job({ kind: 'download_zip' });
    const done = { ...zip, status: 'completed' as const, completed: 4 };
    jobs.track(zip);
    jobs.downloadWhenDone(zip.id);
    jobs.receive(done);
    mockedListJobs.mockResolvedValue([{ ...zip, status: 'running' }]);
    await jobs.load();
    mockedListJobs.mockResolvedValue([done]);
    await jobs.load();

    expect(downloadUrl).toHaveBeenCalledExactlyOnceWith('/api/jobs/job-1/download');
  });

  it('downloads neither an empty ZIP nor one nobody asked for', () => {
    vi.mocked(downloadUrl).mockClear();
    const zip = job({ kind: 'download_zip' });
    jobs.track(zip);
    jobs.downloadWhenDone(zip.id);
    jobs.receive({ ...zip, status: 'completed', failed: 4 });
    jobs.track(job({ id: 'job-2', kind: 'download_zip' }));
    jobs.receive(job({ id: 'job-2', kind: 'download_zip', status: 'completed', completed: 4 }));

    expect(downloadUrl).not.toHaveBeenCalled();
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

  it('takes earlier exports a preset job replaced out of the grid', async () => {
    const replace = job({
      id: 'job-presets',
      kind: 'export_immich',
      params: { variants: [{ preset: 'Web', params: { on_conflict: 'replace' } }] }
    });
    mockedGetJob.mockResolvedValue({
      job: replace,
      items: [
        { asset_id: 'a', result: { replaced_asset_id: 'old-a' } } as JobItem,
        { asset_id: 'b', result: { status: 'skipped' } } as JobItem
      ]
    });
    vi.mocked(trash.removeTrashed).mockClear();
    jobs.track(replace);
    jobs.receive({ ...replace, status: 'completed', completed: 2 });
    await vi.advanceTimersByTimeAsync(0);
    expect(trash.removeTrashed).toHaveBeenCalledWith(
      ['old-a'],
      'Moved 1 earlier export to the Immich trash'
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

  it('reopens a failed job, watches it again and refreshes its details', async () => {
    const failed = job({ status: 'completed', completed: 3, failed: 1 });
    const reopened = job({ status: 'pending', completed: 3, updated_at: '2024-01-01T00:01:00Z' });
    mockedRetryJob.mockResolvedValue(reopened);
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValue({ job: reopened, items: [] });
    jobs.track(job({}));
    jobs.receive(failed);
    jobs.items = { 'job-1': [] };

    await jobs.retry('job-1');

    expect(mockedRetryJob).toHaveBeenCalledWith('job-1');
    expect(jobs.jobs[0]?.status).toBe('pending');
    expect(jobs.watched).toEqual(['job-1']);
    expect(jobs.attention).toBe(false);
    expect(mockedGetJob).toHaveBeenCalledWith('job-1');
  });

  it('ignores the failed record a poll in flight still returns after a retry', async () => {
    const failed = job({ status: 'completed', completed: 3, failed: 1 });
    const reopened = job({ status: 'pending', completed: 3, updated_at: '2024-01-01T00:01:00Z' });
    mockedRetryJob.mockResolvedValue(reopened);
    jobs.track(job({}));
    jobs.receive(failed);
    await jobs.retry('job-1');

    mockedListJobs.mockResolvedValue([failed]);
    await jobs.load();
    jobs.receive(failed);

    expect(jobs.jobs[0]?.status).toBe('pending');
    jobs.receive({ ...reopened, status: 'completed', completed: 4 });
    expect(jobs.jobs[0]?.status).toBe('completed');
    expect(jobs.jobs[0]?.failed).toBe(0);
  });

  it('watches a job another tab retried', async () => {
    const streams: string[] = [];
    vi.stubGlobal(
      'EventSource',
      class {
        constructor(url: string) {
          streams.push(url);
        }
        addEventListener(): void {}
        close(): void {}
      }
    );
    jobs.jobs = [job({ status: 'completed', completed: 3, failed: 1 })];
    jobs.receive(job({ status: 'pending', completed: 3, updated_at: '2024-01-01T00:01:00Z' }));

    expect(jobs.jobs[0]?.status).toBe('pending');
    expect(streams).toEqual(['/api/jobs/job-1/events']);
  });

  it('keeps the job and says so when a retry is refused', async () => {
    const failed = job({ status: 'completed', completed: 3, failed: 1 });
    mockedRetryJob.mockRejectedValue(
      new ApiError(409, 'conflict', 'job belongs to a previous server')
    );
    jobs.jobs = [failed];

    await jobs.retry('job-1');

    expect(jobs.jobs[0]).toEqual(failed);
    expect(toasts.items.map((t) => t.message)).toEqual([
      'Failed to retry job: job belongs to a previous server'
    ]);
  });

  it('refreshes the items of an expanded job when it ends', async () => {
    const running = job({ status: 'running' });
    const done = job({ status: 'completed', completed: 4 });
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValueOnce({ job: running, items: [item('pending')] });
    mockedGetJob.mockResolvedValue({ job: done, items: [item('completed')] });
    jobs.track(running);
    jobs.toggleExpanded('job-1');
    await vi.advanceTimersByTimeAsync(0);
    expect(jobs.items['job-1']?.[0]?.status).toBe('pending');

    jobs.receive(done);
    await vi.advanceTimersByTimeAsync(0);

    expect(jobs.items['job-1']?.[0]?.status).toBe('completed');
    expect(mockedGetJob).toHaveBeenCalledTimes(2);
  });

  it('refreshes the items of an expanded job when the drawer poll sees the end', async () => {
    const running = job({ status: 'running' });
    const done = job({ status: 'completed', completed: 4 });
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValueOnce({ job: running, items: [item('pending')] });
    mockedGetJob.mockResolvedValue({ job: done, items: [item('completed')] });
    jobs.track(running);
    jobs.toggleExpanded('job-1');
    await vi.advanceTimersByTimeAsync(0);
    mockedListJobs.mockResolvedValue([done]);

    await jobs.load();
    await vi.advanceTimersByTimeAsync(0);

    expect(jobs.items['job-1']?.[0]?.status).toBe('completed');
  });

  it('leaves the cached items of a collapsed job until it opens again', async () => {
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValue({ job: job({}), items: [item('pending')] });
    jobs.track(job({}));
    jobs.toggleExpanded('job-1');
    await vi.advanceTimersByTimeAsync(0);
    jobs.toggleExpanded('job-1');

    jobs.receive(job({ status: 'completed', completed: 4 }));
    await vi.advanceTimersByTimeAsync(0);

    expect(mockedGetJob).toHaveBeenCalledOnce();
  });

  it('refreshes items as an expanded job progresses without stacking requests', async () => {
    let release: () => void = () => undefined;
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValueOnce({ job: job({}), items: [item('pending')] });
    jobs.track(job({}));
    jobs.toggleExpanded('job-1');
    await vi.advanceTimersByTimeAsync(0);
    const latest = job({ completed: 3 });
    mockedGetJob.mockImplementation(
      () =>
        new Promise((resolve) => {
          release = () => resolve({ job: latest, items: [item('completed')] });
        })
    );

    jobs.receive(job({ completed: 1 }));
    jobs.receive(job({ completed: 2 }));
    jobs.receive(latest);
    release();
    await vi.advanceTimersByTimeAsync(0);
    release();
    await vi.advanceTimersByTimeAsync(0);

    expect(mockedGetJob).toHaveBeenCalledTimes(3);
    expect(jobs.items['job-1']?.[0]?.status).toBe('completed');
  });

  it('keeps the last items quietly when a background refresh fails', async () => {
    mockedGetJob.mockReset();
    mockedGetJob.mockResolvedValueOnce({ job: job({}), items: [item('pending')] });
    mockedGetJob.mockRejectedValue(new Error('offline'));
    jobs.track(job({}));
    jobs.toggleExpanded('job-1');
    await vi.advanceTimersByTimeAsync(0);

    jobs.receive(job({ completed: 1 }));
    await vi.advanceTimersByTimeAsync(0);

    expect(jobs.items['job-1']?.[0]?.status).toBe('pending');
    expect(toasts.items).toEqual([]);
  });
});
