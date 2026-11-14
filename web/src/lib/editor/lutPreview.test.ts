import { neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { withLut } from './lutPreview';

describe('withLut', () => {
  it('swaps only the LUT and keeps the current amount', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 0.4;
    edits.color.lut_3d = { lut_id: 'kodak', amount: 80 };

    const previewed = withLut(edits, 'teal');

    expect(previewed.color.lut_3d).toEqual({ lut_id: 'teal', amount: 80 });
    expect({ ...previewed, color: { ...previewed.color, lut_3d: edits.color.lut_3d } }).toEqual(
      edits
    );
  });

  it('leaves the source edits untouched', () => {
    const edits = neutralEdits();

    withLut(edits, 'teal');

    expect(edits.color.lut_3d.lut_id).toBeNull();
  });
});
