<script lang="ts">
  import type { MarkerFilter } from '#lib/api/map.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';
  import { provideTileDrag } from '#lib/browse/tileDrag.ts';
  import BulkBar from '#lib/components/browse/bulk/BulkBar.svelte';
  import SelectionAppBar from '#lib/components/browse/bulk/SelectionAppBar.svelte';
  import TrackCard from '#lib/components/geotag/TrackCard.svelte';
  import TrackPanel from '#lib/components/geotag/TrackPanel.svelte';
  import TrackPinPreview from '#lib/components/geotag/TrackPinPreview.svelte';
  import TrackSheet from '#lib/components/geotag/TrackSheet.svelte';
  import MapActions from '#lib/components/map/MapActions.svelte';
  import MapCanvas from '#lib/components/map/MapCanvas.svelte';
  import MapDropConfirm from '#lib/components/map/MapDropConfirm.svelte';
  import MapDropZone from '#lib/components/map/MapDropZone.svelte';
  import MapFilters from '#lib/components/map/MapFilters.svelte';
  import MapPhotos from '#lib/components/map/MapPhotos.svelte';
  import MapPlaceCard from '#lib/components/map/MapPlaceCard.svelte';
  import MapPlaceEdit from '#lib/components/map/MapPlaceEdit.svelte';
  import MapPlaceMenu from '#lib/components/map/MapPlaceMenu.svelte';
  import MapPlaceSheet from '#lib/components/map/MapPlaceSheet.svelte';
  import MapSidePanel from '#lib/components/map/MapSidePanel.svelte';
  import MapTray from '#lib/components/map/MapTray.svelte';
  import MapViewBar from '#lib/components/map/MapViewBar.svelte';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { idsNear } from '#lib/geotag/spots.ts';
  import { openTrackPhoto } from '#lib/geotag/trackLoupe.ts';
  import { onLocationsMoved } from '#lib/map/locationMoves.ts';
  import { parseView, readMapPhoto, WORLD_VIEW, type MapView } from '#lib/map/mapArea.ts';
  import { boxOf, MapCamera } from '#lib/map/mapCamera.svelte.ts';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import { MapDrops } from '#lib/map/mapDrops.ts';
  import { mapFullscreen } from '#lib/map/mapFullscreen.svelte.ts';
  import { forgetMap, MAP_EXITS, mapMemory, mapStart } from '#lib/map/mapMemory.ts';
  import { mapPageState } from '#lib/map/mapPage.svelte.ts';
  import { MapSide } from '#lib/map/mapSide.svelte.ts';
  import { MapTrackLayer } from '#lib/map/mapTrackLayer.svelte.ts';
  import { MapTrayState } from '#lib/map/mapTray.svelte.ts';
  import { writeMapUrl } from '#lib/map/mapUrl.ts';
  import { PhotoMarkers } from '#lib/map/photoMarkers.ts';
  import { PLACE_ZOOM, PlaceEditor, type MapPlacesHost } from '#lib/map/placeEditor.svelte.ts';
  import { watchPlaceMenu } from '#lib/map/placeMenuTriggers.ts';
  import { PlaceStars } from '#lib/map/placeStars.ts';
  import { SheetLift } from '#lib/map/sheetLift.svelte.ts';
  import { TrackFileDrop } from '#lib/map/trackFileDrop.svelte.ts';
  import { onUserMove } from '#lib/map/userMoves.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { afterNavigate, beforeNavigate } from '$app/navigation';
  import { page } from '$app/state';
  import type { Map as MapLibreMap } from 'maplibre-gl';
  import { onMount, untrack } from 'svelte';

  const PILL_SPACE = 72;
  const FIT_PAD = 64;

  const start = mapStart(page.url.searchParams, mapMemory);
  const startView = start.view ?? WORLD_VIEW;
  const map = mapPageState(start.filter);
  map.tab = mapMemory.tab;
  const tray = new MapTrayState();
  const lift = new SheetLift();
  let cardHeight = $state(0);
  const full = $derived(mapFullscreen.on);
  const camera = new MapCamera(() => {
    if (full || !layout.compact) {
      const card = side.card ? cardHeight : 0;
      return { top: FIT_PAD, left: FIT_PAD, right: FIT_PAD, bottom: FIT_PAD + card };
    }
    const height = lift.area?.getBoundingClientRect().height ?? 0;
    const sheet = tray.state === 'open' || trackSession.active;
    const bottom = sheet ? Math.round(height / 2) + FIT_PAD / 4 : PILL_SPACE;
    return { top: FIT_PAD, left: FIT_PAD, right: FIT_PAD, bottom };
  });
  let stars = $state.raw<PlaceStars | null>(null);
  let starsShown = $state(mapMemory.stars);
  let photosShown = $state(mapMemory.photos);
  const editor = new PlaceEditor();
  const fileDrop = new TrackFileDrop((files) => void trackSession.addFiles(files));
  const tracking = $derived(trackSession.active);
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
  const side = new MapSide();
  let wanted = $state<string | null>(readMapPhoto(page.url.searchParams));

  const count = $derived(map.loaded ? countOf(map.markers.length, 'photo') : undefined);

  function fitAll(animate = true): void {
    camera.fitAll(map.extent, animate);
  }

  function pick(ids: readonly string[]): void {
    map.pick(ids);
    if (full) return;
    if (layout.compact) tray.open();
    else side.photos = true;
  }

  function flyTo(at: GeoPoint): void {
    const target = camera.map;
    if (!target) return;
    target.flyTo({ center: [at.lon, at.lat], zoom: Math.max(target.getZoom(), PLACE_ZOOM) });
  }

  function mapCentre(): GeoPoint | null {
    const centre = camera.map?.getCenter();
    return centre ? { lat: centre.lat, lon: centre.lng } : null;
  }

  function leaveSheet(): void {
    starsShown = true;
    if (layout.compact) tray.close();
  }

  const host: MapPlacesHost = {
    hover: (place) => stars?.hit(place?.id ?? null),
    show: (place) => {
      leaveSheet();
      flyTo(place);
      if (layout.mode !== 'touch') editor.open(place);
    },
    goTo: (at) => {
      if (layout.compact) tray.close();
      flyTo(at);
    },
    create: (at, name) => {
      const spot = at ?? mapCentre();
      if (!spot) return;
      leaveSheet();
      editor.startCreate(spot, name);
    },
    move: (place) => {
      leaveSheet();
      if (layout.mode !== 'touch') flyTo(place);
      editor.startMove(place);
    },
    photos: (place) => {
      map.tab = 'view';
      pick(idsNear(map.markers, place));
      flyTo(place);
    }
  };

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
    const starLayer = new PlaceStars(target, lib, (place) => editor.open(place));
    const stopMenu = watchPlaceMenu(target, (menu) => {
      if (!editor.edit && !tracking) editor.menu = menu;
    });
    const stopTrack = track.attach(target);
    const closeCard = (): void => {
      editor.close();
    };
    target.on('click', closeCard);
    camera.map = target;
    markers = manager;
    stars = starLayer;
    map.box = boxOf(target);
    return () => {
      stopMoves();
      stopMenu();
      target.off('click', closeCard);
      starLayer.destroy();
      stars = null;
      stopTrack();
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
    if (!navigation.to || !MAP_EXITS.has(navigation.to.url.pathname)) return;
    forgetMap();
    trackSession.close();
  });

  afterNavigate((navigation) => {
    if (navigation.type === 'enter' || navigation.type === 'popstate') return;
    if (navigation.from?.url.pathname !== navigation.to?.url.pathname) return;
    const at = navigation.to?.url.searchParams.get('at') ?? null;
    if (at === (navigation.from?.url.searchParams.get('at') ?? null)) return;
    const view = parseView(at);
    const photo = navigation.to ? readMapPhoto(navigation.to.url.searchParams) : null;
    if (!view || (!camera.follow(view) && !photo)) return;
    fitted = true;
    map.showAll();
    wanted = photo;
  });

  $effect(() => {
    const id = wanted;
    if (!id || !map.loaded || !markers) return;
    wanted = null;
    if (!map.markers.some((marker) => marker.id === id)) return;
    untrack(() => {
      pick([id]);
      camera.focus(map.markers, [id]);
    });
  });

  $effect(() => {
    const manager = markers;
    const list = map.markers;
    const drawn = tracking && !photosShown ? [] : list;
    if (!manager) return;
    untrack(() => manager.set(drawn));
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
    mapMemory.panel = side.photos;
    mapMemory.stars = starsShown;
    mapMemory.photos = photosShown;
  });

  $effect(() => {
    const layer = stars;
    const list = starsShown ? places.saved : [];
    if (layer) untrack(() => layer.set(list));
    if (!starsShown) editor.close();
  });

  $effect(() => {
    const client = drops.drag.over ? drops.drag.client : null;
    untrack(() => drops.hover(client));
  });

  const track = new MapTrackLayer(camera);

  $effect(() => {
    if (tracking || browseView.loupeId !== null || !map.loaded) return;
    const tab = map.tab;
    if (tab === 'view') untrack(() => map.feed.attach());
    else if (tab === 'unplaced') untrack(() => map.unplaced.attach());
  });

  $effect(() => {
    if (tracking) return;
    if (map.tab === 'view') {
      if (!map.loaded) return;
      const ids = map.shownIds;
      untrack(() => void map.feed.show(ids));
    } else if (map.tab === 'unplaced') {
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
  <MapActions panel={side.shown} onPanel={() => side.toggle()} />
{/snippet}

{#snippet mapView(zone: string, canvas: string)}
  <MapDropZone
    drag={drops.drag}
    onDrop={drops.dropped}
    class="{zone} {tracking || editor.edit !== null
      ? '[&_[data-map-cluster]]:pointer-events-none [&_[data-map-cluster]]:opacity-35! [&_[data-map-photo]]:pointer-events-none [&_[data-map-photo]]:opacity-35!'
      : ''}"
  >
    <MapCanvas
      class={canvas}
      label="Map of your photos"
      style={mapConfig.style}
      view={startView}
      onReady={ready}
      onMoved={moved}
      zoomBelow
    />
    <MapDropConfirm confirm={drops.confirm} map={camera.map} />
    {#if track.card}
      {@const card = track.card}
      <TrackPinPreview
        id={card.id}
        at={card.at}
        compact={layout.mode === 'touch'}
        onOpen={layout.mode === 'touch' ? () => openTrackPhoto(card.id) : undefined}
      />
    {/if}
    {#if editor.card && layout.mode === 'touch'}
      <MapPlaceSheet
        place={editor.card}
        markers={map.markers}
        onMove={host.move}
        onClose={editor.close}
      />
    {:else if editor.card}
      <MapPlaceCard
        place={editor.card}
        map={camera.map}
        markers={map.markers}
        onMove={host.move}
        onClose={editor.close}
      />
    {/if}
    {#if editor.menu}
      <MapPlaceMenu
        menu={editor.menu}
        onSave={(menu) => host.create(menu.at)}
        onClose={() => (editor.menu = null)}
      />
    {/if}
    <MapPlaceEdit {editor} map={camera.map} {stars} touch={layout.mode === 'touch'} />
    <MapViewBar
      bind:stars={starsShown}
      bind:photos={photosShown}
      canFit={!!map.extent}
      onFit={() => fitAll()}
    />
    {#if side.card}
      <TrackCard bind:height={cardHeight} />
    {/if}
    {#if fileDrop.over}
      <div
        class="pointer-events-none absolute inset-2.5 z-30 grid place-items-center rounded-xl bg-light/35 ring-2 ring-primary"
        role="status"
      >
        <span class="rounded-xl bg-light-100 px-4 py-2.5 text-sm text-light-950 shadow-popover">
          Drop to load the track
        </span>
      </div>
    {/if}
  </MapDropZone>
{/snippet}

<div
  class={full
    ? 'fixed inset-0 z-36 flex h-(--app-height) flex-col bg-light safe-frame'
    : 'flex min-h-0 min-w-0 flex-1 flex-col'}
  data-map-fullscreen={full || undefined}
  {@attach fileDrop.attach}
>
  {#if !full}
    <PageHeader
      title="Map"
      {count}
      badge={map.loaded ? String(map.markers.length) : undefined}
      {actions}
      compactActions={actions}
      toolbar={filters}
      selecting={bulkScope.selecting ? selectionBar : undefined}
    />
  {/if}
  {#if map.error}
    <p class="px-6 py-3 text-sm text-danger" role="alert">{map.error}</p>
  {/if}
  {#if layout.compact && tracking && layout.landscape}
    <div class="relative flex min-h-0 flex-1">
      {@render mapView('flex min-w-0 flex-1', 'min-w-0 flex-1')}
      <aside
        aria-label="Geotag from track"
        class="w-75 shrink-0 flex-col border-s border-hairline bg-light {full ? 'hidden' : 'flex'}"
      >
        <TrackPanel />
      </aside>
    </div>
  {:else if layout.compact}
    <div class="relative min-h-0 flex-1" {@attach lift.attach}>
      <div class="absolute inset-0">
        {@render mapView('size-full', 'size-full')}
      </div>
      {#if tracking}
        <TrackSheet lift={lift.value} hidden={full || browseView.loupeId !== null} />
      {:else}
        <MapTray
          {map}
          {host}
          {tray}
          lift={lift.value}
          hidden={full || browseView.loupeId !== null || editor.edit !== null}
        />
      {/if}
    </div>
  {:else}
    <div class="relative flex min-h-0 flex-1">
      {@render mapView('flex min-w-0 flex-1', 'min-w-0 flex-1')}
      <MapSidePanel
        label={tracking ? 'Geotag from track' : 'Photos in view'}
        shown={!full && side.shown}
      >
        {#if tracking}
          <TrackPanel />
        {:else}
          <MapPhotos {map} {host} />
        {/if}
      </MapSidePanel>
      {#if !full}
        <BulkBar placement="float" />
      {/if}
    </div>
  {/if}
  {#if !full}
    <BulkBar placement="dock" />
  {/if}
</div>
