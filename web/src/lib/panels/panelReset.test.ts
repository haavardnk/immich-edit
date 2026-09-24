import { describe, expect, it } from 'vitest';
import { modifiedDevelopPanels } from '$lib/editorModified';
import { neutralEdits, type Edits } from '$lib/types/edits';
import { resetDevelopPanel } from './panelReset';

function everyPanelModified(): Edits {
  const edits = neutralEdits();
  edits.color.dcp.mode = 'flat';
  edits.basic.exposure_ev = 1;
  edits.tone.shadows = 20;
  edits.basic.curves.composite = [
    { x: 0, y: 0 },
    { x: 0.5, y: 0.6 },
    { x: 1, y: 1 }
  ];
  edits.color.hsl.bands[2]!.sat = 20;
  edits.color.color_grade.shadows.sat = 30;
  edits.color.lut_3d.lut_id = 'lut-1';
  edits.detail.luma_nr_amount = 25;
  edits.lens.k1 = 0.1;
  edits.effects.grain_amount = 10;
  edits.geometry.rotate = 90;
  return edits;
}

const PANELS = [...modifiedDevelopPanels(everyPanelModified())];

describe('resetDevelopPanel', () => {
  it('covers every panel that can show as modified', () => {
    expect(PANELS).toEqual([
      'dcp',
      'basic',
      'curves',
      'hsl',
      'color-grading',
      'lut',
      'detail',
      'lens',
      'effects'
    ]);
  });

  it.each(PANELS)('resets %s and leaves the other panels alone', (panelId) => {
    const edits = everyPanelModified();
    const reset = resetDevelopPanel(edits, panelId);

    expect([...modifiedDevelopPanels(reset)]).toEqual(PANELS.filter((id) => id !== panelId));
    expect(reset.geometry).toEqual(edits.geometry);
  });
});
