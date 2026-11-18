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
    store.push('info', 'First', { ttlMs: 0 });
    store.push('info', 'First', { ttlMs: 0 });
    expect(store.items).toHaveLength(1);

    store.push('warn', 'Second', { ttlMs: 0 });
    store.push('error', 'Third', { ttlMs: 0 });
    store.push('success', 'Fourth', { ttlMs: 0 });

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

  it('keeps an action with its toast', () => {
    const store = new ToastStore();
    const run = vi.fn();
    store.push('success', 'Moved', { ttlMs: 0, action: { label: 'Undo', run } });

    store.items[0]?.action?.run();

    expect(store.items[0]?.action?.label).toBe('Undo');
    expect(run).toHaveBeenCalledOnce();
  });

  it('keeps a description with its toast', () => {
    const store = new ToastStore();
    store.push('info', 'Updated', { ttlMs: 0, description: 'Reload when ready.' });

    expect(store.items[0]?.description).toBe('Reload when ready.');
  });

  it('never swallows a repeated toast that carries an action', () => {
    const store = new ToastStore();
    const action = { label: 'Undo', run: vi.fn() };
    store.push('success', 'Moved 1 photo', { ttlMs: 0, action });
    store.push('success', 'Moved 1 photo', { ttlMs: 0, action });
    expect(store.items).toHaveLength(2);
  });

  it('evicts the oldest unpinned toast and keeps a pinned one', () => {
    const store = new ToastStore();
    store.push('info', 'Pinned', { ttlMs: 0, pinned: true });
    store.push('info', 'One', { ttlMs: 0 });
    store.push('info', 'Two', { ttlMs: 0 });
    store.push('info', 'Three', { ttlMs: 0 });

    expect(store.items.map((toast) => toast.message)).toEqual(['Pinned', 'Two', 'Three']);
  });

  it('tells the owner when a toast is dismissed, evicted or expires', () => {
    const store = new ToastStore();
    const closed = vi.fn();
    const evicted = vi.fn();
    const expired = vi.fn();
    store.push('info', 'Closed', { ttlMs: 0, pinned: true, onDismiss: closed });
    store.push('info', 'Evicted', { ttlMs: 0, onDismiss: evicted });
    store.push('info', 'Expires', { ttlMs: 100, onDismiss: expired });
    store.push('info', 'Newest', { ttlMs: 0 });

    expect(evicted).toHaveBeenCalledOnce();
    store.dismiss(store.items[0]?.id ?? -1);
    expect(closed).toHaveBeenCalledOnce();
    vi.advanceTimersByTime(100);
    expect(expired).toHaveBeenCalledOnce();
    store.dismiss(-1);
    expect(closed).toHaveBeenCalledOnce();
  });
});
