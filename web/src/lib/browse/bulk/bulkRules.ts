import type { MultiMode } from '#lib/browse/compareEntry.ts';
import { COMPARE_PANES } from '#lib/stores/compare.svelte.ts';

export function scopeLine(
  loaded: number,
  total: number | undefined,
  partial: boolean
): string | null {
  if (!partial || total === undefined || loaded >= total) return null;
  return `${loaded} of ${total} loaded`;
}

function multiCap(mode: MultiMode, panes: number): number {
  return mode === 'compare' ? COMPARE_PANES : panes;
}

export function multiDetail(mode: MultiMode, count: number, panes: number): string | null {
  const cap = multiCap(mode, panes);
  return count > cap ? `first ${cap}` : null;
}

export function fileStem(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
}
