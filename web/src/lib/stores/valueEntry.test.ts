import { describe, expect, it, vi } from 'vitest';
import { ValueEntryStore, type ValueEntryTarget } from './valueEntry.svelte';

function target(label: string): ValueEntryTarget {
  return {
    label,
    range: { min: 0, max: 1, step: 0.1, fineStep: 0.1, format: String },
    start: 0,
    resetValue: 0,
    owner: {} as HTMLElement,
    live: vi.fn(),
    finish: vi.fn()
  };
}

describe('value entry', () => {
  it('commits the previous target when another opens', () => {
    const store = new ValueEntryStore();
    const first = target('Exposure');
    const second = target('Contrast');
    store.open(first);
    store.open(second);
    expect(first.finish).toHaveBeenCalledWith(true);
    expect(store.target).toBe(second);
  });

  it('closes once with the given outcome', () => {
    const store = new ValueEntryStore();
    const entry = target('Exposure');
    store.open(entry);
    store.close(false);
    store.close(true);
    expect(entry.finish).toHaveBeenCalledTimes(1);
    expect(entry.finish).toHaveBeenCalledWith(false);
    expect(store.target).toBeNull();
  });

  it('releases only its own target', () => {
    const store = new ValueEntryStore();
    const open = target('Exposure');
    store.open(open);
    store.release(target('Contrast'));
    expect(store.target).toBe(open);
    store.release(open);
    expect(open.finish).toHaveBeenCalledWith(true);
    expect(store.target).toBeNull();
  });
});
