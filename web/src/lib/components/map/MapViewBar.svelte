<script lang="ts">
  import { mapFullscreen } from '#lib/map/mapFullscreen.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { IconButton } from '@immich/ui';
  import {
    mdiFitToScreenOutline,
    mdiFullscreen,
    mdiFullscreenExit,
    mdiImageMultiple,
    mdiImageMultipleOutline,
    mdiStar,
    mdiStarOutline
  } from '@mdi/js';

  let {
    stars = $bindable(),
    photos = $bindable(),
    canFit,
    onFit
  }: { stars: boolean; photos: boolean; canFit: boolean; onFit: () => void } = $props();

  const BUTTON = {
    size: 'small',
    variant: 'ghost',
    shape: 'rectangle',
    class: 'touch:size-11'
  } as const;
  const full = $derived(mapFullscreen.on);
</script>

<div
  class="absolute top-3 right-3 z-10 flex divide-x divide-hairline overflow-hidden rounded-xl border border-hairline bg-light-100 shadow-popover"
  role="group"
  aria-label="Map view"
  data-map-view-bar
>
  {#if trackSession.active}
    <IconButton
      {...BUTTON}
      color={photos ? 'primary' : 'secondary'}
      icon={photos ? mdiImageMultiple : mdiImageMultipleOutline}
      title={photos ? 'Hide photos on the map' : 'Show photos on the map'}
      aria-label="Photos on the map"
      aria-pressed={photos}
      onclick={() => (photos = !photos)}
    />
  {/if}
  <IconButton
    {...BUTTON}
    color={stars ? 'primary' : 'secondary'}
    icon={stars ? mdiStar : mdiStarOutline}
    title={stars ? 'Hide saved places' : 'Show saved places'}
    aria-label="Saved places"
    aria-pressed={stars}
    onclick={() => (stars = !stars)}
  />
  <IconButton
    {...BUTTON}
    color="secondary"
    icon={mdiFitToScreenOutline}
    title="Fit all photos"
    aria-label="Fit all photos"
    disabled={!canFit}
    onclick={onFit}
  />
  {#if full}
    <IconButton
      {...BUTTON}
      color="secondary"
      icon={mdiFullscreenExit}
      title={hint('Exit fullscreen', 'mapExitFullscreen')}
      aria-label="Exit fullscreen"
      onclick={() => mapFullscreen.exit()}
    />
  {:else}
    <IconButton
      {...BUTTON}
      color="secondary"
      icon={mdiFullscreen}
      title={hint('Fullscreen', 'fullscreen')}
      aria-label="Fullscreen"
      onclick={() => mapFullscreen.enter()}
    />
  {/if}
</div>
