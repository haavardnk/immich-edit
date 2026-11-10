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
  it('opens the editor on a click until the account says otherwise', async () => {
    const preferences = await freshStore();
    expect(preferences.clickTarget).toBe('editor');
    api.getPreferences.mockResolvedValue({ clickTarget: 'loupe', pinnedSettings: null });
    await preferences.load();
    expect(preferences.clickTarget).toBe('loupe');
  });

  it('keeps the default when the preferences do not load', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockRejectedValue(new Error('offline'));
    await preferences.load();
    expect(preferences.clickTarget).toBe('editor');
    expect(fail).not.toHaveBeenCalled();
  });

  it('applies a choice at once and saves it', async () => {
    const preferences = await freshStore();
    api.patchPreferences.mockResolvedValue({ clickTarget: 'loupe', pinnedSettings: null });
    const saving = preferences.setClickTarget('loupe');
    expect(preferences.clickTarget).toBe('loupe');
    await saving;
    expect(api.patchPreferences).toHaveBeenCalledWith({ clickTarget: 'loupe' });
  });

  it('keeps a choice made while the account preferences load', async () => {
    const preferences = await freshStore();
    let answer: (value: Preferences) => void = () => {};
    api.getPreferences.mockReturnValue(new Promise((resolve) => (answer = resolve)));
    api.patchPreferences.mockResolvedValue({ clickTarget: 'loupe', pinnedSettings: null });
    const loading = preferences.load();
    await preferences.setClickTarget('loupe');
    answer({ clickTarget: 'editor', pinnedSettings: null });
    await loading;
    expect(preferences.clickTarget).toBe('loupe');
  });

  it('reverts a choice the server refuses', async () => {
    const preferences = await freshStore();
    const error = new Error('refused');
    api.patchPreferences.mockRejectedValue(error);
    await preferences.setClickTarget('loupe');
    expect(preferences.clickTarget).toBe('editor');
    expect(fail).toHaveBeenCalledWith('Could not save preferences', error);
  });

  it('saves choices in order', async () => {
    const preferences = await freshStore();
    const order: string[] = [];
    api.patchPreferences.mockImplementation(async (patch) => {
      order.push(patch.clickTarget ?? '');
      return { clickTarget: patch.clickTarget ?? 'editor', pinnedSettings: null };
    });
    void preferences.setClickTarget('loupe');
    await preferences.setClickTarget('editor');
    expect(order).toEqual(['loupe', 'editor']);
    expect(preferences.clickTarget).toBe('editor');
  });

  it('forgets the account choice on sign-out', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue({ clickTarget: 'loupe', pinnedSettings: null });
    await preferences.load();
    preferences.reset();
    expect(preferences.clickTarget).toBe('editor');
  });

  it('shows no pins until the account answers, then the default set', async () => {
    const preferences = await freshStore();
    expect(preferences.pins).toEqual([]);
    api.getPreferences.mockResolvedValue({ clickTarget: 'editor', pinnedSettings: null });
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
    api.getPreferences.mockResolvedValue({ clickTarget: 'editor', pinnedSettings: ['exposure'] });
    api.patchPreferences.mockResolvedValue({ clickTarget: 'editor', pinnedSettings: null });
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
    api.getPreferences.mockResolvedValue({ clickTarget: 'editor', pinnedSettings: ['exposure'] });
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
    api.getPreferences.mockResolvedValue({ clickTarget: 'editor', pinnedSettings: full });
    await preferences.load();
    await preferences.togglePin('exposure');
    expect(preferences.pins).toHaveLength(MAX_PINNED_SETTINGS);
    expect(api.patchPreferences).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalled();
  });
});
