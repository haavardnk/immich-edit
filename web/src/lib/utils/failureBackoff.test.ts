import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FailureBackoff } from './failureBackoff';

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date'] });
});

afterEach(() => {
  vi.useRealTimers();
});

describe('failure backoff', () => {
  it('holds a failed key back until its wait has passed', () => {
    const backoff = new FailureBackoff(1000);
    backoff.fail(['a']);
    expect(backoff.waiting('a')).toBe(true);
    expect(backoff.waiting('b')).toBe(false);
    vi.setSystemTime(Date.now() + 999);
    expect(backoff.waiting('a')).toBe(true);
    vi.setSystemTime(Date.now() + 1);
    expect(backoff.waiting('a')).toBe(false);
  });

  it('restarts the wait on a new failure', () => {
    const backoff = new FailureBackoff(1000);
    backoff.fail(['a']);
    vi.setSystemTime(Date.now() + 600);
    backoff.fail(['a']);
    vi.setSystemTime(Date.now() + 600);
    expect(backoff.waiting('a')).toBe(true);
  });

  it('releases forgiven and cleared keys at once', () => {
    const backoff = new FailureBackoff(1000);
    backoff.fail(['a', 'b', 'c']);
    backoff.forgive(['a']);
    expect(backoff.waiting('a')).toBe(false);
    expect(backoff.waiting('b')).toBe(true);
    backoff.clear();
    expect(backoff.waiting('b')).toBe(false);
    expect(backoff.waiting('c')).toBe(false);
  });
});
