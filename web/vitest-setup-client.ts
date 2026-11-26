import { vi } from 'vitest';

class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

const entries = new Map<string, string>();

const memoryStorage: Storage = {
  get length(): number {
    return entries.size;
  },
  clear(): void {
    entries.clear();
  },
  getItem(key: string): string | null {
    return entries.get(key) ?? null;
  },
  key(index: number): string | null {
    return [...entries.keys()][index] ?? null;
  },
  removeItem(key: string): void {
    entries.delete(key);
  },
  setItem(key: string, value: string): void {
    entries.set(key, value);
  }
};

globalThis.ResizeObserver = ResizeObserverStub;
vi.stubGlobal('localStorage', memoryStorage);

window.matchMedia = (query: string): MediaQueryList =>
  ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: (): void => {},
    removeEventListener: (): void => {},
    addListener: (): void => {},
    removeListener: (): void => {},
    dispatchEvent: (): boolean => false
  }) as MediaQueryList;

class AnimationStub {
  currentTime = 0;
  effect = null;
  playState = 'finished';
  finished = Promise.resolve(this);
  #onfinish: (() => void) | null = null;

  get onfinish(): (() => void) | null {
    return this.#onfinish;
  }

  set onfinish(fn: (() => void) | null) {
    this.#onfinish = fn;
    queueMicrotask(() => this.#onfinish?.());
  }

  cancel(): void {
    this.#onfinish = null;
  }

  finish(): void {}
}

Element.prototype.animate = (): Animation => new AnimationStub() as unknown as Animation;
Element.prototype.scrollTo = (): void => {};
Element.prototype.scrollIntoView = (): void => {};

Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
  configurable: true,
  get: (): number => 1000
});

Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
  configurable: true,
  get: (): number => 800
});
