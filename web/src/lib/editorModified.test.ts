import { describe, expect, it } from 'vitest';
import { modifiedDevelopPanels, modifiedTabCount } from './editorModified';
import { neutralEdits } from '$lib/types/edits';

describe('editor modified state', () => {
  it('maps active operators to their develop panels', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 1;
    edits.color.hsl.bands[2].sat = 20;
    edits.effects.grain_amount = 10;

    expect([...modifiedDevelopPanels(edits)]).toEqual(['basic', 'hsl', 'effects']);
    expect(modifiedTabCount(edits, 'develop')).toBe(3);
  });

  it('counts mask and retouch items and geometry as one section', () => {
    const edits = neutralEdits();
    edits.geometry.rotate = 90;
    edits.masks = [{ id: 'mask' } as never];
    edits.retouch = [{ id: 'stroke' } as never, { id: 'stroke-2' } as never];

    expect(modifiedTabCount(edits, 'geometry')).toBe(1);
    expect(modifiedTabCount(edits, 'masks')).toBe(1);
    expect(modifiedTabCount(edits, 'retouch')).toBe(2);
  });

  it('reports no modified sections for neutral edits', () => {
    const edits = neutralEdits();

    expect(modifiedDevelopPanels(edits).size).toBe(0);
    expect(modifiedTabCount(edits, 'develop')).toBe(0);
  });
});
