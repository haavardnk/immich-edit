import type { Command } from '#lib/commands/types.ts';
import {
  BRUSH_HARDNESS,
  BRUSH_SIZE,
  RETOUCH_HARDNESS,
  RETOUCH_SIZE
} from '#lib/editor/params/brush.ts';
import type { KeybindId } from '#lib/shortcuts/table.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { MaskComponentKind } from '#lib/types/edits.ts';
import { defaultLinear, defaultRadial } from '#lib/types/masks.ts';
import { stepBrush } from '#lib/utils/brushSize.ts';

function addShapeLayer(id: string, bind: KeybindId, kind: () => MaskComponentKind): Command {
  return {
    id,
    contexts: ['editor'],
    bind,
    run: () => {
      ui.openTab('masks');
      void editor.addMaskLayer(kind());
    }
  };
}

function pickWhiteBalance(): void {
  if (ui.editorTab !== 'develop') {
    ui.openTab('develop');
    if (editor.wbPicking) return;
  }
  editor.toggleWbPicker();
}

function stepSize(key: string): void {
  if (ui.editorTab === 'retouch') {
    editor.setRetouchTool({ size: stepBrush(editor.retouchTool.size, key, RETOUCH_SIZE) });
  } else {
    editor.setBrushTool({ size: stepBrush(editor.brushTool.size, key, BRUSH_SIZE) });
  }
}

function stepHardness(key: string): void {
  if (ui.editorTab === 'retouch') {
    editor.setRetouchTool({
      hardness: stepBrush(editor.retouchTool.hardness, key, RETOUCH_HARDNESS)
    });
  } else {
    editor.setBrushTool({ hardness: stepBrush(editor.brushTool.hardness, key, BRUSH_HARDNESS) });
  }
}

export const TOOL_COMMANDS: readonly Command[] = [
  {
    id: 'editor.pickWhiteBalance',
    contexts: ['editor'],
    bind: 'pickWhiteBalance',
    enabled: () => !editor.wbBusy,
    run: pickWhiteBalance
  },
  {
    id: 'geometry.straighten',
    contexts: ['geometry'],
    bind: 'straighten',
    run: () => ui.toggleGeometryTool('straighten')
  },
  {
    id: 'geometry.cropGrid',
    contexts: ['geometry'],
    bind: 'cropGrid',
    run: () => ui.cycleCropGrid()
  },
  {
    id: 'masks.overlay',
    contexts: ['masks'],
    bind: 'maskOverlay',
    run: () => editor.toggleMaskOverlay()
  },
  {
    id: 'masks.addBrush',
    contexts: ['editor'],
    bind: 'addBrushLayer',
    run: () => {
      ui.openTab('masks');
      void editor.addBrushLayer();
    }
  },
  addShapeLayer('masks.addRadial', 'addRadialLayer', defaultRadial),
  addShapeLayer('masks.addLinear', 'addLinearLayer', defaultLinear),
  {
    id: 'retouch.heal',
    contexts: ['retouch'],
    bind: 'retouchHeal',
    run: () => editor.setRetouchMode('heal')
  },
  {
    id: 'retouch.clone',
    contexts: ['retouch'],
    bind: 'retouchClone',
    run: () => editor.setRetouchMode('clone')
  },
  {
    id: 'retouch.delete',
    contexts: ['retouch'],
    bind: 'retouchDelete',
    run: () => {
      if (editor.activeRetouchId) void editor.removeRetouchStroke(editor.activeRetouchId);
    }
  },
  {
    id: 'brush.size',
    contexts: ['masks', 'retouch'],
    bind: 'brushSize',
    run: (trigger) => {
      if (trigger) stepSize(trigger.key);
    }
  },
  {
    id: 'brush.hardness',
    contexts: ['masks', 'retouch'],
    bind: 'brushHardness',
    run: (trigger) => {
      if (trigger) stepHardness(trigger.key);
    }
  }
];
