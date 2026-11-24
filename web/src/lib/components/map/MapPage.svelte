<script lang="ts">
  import { page } from '$app/state';
  import { afterNavigate, beforeNavigate } from '$app/navigation';
  import MapCanvas from '#lib/components/map/MapCanvas.svelte';
  import MapDropConfirm from '#lib/components/map/MapDropConfirm.svelte';
  import MapDropZone from '#lib/components/map/MapDropZone.svelte';
  import MapFilters from '#lib/components/map/MapFilters.svelte';
  import MapPlaceCard from '#lib/components/map/MapPlaceCard.svelte';
  import MapSidePanel from '#lib/components/map/MapSidePanel.svelte';
  import MapTray from '#lib/components/map/MapTray.svelte';
  import BulkBar from '#lib/components/browse/bulk/BulkBar.svelte';
  import SelectionAppBar from '#lib/components/browse/bulk/SelectionAppBar.svelte';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { provideTileDrag } from '#lib/browse/tileDrag.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { MarkerFilter } from '#lib/api/map.ts';
  import type { SavedPlace } from '#lib/api/places.ts';
  import { onLocationsMoved } from '#lib/map/locationMoves.ts';
  import { WORLD_VIEW, parseView, type MapView } from '#lib/map/mapArea.ts';
  import { boxOf, MapCamera } from '#lib/map/mapCamera.svelte.ts';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import { MapDrops } from '#lib/map/mapDrops.ts';
  import { forgetMap, MAP_EXITS, mapMemory, mapStart } from '#lib/map/mapMemory.ts';
  import { MapPageState } from '#lib/map/mapPage.svelte.ts';
  import { MapTrayState } from '#lib/map/mapTray.svelte.ts';
  import { writeMapUrl } from '#lib/map/mapUrl.ts';
  import { PhotoMarkers } from '#lib/map/photoMarkers.ts';
  import { PlaceStars } from '#lib/map/placeStars.ts';
  import { SheetLift } from '#lib/map/sheetLift.svelte.ts';
  import { onUserMove } from '#lib/map/userMoves.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDockRight, mdiFitToScreenOutline, mdiStar, mdiStarOutline } from '@mdi/js';
  import type { Map as MapLibreMap } from 'maplibre-gl';
  import { onMount, untrack } from 'svelte';

  const PILL_SPACE = 72;
  const FIT_PAD = 64;

  const start = mapStart(page.url.searchParams, mapMemory);
  const startView = start.view ?? WORLD_VIEW;
  const map = new MapPageState(start.filter);
  map.tab = mapMemory.tab;
  const tray = new MapTrayState();
  const lift = new SheetLift();
  const camera = new MapCamera(() => {
    if (!layout.compact) return FIT_PAD;
    const height = lift.area?.getBoundingClientRect().height ?? 0;
    const bottom = tray.state === 'open' ? Math.round(height / 2) + FIT_PAD / 4 : PILL_SPACE;
    return { top: FIT_PAD, left: FIT_PAD, right: FIT_PAD, bottom };
  });
  let stars = $state.raw<PlaceStars | null>(null);
  let starsShown = $state(mapMemory.stars);
  let picked = $state.raw<SavedPlace | null>(null);
  const drops = new MapDrops({
    map: () => camera.map,
    area: () => lift.area,
    tray,
    stars: () => (starsShown ? stars : null)
  });
  provideTileDrag(drops.drag);

  let markers = $state.raw<PhotoMarkers | null>(null);
  let fitted = start.view !== null;
  let settled = start.view !== null;
  let panelShown = $state(mapMemory.panel ?? layout.mode !== 'touch');

  const count = $derived(map.loaded ? countOf(map.markers.length, 'photo') : undefined);

  function fitAll(animate = true): void {
    camera.fitAll(map.extent, animate);
  }

  function pick(ids: readonly string[]): void {
    map.pick(ids);
    if (layout.compact) tray.open();
    else panelShown = true;
  }

  function ready(target: MapLibreMap, lib: typeof import('maplibre-gl')): () => void {
    const manager = new PhotoMarkers(target, lib, {
      pick,
      focus: (ids) => camera.focus(map.markers, ids),
      mark: (ids) => selection.setMany(ids, !ids.every(selection.has)),
      move: (ids, lat, lon) => drops.ask(ids, lat, lon, 'move', false)
    });
    const stopMoves = onUserMove(target, () => {
      map.showAll();
      if (tray.state === 'open') tray.close();
    });
    const starLayer = new PlaceStars(target, lib, (place) => (picked = place));
    const closeCard = (): void => {
      picked = null;
    };
    target.on('click', closeCard);
    camera.map = target;
    markers = manager;
    stars = starLayer;
    map.box = boxOf(target);
    return () => {
      stopMoves();
      target.off('click', closeCard);
      starLayer.destroy();
      stars = null;
      manager.destroy();
      camera.map = null;
      markers = null;
    };
  }

  function moved(view: MapView): void {
    camera.shown = view;
    mapMemory.view = view;
    if (camera.map) map.box = boxOf(camera.map);
    writeMapUrl(view, map.filter);
  }

  function setFilter(filter: MarkerFilter): void {
    map.setFilter(filter);
    mapMemory.filter = filter;
    writeMapUrl(null, filter);
    fitted = false;
  }

  beforeNavigate((navigation) => {
    if (navigation.to && MAP_EXITS.has(navigation.to.url.pathname)) forgetMap();
  });

  afterNavigate((navigation) => {
    if (navigation.type === 'enter' || navigation.type === 'popstate') return;
    const at = navigation.to?.url.searchParams.get('at') ?? null;
    if (at === (navigation.from?.url.searchParams.get('at') ?? null)) return;
    const view = parseView(at);
    if (!view || !camera.follow(view)) return;
    fitted = true;
    map.showAll();
  });

  $effect(() => {
    const manager = markers;
    const list = map.markers;
    if (!manager) return;
    untrack(() => manager.set(list));
    if (!fitted && list.length > 0) {
      fitted = true;
      const animate = settled;
      settled = true;
      untrack(() => fitAll(animate));
    }
  });

  $effect(() => {
    const manager = markers;
    void selection.selected;
    if (manager) untrack(() => manager.mark(selection.has));
  });

  $effect(() => {
    mapMemory.tab = map.tab;
    mapMemory.panel = panelShown;
    mapMemory.stars = starsShown;
  });

  $effect(() => {
    const layer = stars;
    const list = starsShown ? places.saved : [];
    if (layer) untrack(() => layer.set(list));
    if (!starsShown) picked = null;
  });

  $effect(() => {
    const client = drops.drag.over ? drops.drag.client : null;
    untrack(() => drops.hover(client));
  });

  $effect(() => {
    if (map.tab === 'view') {
      const ids = map.shownIds;
      untrack(() => void map.feed.show(ids));
    } else {
      const filter = map.filter;
      untrack(() => void map.unplaced.show(filter));
    }
  });

  onMount(() => {
    void mapConfig.ensure().catch(() => undefined);
    void places.ensure();
    void map.load();
    return onLocationsMoved((moves) => map.move(moves));
  });
