import { afterEach, describe, expect, it, vi } from 'vitest';
import { canCopyImages, copyImage } from './clipboardImage';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('copy image', () => {
  it('is offered only when the clipboard takes images', () => {
    vi.stubGlobal('ClipboardItem', undefined);
    expect(canCopyImages()).toBe(false);
    vi.stubGlobal('ClipboardItem', class {});
    vi.stubGlobal('navigator', { clipboard: { write: vi.fn() } });
    expect(canCopyImages()).toBe(true);
  });

  it('writes a PNG export to the clipboard as it is', async () => {
    const items: Array<Record<string, Promise<Blob>>> = [];
    vi.stubGlobal(
      'ClipboardItem',
      class {
        constructor(item: Record<string, Promise<Blob>>) {
          items.push(item);
        }
      }
    );
    const write = vi.fn(() => Promise.resolve());
    vi.stubGlobal('navigator', { clipboard: { write } });
    const blob = new Blob(['png'], { type: 'image/png' });

    await copyImage(blob, 'png');

    expect(write).toHaveBeenCalledTimes(1);
    expect(Object.keys(items[0] ?? {})).toEqual(['image/png']);
    expect(await items[0]?.['image/png']).toBe(blob);
  });
});
