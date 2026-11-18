import type { FaceBox } from '#lib/api/faces.ts';
import type { FaceData } from '#lib/stores/zoomTargets.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { viewTransform } from '#lib/utils/canvasCoords.ts';
import { FIT_VIEW, viewportMetrics, type View } from '#lib/utils/imageViewport.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ZoomCycle, type ZoomCycleSpec } from './zoomCycle';

const faceData = vi.hoisted(() => ({
  cached: new Map<string, FaceData>(),
  pending: new Map<string, FaceData>()
}));

vi.mock('#lib/stores/zoomTargets.ts', () => ({
  cachedFaceData: (id: string) => faceData.cached.get(id) ?? null,
  loadFaceData: async (id: string) => {
    const data = faceData.pending.get(id) ?? { faces: [], savedView: null };
    faceData.cached.set(id, data);
    return data;
  }
}));

const dims = { source_w: 1000, source_h: 800 };

const METRICS = viewportMetrics({ w: 800, h: 600 }, { w: 800, h: 600 }, 1600, 1);

function face(x: number, y: number, w: number, h: number): FaceBox {
  return { ...dims, x, y, w, h };
}

const TWO_FACES: FaceData = {
  faces: [face(0.1, 0.1, 0.05, 0.05), face(0.6, 0.6, 0.2, 0.2)],
  savedView: viewTransform(neutralEdits(), dims)
};

function setup(extra: Partial<ZoomCycleSpec> = {}) {
  let shown: string | null = 'a';
  let view: View = FIT_VIEW;
  const apply = vi.fn((_id: string, next: View) => (view = next));
  const cycle = new ZoomCycle({
    shown: () => shown,
    transform: (data) => data.savedView,
    image: () => null,
    metrics: () => null,
    zoomLevel: () => 300,
    view: () => view,
    apply,
    ...extra
  });
  return {
    cycle,
    apply,
    show: (id: string | null) => (shown = id),
    setView: (next: View) => (view = next),
    current: () => view
  };
}

afterEach(() => {
  faceData.cached.clear();
  faceData.pending.clear();
});

describe('ZoomCycle', () => {
  it('steps through faces from the largest, then back to fit', () => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, current } = setup();
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.125, cy: 0.125 });
    cycle.next();
    expect(current()).toEqual(FIT_VIEW);
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });

  it('zooms to the remembered level', () => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, current } = setup({ zoomLevel: () => 150 });
    cycle.next();
    expect(current()).toEqual({ zoom: 150, cx: 0.7, cy: 0.7 });
  });

  it('restarts from the first target after the view changed', () => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, setView, current } = setup();
    cycle.next();
    setView({ zoom: 300, cx: 0.6, cy: 0.7 });
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });

  it('restarts after a reset', () => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, current } = setup();
    cycle.next();
    cycle.reset();
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });

  it('stores targets limited to the photo edge and keeps stepping', () => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, current } = setup({ metrics: () => METRICS, zoomLevel: () => 100 });
    cycle.next();
    expect(current()).toEqual({ zoom: 100, cx: 0.7, cy: 0.7 });
    cycle.next();
    expect(current()).toEqual({ zoom: 100, cx: 0.25, cy: 0.25 });
    cycle.next();
    expect(current()).toEqual(FIT_VIEW);
  });

  it.each([50, 40])('goes straight to fit when level %s is not above fit', (level) => {
    faceData.cached.set('a', TWO_FACES);
    const { cycle, setView, current, apply } = setup({
      metrics: () => METRICS,
      zoomLevel: () => level
    });
    setView({ zoom: 300, cx: 0.7, cy: 0.7 });
    cycle.next();
    expect(current()).toEqual(FIT_VIEW);
    cycle.next();
    expect(current()).toEqual(FIT_VIEW);
    expect(apply).toHaveBeenCalledTimes(2);
  });

  it('starts from the first target once the level rises above fit', () => {
    faceData.cached.set('a', TWO_FACES);
    let level = 40;
    const { cycle, current } = setup({ metrics: () => METRICS, zoomLevel: () => level });
    cycle.next();
    level = 300;
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });

  it('loads face data before the first step', async () => {
    faceData.pending.set('a', TWO_FACES);
    const { cycle, apply, current } = setup();
    cycle.next();
    expect(apply).not.toHaveBeenCalled();
    await vi.waitFor(() => expect(apply).toHaveBeenCalledOnce());
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });

  it('drops a step whose photo is no longer shown', async () => {
    faceData.pending.set('a', TWO_FACES);
    const { cycle, apply, show } = setup();
    cycle.next();
    show('b');
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(faceData.cached.has('a')).toBe(true);
    expect(apply).not.toHaveBeenCalled();
  });

  it('does nothing without a photo', () => {
    const { cycle, apply, show } = setup();
    show(null);
    cycle.next();
    expect(apply).not.toHaveBeenCalled();
  });

  it('falls back to the centre when faces cannot be placed', () => {
    faceData.cached.set('a', { ...TWO_FACES, savedView: null });
    const { cycle, current } = setup();
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.5, cy: 0.5 });
    cycle.next();
    expect(current()).toEqual(FIT_VIEW);
  });

  it('uses the transform the caller supplies', () => {
    faceData.cached.set('a', { ...TWO_FACES, savedView: null });
    const { cycle, current } = setup({ transform: () => viewTransform(neutralEdits(), dims) });
    cycle.next();
    expect(current()).toEqual({ zoom: 300, cx: 0.7, cy: 0.7 });
  });
});
