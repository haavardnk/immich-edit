import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({ push: vi.fn(), fail: vi.fn() }));

vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { push: h.push, fail: h.fail } }));

import { copyPath } from '#lib/browse/info/copyPath.ts';

const writeText = vi.fn<(text: string) => Promise<void>>();

beforeEach(() => {
  h.push.mockClear();
  h.fail.mockClear();
  writeText.mockReset();
  vi.stubGlobal('navigator', { clipboard: { writeText } });
});

afterEach(() => vi.unstubAllGlobals());

describe('copyPath', () => {
  it('copies the path and confirms it', async () => {
    writeText.mockResolvedValue();
    await copyPath('/library/2026/IMG_4120.ARW');
    expect(writeText).toHaveBeenCalledWith('/library/2026/IMG_4120.ARW');
    expect(h.push).toHaveBeenCalledWith('success', 'Copied the file path', { ttlMs: 2500 });
  });

  it('reports a refused clipboard', async () => {
    const denied = new Error('denied');
    writeText.mockRejectedValue(denied);
    await copyPath('/a');
    expect(h.fail).toHaveBeenCalledWith('Could not copy the path', denied, 6000);
    expect(h.push).not.toHaveBeenCalled();
  });
});
