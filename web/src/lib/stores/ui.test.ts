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
    { strip: false, collapsed: false, toggles: ['presets'], end: [true, false] },
    { strip: true, collapsed: false, toggles: ['presets'], end: [false, true] },
    { strip: true, collapsed: false, toggles: ['filmstrip'], end: [false, false] },
    { strip: false, collapsed: true, toggles: ['presets', 'presets'], end: [false, true] },
    { strip: false, collapsed: true, toggles: ['presets', 'filmstrip'], end: [false, false] },
    { strip: true, collapsed: true, toggles: ['chrome'], end: [false, true] }
  ])(
    'strip $strip, collapsed $collapsed ends at $end after $toggles',
    async ({ strip, collapsed, toggles, end }) => {
      vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => {} });
      const { ui } = await import('./ui.svelte');
      ui.presetStrip = strip;
      ui.editorFilmstripCollapsed = collapsed;
      const actions = {
        presets: ui.togglePresetStrip,
        filmstrip: ui.toggleEditorFilmstrip,
        chrome: ui.toggleChrome
      };
      for (const t of toggles) actions[t as keyof typeof actions]();
      expect([ui.presetStrip, ui.editorFilmstripCollapsed]).toEqual(end);
    }
  );

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

async function viewStore(values = new Map<string, string>()) {
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  const { ui } = await import('./ui.svelte');
  ui.setFitZoom(50);
  return ui;
}

describe('editor view', () => {
  it('starts at fit, centred', async () => {
    const ui = await viewStore();
    expect(ui.view).toEqual({ zoom: null, cx: 0.5, cy: 0.5 });
    expect(ui.zoom).toBe(50);
  });

  it('sets a zoomed view without remembering its zoom', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 200, cx: 0.3, cy: 0.7 });
    expect(ui.view).toEqual({ zoom: 200, cx: 0.3, cy: 0.7 });
    expect(ui.fitMode).toBe(false);
    expect(ui.zoomed).toBe(true);
    expect(ui.zoomLevel).toBe(100);
  });

  it('centres a view that is not above fit', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 200, cx: 0.3, cy: 0.7 });
    ui.setView({ zoom: 40, cx: 0.1, cy: 0.9 });
    expect(ui.view).toEqual({ zoom: 40, cx: 0.5, cy: 0.5 });
  });

  it('clamps the zoom through setZoom', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 5000, cx: 0.3, cy: 0.7 });
    expect(ui.view).toEqual({ zoom: 800, cx: 0.3, cy: 0.7 });
  });

  it('goes back to fit for a fit view', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 200, cx: 0.3, cy: 0.7 });
    ui.setView({ zoom: null, cx: 0.2, cy: 0.2 });
    expect(ui.view).toEqual({ zoom: null, cx: 0.5, cy: 0.5 });
    expect(ui.zoom).toBe(50);
  });

  it('remembers the zoom of a zoom gesture only when zoomed', async () => {
    const values = new Map<string, string>();
    const ui = await viewStore(values);
    ui.applyView({ zoom: 40, cx: 0.3, cy: 0.7 }, 'zoom');
    expect(ui.zoomLevel).toBe(100);
    ui.applyView({ zoom: 250, cx: 0.3, cy: 0.7 }, 'zoom');
    expect(ui.zoomLevel).toBe(250);
    expect(ui.view).toEqual({ zoom: 250, cx: 0.3, cy: 0.7 });
    expect(JSON.parse(values.get('immich-edit:zoomLevel') ?? '')).toEqual({ zoom: 250 });
  });

  it('does not remember the zoom of a pan', async () => {
    const values = new Map<string, string>();
    const ui = await viewStore(values);
    ui.applyView({ zoom: 250, cx: 0.3, cy: 0.7 }, 'pan');
    expect(ui.view).toEqual({ zoom: 250, cx: 0.3, cy: 0.7 });
    expect(ui.zoomLevel).toBe(100);
    expect(values.has('immich-edit:zoomLevel')).toBe(false);
  });

  it('remembers a zoom level only above the given fit', async () => {
    const values = new Map<string, string>();
    const ui = await viewStore(values);
    ui.rememberZoomLevel(150, 200);
    expect(ui.zoomLevel).toBe(100);
    ui.rememberZoomLevel(150, 120);
    expect(ui.zoomLevel).toBe(150);
    ui.rememberZoomLevel(40);
    expect(ui.zoomLevel).toBe(150);
    expect(JSON.parse(values.get('immich-edit:zoomLevel') ?? '')).toEqual({ zoom: 150 });
  });

  it('keeps the view centre when zooming', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 200, cx: 0.3, cy: 0.7 });
    ui.zoomIn();
    expect(ui.view.cx).toBe(0.3);
    expect(ui.view.cy).toBe(0.7);
    ui.setZoom(400);
    expect(ui.view).toEqual({ zoom: 400, cx: 0.3, cy: 0.7 });
  });

  it.each(['zoomFit', 'zoomOut'] as const)('%s recentres the view', async (action) => {
    const ui = await viewStore();
    ui.setView({ zoom: 66, cx: 0.4, cy: 0.6 });
    ui[action]();
    expect(ui.view.cx).toBe(0.5);
    expect(ui.view.cy).toBe(0.5);
  });

  it('keeps the centre when the fit zoom changes', async () => {
    const ui = await viewStore();
    ui.setView({ zoom: 200, cx: 0.3, cy: 0.7 });
    ui.setFitZoom(80);
    expect(ui.view).toEqual({ zoom: 200, cx: 0.3, cy: 0.7 });
  });
});
