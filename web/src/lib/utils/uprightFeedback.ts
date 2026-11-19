import type { Point } from '#lib/utils/perspective.ts';
import {
  MAX_GUIDE_TILT_DEG,
  MAX_GUIDES_PER_KIND,
  MIN_GUIDE_FRACTION,
  type GuideKind,
  type UprightGuide
} from '#lib/utils/upright.ts';

export type GuideNote = { kind: 'dropped'; degrees: number } | { kind: 'unsolved' };

export const GUIDE_KINDS: readonly GuideKind[] = ['vertical', 'horizontal'];

export const KIND_NAME: Record<GuideKind, string> = {
  vertical: 'Vertical',
  horizontal: 'Horizontal'
};

const PREFIX: Record<GuideKind, string> = { vertical: 'V', horizontal: 'H' };

export function tiltFromLevel(from: Point, to: Point): number {
  const dx = Math.abs(to[0] - from[0]);
  const dy = Math.abs(to[1] - from[1]);
  return (Math.atan2(dy, dx) * 180) / Math.PI;
}

export function isGuideLength(from: Point, to: Point, diagonal: number): boolean {
  return Math.hypot(to[0] - from[0], to[1] - from[1]) >= MIN_GUIDE_FRACTION * diagonal;
}

export function guideLabels(guides: readonly UprightGuide[]): string[] {
  const seen: Record<GuideKind, number> = { vertical: 0, horizontal: 0 };
  return guides.map((g) => `${PREFIX[g.kind]}${++seen[g.kind]}`);
}

export function guideCount(guides: readonly UprightGuide[], kind: GuideKind): number {
  return guides.filter((g) => g.kind === kind).length;
}

export function replacedLabel(guides: readonly UprightGuide[], kind: GuideKind): string | null {
  return guideCount(guides, kind) >= MAX_GUIDES_PER_KIND ? `${PREFIX[kind]}1` : null;
}

export function countText(guides: readonly UprightGuide[], kind: GuideKind): string {
  return `${KIND_NAME[kind]} ${guideCount(guides, kind)} of ${MAX_GUIDES_PER_KIND}`;
}

export function drawingHint(
  guides: readonly UprightGuide[],
  from: Point,
  to: Point,
  kind: GuideKind | null,
  diagonal: number
): string | null {
  if (!isGuideLength(from, to, diagonal)) return null;
  const tilt = tiltFromLevel(from, to);
  if (!kind) return `${Math.round(tilt)}° · too diagonal, release drops it`;
  const offset = Math.round(kind === 'vertical' ? 90 - tilt : tilt);
  const replaced = replacedLabel(guides, kind);
  const next = `${PREFIX[kind]}${Math.min(guideCount(guides, kind) + 1, MAX_GUIDES_PER_KIND)}`;
  return replaced ? `${next} · ${offset}° · replaces ${replaced}` : `${next} · ${offset}°`;
}

export type GuideStatus = { tone: 'warn' | 'info'; text: string };

export function guideStatus(
  guides: readonly UprightGuide[],
  note: GuideNote | null
): GuideStatus[] {
  const lines: GuideStatus[] = [];
  if (note?.kind === 'dropped') {
    lines.push({
      tone: 'warn',
      text: `Last line dropped: ${Math.round(note.degrees)}° is neither level nor upright (within ${MAX_GUIDE_TILT_DEG}°)`
    });
  }
  if (note?.kind === 'unsolved') {
    lines.push({ tone: 'warn', text: 'These lines cannot be corrected together' });
  }
  if (guides.length === 1) lines.push({ tone: 'info', text: 'Draw one more line' });
  for (const kind of GUIDE_KINDS) {
    const replaced = replacedLabel(guides, kind);
    if (replaced) {
      lines.push({
        tone: 'info',
        text: `A third ${KIND_NAME[kind].toLowerCase()} replaces ${replaced}, the oldest`
      });
    }
  }
  return lines;
}
