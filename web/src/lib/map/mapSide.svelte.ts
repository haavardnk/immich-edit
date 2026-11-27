import { layout } from '#lib/device/layout.svelte.ts';
import { mapFullscreen } from '#lib/map/mapFullscreen.svelte.ts';
import { mapMemory } from '#lib/map/mapMemory.ts';
import { trackSession } from '#lib/stores/trackSession.svelte.ts';

export class MapSide {
  photos = $state(mapMemory.panel ?? layout.mode !== 'touch');
  track = $state(true);

  readonly shown = $derived(trackSession.active ? this.track : this.photos);
  readonly card = $derived(
    trackSession.active &&
      trackSession.tracks.length > 0 &&
      (mapFullscreen.on || (!layout.compact && !this.track))
  );

  constructor() {
    $effect(() => {
      if (!trackSession.active) this.track = true;
    });
  }

  toggle(): void {
    if (trackSession.active) this.track = !this.track;
    else this.photos = !this.photos;
  }
}
