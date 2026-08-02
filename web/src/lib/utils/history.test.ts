import { describe, it, expect } from 'vitest';
import { historyDetails, historyLabel } from './history';
import { neutralEdits } from '$lib/types/edits';
import type { EditHistoryEntry } from '$lib/api/edits';
import type { MaskLayer } from '$lib/types/edits';

function entry(over: Partial<EditHistoryEntry>): EditHistoryEntry {
  return {
    id: 1,
    manifest_hash: 'abcdef1234567890',
    deleted: false,
    edits: neutralEdits(),
    created_at: '2024-01-01T00:00:00Z',
    action: null,
    ...over
  };
}

function mask(id: string, name: string): MaskLayer {
  return {
    id,
    name,
    enabled: true,
    color: '#ff0000',
    amount: 1,
    invert: false,
    components: [
      {
        id: `${id}-component`,
        enabled: true,
        mode: 'add',
        invert: false,
        kind: { kind: 'linear', p0: { x: 0, y: 0 }, p1: { x: 1, y: 1 }, feather: 0.5 },
        source: 'manual'
      }
    ],
    edits: { exposure_ev: 1 }
  };
}

describe('historyLabel', () => {
  it('labels a deletion with its action or a reset fallback', () => {
    expect(historyLabel(entry({ deleted: true, action: 'Reset' }), null).label).toBe('Reset');
    expect(historyLabel(entry({ deleted: true, action: null }), null).label).toBe(
      'Reset to original'
    );
  });

  it('names a single changed field with its delta', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1.5;
    const out = historyLabel(entry({ edits: curr }), null);
    expect(out.label).toBe('Exposure');
    expect(out.delta).toBe('+1.50');
  });

  it('formats a negative single-field delta', () => {
    const curr = neutralEdits();
    curr.tone.shadows = -20;
    const out = historyLabel(entry({ edits: curr }), null);
    expect(out.label).toBe('Shadows');
    expect(out.delta).toBe('-20');
  });

  it('reports multiple changes when more than one field differs', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1;
    curr.basic.contrast = 10;
    const out = historyLabel(entry({ edits: curr, action: null }), null);
    expect(out.label).toBe('Multiple changes');
  });

  it('prefers an explicit action over the multi-change label', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1;
    curr.basic.contrast = 10;
    const out = historyLabel(entry({ edits: curr, action: 'Pasted edits' }), null);
    expect(out.label).toBe('Pasted edits');
  });

  it('diffs against the previous entry, not neutral', () => {
    const prev = neutralEdits();
    prev.basic.exposure_ev = 1;
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1.5;
    const out = historyLabel(
      entry({ id: 2, edits: curr }),
      entry({ id: 1, edits: prev })
    );
    expect(out.label).toBe('Exposure');
    expect(out.delta).toBe('+0.50');
  });

  it('falls back to the hash when nothing changed', () => {
    const out = historyLabel(entry({ edits: neutralEdits(), action: null }), null);
    expect(out.label).toBe('abcdef12');
  });
});

