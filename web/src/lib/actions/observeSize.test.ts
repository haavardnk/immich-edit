import { beforeEach, describe, expect, it, vi } from 'vitest';
import { observeSize } from './observeSize';

class ResizeObserverFake {
  static instance: ResizeObserverFake | null = null;

  observed: Element[] = [];
  disconnected = false;

  constructor() {
    ResizeObserverFake.instance = this;
  }

  observe(node: Element): void {
    this.observed.push(node);
  }

  disconnect(): void {
    this.disconnected = true;
  }
}

describe('observeSize', () => {
  beforeEach(() => {
    ResizeObserverFake.instance = null;
    vi.stubGlobal('ResizeObserver', ResizeObserverFake);
  });

  it('measures immediately, observes the node, and disconnects on destroy', () => {
    const node = {} as Element;
    let measurements = 0;
    const action = observeSize(node, () => {
      measurements += 1;
    });
    const observer = ResizeObserverFake.instance;

    expect(measurements).toBe(1);
    expect(observer?.observed).toEqual([node]);
    if (!action?.destroy) throw new Error('observeSize must return a destroy handler');
    action.destroy();
    expect(observer?.disconnected).toBe(true);
  });
});
