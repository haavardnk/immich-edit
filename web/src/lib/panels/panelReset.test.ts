import { describe, expect, it } from 'vitest';
import { modifiedDevelopPanels } from '#lib/editor/modified.ts';
import { everyDevelopOpModified } from '#lib/testing/developEdits.ts';
import { resetDevelopPanel } from './panelReset';

const PANELS = [...modifiedDevelopPanels(everyDevelopOpModified())];

describe('resetDevelopPanel', () => {
  it('covers every panel that can show as modified', () => {
    expect(PANELS).toEqual([
      'dcp',
      'basic',
      'curves',
      'hsl',
      'color-grading',
      'bw',
      'lut',
      'detail',
      'lens',
      'lens-blur',
      'effects'
    ]);
  });

  it.each(PANELS)('resets %s and leaves the other panels alone', (panelId) => {
    const edits = everyDevelopOpModified();
    const reset = resetDevelopPanel(edits, panelId, false);

    expect([...modifiedDevelopPanels(reset)]).toEqual(PANELS.filter((id) => id !== panelId));
    expect(reset.geometry).toEqual(edits.geometry);
    expect(reset.lens_blur.depth_raster_id).toBe('depth');
  });
});
