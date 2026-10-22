import type { DenoiseStatus } from '$lib/api/denoise';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  start: vi.fn<(id: string) => Promise<DenoiseStatus>>(),
  status: vi.fn<(id: string) => Promise<DenoiseStatus>>()
}));

vi.mock('$lib/api/denoise', () => ({
  startDenoise: api.start,
  getDenoiseStatus: api.status
}));

const { AiDenoiseTracker } = await import('./aiDenoise.svelte');

async function flush(): Promise<void> {
  for (let i = 0; i < 10; i++) await Promise.resolve();
}

class FakeDocument extends EventTarget {
  visibilityState: DocumentVisibilityState = 'visible';

  show(state: DocumentVisibilityState): void {
    this.visibilityState = state;
    this.dispatchEvent(new Event('visibilitychange'));
  }
}

describe('AI denoise tracker', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    api.start.mockReset();
    api.status.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('starts a bake, polls its progress and re-renders once when it lands', async () => {
    const onReady = vi.fn();
    const tracker = new AiDenoiseTracker(onReady);
    api.start.mockResolvedValue({ state: 'queued' });
    api.status
      .mockResolvedValueOnce({ state: 'running', done: 3, total: 12 })
      .mockResolvedValueOnce({ state: 'ready' });

    tracker.sync('a', true);
    await flush();
    expect(api.start).toHaveBeenCalledWith('a');
    expect(tracker.revision).toBe('');

    await vi.advanceTimersByTimeAsync(1500);
    expect(tracker.status).toEqual({ state: 'running', done: 3, total: 12 });

    await vi.advanceTimersByTimeAsync(1500);
    expect(tracker.status).toEqual({ state: 'ready' });
    expect(tracker.revision).not.toBe('');
    expect(onReady).toHaveBeenCalledOnce();

    await vi.advanceTimersByTimeAsync(5000);
    expect(api.status).toHaveBeenCalledTimes(2);
  });

  it('keeps the first source when the plane was already baked', async () => {
    const onReady = vi.fn();
    const tracker = new AiDenoiseTracker(onReady);
    api.start.mockResolvedValue({ state: 'ready' });
    tracker.sync('a', true);
    await flush();
    expect(tracker.status).toEqual({ state: 'ready' });
    expect(tracker.revision).toBe('');
    expect(onReady).not.toHaveBeenCalled();
  });

  it('sends one start while the first answer is pending', async () => {
    const tracker = new AiDenoiseTracker(vi.fn());
    api.start.mockResolvedValue({ state: 'queued' });
    tracker.sync('a', true);
    tracker.sync('a', true);
    tracker.sync('a', true);
    await flush();
    expect(api.start).toHaveBeenCalledOnce();
  });

  it('stays quiet while AI denoise is off and drops answers for a previous asset', async () => {
    const tracker = new AiDenoiseTracker(vi.fn());
    tracker.sync('a', false);
    expect(api.start).not.toHaveBeenCalled();

    let land: (status: DenoiseStatus) => void = () => {};
    api.start.mockReturnValueOnce(new Promise((resolve) => (land = resolve)));
    tracker.sync('a', true);
    tracker.sync('b', false);
    land({ state: 'ready' });
    await flush();
    expect(tracker.status).toBeNull();
  });

  it('does not retry a failed bake until asked', async () => {
    const tracker = new AiDenoiseTracker(vi.fn());
    api.start.mockResolvedValue({ state: 'failed', error: 'boom' });
    tracker.sync('a', true);
    await flush();
    tracker.sync('a', true);
    await flush();
    expect(api.start).toHaveBeenCalledOnce();
    tracker.retry();
    await flush();
    expect(api.start).toHaveBeenCalledTimes(2);
  });

  it('restarts the bake when a poll finds the server forgot it', async () => {
    const tracker = new AiDenoiseTracker(vi.fn());
    api.start.mockResolvedValue({ state: 'queued' });
    api.status.mockResolvedValueOnce({ state: 'idle' });

    tracker.sync('a', true);
    await flush();
    await vi.advanceTimersByTimeAsync(1500);

    expect(api.start).toHaveBeenCalledTimes(2);
    expect(tracker.status).toEqual({ state: 'queued' });
  });

  it('pauses polling while the page is hidden and resumes when it is shown', async () => {
    const doc = new FakeDocument();
    vi.stubGlobal('document', doc);
    const tracker = new AiDenoiseTracker(vi.fn());
    api.start.mockResolvedValue({ state: 'queued' });
    api.status.mockResolvedValue({ state: 'running', done: 1, total: 4 });

    tracker.sync('a', true);
    await flush();
    doc.show('hidden');
    await vi.advanceTimersByTimeAsync(6000);
    expect(api.status).not.toHaveBeenCalled();

    doc.show('visible');
    await flush();
    expect(api.status).toHaveBeenCalledOnce();

    doc.show('hidden');
    await vi.advanceTimersByTimeAsync(1500);
    tracker.reset();
    doc.show('visible');
    await flush();
    expect(api.status).toHaveBeenCalledOnce();
  });
});
