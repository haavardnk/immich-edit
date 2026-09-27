import {
  MAX_RETOUCH_POINTS,
  MAX_RETOUCH_STROKES,
  type Edits,
  type RetouchStroke,
  type Vec2f
} from '$lib/types/edits';
import { parseVec2f } from './masks';

export function decodeRetouch(ops: Record<string, unknown>, edits: Edits): void {
  const retouch = ops.retouch as { strokes?: unknown[] } | undefined;
  if (!retouch?.strokes) return;
  edits.retouch = retouch.strokes
    .map((raw) => parseRetouchStroke(raw))
    .filter((stroke): stroke is RetouchStroke => stroke !== null)
    .slice(0, MAX_RETOUCH_STROKES);
}

function parseRetouchStroke(raw: unknown): RetouchStroke | null {
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  if (typeof record.id !== 'string') return null;
  const source = parseVec2f(record.source);
  if (!source) return null;
  const points = (Array.isArray(record.points) ? record.points : [])
    .map((point) => parseVec2f(point))
    .filter((point): point is Vec2f => point !== null)
    .slice(0, MAX_RETOUCH_POINTS);
  if (points.length === 0) return null;
  return {
    id: record.id,
    mode: record.mode === 'clone' ? 'clone' : 'heal',
    points,
    radius: typeof record.radius === 'number' ? record.radius : 0,
    hardness: typeof record.hardness === 'number' ? record.hardness : 0.5,
    opacity: typeof record.opacity === 'number' ? record.opacity : 1,
    source,
    enabled: record.enabled !== false
  };
}
