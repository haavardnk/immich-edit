import { loupeCommand, VIEWING } from '#lib/commands/loupe/context.ts';
import { loupeHost } from '#lib/commands/loupe/loupeHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { panStep } from '#lib/utils/imageViewport.ts';

export const LOUPE_VIEW_COMMANDS: readonly Command[] = [
  loupeCommand({
    id: 'loupe.zoom',
    contexts: VIEWING,
    bind: 'zoomToggle',
    run: () => loupeHost.get()?.toggleZoom()
  }),
  loupeCommand({
    id: 'loupe.pan',
    contexts: VIEWING,
    bind: 'zoomPan',
    run: (trigger) => {
      const step = trigger ? panStep(trigger.key) : null;
      if (step) loupeHost.get()?.panBy(step[0], step[1]);
    }
  }),
  loupeCommand({
    id: 'loupe.info',
    contexts: VIEWING,
    bind: 'toggleInfo',
    run: () => browseView.toggleLoupeInfo()
  }),
  loupeCommand({
    id: 'loupe.tags',
    contexts: ['loupe'],
    bind: 'toggleTags',
    run: () => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)
  }),
  loupeCommand({
    id: 'loupe.clipWarn',
    contexts: VIEWING,
    bind: 'clipWarn',
    run: () => ui.toggleClipWarn()
  }),
  loupeCommand({
    id: 'loupe.fullscreen',
    contexts: VIEWING,
    bind: 'fullscreen',
    run: () => ui.toggleFullscreen()
  }),
  loupeCommand({
    id: 'loupe.filmstrip',
    contexts: VIEWING,
    bind: 'filmstrip',
    run: () => ui.toggleLoupeFilmstrip()
  })
];
