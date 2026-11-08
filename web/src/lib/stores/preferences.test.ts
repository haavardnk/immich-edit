import type { Preferences } from '#lib/api/preferences.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  getPreferences: vi.fn<() => Promise<Preferences>>(),
  patchPreferences: vi.fn<(patch: Partial<Preferences>) => Promise<Preferences>>()
}));

const fail = vi.hoisted(() => vi.fn<(context: string, error: unknown) => void>());

vi.mock('#lib/api/preferences.ts', () => api);
vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { fail } }));

async function freshStore(): Promise<(typeof import('./preferences.svelte'))['preferences']> {
  vi.resetModules();
  return (await import('./preferences.svelte')).preferences;
}

beforeEach(() => {
  api.getPreferences.mockReset();
  api.patchPreferences.mockReset();
  fail.mockReset();
});

describe('preferences', () => {
  it('opens the editor on a click until the account says otherwise', async () => {
    const preferences = await freshStore();
    expect(preferences.clickTarget).toBe('editor');
    api.getPreferences.mockResolvedValue({ clickTarget: 'loupe' });
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
    api.patchPreferences.mockResolvedValue({ clickTarget: 'loupe' });
    const saving = preferences.setClickTarget('loupe');
    expect(preferences.clickTarget).toBe('loupe');
    await saving;
    expect(api.patchPreferences).toHaveBeenCalledWith({ clickTarget: 'loupe' });
  });

  it('keeps a choice made while the account preferences load', async () => {
    const preferences = await freshStore();
    let answer: (value: Preferences) => void = () => {};
    api.getPreferences.mockReturnValue(new Promise((resolve) => (answer = resolve)));
    api.patchPreferences.mockResolvedValue({ clickTarget: 'loupe' });
    const loading = preferences.load();
    await preferences.setClickTarget('loupe');
    answer({ clickTarget: 'editor' });
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
      return { clickTarget: patch.clickTarget ?? 'editor' };
    });
    void preferences.setClickTarget('loupe');
    await preferences.setClickTarget('editor');
    expect(order).toEqual(['loupe', 'editor']);
    expect(preferences.clickTarget).toBe('editor');
  });

  it('forgets the account choice on sign-out', async () => {
    const preferences = await freshStore();
    api.getPreferences.mockResolvedValue({ clickTarget: 'loupe' });
    await preferences.load();
    preferences.reset();
    expect(preferences.clickTarget).toBe('editor');
  });
});
