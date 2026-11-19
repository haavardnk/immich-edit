import { describe, expect, it } from 'vitest';
import { watermarkBox } from './watermark';

const photo = { w: 600, h: 400 };
const mark = { width: 200, height: 100 };

describe('watermark preview box', () => {
  it('sizes the long side from the short edge and insets from the anchor corner', () => {
    expect(watermarkBox(photo, mark, { size: 20, inset: 3, anchor: 'bottom_right' })).toEqual({
      left: 508,
      top: 348,
      width: 80,
      height: 40
    });
    expect(watermarkBox(photo, mark, { size: 20, inset: 3, anchor: 'top_left' })).toEqual({
      left: 12,
      top: 12,
      width: 80,
      height: 40
    });
  });

  it('centres on the middle anchors and never leaves the photo', () => {
    expect(watermarkBox(photo, mark, { size: 20, inset: 50, anchor: 'center' })).toEqual({
      left: 260,
      top: 180,
      width: 80,
      height: 40
    });
    const big = watermarkBox(photo, mark, { size: 100, inset: 50, anchor: 'right' });
    expect(big).toEqual({ left: 0, top: 100, width: 400, height: 200 });
  });
});

describe('watermark preview box against the export renderer', () => {
  it('matches the anchors_place_the_mark_inside_the_inset cases in finish/watermark.rs', () => {
    const photo = { w: 100, h: 50 };
    const mark = { width: 4, height: 4 };
    const at = (anchor: 'top_left' | 'center' | 'bottom_right') =>
      watermarkBox(photo, mark, { size: 20, inset: 10, anchor });
    expect(at('top_left')).toMatchObject({ left: 5, top: 5, width: 10, height: 10 });
    expect(at('center')).toMatchObject({ left: 45, top: 20 });
    expect(at('bottom_right')).toMatchObject({ left: 85, top: 35 });
  });
});
