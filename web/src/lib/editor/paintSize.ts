import { BRUSH_SIZE } from '#lib/editor/params/brush.ts';
import type { ToolRange } from '#lib/editor/params/types.ts';
import {
  commitRetouchSetting,
  liveRetouch,
  RETOUCH_RANGES,
  retouchValue
} from '#lib/editor/retouch/settings.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { paintInput } from '#lib/stores/editor/paintInput.svelte.ts';

export type PaintSize = {
  value: number;
  range: ToolRange;
  set: (value: number) => void;
  commit: () => void;
};

export function paintSize(): PaintSize | null {
  if (paintInput.retouchActive) {
    return {
      value: retouchValue('size'),
      range: RETOUCH_RANGES.size,
      set: (value) => liveRetouch('size', value),
      commit: commitRetouchSetting
    };
  }
  if (!paintInput.brushActive) return null;
  return {
    value: editor.brushTool.size,
    range: BRUSH_SIZE,
    set: (size) => editor.setBrushTool({ size }),
    commit: () => {}
  };
}
