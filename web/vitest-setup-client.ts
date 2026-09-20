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

vi.stubGlobal('ResizeObserver', ResizeObserverStub);
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

Element.prototype.scrollTo = (): void => {};

Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
  configurable: true,
  get: (): number => 1000
});

Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
  configurable: true,
  get: (): number => 800
});
