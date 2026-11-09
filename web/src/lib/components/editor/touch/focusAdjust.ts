import {
  dragBounds,
  dragPosition,
  positionOf,
  valueAt
} from '#lib/components/editor/controls/relativeDrag.ts';
import type { ParamDef } from '#lib/editor/params/types.ts';
import type { ViewerAdjust } from '#lib/editor/viewerGestures.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export function focusAdjust(param: ParamDef): ViewerAdjust {
  let start = 0;
  let position = 0;

  function current(): number {
    return param.get(editor.edits, editor.isRaw);
  }

  return {
    start() {
      start = current();
      position = positionOf(start, param);
      editor.beginDrag();
    },
    move(stepX, width) {
      position = dragPosition(position, stepX, 0, { ...dragBounds(param), width });
      const next = valueAt(position, param);
      if (next === current()) return;
      param.set(editor.edits, next, editor.isRaw);
      editor.onLive();
    },
    end() {
      editor.endDrag();
      if (current() !== start) void editor.onCommit(param.history);
    }
  };
}
