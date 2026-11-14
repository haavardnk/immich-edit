import { countOf } from '#lib/utils/plural.ts';
import type { Destination } from './settings';

export interface ExportRun {
  photos: number;
  presets: readonly Destination[];
  destination: Destination;
  format: string;
  albums: readonly string[];
}

function presetKinds(run: ExportRun): Set<Destination> {
  return new Set(run.presets);
}

export function runFiles(run: ExportRun): number {
  return run.photos * Math.max(1, run.presets.length);
}

export function runDirect(run: ExportRun): boolean {
  return run.presets.length === 0 && run.destination === 'download' && run.photos === 1;
}

export function runZips(run: ExportRun): boolean {
  if (run.presets.length > 0) return run.presets.includes('download');
  return run.destination === 'download' && run.photos > 1;
}

export function runLabel(run: ExportRun): string {
  if (run.presets.length === 0) {
    if (run.destination === 'immich') return `Export ${run.photos} to Immich`;
    if (run.photos === 1) return `Download ${run.format}`;
    return `Download ${run.photos} as ${run.format} ZIP`;
  }
  const files = runFiles(run);
  const kinds = presetKinds(run);
  if (kinds.size > 1) return `Export ${files} files`;
  return kinds.has('immich') ? `Export ${files} to Immich` : `Download ${files} as ZIP`;
}

function runTarget(run: ExportRun): string {
  if (run.presets.length > 0) {
    const kinds = presetKinds(run);
    if (kinds.size > 1) return 'ZIP and Immich';
    return kinds.has('immich') ? 'to Immich' : 'ZIP';
  }
  if (run.destination === 'download') return run.photos === 1 ? 'download' : 'ZIP';
  const [album] = run.albums;
  if (run.albums.length === 1 && album) return `to Immich album ${album}`;
  if (run.albums.length > 1) return `to Immich, ${countOf(run.albums.length, 'album')}`;
  return 'to Immich';
}

export function runSummary(run: ExportRun): string {
  return [
    countOf(run.photos, 'photo'),
    run.presets.length > 0 ? countOf(run.presets.length, 'preset') : null,
    runTarget(run)
  ]
    .filter((part) => part !== null)
    .join(' · ');
}

export function runNote(run: ExportRun): string | null {
  if (runDirect(run)) return null;
  const jobs = run.presets.length > 0 ? presetKinds(run).size : 1;
  const runs = jobs === 1 ? 'a background job' : `${jobs} background jobs`;
  return `Runs as ${runs}. Track progress in Jobs.`;
}
