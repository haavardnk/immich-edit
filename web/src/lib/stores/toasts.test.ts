import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ToastStore, type ToastKind } from './toasts.svelte';

describe('toast store', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(10_000);
  });

  afterEach(() => vi.useRealTimers());

  it('deduplicates recent messages and keeps the newest three', () => {
    const store = new ToastStore();
    store.push('info', 'First', 0);
    store.push('info', 'First', 0);
    expect(store.items).toHaveLength(1);

    store.push('warn', 'Second', 0);
    store.push('error', 'Third', 0);
    store.push('success', 'Fourth', 0);

    expect(store.items.map((toast) => toast.message)).toEqual(['Second', 'Third', 'Fourth']);
  });

  it.each<[ToastKind, number]>([
    ['info', 5000],
    ['warn', 8000],
    ['success', 10_000],
    ['error', 12_000]
  ])('expires %s messages after %d ms', (kind, ttl) => {
    const store = new ToastStore();
    store.push(kind, 'Message');

    vi.advanceTimersByTime(ttl - 1);
    expect(store.items).toHaveLength(1);

    vi.advanceTimersByTime(1);
    expect(store.items).toHaveLength(0);
  });
});
