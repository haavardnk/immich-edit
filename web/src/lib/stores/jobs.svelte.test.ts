import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { jobs } from './jobs.svelte';
import { toasts } from './toasts.svelte';
import { listJobs } from '$lib/api/jobs';

vi.mock('$lib/api/jobs', () => ({
  listJobs: vi.fn(),
  getJob: vi.fn(),
  cancelJob: vi.fn(),
  clearJobs: vi.fn()
}));

const mockedListJobs = vi.mocked(listJobs);

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
