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
        loupeFilmstripCollapsed: true,
        developOpenPanels: ['curves'],
        greyCanvas: true
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
    expect(ui.developOpenPanels).toEqual(['curves']);
    expect(ui.greyCanvas).toBe(true);

    ui.togglePanels();
    ui.toggleEditorFilmstrip();
    ui.toggleLoupeFilmstrip();
    ui.toggleGreyCanvas();
    ui.setDevelopPanels(['curves', 'hsl']);

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({
      inspectorWidth: 410,
      filmstripHeight: 88,
      rightCollapsed: false,
      editorFilmstripCollapsed: false,
      loupeFilmstripCollapsed: false,
      developOpenPanels: ['curves', 'hsl'],
      brushTool: { size: 0.08, hardness: 0.5, flow: 0.8, mode: 'paint' },
      retouchTool: { mode: 'heal', size: 0.05, hardness: 0.5, opacity: 1 },
      greyCanvas: false,
      cropGrid: 'thirds'
    });
  });

  it.each([
    ['golden', 'diagonal'],
    ['off', 'thirds'],
    ['bogus', 'golden']
  ])('restores crop guide %s and cycles to %s', async (stored, next) => {
    const values = new Map<string, string>([[storageKey, JSON.stringify({ cropGrid: stored })]]);
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value)
    });

    const { ui } = await import('./ui.svelte');
    ui.cycleCropGrid();

    expect(ui.cropGrid).toBe(next);
    expect(JSON.parse(values.get(storageKey) ?? '').cropGrid).toBe(next);
  });

  it('restores brush and retouch tools inside their ranges', async () => {
    const values = new Map<string, string>();
    values.set(
      storageKey,
      JSON.stringify({
        brushTool: { size: 0.12, hardness: 0.3, flow: 5, mode: 'erase' },
        retouchTool: { mode: 'clone', size: 'big', hardness: -1, opacity: 0.4 }
      })
    );
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value)
    });

    const { ui } = await import('./ui.svelte');

    expect(ui.brushTool).toEqual({ size: 0.12, hardness: 0.3, flow: 1, mode: 'erase' });
    expect(ui.retouchTool).toEqual({ mode: 'clone', size: 0.05, hardness: 0, opacity: 0.4 });

    ui.setRetouchTool({ ...ui.retouchTool, size: 0.02 });
    expect(JSON.parse(values.get(storageKey) ?? '').retouchTool.size).toBe(0.02);
  });
});
