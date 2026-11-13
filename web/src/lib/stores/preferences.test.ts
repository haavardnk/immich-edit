import type { Preferences } from '#lib/api/preferences.ts';
import {
  DEFAULT_PINNED_SETTINGS,
  MAX_CROP_RATIOS,
  MAX_PINNED_SETTINGS
} from './preferences.svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  getPreferences: vi.fn<() => Promise<Preferences>>(),
  patchPreferences: vi.fn<(patch: Partial<Preferences>) => Promise<Preferences>>()
}));

const fail = vi.hoisted(() => vi.fn<(context: string, error: unknown) => void>());
const push = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/preferences.ts', () => api);
vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { fail, push } }));

function remote(fields: Partial<Preferences>): Preferences {
  return {
    clickTarget: 'editor',
    pinnedSettings: null,
    shortcuts: {},
    desktopSections: null,
    touchSubviews: null,
    cropRatios: null,
    maskOverlayColor: null,
    ...fields
  };
}

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
    api.getPreferences.mockResolvedValue(remote({ clickTarget: 'loupe', pinnedSettings: null }));
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
    api.patchPreferences.mockResolvedValue(remote({ clickTarget: 'loupe', pinnedSettings: null }));
    const saving = preferences.setClickTarget('loupe');
    expect(preferences.clickTarget).toBe('loupe');
    await saving;
    expect(api.patchPreferences).toHaveBeenCalledWith({ clickTarget: 'loupe' });
  });

  it('keeps a choice made while the account preferences load', async () => {
    const preferences = await freshStore();
    let answer: (value: Preferences) => void = () => {};
    api.getPreferences.mockReturnValue(new Promise((resolve) => (answer = resolve)));
    api.patchPreferences.mockResolvedValue(remote({ clickTarget: 'loupe', pinnedSettings: null }));
    const loading = preferences.load();
    await preferences.setClickTarget('loupe');
    answer(remote({ clickTarget: 'editor', pinnedSettings: null }));
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
      return remote({ clickTarget: patch.clickTarget ?? 'editor' });
    });
    void preferences.setClickTarget('loupe');
    await preferences.setClickTarget('editor');
    expect(order).toEqual(['loupe', 'editor']);
    expect(preferences.clickTarget).toBe('editor');
  });

  it('forgets the account choice on sign-out', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue(remote({ clickTarget: 'loupe', pinnedSettings: null }));
    await preferences.load();
    preferences.reset();
    expect(preferences.clickTarget).toBe('editor');
  });

  it('shows no pins until the account answers, then the default set', async () => {
    const preferences = await freshStore();
    expect(preferences.pins).toEqual([]);
    api.getPreferences.mockResolvedValue(remote({ clickTarget: 'editor', pinnedSettings: null }));
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
    api.getPreferences.mockResolvedValue(
      remote({ clickTarget: 'editor', pinnedSettings: ['exposure'] })
    );
    api.patchPreferences.mockResolvedValue(remote({ clickTarget: 'editor', pinnedSettings: null }));
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
    api.getPreferences.mockResolvedValue(
      remote({ clickTarget: 'editor', pinnedSettings: ['exposure'] })
    );
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
    api.getPreferences.mockResolvedValue(remote({ clickTarget: 'editor', pinnedSettings: full }));
    await preferences.load();
    await preferences.togglePin('exposure');
    expect(preferences.pins).toHaveLength(MAX_PINNED_SETTINGS);
    expect(api.patchPreferences).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalled();
  });

  it('keeps shortcuts and section lists from the account', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue(
      remote({
        shortcuts: { autoAdjust: ['Mod+Shift+a'] },
        desktopSections: [{ id: 'lens-blur', hidden: true }],
        touchSubviews: [{ id: 'profile.lut', hidden: true }]
      })
    );
    await preferences.load();
    expect(preferences.shortcuts).toEqual({ autoAdjust: ['Mod+Shift+a'] });
    expect(preferences.desktopSections).toEqual([{ id: 'lens-blur', hidden: true }]);
    expect(preferences.touchSubviews).toEqual([{ id: 'profile.lut', hidden: true }]);
    preferences.reset();
    expect(preferences.shortcuts).toEqual({});
  });

  it('keeps the saved crop ratios from the account and forgets them on sign-out', async () => {
    const preferences = await freshStore();
    expect(preferences.savedRatios).toEqual([]);
    api.getPreferences.mockResolvedValue(remote({ cropRatios: [{ num: 3, den: 2 }] }));
    await preferences.load();
    expect(preferences.savedRatios).toEqual([{ num: 3, den: 2 }]);
    preferences.reset();
    expect(preferences.cropRatios).toBeNull();
    expect(preferences.savedRatios).toEqual([]);
  });

  it('saves a crop ratio long side first and only once', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue(remote({ cropRatios: [{ num: 3, den: 2 }] }));
    api.patchPreferences.mockResolvedValue(remote({}));
    await preferences.load();
    await preferences.saveCropRatio({ num: 9, den: 16 });
    expect(preferences.savedRatios).toEqual([
      { num: 3, den: 2 },
      { num: 16, den: 9 }
    ]);
    expect(api.patchPreferences).toHaveBeenLastCalledWith({
      cropRatios: [
        { num: 3, den: 2 },
        { num: 16, den: 9 }
      ]
    });
    await preferences.saveCropRatio({ num: 2, den: 3 });
    expect(api.patchPreferences).toHaveBeenCalledTimes(1);
  });

  it('stops saving crop ratios at the server limit', async () => {
    const preferences = await freshStore();
    const full = Array.from({ length: MAX_CROP_RATIOS }, (_, index) => ({
      num: index + 2,
      den: 1
    }));
    api.getPreferences.mockResolvedValue(remote({ cropRatios: full }));
    await preferences.load();
    await preferences.saveCropRatio({ num: 5, den: 4 });
    expect(preferences.savedRatios).toHaveLength(MAX_CROP_RATIOS);
    expect(api.patchPreferences).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalled();
  });

  it('removes a crop ratio in either orientation and restores it when refused', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue(
      remote({
        cropRatios: [
          { num: 3, den: 2 },
          { num: 16, den: 9 }
        ]
      })
    );
    api.patchPreferences.mockResolvedValue(remote({}));
    await preferences.load();
    await preferences.removeCropRatio({ num: 2, den: 3 });
    expect(preferences.savedRatios).toEqual([{ num: 16, den: 9 }]);
    expect(api.patchPreferences).toHaveBeenLastCalledWith({ cropRatios: [{ num: 16, den: 9 }] });
    await preferences.removeCropRatio({ num: 4, den: 3 });
    expect(api.patchPreferences).toHaveBeenCalledTimes(1);
    api.patchPreferences.mockRejectedValue(new Error('refused'));
    await preferences.removeCropRatio({ num: 16, den: 9 });
    expect(preferences.savedRatios).toEqual([{ num: 16, den: 9 }]);
    expect(fail).toHaveBeenCalled();
  });

  it('saves the whole shortcut map and restores it when refused', async () => {
    const preferences = await freshStore();
    api.patchPreferences.mockRejectedValue(new Error('refused'));
    await preferences.setShortcuts({ undo: ['Mod+y'] });
    expect(api.patchPreferences).toHaveBeenCalledWith({ shortcuts: { undo: ['Mod+y'] } });
    expect(preferences.shortcuts).toEqual({});
  });
});