describe('historyDetails', () => {
  it('formats scalar and boolean values against the previous entry', () => {
    const prev = neutralEdits();
    prev.basic.exposure_ev = 0.5;
    prev.detail.sharpen_radius = 1.2;
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1.5;
    curr.detail.sharpen_radius = 2;
    curr.lens.profile_enabled = true;

    const details = historyDetails(entry({ id: 2, edits: curr }), entry({ id: 1, edits: prev }));

    expect(details).toContainEqual({
      key: 'basic',
      label: 'Basic',
      items: [{ kind: 'value', label: 'Exposure', before: '+0.50', after: '+1.50' }]
    });
    expect(details).toContainEqual({
      key: 'detail',
      label: 'Detail',
      items: [{ kind: 'value', label: 'Sharpen Radius', before: '1.2', after: '2.0' }]
    });
    expect(details).toContainEqual({
      key: 'lens',
      label: 'Lens',
      items: [{ kind: 'value', label: 'Profile Corrections', before: 'Off', after: 'On' }]
    });
  });

  it('shows a deletion returning prior values to neutral', () => {
    const prev = neutralEdits();
    prev.tone.shadows = -20;

    const details = historyDetails(
      entry({ id: 2, deleted: true, edits: null }),
      entry({ id: 1, edits: prev })
    );

    expect(details).toContainEqual({
      key: 'tone',
      label: 'Tone',
      items: [{ kind: 'value', label: 'Shadows', before: '-20', after: '0' }]
    });
  });

  it('summarizes curve channels and point counts', () => {
    const prev = neutralEdits();
    const curr = neutralEdits();
    curr.basic.curves.composite = [{ x: 0, y: 0 }, { x: 1, y: 0.9 }];
    curr.basic.curves.r = [{ x: 0, y: 0 }, { x: 0.5, y: 0.6 }, { x: 1, y: 1 }];

    const group = historyDetails(entry({ edits: curr }), entry({ edits: prev }))
      .find((detail) => detail.key === 'curves');

    expect(group?.items).toEqual([
      { kind: 'summary', text: 'RGB: adjusted' },
      { kind: 'summary', text: 'Red: 2 → 3 points' }
    ]);
  });

  it('summarizes mask additions, removals, modifications, and reordering', () => {
    const prev = neutralEdits();
    prev.masks = [mask('a', 'A'), mask('b', 'B')];
    const changed = neutralEdits();
    changed.masks = [mask('b', 'Changed'), mask('c', 'C')];
    const reordered = neutralEdits();
    reordered.masks = [mask('b', 'B'), mask('a', 'A')];

    const changedGroup = historyDetails(entry({ edits: changed }), entry({ edits: prev }))
      .find((detail) => detail.key === 'masks');
    const reorderedGroup = historyDetails(entry({ edits: reordered }), entry({ edits: prev }))
      .find((detail) => detail.key === 'masks');

    expect(changedGroup?.items).toEqual([
      { kind: 'summary', text: '1 added, 1 removed, 1 modified' }
    ]);
    expect(reorderedGroup?.items).toEqual([{ kind: 'summary', text: 'reordered' }]);
  });

  it('summarizes geometry changes', () => {
    const prev = neutralEdits();
    const curr = neutralEdits();
    curr.geometry.rotate = 90;
    curr.geometry.rotate_angle = 2.5;
    curr.geometry.flip_h = true;
    curr.geometry.aspect = { kind: 'ratio', num: 3, den: 2 };
    curr.geometry.crop = { x: 0.1, y: 0.1, w: 0.8, h: 0.8 };

    const group = historyDetails(entry({ edits: curr }), entry({ edits: prev }))
      .find((detail) => detail.key === 'geometry');

    expect(group?.items).toEqual([
      { kind: 'summary', text: 'Rotation: 0° → 90°' },
      { kind: 'summary', text: 'Angle: 0.0° → 2.5°' },
      { kind: 'summary', text: 'Horizontal flip: Off → On' },
      { kind: 'summary', text: 'Aspect: Original → 3:2' },
      { kind: 'summary', text: 'Crop added' }
    ]);
  });

  it('summarizes hidden lens profile changes', () => {
    const prev = neutralEdits();
    const curr = neutralEdits();
    curr.lens.k1 = 0.1;
    curr.lens.ca_red_scale_x10000 = 12;

    const details = historyDetails(entry({ edits: curr }), entry({ edits: prev }));

    expect(details).toContainEqual({
      key: 'lens',
      label: 'Lens',
      items: [{ kind: 'summary', text: 'Profile data changed' }]
    });
  });

  it('summarizes camera-profile and LUT selection changes', () => {
    const prev = neutralEdits();
    const curr = neutralEdits();
    curr.color.dcp.mode = 'off';
    curr.color.lut_3d.lut_id = 'film';
    const group = historyDetails(entry({ edits: curr }), entry({ edits: prev }))
      .find((detail) => detail.key === 'color');
    expect(group?.items).toContainEqual({
      kind: 'summary',
      text: 'Camera profile: Auto → Default Color'
    });
    expect(group?.items).toContainEqual({ kind: 'summary', text: 'LUT selection changed' });
  });

  it('returns no details for equivalent snapshots', () => {
    expect(historyDetails(entry({ edits: neutralEdits() }), null)).toEqual([]);
  });
});
