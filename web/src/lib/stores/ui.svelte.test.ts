import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:editorUi';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('editor layout persistence', () => {
  it('restores and updates collapsed panels and filmstrips', async () => {
    const values = new Map<string, string>();
    values.set(
      storageKey,
      JSON.stringify({
        inspectorWidth: 410,
        filmstripHeight: 88,
        rightCollapsed: true,
        editorFilmstripCollapsed: true,
        loupeFilmstripCollapsed: true
      })
    );
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value)
    });

    const { ui } = await import('./ui.svelte');

    expect(ui.inspectorWidth).toBe(410);
    expect(ui.filmstripHeight).toBe(88);
    expect(ui.rightCollapsed).toBe(true);
    expect(ui.editorFilmstripCollapsed).toBe(true);
    expect(ui.loupeFilmstripCollapsed).toBe(true);

    ui.togglePanels();
    ui.toggleEditorFilmstrip();
    ui.toggleLoupeFilmstrip();

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({
      inspectorWidth: 410,
      filmstripHeight: 88,
      rightCollapsed: false,
      editorFilmstripCollapsed: false,
      loupeFilmstripCollapsed: false
    });
  });
});
