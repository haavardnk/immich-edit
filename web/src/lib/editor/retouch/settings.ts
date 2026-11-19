import { RETOUCH_HARDNESS, RETOUCH_OPACITY, RETOUCH_SIZE } from '#lib/editor/params/brush.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import type { RetouchStroke } from '#lib/types/edits.ts';

export type RetouchSetting = 'size' | 'hardness' | 'opacity';

export const RETOUCH_RANGES = {
  size: RETOUCH_SIZE,
  hardness: RETOUCH_HARDNESS,
  opacity: RETOUCH_OPACITY
} as const;

const STROKE_FIELD = { size: 'radius', hardness: 'hardness', opacity: 'opacity' } as const;

export type SelectedStroke = { stroke: RetouchStroke; index: number };

export function selectedStroke(): SelectedStroke | null {
  const id = editor.activeRetouchId;
  if (!id) return null;
  const index = editor.edits.retouch.findIndex((stroke) => stroke.id === id);
  const stroke = editor.edits.retouch[index];
  return stroke ? { stroke, index } : null;
}

export function retouchValue(setting: RetouchSetting): number {
  const selected = selectedStroke();
  return selected ? selected.stroke[STROKE_FIELD[setting]] : editor.retouchTool[setting];
}

export function liveRetouch(setting: RetouchSetting, value: number): void {
  const selected = selectedStroke();
  if (selected) {
    void editor.setRetouchStroke(selected.stroke.id, { [STROKE_FIELD[setting]]: value }, false);
    return;
  }
  editor.setRetouchTool({ [setting]: value });
}

export function commitRetouchSetting(): void {
  if (editor.activeRetouchId) void editor.commitRetouch();
}
