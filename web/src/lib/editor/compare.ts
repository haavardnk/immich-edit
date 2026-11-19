import type { Box, ClientPoint } from '#lib/gestures/point.ts';

export type CompareMode = 'off' | 'split' | 'side';

export type CompareSource = { kind: 'original' } | { kind: 'snapshot'; id: string };

type Named = { id: string; name: string };

export const ORIGINAL_SOURCE: CompareSource = { kind: 'original' };

export const COMPARE_DIVIDER = 2;

export function nextCompareMode(mode: CompareMode, sideAvailable: boolean): CompareMode {
  if (mode === 'off') return 'split';
  if (mode === 'split' && sideAvailable) return 'side';
  return 'off';
}

export function availableMode(mode: CompareMode, sideAvailable: boolean): CompareMode {
  return mode === 'side' && !sideAvailable ? 'split' : mode;
}

export function preferredMode(sideAvailable: boolean): CompareMode {
  return sideAvailable ? 'side' : 'split';
}

export function sameSource(a: CompareSource, b: CompareSource): boolean {
  if (a.kind === 'original' || b.kind === 'original') return a.kind === b.kind;
  return a.kind === b.kind && a.id === b.id;
}

export function compareName(source: CompareSource, snapshots: readonly Named[]): string {
  if (source.kind === 'snapshot') {
    return snapshots.find((item) => item.id === source.id)?.name ?? 'Snapshot';
  }
  return 'Original';
}

export function compareLabel(source: CompareSource, snapshots: readonly Named[]): string {
  const name = compareName(source, snapshots);
  return source.kind === 'snapshot' ? `Snapshot · ${name}` : name;
}

export function compareViewport(
  stage: { w: number; h: number },
  mode: CompareMode
): { w: number; h: number } {
  if (mode !== 'side') return stage;
  return { w: Math.max(0, (stage.w - COMPARE_DIVIDER) / 2), h: stage.h };
}

export function comparePane(stage: Box, point: ClientPoint, mode: CompareMode): Box {
  if (mode !== 'side') return stage;
  const width = Math.max(0, (stage.width - COMPARE_DIVIDER) / 2);
  const right = point.clientX >= stage.left + width + COMPARE_DIVIDER / 2;
  return {
    left: right ? stage.left + width + COMPARE_DIVIDER : stage.left,
    top: stage.top,
    width,
    height: stage.height
  };
}
