import { neutralEdits, type Edits, type RetouchStroke } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import {
  COPY_TOOLS,
  DEFAULT_COPY_SELECTION,
  DEVELOP_ROWS,
  DEVELOP_TOOLS,
  TOOL_LABELS,
  applyCopyTools,
  selectedTools,
  selectionOf,
  toolsState,
  withTools
} from './copyTools';

function stroke(id: string): RetouchStroke {
  return {
    id,
    mode: 'heal',
    points: [{ x: 0.5, y: 0.5 }],
    radius: 0.02,
    hardness: 0.5,
    opacity: 1,
    source: { x: 0.6, y: 0.6 },
    enabled: true
  };
}

function perturb(value: unknown): unknown {
  if (value === null) return 7;
  if (typeof value === 'number') return value + 1.5;
  if (typeof value === 'boolean') return !value;
  if (typeof value === 'string') return `${value}~`;
  if (Array.isArray(value)) return value.map(perturb);
  if (typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, perturb(v)]));
  }
  return value;
}

function incoming(): Edits {
  const edits = neutralEdits();
  edits.basic.white_balance = { mode: 'absolute', kelvin: 4200, tint: -20 };
  edits.basic.exposure_ev = 1;
  edits.basic.clarity = 15;
  edits.basic.curves.composite = [
    { x: 0, y: 0 },
    { x: 1, y: 0.8 }
  ];
  edits.tone.highlights = -30;
  edits.color.hsl.bands[0] = { hue: 12, sat: 0, lum: 0 };
  edits.color.bw.enabled = true;
  edits.detail.luma_nr_amount = 20;
  edits.detail.sharpen_amount = 60;
  edits.effects.vignette_amount = 40;
  edits.effects.grain_amount = 30;
  edits.geometry.rotate = 90;
  edits.retouch = [stroke('incoming')];
  return edits;
}

describe('copy tool catalog', () => {
  it('lists every develop section once, in inspector order', () => {
    expect(DEVELOP_TOOLS).toEqual([
      'dcp',
      'white_balance',
      'tone',
      'presence',
      'curves',
      'hsl',
      'color_grading',
      'bw',
      'lut',
      'sharpening',
      'noise_reduction',
      'lens',
      'lens_blur',
      'vignette',
      'grain'
    ]);
    expect(COPY_TOOLS).toHaveLength(18);
  });

  it('nests the sections of multi-section panels under the panel', () => {
    const nested = DEVELOP_ROWS.filter((row) => row.children.length > 0);
    expect(nested.map((row) => [row.label, row.children.map((c) => c.label)])).toEqual([
      ['Basic', ['White Balance', 'Tone', 'Presence']],
      ['Detail', ['Sharpening', 'Noise Reduction']],
      ['Effects', ['Vignette', 'Grain']]
    ]);
  });

  it('names single-section tools after their panel', () => {
    expect(TOOL_LABELS.bw).toBe('Black & White');
    expect(TOOL_LABELS.lens).toBe('Lens Corrections');
    expect(TOOL_LABELS.lens_blur).toBe('Lens Blur');
    expect(TOOL_LABELS.tone).toBe('Tone');
    expect(TOOL_LABELS.geometry).toBe('Geometry & crop');
  });
});

describe('copy selection', () => {
  it('checks every develop tool and leaves photo-specific tools off by default', () => {
    expect(selectedTools(DEFAULT_COPY_SELECTION)).toEqual(DEVELOP_TOOLS);
  });

  it('reports a partly checked group as mixed', () => {
    const tools = ['white_balance', 'tone', 'presence'] as const;
    expect(toolsState(selectionOf([]), tools)).toBe(false);
    expect(toolsState(selectionOf(['tone']), tools)).toBe('mixed');
    expect(toolsState(withTools(selectionOf([]), tools, true), tools)).toBe(true);
  });
});

describe('applyCopyTools', () => {
  it('copies HSL alone and leaves curves and black & white', () => {
    const out = applyCopyTools(neutralEdits(), incoming(), selectionOf(['hsl']));
    expect(out.color.hsl).toEqual(incoming().color.hsl);
    expect(out.basic.curves).toEqual(neutralEdits().basic.curves);
    expect(out.color.bw.enabled).toBe(false);
    expect(out.basic.exposure_ev).toBe(0);
  });

  it('copies white balance alone and leaves tone and presence', () => {
    const out = applyCopyTools(neutralEdits(), incoming(), selectionOf(['white_balance']));
    expect(out.basic.white_balance).toEqual(incoming().basic.white_balance);
    expect(out.basic.exposure_ev).toBe(0);
    expect(out.basic.clarity).toBe(0);
    expect(out.tone.highlights).toBe(0);
  });

  it('copies tone across the basic exposure and the tone regions', () => {
    const out = applyCopyTools(neutralEdits(), incoming(), selectionOf(['tone']));
    expect(out.basic.exposure_ev).toBe(1);
    expect(out.tone.highlights).toBe(-30);
    expect(out.basic.clarity).toBe(0);
  });

  it('splits detail and effects per tool', () => {
    const out = applyCopyTools(
      neutralEdits(),
      incoming(),
      selectionOf(['noise_reduction', 'grain'])
    );
    expect(out.detail.luma_nr_amount).toBe(20);
    expect(out.detail.sharpen_amount).toBeNull();
    expect(out.effects.grain_amount).toBe(30);
    expect(out.effects.vignette_amount).toBe(0);
  });

  it('copies lens blur settings but never the depth map', () => {
    const current = neutralEdits();
    current.lens_blur.depth_raster_id = 'mine';
    const other = neutralEdits();
    other.lens_blur = { ...other.lens_blur, amount: 60, focus_max: 0.6, depth_raster_id: 'theirs' };

    const out = applyCopyTools(current, other, selectionOf(['lens_blur']));
    expect(out.lens_blur).toEqual({ ...other.lens_blur, depth_raster_id: 'mine' });

    const kept = applyCopyTools(current, other, selectionOf(['hsl']));
    expect(kept.lens_blur).toEqual(current.lens_blur);
  });

  it('carries geometry, masks and retouch only when checked', () => {
    const current = neutralEdits();
    current.retouch = [stroke('current')];
    const other = incoming();

    const kept = applyCopyTools(current, other, DEFAULT_COPY_SELECTION);
    expect(kept.retouch).toEqual([stroke('current')]);
    expect(kept.geometry.rotate).toBe(0);

    const pasted = applyCopyTools(current, other, selectionOf(['geometry', 'retouch']));
    expect(pasted.retouch).toEqual([stroke('incoming')]);
    expect(pasted.geometry.rotate).toBe(90);
    expect(pasted.effects.vignette_amount).toBe(0);
  });

  it('copies every field but the depth map when every tool is checked', () => {
    const current = neutralEdits();
    const other = perturb(neutralEdits()) as Edits;
    const out = applyCopyTools(current, other, selectionOf(COPY_TOOLS));
    expect(out).toEqual({
      ...other,
      lens_blur: { ...other.lens_blur, depth_raster_id: current.lens_blur.depth_raster_id }
    });
  });

  it('keeps the current edits when nothing is checked', () => {
    const current = incoming();
    expect(applyCopyTools(current, neutralEdits(), selectionOf([]))).toEqual(current);
  });
});
