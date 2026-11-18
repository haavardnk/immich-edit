import { describe, expect, it } from 'vitest';
import { viewportInset, type ViewportSource } from '#lib/components/sheet/visualViewport.svelte.ts';

function fakeViewport(height: number, offsetTop = 0) {
  const listeners = new Map<string, Set<() => void>>();
  const source: ViewportSource = {
    height,
    offsetTop,
    addEventListener: (type, listener) => {
      const set = listeners.get(type) ?? new Set<() => void>();
      set.add(listener);
      listeners.set(type, set);
    },
    removeEventListener: (type, listener) => {
      listeners.get(type)?.delete(listener);
    }
  };
  function emit(type: string): void {
    for (const listener of listeners.get(type) ?? []) listener();
  }
  function count(): number {
    return [...listeners.values()].reduce((total, set) => total + set.size, 0);
  }
  return { source, emit, count };
}

describe('viewportInset', () => {
  it('reads the inset immediately when watching starts', () => {
    const viewport = fakeViewport(500);
    const inset = viewportInset();
    const stop = inset.watch({ innerHeight: 800, visualViewport: viewport.source });
    expect(inset.bottom).toBe(300);
    expect(inset.top).toBe(0);
    stop();
  });

  it('updates on visual viewport resize and scroll', () => {
    const viewport = fakeViewport(800);
    const inset = viewportInset();
    const stop = inset.watch({ innerHeight: 800, visualViewport: viewport.source });
    expect(inset.bottom).toBe(0);
    viewport.source.height = 450;
    viewport.emit('resize');
    expect(inset.bottom).toBe(350);
    viewport.source.offsetTop = 100;
    viewport.emit('scroll');
    expect(inset.top).toBe(100);
    expect(inset.bottom).toBe(250);
    stop();
  });

  it('removes its listeners and resets the inset when stopped', () => {
    const viewport = fakeViewport(500);
    const inset = viewportInset();
    const stop = inset.watch({ innerHeight: 800, visualViewport: viewport.source });
    expect(viewport.count()).toBe(2);
    stop();
    expect(viewport.count()).toBe(0);
    expect(inset.bottom).toBe(0);
    viewport.source.height = 200;
    viewport.emit('resize');
    expect(inset.bottom).toBe(0);
  });

  it('stays empty without a visual viewport', () => {
    const inset = viewportInset();
    const stop = inset.watch({ innerHeight: 800, visualViewport: null });
    expect(inset.top).toBe(0);
    expect(inset.bottom).toBe(0);
    stop();
  });
});
