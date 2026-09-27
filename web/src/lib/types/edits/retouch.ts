import type { Vec2f } from './masks';

export type RetouchMode = 'heal' | 'clone';

export const MAX_RETOUCH_STROKES = 64;
export const MAX_RETOUCH_POINTS = 256;

export interface RetouchStroke {
  id: string;
  mode: RetouchMode;
  points: Vec2f[];
  radius: number;
  hardness: number;
  opacity: number;
  source: Vec2f;
  enabled: boolean;
}
