import type { Edits } from '#lib/types/edits.ts';

export function withLut(edits: Edits, lutId: string): Edits {
  return {
    ...edits,
    color: { ...edits.color, lut_3d: { ...edits.color.lut_3d, lut_id: lutId } }
  };
}
