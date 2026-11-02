import { DEFAULT_APPEARANCE } from '#lib/theme/registry.ts';
import type { Appearance } from '#lib/theme/types.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  getAppearance: vi.fn<() => Promise<Appearance>>(),
  putAppearance: vi.fn<(appearance: Appearance) => Promise<Appearance>>()
}));

const ui = vi.hoisted(() => ({
  setPreference: vi.fn<(preference: string) => void>(),
  fail: vi.fn<(context: string, error: unknown) => void>()
}));

vi.mock('#lib/api/appearance.ts', () => api);

vi.mock('#lib/theme/document.ts', () => ({
  pageThemeRevision: () => 'test',
  readThemeCache: () => null,
  writeTheme: vi.fn(),
  writeSchemeChrome: vi.fn()
}));

vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { fail: ui.fail } }));

vi.mock('@immich/ui', () => ({
  Theme: { Light: 'light', Dark: 'dark' },
  ThemePreference: { Light: 'light', Dark: 'dark', System: 'system' },
  themeManager: { value: 'dark', setPreference: ui.setPreference }
}));

const REMOTE: Appearance = {
  mode: 'dark',
  lightTheme: 'catppuccin-latte',
  darkTheme: 'nord',
  accents: { nord: 'green' }
};

async function freshStore(): Promise<(typeof import('./appearance.svelte'))['appearance']> {
  vi.resetModules();
  return (await import('./appearance.svelte')).appearance;
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((done) => (resolve = done));
  return { promise, resolve };
}

describe('appearance store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.putAppearance.mockImplementation((appearance) => Promise.resolve(appearance));
  });

  it('replays edits made during the boot load onto the server value', async () => {
    const remote = deferred<Appearance>();
    api.getAppearance.mockReturnValue(remote.promise);
    const store = await freshStore();
    store.apply();

    const loading = store.load();
    store.setMode('light');
    expect(api.putAppearance).not.toHaveBeenCalled();
    remote.resolve(REMOTE);
    await loading;
    await vi.waitFor(() => expect(api.putAppearance).toHaveBeenCalledTimes(1));

    expect(store.current).toEqual({ ...REMOTE, mode: 'light' });
    expect(api.putAppearance).toHaveBeenCalledWith({ ...REMOTE, mode: 'light' });
  });

  it('applies the server value without saving it back', async () => {
    api.getAppearance.mockResolvedValue(REMOTE);
    const store = await freshStore();
    store.apply();

    await store.load();

    expect(store.current).toEqual(REMOTE);
    expect(api.putAppearance).not.toHaveBeenCalled();
  });

  it('keeps local state and saves later edits when the load fails', async () => {
    api.getAppearance.mockRejectedValue(new Error('offline'));
    const store = await freshStore();
    store.apply();

    await store.load();
    store.setAccent('immich-edit', 'teal');

    await vi.waitFor(() => expect(api.putAppearance).toHaveBeenCalledTimes(1));
    expect(store.current.accents).toEqual({ 'immich-edit': 'teal' });
  });

  it('only sets the package preference when the mode changes', async () => {
    const store = await freshStore();
    store.apply();
    store.setAccent('immich-edit', 'rose');
    store.setTheme('light', 'nord');
    store.setMode('system');

    expect(ui.setPreference.mock.calls).toEqual([['dark'], ['system']]);
  });

  it('reset restores the defaults locally without saving', async () => {
    api.getAppearance.mockResolvedValue({ ...REMOTE, mode: 'light' });
    const store = await freshStore();
    store.apply();
    await store.load();
    ui.setPreference.mockClear();

    store.reset();

    expect(store.current).toEqual(DEFAULT_APPEARANCE);
    expect(ui.setPreference.mock.calls).toEqual([['dark']]);
    expect(api.putAppearance).not.toHaveBeenCalled();
  });

  it('reports a failed save', async () => {
    api.putAppearance.mockRejectedValue(new Error('boom'));
    const store = await freshStore();
    store.apply();

    store.setMode('light');

    await vi.waitFor(() => expect(ui.fail).toHaveBeenCalledTimes(1));
    expect(ui.fail.mock.calls[0]?.[0]).toBe('Could not save appearance');
  });
});
