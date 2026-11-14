import type { OnConflict } from '#lib/api/export.ts';
import type { EarlierExports, ExportedFile } from '#lib/api/exportOutputs.ts';
import type { Option } from '#lib/panels/export/settings.ts';
import { formatAgo } from '#lib/utils/datetime.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface EarlierRun {
  found: EarlierExports;
  total: number;
  presets: number;
}

function onePhoto(run: EarlierRun): boolean {
  return run.total === 1;
}

function oneFile(run: EarlierRun): ExportedFile | null {
  const [file, ...rest] = run.found.files;
  return onePhoto(run) && file && rest.length === 0 ? file : null;
}

function presetPhrase(run: EarlierRun): string {
  if (run.presets === 0) return '';
  return run.presets === 1 ? ' with this preset' : ' with these presets';
}

export function earlierChoices(run: EarlierRun): Option<OnConflict>[] {
  const keep = { value: 'keep_both' as const, label: 'Keep both' };
  if (oneFile(run)) return [{ value: 'replace', label: 'Replace it' }, keep];
  if (onePhoto(run)) return [{ value: 'replace', label: 'Replace them' }, keep];
  return [{ value: 'replace', label: 'Replace them' }, { value: 'skip', label: 'Skip them' }, keep];
}

export function earlierTitle(run: EarlierRun): string {
  const preset = presetPhrase(run);
  if (onePhoto(run)) return `Exported to Immich before${preset}`;
  const { photos } = run.found;
  const verb = photos === 1 ? 'was' : 'were';
  const which =
    photos === run.total
      ? `All ${countOf(photos, 'photo')}`
      : `${photos} of ${countOf(run.total, 'photo')}`;
  return `${which} ${verb} exported to Immich before${preset}`;
}

export function earlierFiles(files: readonly ExportedFile[], now = new Date()): string | null {
  const [latest, ...rest] = files;
  if (!latest) return null;
  const more = rest.length > 0 ? ` and ${rest.length} more` : '';
  return `${latest.filename} · ${formatAgo(latest.exported_at, now)}${more}`;
}

export function earlierHint(run: EarlierRun, policy: OnConflict): string {
  if (policy === 'skip') {
    const rest = run.total - run.found.photos;
    if (rest === 0) return 'Nothing is uploaded.';
    return rest === 1
      ? 'Only the other photo is uploaded.'
      : `Only the other ${rest} photos are uploaded.`;
  }
  if (policy === 'keep_both') {
    return oneFile(run)
      ? 'The new export is added next to the earlier one.'
      : 'The new exports are added next to the earlier ones.';
  }
  const file = oneFile(run);
  if (file)
    return `${file.filename} moves to the Immich trash; the new export takes its albums and stack.`;
  const each = run.presets > 1 ? 'photo and preset' : 'photo';
  return `The latest earlier export of each ${each} moves to the Immich trash; the new one takes its albums and stack.`;
}
