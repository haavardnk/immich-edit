import { loupeCommand, VIEWING } from '#lib/commands/loupe/context.ts';
import { loupeHost } from '#lib/commands/loupe/loupeHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { panStep } from '#lib/utils/imageViewport.ts';
import { clipWarnView, fullscreenView, infoView } from '#lib/commands/viewSpecs.ts';
import { mdiSkipNextOutline } from '@mdi/js';

export const loupeInfo = loupeCommand({
  id: 'loupe.info',
  contexts: VIEWING,
  ...infoView,
  pressed: () => browseView.loupeInfoOpen,
  run: () => browseView.toggleLoupeInfo()
});

export const loupeClipWarn = loupeCommand({
  id: 'loupe.clipWarn',
  contexts: VIEWING,
  ...clipWarnView,
  pressed: () => ui.clipWarn,
  run: () => ui.toggleClipWarn()
});

export const loupeFullscreen = loupeCommand({
  id: 'loupe.fullscreen',
  contexts: VIEWING,
  ...fullscreenView,
  run: () => ui.toggleFullscreen()
});

export const loupeAutoAdvance = loupeCommand({
  id: 'loupe.autoAdvance',
  contexts: VIEWING,
  label: 'Auto-advance',
  icon: mdiSkipNextOutline,
  pressed: () => browseView.loupeAutoAdvance,
  run: () => browseView.setLoupeAutoAdvance(!browseView.loupeAutoAdvance)
});

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
  loupeInfo,
  loupeCommand({
    id: 'loupe.tags',
    contexts: ['loupe'],
    bind: 'toggleTags',
    run: () => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)
  }),
  loupeClipWarn,
  loupeFullscreen,
  loupeAutoAdvance,
  loupeCommand({
    id: 'loupe.filmstrip',
    contexts: VIEWING,
    bind: 'filmstrip',
    run: () => ui.toggleLoupeFilmstrip()
  })
];
