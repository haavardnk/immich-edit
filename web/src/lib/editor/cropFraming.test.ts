import { describe, expect, it } from 'vitest';
import { CROP_FILL, cropFit, framedView, previewView, previewZoom } from './cropFraming';

const box = { w: 400, h: 800 };
const bbox = { w: 6000, h: 4000 };
const fit = cropFit(box, bbox);

describe('cropFraming', () => {
  it('fits the rotated bbox inside the fill margin', () => {
    expect(bbox.w * fit).toBeCloseTo(box.w * CROP_FILL);
  });

  it('matches the preview fit of the cropped photo', () => {
    const crop = { x: 0.25, y: 0, w: 0.5, h: 1 };
    const zoom = previewZoom(crop, bbox, fit, box);
    expect(crop.w * bbox.w * fit * zoom).toBeCloseTo(box.w);
    expect(previewView(crop, bbox, fit, box)).toEqual({ zoom, cx: 0.5, cy: 0.5 });
  });

  it('frames the crop a fill margin smaller than the preview, centred on it', () => {
    const crop = { x: 0.1, y: 0.2, w: 0.4, h: 0.4 };
    const framed = framedView(crop, bbox, fit, box);
    expect(framed.zoom).toBeCloseTo(previewZoom(crop, bbox, fit, box) * CROP_FILL);
    expect(framed.cx).toBeCloseTo(0.3);
    expect(framed.cy).toBeCloseTo(0.4);
  });

  it('never zooms the framed view below the whole photo or past the cap', () => {
    const whole = { x: 0, y: 0, w: 1, h: 1 };
    expect(framedView(whole, bbox, fit, box).zoom).toBe(1);
    expect(previewZoom(whole, bbox, fit, box)).toBeCloseTo(1 / CROP_FILL);
    const tiny = { x: 0.5, y: 0.5, w: 0.01, h: 0.01 };
    expect(framedView(tiny, bbox, fit, box).zoom).toBe(4);
  });
});
