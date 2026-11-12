import type { Command } from '#lib/commands/types.ts';
import { openTouchSearch } from '#lib/components/editor/touch/modes.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import {
  BRUSH_HARDNESS,
  BRUSH_SIZE,
  RETOUCH_HARDNESS,
  RETOUCH_SIZE
} from '#lib/editor/params/brush.ts';
import type { KeybindId } from '#lib/shortcuts/table.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { MaskComponentKind } from '#lib/types/edits.ts';
import { defaultLinear, defaultRadial } from '#lib/types/masks.ts';
import { stepBrush } from '#lib/utils/brushSize.ts';
import { mdiMagnify } from '@mdi/js';
import { showTargetTool } from '#lib/editor/targetSession.ts';
import { TARGET_TOOLS, targetAvailable } from '#lib/editor/targetTools.ts';
import { targetAdjust } from '#lib/stores/targetAdjust.svelte.ts';

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

function toggleTargetAdjust(): void {
  ui.openTab('develop');
  targetAdjust.toggle(targetAvailable(targetAdjust.last, editor.edits) ? targetAdjust.last : 'hsl');
  if (targetAdjust.tool) showTargetTool(TARGET_TOOLS[targetAdjust.tool]);
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

export const searchSettings = {
  id: 'editor.searchSettings',
  contexts: ['editor'],
  label: 'Search settings',
  icon: mdiMagnify,
  bind: 'searchSettings',
  gate: 'always',
  run: () => {
    if (layout.mode === 'touch') openTouchSearch();
    else settingSearch.paletteOpen = !settingSearch.paletteOpen;
  }
} satisfies Command;

export const TOOL_COMMANDS: readonly Command[] = [
  searchSettings,
  {
    id: 'editor.pickWhiteBalance',
    contexts: ['editor'],
    bind: 'pickWhiteBalance',
    enabled: () => !editor.wbBusy,
    run: pickWhiteBalance
  },
  {
    id: 'editor.targetAdjust',
    contexts: ['editor'],
    bind: 'targetAdjust',
    enabled: () => !!editor.assetId,
    run: toggleTargetAdjust
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
    id: 'geometry.cropGuideTurn',
    contexts: ['geometry'],
    bind: 'cropGuideTurn',
    run: () => ui.turnCropGuide()
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
