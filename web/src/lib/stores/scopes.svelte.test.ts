import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ScopeGrid, ScopeKind } from '$lib/types/preview';

const storageKey = 'immich-edit:scopes';

const getPreviewScope = vi.fn(
  async (_assetId: string, _metaId: string, kind: ScopeKind): Promise<ScopeGrid> => ({
    kind,
    width: 2,
    height: 2,
    channels: 1,
    maxCount: 4,
    data: new Uint8Array(4)
  })
);

vi.mock('$lib/api/preview', () => ({ getPreviewScope }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  getPreviewScope.mockClear();
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

describe('scopes store', () => {
  it('restores stored view settings and clamps the gain', async () => {
    stubStorage({ mode: 'vectorscope', channels: 'rgb', gain: 99, zoom: 2 });

    const { scopes } = await import('./scopes.svelte');

    expect(scopes.mode).toBe('vectorscope');
    expect(scopes.channels).toBe('rgb');
    expect(scopes.gain).toBe(8);
    expect(scopes.zoom).toBe(2);
  });

  it('ignores unknown stored values', async () => {
    stubStorage({ mode: 'oscilloscope', channels: 'cmyk', gain: 'loud', zoom: 5 });

    const { scopes } = await import('./scopes.svelte');

    expect(scopes.mode).toBe('histogram');
    expect(scopes.channels).toBe('luma');
    expect(scopes.gain).toBe(1);
    expect(scopes.zoom).toBe(1);
  });

  it('stays idle while the histogram is selected', async () => {
    stubStorage();

    const { scopes } = await import('./scopes.svelte');
    scopes.onMeta('asset-1', 'meta-1', true);

    expect(scopes.wants).toBe(false);
    expect(getPreviewScope).not.toHaveBeenCalled();
  });

  it('fetches the parade grid for an RGB waveform', async () => {
    stubStorage({ mode: 'waveform', channels: 'rgb', gain: 1, zoom: 1 });

    const { scopes } = await import('./scopes.svelte');
    scopes.onMeta('asset-1', 'meta-1', true);
    await vi.waitFor(() => expect(scopes.grid).not.toBeNull());

    expect(getPreviewScope).toHaveBeenCalledWith(
      'asset-1',
      'meta-1',
      'parade',
      expect.any(AbortSignal)
    );
    expect(scopes.loading).toBe(false);
  });

  it('asks for a re-render when the meta carries no scopes', async () => {
    stubStorage({ mode: 'waveform', channels: 'luma', gain: 1, zoom: 1 });

    const { scopes } = await import('./scopes.svelte');
    scopes.onMeta('asset-1', 'meta-1', false);

    expect(scopes.needsRender).toBe(true);
    expect(getPreviewScope).not.toHaveBeenCalled();
  });

  it('loads until the first grid arrives, including before any meta', async () => {
    stubStorage({ mode: 'waveform', channels: 'luma', gain: 1, zoom: 1 });

    const { scopes } = await import('./scopes.svelte');
    expect(scopes.loading).toBe(true);

    scopes.onMeta('asset-1', 'meta-1', true);
    await vi.waitFor(() => expect(scopes.grid).not.toBeNull());
    expect(scopes.loading).toBe(false);
  });

  it('stops loading when the grid request fails', async () => {
    stubStorage({ mode: 'waveform', channels: 'luma', gain: 1, zoom: 1 });
    getPreviewScope.mockRejectedValueOnce(new Error('missing'));

    const { scopes } = await import('./scopes.svelte');
    scopes.onMeta('asset-1', 'meta-1', true);

    await vi.waitFor(() => expect(scopes.loading).toBe(false));
    expect(scopes.grid).toBeNull();
  });

  it('drops the cached grid and refetches on a mode change', async () => {
    stubStorage({ mode: 'waveform', channels: 'luma', gain: 1, zoom: 1 });

    const { scopes } = await import('./scopes.svelte');
    scopes.onMeta('asset-1', 'meta-1', true);
    await vi.waitFor(() => expect(scopes.grid).not.toBeNull());

    scopes.setMode('vectorscope');
    expect(scopes.grid).toBeNull();
    await vi.waitFor(() => expect(scopes.grid?.kind).toBe('vectorscope'));
  });

  it('persists every view setting on change', async () => {
    const values = stubStorage();

    const { scopes } = await import('./scopes.svelte');
    scopes.setMode('parade');
    scopes.setGain(4);
    scopes.setZoom(2);

    expect(JSON.parse(values.get(storageKey) ?? '{}')).toEqual({
      mode: 'parade',
      channels: 'luma',
      gain: 4,
      zoom: 2
    });
  });
});
