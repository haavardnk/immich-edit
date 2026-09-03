import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:browseView';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(storageKey, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  return values;
}

describe('browse view persistence', () => {
  it('restores the stored grid size and auto-advance flag', async () => {
    stubStorage({ gridSize: 'xl', loupeAutoAdvance: true });

    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('xl');
    expect(browseView.loupeAutoAdvance).toBe(true);
  });

  it('ignores an unknown grid size', async () => {
    stubStorage({ gridSize: 'huge', loupeAutoAdvance: 'yes' });

    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('md');
    expect(browseView.loupeAutoAdvance).toBe(false);
  });

  it('writes both fields whenever either one changes', async () => {
    const values = stubStorage();

    const { browseView } = await import('./browseView.svelte');
    browseView.setLoupeAutoAdvance(true);
    browseView.stepGridSize(1);

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({
      gridSize: 'lg',
      loupeAutoAdvance: true
    });
  });
});
