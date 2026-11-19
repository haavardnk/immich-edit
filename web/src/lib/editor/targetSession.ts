import { settleValue } from '#lib/components/editor/controls/sliderStep.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import type { ParamDef, ParamRange } from '#lib/editor/params/types.ts';
import { revealSetting } from '#lib/editor/revealSetting.ts';
import { CHANNEL_LABELS, dragPoint, pickPoint } from '#lib/panels/curves/curves.ts';
import { setDevelopPanel } from '#lib/panels/developPanels.ts';
import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { touchUi } from '#lib/stores/touchUi.svelte.ts';
import type { CurveChannel } from '#lib/types/edits.ts';
import { beginParamDrag } from './paramDrag';
import type { TargetAxes, TargetHit, TargetToolDef } from './targetTools';

export const TARGET_DRAG_PX = 300;

export type TargetAxis = keyof TargetAxes;

export interface TargetDrag {
  readonly param: ParamDef | null;
  move(dy: number): void;
  end(): void;
}

export function draggedValue(range: ParamRange, start: number, dy: number): number {
  return settleValue(start - (dy / TARGET_DRAG_PX) * (range.max - range.min), range);
}

function commitCurve(channel: CurveChannel): void {
  void editor.onCommit(`Curve ${CHANNEL_LABELS[channel]}`);
}

function pickCurvePoint(channel: CurveChannel, level: number): { index: number; added: boolean } {
  const pts = editor.edits.basic.curves[channel];
  const { points, index } = pickPoint(pts, level);
  const added = points !== pts;
  if (added) editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: points };
  curveEditor.selected = index;
  return { index, added };
}

export function showTargetTool(def: TargetToolDef): void {
  if (layout.mode !== 'touch') {
    setDevelopPanel(def.panel, true);
    return;
  }
  if (!def.touch) return;
  touchUi.setGroup(def.touch.group);
  touchUi.setView(def.touch.view);
}

export function selectTarget(def: TargetToolDef, hit: TargetHit): void {
  showTargetTool(def);
  if (hit.kind === 'param') {
    void revealSetting(hit.axes.main.id);
    return;
  }
  if (pickCurvePoint(hit.channel, hit.level).added) commitCurve(hit.channel);
}

function paramDrag(param: ParamDef): TargetDrag {
  const drag = beginParamDrag(param);
  return {
    param,
    move: (dy) => drag.set(draggedValue(param, drag.start, dy)),
    end: drag.end
  };
}

function curveDrag(channel: CurveChannel, level: number): TargetDrag {
  const { index, added } = pickCurvePoint(channel, level);
  const start = editor.edits.basic.curves[channel][index]?.y ?? level;
  let moved = false;
  editor.beginDrag();
  if (added) editor.onLive();
  return {
    param: null,
    move(dy) {
      const pts = editor.edits.basic.curves[channel];
      const point = pts[index];
      if (!point) return;
      const y = Math.max(0, Math.min(1, start - dy / TARGET_DRAG_PX));
      const next = y === point.y ? null : dragPoint(pts, index, { x: point.x, y });
      if (!next) return;
      editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: next };
      moved = true;
      editor.onLive();
    },
    end() {
      editor.endDrag();
      if (added || moved) commitCurve(channel);
    }
  };
}

export function beginTargetDrag(def: TargetToolDef, hit: TargetHit, axis: TargetAxis): TargetDrag {
  showTargetTool(def);
  if (hit.kind === 'curve') return curveDrag(hit.channel, hit.level);
  const param = hit.axes[axis] ?? hit.axes.main;
  void revealSetting(param.id);
  return paramDrag(param);
}
