import { countOf } from '#lib/utils/plural.ts';
import type { Destination } from './settings';

export interface ExportRun {
  photos: number;
  destination: Destination;
  format: string;
  albums: readonly string[];
}

export function runDirect(run: ExportRun): boolean {
  return run.destination === 'download' && run.photos === 1;
}

export function runZips(run: ExportRun): boolean {
  return run.destination === 'download' && run.photos > 1;
}

export function runLabel(run: ExportRun): string {
  if (run.destination === 'immich') return `Export ${run.photos} to Immich`;
  if (run.photos === 1) return `Download ${run.format}`;
  return `Download ${run.photos} as ${run.format} ZIP`;
}

function runTarget(run: ExportRun): string {
  if (run.destination === 'download') return run.photos === 1 ? 'download' : 'ZIP';
  const [album] = run.albums;
  if (run.albums.length === 1 && album) return `to Immich album ${album}`;
  if (run.albums.length > 1) return `to Immich, ${countOf(run.albums.length, 'album')}`;
  return 'to Immich';
}

export function runSummary(run: ExportRun): string {
  return `${countOf(run.photos, 'photo')} · ${runTarget(run)}`;
}

export function runNote(run: ExportRun): string | null {
  return runDirect(run) ? null : 'Runs as a background job. Track progress in Jobs.';
}
