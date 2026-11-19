import type { ParamDef } from '#lib/editor/params/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export interface ParamDrag {
  readonly param: ParamDef;
  readonly start: number;
  set(value: number): void;
  end(): void;
}

export function beginParamDrag(param: ParamDef): ParamDrag {
  const start = param.get(editor.edits, editor.isRaw);
  editor.beginDrag();
  return {
    param,
    start,
    set(value) {
      if (value === param.get(editor.edits, editor.isRaw)) return;
      param.set(editor.edits, value, editor.isRaw);
      editor.onLive();
    },
    end() {
      editor.endDrag();
      if (param.get(editor.edits, editor.isRaw) !== start) void editor.onCommit(param.history);
    }
  };
}
