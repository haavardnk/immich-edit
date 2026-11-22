<script lang="ts">
  import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';
  import MapCanvas from '#lib/components/map/MapCanvas.svelte';
  import { mapHref } from '#lib/map/mapArea.ts';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import { placePin } from '#lib/map/pin.ts';
  import { Icon, Tooltip } from '@immich/ui';
  import { mdiArrowExpand } from '@mdi/js';
  import type { Map as MapLibreMap } from 'maplibre-gl';
  import { onMount } from 'svelte';

  let { point, label }: { point: GeoPoint; label: string } = $props();

  const ZOOM = 11;

  function ready(map: MapLibreMap, lib: typeof import('maplibre-gl')): () => void {
    const pin = placePin(lib, map, point, { size: 'small' });
    return () => pin.remove();
  }

  onMount(() => {
    void mapConfig.ensure().catch(() => undefined);
  });
</script>

<Tooltip text="Open on the map">
  {#snippet child({ props })}
    <a
      {...props}
      href={mapHref({ ...point, zoom: 14 })}
      aria-label="Open {label} on the map"
      class="group relative block h-38 overflow-hidden rounded-lg border border-hairline outline-none focus-visible:ring-2 focus-visible:ring-primary touch:h-44"
    >
      {#key `${point.lat},${point.lon}`}
        <MapCanvas
          class="pointer-events-none size-full"
          label="Map of {label}"
          style={mapConfig.style}
          view={{ ...point, zoom: ZOOM }}
          interactive={false}
          onReady={ready}
        />
      {/key}
      <span
        class="absolute top-2 right-2 grid size-7 place-items-center rounded-lg bg-light-100/90 text-light-950 shadow-sm group-hover:text-primary"
        aria-hidden="true"
      >
        <Icon icon={mdiArrowExpand} size="14px" />
      </span>
    </a>
  {/snippet}
</Tooltip>
