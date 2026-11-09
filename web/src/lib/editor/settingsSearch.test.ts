import { PARAMS } from '#lib/editor/params/catalog.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import {
  SETTINGS,
  byTouchHome,
  modifiedSettings,
  searchSettings,
  settingValue,
  type Setting
} from './settingsSearch';

function setting(id: string): Setting {
  const found = SETTINGS.find((entry) => entry.id === id);
  if (!found) throw new Error(`no setting ${id}`);
  return found;
}

function labels(query: string): string[] {
  return searchSettings(query).map((match) => match.setting.label);
}

describe('settings search', () => {
  it('lists every catalog parameter once, plus the sections without sliders', () => {
    const ids = SETTINGS.map((entry) => entry.id);

    expect(new Set(ids).size).toBe(ids.length);
    expect(ids.filter((id) => !id.startsWith('section:')).sort()).toEqual(
      PARAMS.map((param) => param.id).sort()
    );
    expect(ids.filter((id) => id.startsWith('section:')).sort()).toEqual([
      'section:curves',
      'section:dcp',
      'section:white_balance'
    ]);
  });

  it('names band and region rows after their band and shows both paths', () => {
    expect(setting('hsl_aqua_sat')).toMatchObject({
      label: 'Aqua saturation',
      desktopPath: 'HSL › Aqua',
      touch: { group: 'color', view: 'mix', path: 'Color › Mix › Aqua' }
    });
    expect(setting('grade_shadows_hue')).toMatchObject({
      label: 'Shadows hue',
      desktopPath: 'Color Grading › Shadows',
      touch: { path: 'Color › Grading › Shadows' }
    });
    expect(setting('exposure')).toMatchObject({
      desktopPath: 'Basic › Tone',
      touch: { group: 'light', view: 'tone', path: 'Light › Tone' }
    });
    expect(setting('section:white_balance').touch).toMatchObject({ group: 'color', view: 'basic' });
  });

  it('ranks label prefixes, then word starts, then substrings, then synonyms and paths', () => {
    const results = labels('sha');

    const wordStart = results.indexOf('B&W shadows hue');

    expect(results.slice(0, 2)).toEqual(['Shadows', 'Shadows hue']);
    expect(results.indexOf('Sharpen amount')).toBeLessThan(wordStart);
    expect(results.indexOf('Shadows hue')).toBeLessThan(wordStart);
    expect(wordStart).toBeGreaterThan(0);
    expect(labels('wb')).toEqual(['White Balance']);
    expect(labels('teal')).toEqual([]);
    expect(labels('aqua sat')[0]).toBe('Aqua saturation');
  });

  it('marks the matched part of the label', () => {
    const [first] = searchSettings('posure');

    expect(first?.setting.label).toBe('Exposure');
    expect(first?.highlight).toEqual([2, 8]);
    expect(searchSettings('stops')[0]?.highlight).toBeNull();
  });

  it('returns every setting for an empty query', () => {
    expect(searchSettings('  ')).toHaveLength(SETTINGS.length);
  });

  it('formats the current value and lists modified settings by touch view', () => {
    const edits = neutralEdits();
    edits.basic.exposure_ev = 0.35;
    edits.basic.vibrance = 12;
    edits.basic.white_balance = { mode: 'relative', temp: 10, tint: 0 };
    edits.color.hsl.bands[1]!.sat = 22;

    expect(settingValue(setting('exposure'), edits, false)).toBe('0.35');
    expect(settingValue(setting('section:white_balance'), edits, false)).toBe('');

    const modified = modifiedSettings(edits, false);
    expect(modified.map((entry) => entry.id)).toEqual([
      'section:white_balance',
      'exposure',
      'vibrance',
      'hsl_orange_sat'
    ]);
    expect(byTouchHome(modified).map((group) => group.home.path)).toEqual([
      'Light › Tone',
      'Color › Basic',
      'Color › Mix › Orange'
    ]);
  });
});
