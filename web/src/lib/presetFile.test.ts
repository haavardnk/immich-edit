import { describe, expect, it } from 'vitest';
import type { Preset } from '$lib/api/presets';
import {
  importSummary,
  newPresets,
  parsePresetFile,
  presetFileJson,
  presetFileName
} from './presetFile';

function preset(name: string, group_name: string | null = null): Preset {
  return {
    id: `id-${name}`,
    name,
    group_name,
    manifest: { schema_version: 3, ops: { basic: { contrast: 20 } } },
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

describe('preset files', () => {
  it('round-trips names, groups and manifests without ids', () => {
    const list = [preset('Warm', 'Film'), preset('Cool')];
    const text = presetFileJson(list);
    expect(text).not.toContain('id-Warm');
    expect(parsePresetFile(text)).toEqual(
      list.map((p) => ({ name: p.name, group_name: p.group_name, manifest: p.manifest }))
    );
  });

  it.each([
    ['not json', 'Not a JSON file'],
    ['{"kind":"other","version":1,"presets":[]}', 'Not an immich-edit preset file'],
    [
      '{"kind":"immich-edit-presets","version":2,"presets":[]}',
      'Unsupported preset file version 2'
    ],
    ['{"kind":"immich-edit-presets","version":1}', 'Preset file has no presets'],
    [
      '{"kind":"immich-edit-presets","version":1,"presets":[{"name":" ","manifest":{"schema_version":3,"ops":{}}}]}',
      'Preset 1 has no name'
    ],
    [
      '{"kind":"immich-edit-presets","version":1,"presets":[{"name":"A","group_name":4,"manifest":{"schema_version":3,"ops":{}}}]}',
      'Preset 1 has an invalid group'
    ],
    [
      '{"kind":"immich-edit-presets","version":1,"presets":[{"name":"A","manifest":{"ops":{}}}]}',
      'Preset 1 has no edit manifest'
    ]
  ])('rejects %s', (text, message) => {
    expect(() => parsePresetFile(text)).toThrow(message);
  });

  it('trims names and turns a blank group into none', () => {
    const text =
      '{"kind":"immich-edit-presets","version":1,"presets":[{"name":" Warm ","group_name":" ","manifest":{"schema_version":3,"ops":{}}}]}';
    expect(parsePresetFile(text)[0]).toMatchObject({ name: 'Warm', group_name: null });
  });

  it('skips names already in the library and repeated in the file', () => {
    const incoming = parsePresetFile(
      presetFileJson([preset('Warm'), preset('Cool'), preset('Cool')])
    );
    const { fresh, skipped } = newPresets(incoming, [preset('Warm')]);
    expect(fresh.map((p) => p.name)).toEqual(['Cool']);
    expect(skipped).toBe(2);
  });

  it.each([
    ['Warm / Film', 'Warm - Film.json'],
    ['***', 'preset.json'],
    ['Warm', 'Warm.json']
  ])('names the file for %s', (name, file) => {
    expect(presetFileName(preset(name))).toBe(file);
  });

  it.each([
    [2, 0, 0, 'Imported 2 presets'],
    [1, 1, 0, 'Imported 1 preset, skipped 1 with a name already in use'],
    [0, 3, 0, 'No presets imported, skipped 3 with a name already in use'],
    [1, 0, 1, 'Imported 1 preset, 1 failed']
  ])('summarises %i imported, %i skipped, %i failed', (imported, skipped, failed, text) => {
    expect(importSummary(imported, skipped, failed)).toBe(text);
  });
});
