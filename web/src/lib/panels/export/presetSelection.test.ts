import { afterEach, describe, expect, it } from 'vitest';
import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { exportPresets } from './exportPresets.svelte';
import { PresetSelection } from './presetSelection.svelte';

function preset(id: string, destination: ExportPreset['destination']): ExportPreset {
  return {
    id,
    name: id,
    form: {},
    destination,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

afterEach(() => {
  exportPresets.presets = [];
});

describe('PresetSelection', () => {
  it('counts one job per destination', () => {
    exportPresets.presets = [
      preset('a', 'download'),
      preset('b', 'download'),
      preset('c', 'immich')
    ];
    const selection = new PresetSelection();
    selection.toggle('a', true);
    expect(selection.note).toBe('');
    selection.toggle('b', true);
    expect(selection.note).toBe('2 selected · runs as one job');
    selection.toggle('c', true);
    expect(selection.note).toBe('3 selected · runs as 2 jobs');

    selection.drop('download');
    expect(selection.picked.map((p) => p.id)).toEqual(['c']);
  });

  it('forgets a deleted preset', () => {
    exportPresets.presets = [preset('a', 'download'), preset('b', 'immich')];
    const selection = new PresetSelection();
    selection.toggle('a', true);
    exportPresets.presets = [preset('b', 'immich')];
    expect(selection.picked).toEqual([]);
  });

  it('caps a job at 16 presets per destination', () => {
    exportPresets.presets = Array.from({ length: 17 }, (_, i) => preset(`p${i}`, 'immich'));
    const selection = new PresetSelection();
    for (const { id } of exportPresets.presets.slice(0, 16)) selection.toggle(id, true);
    expect(selection.tooMany).toBe(false);
    selection.toggle('p16', true);
    expect(selection.tooMany).toBe(true);
    expect(selection.note).toBe('Up to 16 presets per destination');
  });
});
