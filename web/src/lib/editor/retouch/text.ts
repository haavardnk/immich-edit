import { MAX_RETOUCH_STROKES, type RetouchMode } from '#lib/types/edits.ts';
import { countOf } from '#lib/utils/plural.ts';
import { RETOUCH_MODE_LABEL } from './modes';

export const RETOUCH_LIMIT_TEXT = `Limit of ${MAX_RETOUCH_STROKES} retouch strokes reached.`;
export const RETOUCH_LIMIT_HINT =
  'Delete a stroke to paint again. Strokes can still be selected and edited.';
export const SOURCE_MISSING_TEXT = 'Set a source before painting';

export function strokeName(index: number): string {
  return `Stroke ${index + 1}`;
}

export function modeName(mode: RetouchMode): string {
  return RETOUCH_MODE_LABEL[mode];
}

export function strokeTitle(mode: RetouchMode, index: number): string {
  return `${strokeName(index)} · ${modeName(mode)}`;
}

export function strokeCount(count: number): string {
  return `${count}/${MAX_RETOUCH_STROKES}`;
}

export function retouchSummary(count: number): string {
  if (count === 0) return 'No strokes yet';
  const strokes = countOf(count, 'stroke');
  return count >= MAX_RETOUCH_STROKES ? `${strokes} · painting paused` : strokes;
}
