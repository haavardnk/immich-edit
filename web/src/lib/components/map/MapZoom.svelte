<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiMinus, mdiPlus } from '@mdi/js';
  import { ZOOM_STEP_MS } from '#lib/map/motion.ts';
  import type { Map as MapLibreMap } from 'maplibre-gl';

  let { map, below = false }: { map: MapLibreMap | null; below?: boolean } = $props();

  const BUTTONS = [
    { label: 'Zoom in', icon: mdiPlus, step: 1 },
    { label: 'Zoom out', icon: mdiMinus, step: -1 }
  ] as const;
</script>

<div
  class="absolute {below
    ? 'top-16'
    : 'top-3'} right-3 z-10 flex flex-col overflow-hidden rounded-xl border border-hairline bg-light-100 shadow-popover touch:hidden"
>
  {#each BUTTONS as { label, icon, step } (label)}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      shape="rectangle"
      {icon}
      title={label}
      aria-label={label}
      disabled={!map}
      onclick={(event: MouseEvent) =>
        map?.zoomTo(map.getZoom() + step, { duration: ZOOM_STEP_MS }, { originalEvent: event })}
    />
  {/each}
</div>