</script>

{#snippet filters()}
  <MapFilters filter={map.filter} onChange={setFilter} />
{/snippet}

{#snippet selectionBar()}
  <SelectionAppBar />
{/snippet}

{#snippet actions()}
  <IconButton
    size="small"
    variant="ghost"
    color={starsShown ? 'primary' : 'secondary'}
    class="touch:size-11"
    icon={starsShown ? mdiStar : mdiStarOutline}
    title={starsShown ? 'Hide saved places' : 'Show saved places'}
    aria-label="Saved places"
    aria-pressed={starsShown}
    onclick={() => (starsShown = !starsShown)}
  />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class="touch:size-11"
    icon={mdiFitToScreenOutline}
    title="Fit all photos"
    aria-label="Fit all photos"
    disabled={!map.extent}
    onclick={() => fitAll()}
  />
  {#if !layout.compact}
    <IconButton
      size="small"
      variant="ghost"
      color={panelShown ? 'primary' : 'secondary'}
      class="touch:size-11"
      icon={mdiDockRight}
      title={panelShown ? 'Hide photos' : 'Show photos'}
      aria-label="Photos panel"
      aria-pressed={panelShown}
      onclick={() => (panelShown = !panelShown)}
    />
  {/if}
{/snippet}

{#snippet mapView(zone: string, canvas: string)}
  <MapDropZone drag={drops.drag} onDrop={drops.dropped} class={zone}>
    <MapCanvas
      class={canvas}
      label="Map of your photos"
      style={mapConfig.style}
      view={startView}
      onReady={ready}
      onMoved={moved}
    />
    <MapDropConfirm confirm={drops.confirm} map={camera.map} />
    {#if picked}
      <MapPlaceCard
        place={picked}
        map={camera.map}
        markers={map.markers}
        onClose={() => (picked = null)}
      />
    {/if}
  </MapDropZone>
{/snippet}

<div class="flex min-h-0 min-w-0 flex-1 flex-col">
  <PageHeader
    title="Map"
    {count}
    badge={map.loaded ? String(map.markers.length) : undefined}
    {actions}
    compactActions={actions}
    toolbar={filters}
    selecting={bulkScope.selecting ? selectionBar : undefined}
  />
  {#if map.error}
    <p class="px-6 py-3 text-sm text-danger" role="alert">{map.error}</p>
  {/if}
  {#if layout.compact}
    <div class="relative min-h-0 flex-1" {@attach lift.attach}>
      <div class="absolute inset-0">
        {@render mapView('size-full', 'size-full')}
      </div>
      <MapTray {map} {tray} lift={lift.value} hidden={browseView.loupeId !== null} />
    </div>
  {:else}
    <div class="relative flex min-h-0 flex-1">
      {@render mapView('flex min-w-0 flex-1', 'min-w-0 flex-1')}
      <MapSidePanel {map} shown={panelShown} />
      <BulkBar placement="float" />
    </div>
  {/if}
  <BulkBar placement="dock" />
</div>
