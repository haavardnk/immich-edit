import type { Preset, PresetInput } from '$lib/api/presets';

export const PRESET_FILE_KIND = 'immich-edit-presets';
export const PRESET_FILE_VERSION = 1;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function presetFileJson(presets: Preset[]): string {
  const body = {
    kind: PRESET_FILE_KIND,
    version: PRESET_FILE_VERSION,
    presets: presets.map((p) => ({ name: p.name, group_name: p.group_name, manifest: p.manifest }))
  };
  return `${JSON.stringify(body, null, 2)}\n`;
}

export const PRESET_LIBRARY_FILE = `${PRESET_FILE_KIND}.json`;

export function presetFileName(preset: Preset): string {
  const stem = preset.name.replace(/[^\w -]+/g, '-').replace(/^[\s-]+|[\s-]+$/g, '');
  return `${stem || 'preset'}.json`;
}

function parseEntry(entry: unknown, index: number): PresetInput {
  const where = `Preset ${index + 1}`;
  if (!isRecord(entry)) throw new Error(`${where} is not an object`);
  const { name, group_name, manifest } = entry;
  if (typeof name !== 'string' || !name.trim()) throw new Error(`${where} has no name`);
  if (group_name !== undefined && group_name !== null && typeof group_name !== 'string') {
    throw new Error(`${where} has an invalid group`);
  }
  if (
    !isRecord(manifest) ||
    typeof manifest.schema_version !== 'number' ||
    !isRecord(manifest.ops)
  ) {
    throw new Error(`${where} has no edit manifest`);
  }
  return {
    name: name.trim(),
    group_name: typeof group_name === 'string' ? group_name.trim() || null : null,
    manifest: { schema_version: manifest.schema_version, ops: manifest.ops }
  };
}

export function parsePresetFile(text: string): PresetInput[] {
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    throw new Error('Not a JSON file');
  }
  if (!isRecord(body) || body.kind !== PRESET_FILE_KIND) {
    throw new Error('Not an immich-edit preset file');
  }
  if (body.version !== PRESET_FILE_VERSION) {
    throw new Error(`Unsupported preset file version ${String(body.version)}`);
  }
  if (!Array.isArray(body.presets)) throw new Error('Preset file has no presets');
  return body.presets.map(parseEntry);
}

export function newPresets(
  incoming: PresetInput[],
  existing: Preset[]
): { fresh: PresetInput[]; skipped: number } {
  const taken = new Set(existing.map((p) => p.name));
  const fresh = incoming.filter((p) => {
    if (taken.has(p.name)) return false;
    taken.add(p.name);
    return true;
  });
  return { fresh, skipped: incoming.length - fresh.length };
}

function presetCount(n: number): string {
  return `${n} preset${n === 1 ? '' : 's'}`;
}

export function importSummary(imported: number, skipped: number, failed: number): string {
  const parts = [imported > 0 ? `Imported ${presetCount(imported)}` : 'No presets imported'];
  if (skipped > 0) parts.push(`skipped ${skipped} with a name already in use`);
  if (failed > 0) parts.push(`${failed} failed`);
  return parts.join(', ');
}
