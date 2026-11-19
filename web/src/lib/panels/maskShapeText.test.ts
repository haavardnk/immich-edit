import type { MaskLayer } from '#lib/types/edits.ts';
import { defaultLinear, makeComponent } from '#lib/types/masks.ts';
import { describe, expect, it } from 'vitest';
import { shapeLabels, shapeSubtitle } from './maskShapeText';

describe('shapeSubtitle', () => {
  it('describes a gradient by its feather and a hidden shape as hidden', () => {
    const comp = makeComponent(defaultLinear());
    expect(shapeSubtitle(comp)).toBe('Feather 0.50');
    expect(shapeSubtitle({ ...comp, invert: true })).toBe('Feather 0.50 · Inverted');
    expect(shapeSubtitle({ ...comp, enabled: false })).toBe('Hidden');
  });

  it('describes an AI shape by its points, grow and feather', () => {
    const comp = {
      ...makeComponent({ kind: 'brush', raster_id: 'r' }),
      generated: {
        model_id: 'sam',
        kind: 'click',
        prob_raster_id: 'p',
        grow: 4,
        feather: 6,
        points: [{ x: 0.5, y: 0.5, positive: true }]
      }
    };
    expect(shapeSubtitle(comp)).toBe('AI · 1 point · Grow 4 · Feather 6');
  });
});

describe('shapeLabels', () => {
  it('names a generated shape by its kind and class', () => {
    const brush = makeComponent({ kind: 'brush', raster_id: 'r' });
    const scene = (maskClass?: string) => ({
      ...brush,
      generated: {
        model_id: 'seg',
        kind: 'semantic',
        prob_raster_id: 'p',
        grow: 0,
        feather: 0,
        ...(maskClass ? { class: maskClass } : {})
      }
    });
    const layer = { components: [scene('sky'), scene('sky'), scene()] } as unknown as MaskLayer;
    expect(shapeLabels(layer)).toEqual(['Scene · sky 1', 'Scene · sky 2', 'Scene']);
  });
});
