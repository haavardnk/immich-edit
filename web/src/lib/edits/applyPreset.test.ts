import { describe, expect, it } from 'vitest';
import { editsToManifest } from '#lib/edits/manifest.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { defaultLinear, makeLayer } from '#lib/types/masks.ts';
import { presetCarries, presetEdits } from './applyPreset';

function look(): Edits {
  const edits = neutralEdits();
  edits.basic.contrast = 40;
  edits.geometry.rotate = 90;
  edits.masks = [makeLayer('Preset mask', defaultLinear())];
  return edits;
}

function current(): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  edits.geometry.flip_h = true;
  edits.masks = [makeLayer('Own mask', defaultLinear())];
  return edits;
}

describe('presetEdits', () => {
  it.each([
    { includeGeometry: false, includeMasks: false, rotate: 0, mask: 'Own mask' },
    { includeGeometry: true, includeMasks: false, rotate: 90, mask: 'Own mask' },
    { includeGeometry: false, includeMasks: true, rotate: 0, mask: 'Preset mask' }
  ])(
    'replaces the look and keeps geometry $includeGeometry, masks $includeMasks',
    ({ includeGeometry, includeMasks, rotate, mask }) => {
      const base = current();
      const out = presetEdits(base, {
        manifest: editsToManifest(look()),
        opts: { includeGeometry, includeMasks, amount: 50 }
      });
      expect(out.basic.contrast).toBe(20);
      expect(out.basic.exposure_ev).toBe(0);
      expect(out.geometry.rotate).toBe(rotate);
      expect(out.masks.map((l) => l.name)).toEqual([mask]);
      expect(out.retouch).toBe(base.retouch);
    }
  );
});

describe('presetCarries', () => {
  it('reports a crop and masks only when the preset holds them', () => {
    const plain = neutralEdits();
    expect(presetCarries(editsToManifest(plain))).toEqual({ geometry: false, masks: false });
    const cropped = neutralEdits();
    cropped.geometry.crop = { x: 0.1, y: 0.1, w: 0.5, h: 0.5 };
    expect(presetCarries(editsToManifest(cropped))).toEqual({ geometry: true, masks: false });
  });
});
