<script lang="ts">
  import { reverseGeocode, type Place } from '#lib/api/map.ts';
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import { locationOf, type GeoLocation } from '#lib/browse/locationOps.ts';
  import Dialog from '#lib/components/Dialog.svelte';
  import MapCanvas from '#lib/components/map/MapCanvas.svelte';
  import MapPin from '#lib/components/map/MapPin.svelte';
  import PlaceSearch from '#lib/components/map/PlaceSearch.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { WORLD_VIEW } from '#lib/map/mapArea.ts';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import { FLY_MAX_MS } from '#lib/map/motion.ts';
  import { placePin } from '#lib/map/pin.ts';
  import { locationPicker, type LocationRequest } from '#lib/stores/locationPicker.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiMapMarkerOutline } from '@mdi/js';
  import type { Map as MapLibreMap, Marker } from 'maplibre-gl';
  import { onMount, untrack } from 'svelte';

  let { request }: { request: LocationRequest } = $props();

  const ZOOM = 13;
  const PLACE_ZOOM = 15;

  const initial = untrack(() => request);
  const touch = layout.mode === 'touch';
  const located = initial.assets.filter((asset) => locationOf(asset.exifInfo) !== null).length;
  const single = initial.assets.length === 1 ? initial.assets[0] : undefined;

  let at = $state<GeoLocation | null>(initial.start);
  let map: MapLibreMap | null = null;
  let pin: Marker | null = null;
  let lib: typeof import('maplibre-gl') | null = null;
  let naming = 0;

  const title = $derived(single && located === 1 ? 'Edit location' : 'Set location');
  const subtitle = single ? single.originalFileName : countOf(initial.assets.length, 'photo');
  const placeName = $derived.by(() => {
    const place = at?.place;
    const parts = [place?.city, place?.state, place?.country].filter(
      (part): part is string => typeof part === 'string' && part !== ''
    );
    return parts.length > 0 ? parts.join(', ') : 'Unnamed place';
  });
  const lead = $derived.by(() => {
    if (initial.assets.length > 1 && located > 0) {
      return `${located} of them ${located === 1 ? 'has' : 'have'} a location; all ${initial.assets.length} move here`;
    }
    return touch ? 'Move the map under the pin' : 'Click the map or drag the pin';
  });

  function name(lat: number, lon: number): void {
    const ticket = ++naming;
    at = { lat, lon, place: null };
    reverseGeocode(lat, lon)
      .then((place) => {
        if (ticket === naming && at) at = { ...at, place };
      })
      .catch(() => undefined);
  }

  function moveTo(lat: number, lon: number): void {
    name(lat, lon);
    if (touch || !map || !lib) return;
    if (pin) pin.setLngLat([lon, lat]);
    else pin = dragPin(lat, lon);
  }

  function dragPin(lat: number, lon: number): Marker | null {
    if (!map || !lib) return null;
    const marker = placePin(lib, map, { lat, lon }, { draggable: true });
    marker.on('dragend', () => {
      const spot = marker.getLngLat();
      name(spot.lat, spot.lng);
    });
    return marker;
  }

  function ready(target: MapLibreMap, maplibre: typeof import('maplibre-gl')): () => void {
    map = target;
    lib = maplibre;
    if (touch) {
      target.on('moveend', () => {
        const center = target.getCenter();
        if (at && Math.abs(at.lat - center.lat) < 1e-7 && Math.abs(at.lon - center.lng) < 1e-7) {
          return;
        }
        name(center.lat, center.lng);
      });
    } else {
      if (at) pin = dragPin(at.lat, at.lon);
      target.on('click', (event) => moveTo(event.lngLat.lat, event.lngLat.lng));
    }
    return () => {
      pin?.remove();
      pin = null;
      map = null;
    };
  }

  function pickPlace(place: Place): void {
    map?.flyTo({ center: [place.lon, place.lat], zoom: PLACE_ZOOM, maxDuration: FLY_MAX_MS });
    moveTo(place.lat, place.lon);
  }

  function save(): void {
    if (at) void locationPicker.save(at);
  }

  onMount(() => {
    void mapConfig.ensure().catch(() => undefined);
  });
</script>

<Dialog
  {title}
  {subtitle}
  size="large"
  onClose={locationPicker.close}
  bodyClass="flex flex-col gap-3"
>
  {#snippet footer()}
    <div class="flex w-full items-center justify-end gap-2">
      <p class="me-auto min-w-0 truncate text-caption text-dark/65">{lead}</p>
      <Button
        type="button"
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        onclick={locationPicker.close}
      >
        Cancel
      </Button>
      <Button
        type="button"
        size="small"
        class="touch:h-11"
        loading={locationPicker.saving}
        disabled={!at || locationPicker.saving}
        onclick={save}
      >
        Save
      </Button>
    </div>
  {/snippet}

  {#if mapConfig.search}
    <PlaceSearch onPick={pickPlace} />
  {/if}
  <div class="relative h-85 overflow-hidden rounded-lg border border-hairline touch:h-[50vh]">
    <MapCanvas
      class="size-full"
      label="Location map"
      style={mapConfig.style}
      view={initial.start ? { ...initial.start, zoom: ZOOM } : WORLD_VIEW}
      onReady={ready}
    />
    {#if touch}
      <div
        class="pointer-events-none absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-full"
        data-map-pin
        aria-hidden="true"
      >
        <MapPin />
      </div>
    {/if}
  </div>
  <p class="flex items-center gap-2 text-sm text-light-950" role="status">
    <Icon icon={mdiMapMarkerOutline} size="16px" class="shrink-0 text-dark/65" aria-hidden="true" />
    {#if at}
      <span class="min-w-0 flex-1 truncate">{placeName}</span>
      <span class="shrink-0 text-caption text-dark/65 tabular-nums">
        {coordinateLabel(at.lat, at.lon)}
      </span>
    {:else}
      <span class="text-dark/65">No location yet</span>
    {/if}
  </p>
</Dialog>
