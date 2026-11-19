import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Reconnect, retrySeconds } from './reconnect.svelte.ts';

describe('retrySeconds', () => {
  it('backs off 1, 2, 4, 8 and then 10 seconds', () => {
    expect([0, 1, 2, 3, 4, 9].map(retrySeconds)).toEqual([1, 2, 4, 8, 10, 10]);
  });
});

describe('Reconnect', () => {
  let online = true;
  let events: EventTarget;

  beforeEach(() => {
    vi.useFakeTimers();
    online = true;
    events = new EventTarget();
    vi.stubGlobal('navigator', {
      get onLine() {
        return online;
      }
    });
    vi.stubGlobal('window', events);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('counts down, probes and backs off until the probe succeeds', async () => {
    const results = [false, false, true];
    const probe = vi.fn(async () => results.shift() ?? true);
    const onBack = vi.fn();
    const reconnect = new Reconnect(probe, onBack);
    const stop = reconnect.start();

    expect(reconnect.secondsLeft).toBe(1);
    await vi.advanceTimersByTimeAsync(1000);
    expect(probe).toHaveBeenCalledTimes(1);
    expect(reconnect.attempt).toBe(1);
    expect(reconnect.secondsLeft).toBe(2);

    await vi.advanceTimersByTimeAsync(1000);
    expect(reconnect.secondsLeft).toBe(1);
    await vi.advanceTimersByTimeAsync(1000);
    expect(probe).toHaveBeenCalledTimes(2);
    expect(reconnect.secondsLeft).toBe(4);

    await vi.advanceTimersByTimeAsync(4000);
    expect(probe).toHaveBeenCalledTimes(3);
    expect(onBack).toHaveBeenCalledTimes(1);
    stop();
  });

  it('treats a throwing probe as a failure', async () => {
    const reconnect = new Reconnect(() => Promise.reject(new Error('down')), vi.fn());
    const stop = reconnect.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(reconnect.attempt).toBe(1);
    expect(reconnect.probing).toBe(false);
    stop();
  });

  it('retry now resets the backoff', async () => {
    const probe = vi.fn(async () => false);
    const reconnect = new Reconnect(probe, vi.fn());
    const stop = reconnect.start();
    await vi.advanceTimersByTimeAsync(1000);
    await vi.advanceTimersByTimeAsync(2000);
    expect(reconnect.attempt).toBe(2);

    await reconnect.retryNow();
    expect(reconnect.attempt).toBe(1);
    expect(reconnect.secondsLeft).toBe(2);
    stop();
  });

  it('waits for the network instead of counting while offline', async () => {
    online = false;
    const probe = vi.fn(async () => true);
    const onBack = vi.fn();
    const reconnect = new Reconnect(probe, onBack);
    const stop = reconnect.start();
    expect(reconnect.offline).toBe(true);

    await vi.advanceTimersByTimeAsync(5000);
    expect(probe).not.toHaveBeenCalled();

    online = true;
    events.dispatchEvent(new Event('online'));
    await vi.advanceTimersByTimeAsync(0);
    expect(reconnect.offline).toBe(false);
    expect(onBack).toHaveBeenCalledTimes(1);
    stop();
  });

  it('stops ticking once stopped', async () => {
    const probe = vi.fn(async () => false);
    const reconnect = new Reconnect(probe, vi.fn());
    reconnect.start()();
    await vi.advanceTimersByTimeAsync(5000);
    expect(probe).not.toHaveBeenCalled();
  });
});
