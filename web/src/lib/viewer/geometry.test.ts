import { FIT_VIEW } from '#lib/utils/imageViewport.ts';
import { viewerFrame } from '#lib/viewer/geometry.ts';
import { describe, expect, it } from 'vitest';

describe('viewerFrame', () => {
  it('fits and centres the photo in the pane', () => {
    const placed = viewerFrame({ w: 1000, h: 1000 }, { w: 2000, h: 1000 }, FIT_VIEW, 4000, 1);
    expect(placed).toEqual({ frame: { left: 0, top: 250, width: 1000, height: 500 }, zoom: 25 });
  });

  it('keeps a zoom percentage at the viewer size', () => {
    const view = { zoom: 100, cx: 0.5, cy: 0.5 };
    const placed = viewerFrame({ w: 1000, h: 1000 }, { w: 2000, h: 1000 }, view, 4000, 1);
    expect(placed?.zoom).toBe(100);
    expect(placed?.frame.width).toBe(4000);
    expect(placed?.frame.left).toBe(-1500);
  });

  it('waits for a measured pane and image', () => {
    expect(viewerFrame({ w: 0, h: 800 }, { w: 2000, h: 1000 }, FIT_VIEW, 4000, 1)).toBeNull();
    expect(viewerFrame({ w: 800, h: 800 }, { w: 0, h: 0 }, FIT_VIEW, 4000, 1)).toBeNull();
  });
});
