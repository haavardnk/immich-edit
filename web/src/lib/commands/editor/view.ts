import type { Command } from '#lib/commands/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { panStep } from '#lib/utils/imageViewport.ts';
import { selectedShapeNudges } from '#lib/commands/editor/context.ts';
import { clipWarnView, fullscreenView, infoView } from '#lib/commands/viewSpecs.ts';
import { mdiCompare, mdiEyeOutline } from '@mdi/js';

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

export const editorInfo = {
  id: 'editor.info',
  contexts: ['editor'],
  ...infoView,
  pressed: () => ui.metaPopover === 'exif',
  enabled: () => !!editor.asset?.exifInfo,
  run: () => ui.togglePopover('exif')
} satisfies Command;

export const editorClipWarn = {
  id: 'editor.clipWarn',
  contexts: ['editor'],
  ...clipWarnView,
  pressed: () => ui.clipWarn,
  run: () => editor.toggleClipWarn()
} satisfies Command;

export const beforeAfter = {
  id: 'editor.beforeAfter',
  contexts: ['editor'],
  label: 'Before/After split',
  icon: mdiCompare,
  bind: 'beforeAfter',
  pressed: () => editor.splitMode,
  enabled: () => !editor.geometrySession,
  run: () => editor.toggleSplit()
} satisfies Command;

export const holdOriginal = {
  id: 'editor.holdOriginal',
  contexts: ['editor'],
  label: 'Hold for original',
  icon: mdiEyeOutline,
  bind: 'holdOriginal',
  pressed: () => editor.showingOriginal,
  run: showOriginal,
  release: hideOriginal
} satisfies Command;

export const editorFullscreen = {
  id: 'editor.fullscreen',
  contexts: ['editor'],
  ...fullscreenView,
  run: () => ui.toggleFullscreen()
} satisfies Command;

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
  editorInfo,
  {
    id: 'editor.tags',
    contexts: ['editor'],
    bind: 'toggleTags',
    run: () => ui.togglePopover('tags')
  },
  editorClipWarn,
  beforeAfter,
  holdOriginal,
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
  editorFullscreen,
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
