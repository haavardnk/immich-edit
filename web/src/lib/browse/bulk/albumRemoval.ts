import type { ToastKind } from '#lib/stores/toasts.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface AlbumRemoval {
  removed: string[];
  refused: number;
  failed: number;
}

export interface RemovalText {
  kind: ToastKind;
  message: string;
  description?: string;
}

export const NOT_IN_ALBUM = 'not_found';
export const NOT_YOURS = 'no_permission';

function refusedLine(refused: number): string {
  return refused === 1
    ? '1 belongs to someone else and stays'
    : `${refused} belong to someone else and stay`;
}

function notes(outcome: AlbumRemoval): string[] {
  const lines: string[] = [];
  if (outcome.refused > 0) lines.push(refusedLine(outcome.refused));
  if (outcome.failed > 0) lines.push(`${outcome.failed} could not be removed`);
  return lines;
}

export function removalText(outcome: AlbumRemoval, albumName: string): RemovalText {
  const lines = notes(outcome);
  const removed = outcome.removed.length;
  if (removed === 0) {
    const message = lines.length > 0 ? `Nothing removed: ${lines.join('; ')}` : 'Nothing removed';
    return { kind: 'info', message };
  }
  const message = `Removed ${countOf(removed, 'photo')} from ${albumName}`;
  if (lines.length === 0) return { kind: 'success', message };
  return { kind: 'warn', message, description: lines.join('; ') };
}
