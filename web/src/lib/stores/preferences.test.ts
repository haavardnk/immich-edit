import type { Preferences } from '#lib/api/preferences.ts';
import { DEFAULT_PINNED_SETTINGS, MAX_PINNED_SETTINGS } from './preferences.svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  getPreferences: vi.fn<() => Promise<Preferences>>(),
  patchPreferences: vi.fn<(patch: Partial<Preferences>) => Promise<Preferences>>()
}));

const fail = vi.hoisted(() => vi.fn<(context: string, error: unknown) => void>());
const push = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/preferences.ts', () => api);
vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { fail, push } }));

async function freshStore(): Promise<(typeof import('./preferences.svelte'))['preferences']> {
  vi.resetModules();
  return (await import('./preferences.svelte')).preferences;
}

beforeEach(() => {
  api.getPreferences.mockReset();
  api.patchPreferences.mockReset();
  fail.mockReset();
  push.mockReset();
});

describe('preferences', () => {
  it('shows no pins until the account answers, then the default set', async () => {
    const preferences = await freshStore();
    expect(preferences.pins).toEqual([]);
    api.getPreferences.mockResolvedValue({ pinnedSettings: null });
    await preferences.load();
    expect(preferences.pins).toEqual(DEFAULT_PINNED_SETTINGS);
  });

  it('falls back to the default pins when the preferences do not load', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockRejectedValue(new Error('offline'));
    await preferences.load();
    expect(preferences.pins).toEqual(DEFAULT_PINNED_SETTINGS);
  });

  it('pins and unpins by saving the whole list', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue({ pinnedSettings: ['exposure'] });
    api.patchPreferences.mockResolvedValue({ pinnedSettings: null });
    await preferences.load();
    await preferences.togglePin('highlights');
    expect(preferences.pins).toEqual(['exposure', 'highlights']);
    expect(api.patchPreferences).toHaveBeenLastCalledWith({
      pinnedSettings: ['exposure', 'highlights']
    });
    await preferences.togglePin('exposure');
    expect(preferences.pins).toEqual(['highlights']);
    expect(preferences.isPinned('exposure')).toBe(false);
  });

  it('restores the pins the server refuses', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue({ pinnedSettings: ['exposure'] });
    await preferences.load();
    const error = new Error('refused');
    api.patchPreferences.mockRejectedValue(error);
    await preferences.togglePin('contrast');
    expect(preferences.pins).toEqual(['exposure']);
    expect(fail).toHaveBeenCalledWith('Could not save preferences', error);
  });

  it('stops pinning at the server limit', async () => {
    const preferences = await freshStore();
    const full = Array.from({ length: MAX_PINNED_SETTINGS }, (_, index) => `p${index}`);
    api.getPreferences.mockResolvedValue({ pinnedSettings: full });
    await preferences.load();
    await preferences.togglePin('exposure');
    expect(preferences.pins).toHaveLength(MAX_PINNED_SETTINGS);
    expect(api.patchPreferences).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalled();
  });

  it('keeps pins chosen while the account preferences load', async () => {
    const preferences = await freshStore();
    let answer: (value: Preferences) => void = () => {};
    api.getPreferences.mockReturnValue(new Promise((resolve) => (answer = resolve)));
    api.patchPreferences.mockResolvedValue({ pinnedSettings: null });
    const loading = preferences.load();
    await preferences.togglePin('highlights');
    answer({ pinnedSettings: ['exposure'] });
    await loading;
    expect(preferences.pins).toEqual(['highlights']);
  });

  it('saves pin changes in order', async () => {
    const preferences = await freshStore();
    const order: unknown[] = [];
    api.patchPreferences.mockImplementation(async (patch) => {
      order.push(patch.pinnedSettings);
      return { pinnedSettings: patch.pinnedSettings ?? null };
    });
    void preferences.togglePin('exposure');
    await preferences.togglePin('contrast');
    expect(order).toEqual([['exposure'], ['exposure', 'contrast']]);
    expect(preferences.pins).toEqual(['exposure', 'contrast']);
  });

  it('forgets the account pins on sign-out', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue({ pinnedSettings: ['exposure'] });
    await preferences.load();
    preferences.reset();
    expect(preferences.pinnedSettings).toBeNull();
    expect(preferences.pins).toEqual([]);
  });
});
