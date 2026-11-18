import { ui } from '#lib/stores/ui.svelte.ts';
import type { FaceData } from '#lib/stores/zoomTargets.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { viewportMetrics } from '#lib/utils/imageViewport.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ViewportNav } from './viewport';

const faceData = vi.hoisted(() => ({ cached: new Map<string, FaceData>() }));

vi.mock('#lib/stores/zoomTargets.ts', () => ({
  cachedFaceData: (id: string) => faceData.cached.get(id) ?? null,
  loadFaceData: async () => ({ faces: [], savedView: null })
}));

const METRICS = viewportMetrics({ w: 800, h: 600 }, { w: 800, h: 600 }, 1600, 1);

function nav(): ViewportNav {
  return new ViewportNav({ assetId: 'a', edits: neutralEdits(), meta: null });
}

let level: number;

beforeEach(() => {
  level = ui.zoomLevel;
  ui.setFitZoom(50);
  ui.zoomFit();
});

afterEach(() => {
  ui.zoomLevel = level;
  faceData.cached.clear();
});

describe('ViewportNav.panBy', () => {
  it('pans a zoomed view by a tenth of the viewport', () => {
    const viewport = nav();
    viewport.setMetrics(METRICS);
    ui.setView({ zoom: 100, cx: 0.5, cy: 0.5 });
    viewport.panBy([0.1, 0]);
    expect(ui.view.cx).toBeCloseTo(0.55, 9);
    expect(ui.view.cy).toBe(0.5);
  });

  it('starts from the shown view and stops at the photo edge', () => {
    const viewport = nav();
    viewport.setMetrics(METRICS);
    ui.setView({ zoom: 100, cx: 0.95, cy: 0.5 });
    viewport.panBy([0.1, 0]);
    expect(ui.view).toEqual({ zoom: 100, cx: 0.75, cy: 0.5 });
  });

  it('does nothing at fit or before the viewer reports its size', () => {
    const fresh = nav();
    ui.setView({ zoom: 100, cx: 0.5, cy: 0.5 });
    fresh.panBy([0.1, 0]);
    expect(ui.view).toEqual({ zoom: 100, cx: 0.5, cy: 0.5 });
    ui.zoomFit();
    fresh.setMetrics(METRICS);
    fresh.panBy([0.1, 0]);
    expect(ui.view).toEqual({ zoom: null, cx: 0.5, cy: 0.5 });
  });
});

describe('ViewportNav.cycleZoom', () => {
  it('stores the face target limited to the photo edge', () => {
    faceData.cached.set('a', {
      faces: [{ source_w: 1000, source_h: 800, x: 0, y: 0, w: 0.1, h: 0.1 }],
      savedView: null
    });
    ui.zoomLevel = 100;
    const viewport = nav();
    viewport.setMetrics(METRICS);
    viewport.cycleZoom();
    expect(ui.view).toEqual({ zoom: 100, cx: 0.25, cy: 0.25 });
    viewport.cycleZoom();
    expect(ui.view).toEqual({ zoom: null, cx: 0.5, cy: 0.5 });
  });
});
