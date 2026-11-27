import type { Command } from '#lib/commands/types.ts';
import { mapFullscreen } from '#lib/map/mapFullscreen.svelte.ts';

export const MAP_COMMANDS: readonly Command[] = [
  {
    id: 'map.fullscreen',
    contexts: ['map'],
    label: 'Fullscreen',
    bind: 'fullscreen',
    gate: 'outsideControls',
    run: () => mapFullscreen.toggle()
  },
  {
    id: 'map.exitFullscreen',
    contexts: ['map'],
    label: 'Exit fullscreen',
    bind: 'mapExitFullscreen',
    gate: 'outsideFields',
    active: () => mapFullscreen.on,
    run: () => mapFullscreen.exit()
  }
];
