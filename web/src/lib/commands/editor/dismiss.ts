import type { Command, KeyGate } from '#lib/commands/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { targetAdjust } from '#lib/stores/targetAdjust.svelte.ts';

function dismiss(
  id: string,
  active: () => boolean,
  run: () => void,
  gate: KeyGate = 'outsideFields'
): Command {
  return { id, contexts: ['editor'], bind: 'editorEscape', gate, active, run };
}

export const DISMISS_COMMANDS: readonly Command[] = [
  dismiss(
    'editor.closePopovers',
    () => !!ui.metaPopover,
    () => ui.closePopover(),
    'always'
  ),
  dismiss(
    'editor.closeHelp',
    () => ui.keybindsHelpOpen,
    () => ui.closeKeybindsHelp()
  ),
  dismiss(
    'develop.exitTarget',
    () => !!targetAdjust.tool,
    () => targetAdjust.exit()
  ),
  dismiss(
    'geometry.closeTool',
    () => ui.editorTab === 'geometry' && !!ui.geometryTool,
    () => (ui.geometryTool = null)
  ),
  dismiss(
    'retouch.stopSampling',
    () => ui.editorTab === 'retouch' && editor.retouchSampling,
    () => editor.toggleRetouchSampling()
  ),
  dismiss(
    'retouch.deselect',
    () => ui.editorTab === 'retouch' && !!editor.activeRetouchId,
    () => (editor.activeRetouchId = null)
  ),
  dismiss(
    'masks.deselectShape',
    () => !!editor.activeMaskComponentId,
    () => editor.setActiveMaskComponent(null)
  ),
  dismiss(
    'editor.exitFullscreen',
    () => ui.fullscreen,
    () => ui.toggleFullscreen()
  )
];
