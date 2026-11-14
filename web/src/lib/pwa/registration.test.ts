import { describe, expect, it, vi } from 'vitest';
import {
  shouldReport,
  UpdateThrottle,
  watchRegistration,
  type InstallingWorker
} from '#lib/pwa/registration.ts';

type Listener = () => void;

class FakeWorker implements InstallingWorker {
  state = 'installing';
  #listeners: Listener[] = [];

  addEventListener(_type: 'statechange', listener: Listener): void {
    this.#listeners.push(listener);
  }

  become(state: string): void {
    this.state = state;
    for (const listener of this.#listeners) listener();
  }
}

class FakeRegistration {
  installing: FakeWorker | null = null;
  waiting: unknown = null;
  #listeners: Listener[] = [];

  addEventListener(_type: 'updatefound', listener: Listener): void {
    this.#listeners.push(listener);
  }

  found(worker: FakeWorker): void {
    this.installing = worker;
    for (const listener of this.#listeners) listener();
  }
}

describe('watchRegistration', () => {
  it('announces an update found later once it is installed', () => {
    const reg = new FakeRegistration();
    const onWaiting = vi.fn();
    watchRegistration(reg, () => true, onWaiting);
    const worker = new FakeWorker();
    reg.found(worker);
    expect(onWaiting).not.toHaveBeenCalled();
    worker.become('installed');
    expect(onWaiting).toHaveBeenCalledOnce();
  });

  it('catches an update already installing when registration resolves', () => {
    const reg = new FakeRegistration();
    const worker = new FakeWorker();
    reg.installing = worker;
    const onWaiting = vi.fn();
    watchRegistration(reg, () => true, onWaiting);
    worker.become('installed');
    expect(onWaiting).toHaveBeenCalledOnce();
  });

  it('announces a worker that is already waiting', () => {
    const reg = new FakeRegistration();
    reg.waiting = {};
    const onWaiting = vi.fn();
    watchRegistration(reg, () => true, onWaiting);
    expect(onWaiting).toHaveBeenCalledOnce();
  });

  it('stays quiet on the first install without a controller', () => {
    const reg = new FakeRegistration();
    const onWaiting = vi.fn();
    watchRegistration(reg, () => false, onWaiting);
    const worker = new FakeWorker();
    reg.found(worker);
    worker.become('installed');
    worker.become('activated');
    expect(onWaiting).not.toHaveBeenCalled();
  });

  it('ignores states other than installed', () => {
    const reg = new FakeRegistration();
    const onWaiting = vi.fn();
    watchRegistration(reg, () => true, onWaiting);
    const worker = new FakeWorker();
    reg.found(worker);
    worker.become('redundant');
    expect(onWaiting).not.toHaveBeenCalled();
  });
});

describe('shouldReport', () => {
  it('skips insecure-origin registration failures', () => {
    expect(shouldReport(new DOMException('blocked', 'SecurityError'))).toBe(false);
  });

  it('reports other failures', () => {
    expect(shouldReport(new TypeError('bad script'))).toBe(true);
    expect(shouldReport(new DOMException('aborted', 'AbortError'))).toBe(true);
  });
});

describe('UpdateThrottle', () => {
  it('is due before the first check', () => {
    expect(new UpdateThrottle(60_000).due(0)).toBe(true);
  });

  it('waits the full interval after a check', () => {
    const throttle = new UpdateThrottle(60_000);
    throttle.mark(1_000);
    expect(throttle.due(60_999)).toBe(false);
    expect(throttle.due(61_000)).toBe(true);
  });
});
