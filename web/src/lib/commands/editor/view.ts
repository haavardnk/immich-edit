import type { Command } from '#lib/commands/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { panStep } from '#lib/utils/imageViewport.ts';
import { selectedShapeNudges } from '#lib/commands/editor/context.ts';

function showOriginal(): void {
  if (editor.showingOriginal) return;
  editor.showingOriginal = true;
  editor.onLive();
}

function hideOriginal(): void {
  if (!editor.showingOriginal) return;
  editor.showingOriginal = false;
  editor.onLive();
}

export const VIEW_COMMANDS: readonly Command[] = [
  {
    id: 'editor.pan',
    contexts: ['editor'],
    bind: 'zoomPan',
    yields: selectedShapeNudges,
    run: (trigger) => {
      const step = trigger ? panStep(trigger.key) : null;
      if (step) editor.panBy(step);
    }
  },
  { id: 'editor.zoom', contexts: ['editor'], bind: 'zoomToggle', run: () => editor.zoomCycle() },
  {
    id: 'editor.info',
    contexts: ['editor'],
    bind: 'toggleInfo',
    run: () => ui.togglePopover('exif')
  },
  {
    id: 'editor.tags',
    contexts: ['editor'],
    bind: 'toggleTags',
    run: () => ui.togglePopover('tags')
  },
  {
    id: 'editor.clipWarn',
    contexts: ['editor'],
    bind: 'clipWarn',
    run: () => editor.toggleClipWarn()
  },
  {
    id: 'editor.beforeAfter',
    contexts: ['editor'],
    bind: 'beforeAfter',
    run: () => editor.toggleSplit()
  },
  {
    id: 'editor.holdOriginal',
    contexts: ['editor'],
    bind: 'holdOriginal',
    run: showOriginal,
    release: hideOriginal
  },
  {
    id: 'editor.panels',
    contexts: ['editor'],
    bind: 'togglePanels',
    run: () => ui.togglePanels()
  },
  {
    id: 'editor.chrome',
    contexts: ['editor'],
    bind: 'toggleChrome',
    run: () => ui.toggleChrome()
  },
  {
    id: 'editor.fullscreen',
    contexts: ['editor'],
    bind: 'fullscreen',
    run: () => ui.toggleFullscreen()
  },
  {
    id: 'editor.filmstrip',
    contexts: ['editor'],
    bind: 'filmstrip',
    run: () => ui.toggleEditorFilmstrip()
  },
  {
    id: 'editor.presetStrip',
    contexts: ['editor'],
    bind: 'presetStrip',
    run: () => ui.togglePresetStrip()
  }
];
