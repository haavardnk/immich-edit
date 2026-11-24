<script lang="ts">
  import { page } from '$app/state';
  import { afterNavigate, beforeNavigate } from '$app/navigation';
  import MapCanvas from '#lib/components/map/MapCanvas.svelte';
  import MapDropConfirm from '#lib/components/map/MapDropConfirm.svelte';
  import MapDropZone from '#lib/components/map/MapDropZone.svelte';
  import MapFilters from '#lib/components/map/MapFilters.svelte';
  import MapPhotos from '#lib/components/map/MapPhotos.svelte';
  import MapPlaceCard from '#lib/components/map/MapPlaceCard.svelte';
  import MapSidePanel from '#lib/components/map/MapSidePanel.svelte';
  import MapTray from '#lib/components/map/MapTray.svelte';
  import BulkBar from '#lib/components/browse/bulk/BulkBar.svelte';
  import SelectionAppBar from '#lib/components/browse/bulk/SelectionAppBar.svelte';
  import TrackPanel from '#lib/components/geotag/TrackPanel.svelte';
  import TrackPinPreview from '#lib/components/geotag/TrackPinPreview.svelte';
  import TrackSheet from '#lib/components/geotag/TrackSheet.svelte';
  import { openTrackPhoto } from '#lib/geotag/trackLoupe.ts';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { provideTileDrag } from '#lib/browse/tileDrag.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { MarkerFilter } from '#lib/api/map.ts';
  import type { SavedPlace } from '#lib/api/places.ts';
  import { onLocationsMoved } from '#lib/map/locationMoves.ts';
  import { WORLD_VIEW, boundsOf, parseView, type MapView } from '#lib/map/mapArea.ts';
  import { boxOf, MapCamera } from '#lib/map/mapCamera.svelte.ts';
  import { followPoint } from '#lib/map/followPoint.ts';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import { MapDrops } from '#lib/map/mapDrops.ts';
  import { forgetMap, MAP_EXITS, mapMemory, mapStart } from '#lib/map/mapMemory.ts';
  import { MapPageState } from '#lib/map/mapPage.svelte.ts';
  import { MapTrayState } from '#lib/map/mapTray.svelte.ts';
  import { writeMapUrl } from '#lib/map/mapUrl.ts';
  import { PhotoMarkers } from '#lib/map/photoMarkers.ts';
  import { PlaceStars } from '#lib/map/placeStars.ts';
  import { SheetLift } from '#lib/map/sheetLift.svelte.ts';
  import { TrackFileDrop } from '#lib/map/trackFileDrop.svelte.ts';
  import { TrackOverlay, type PinPoint } from '#lib/map/trackOverlay.ts';
  import { onUserMove } from '#lib/map/userMoves.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { IconButton } from '@immich/ui';
  import {
    mdiDockRight,
    mdiFitToScreenOutline,
    mdiMapMarkerPath,
    mdiStar,
    mdiStarOutline
  } from '@mdi/js';
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
    const sheet = tray.state === 'open' || trackSession.active;
    const bottom = sheet ? Math.round(height / 2) + FIT_PAD / 4 : PILL_SPACE;
    return { top: FIT_PAD, left: FIT_PAD, right: FIT_PAD, bottom };
  });
  let stars = $state.raw<PlaceStars | null>(null);
  let starsShown = $state(mapMemory.stars);
  let picked = $state.raw<SavedPlace | null>(null);
  let pinCard = $state.raw<{ id: string; at: PinPoint } | null>(null);
  let rowCard = $state.raw<{ id: string; at: PinPoint } | null>(null);
  const shownCard = $derived(pinCard ?? rowCard);
  let overlay = $state.raw<TrackOverlay | null>(null);
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
    const trackLayer = new TrackOverlay(target, {
      hover: (id, at) => {
        trackSession.hoverPin(id);
        pinCard = id && at ? { id, at } : null;
      },
      open: (id) => {
        if (layout.mode === 'touch' && trackSession.focused !== id) trackSession.hoverPin(id);
        else openTrackPhoto(id);
      },
      blank: () => {
        if (layout.mode === 'touch') trackSession.hoverPin(null);
      }
    });
    const closeCard = (): void => {
      picked = null;
    };
    target.on('click', closeCard);
    camera.map = target;
    markers = manager;
    stars = starLayer;
    overlay = trackLayer;
    map.box = boxOf(target);
    return () => {
      stopMoves();
      target.off('click', closeCard);
      starLayer.destroy();
      stars = null;
      trackLayer.destroy();
      overlay = null;
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
    const layer = overlay;
    const paths = tracking ? trackSession.log.path() : [];
    if (!layer) return;
    untrack(() => layer.setTrack(paths));
    const points = paths.flat().map(([lon, lat]) => ({ lat, lon }));
    if (points.length > 0) untrack(() => camera.fitAll(boundsOf(points)));
  });

  $effect(() => {
    const layer = overlay;
    const pins = tracking
      ? trackSession.matches.flatMap((match) =>
          match.outcome.kind === 'placed'
            ? [{ id: match.id, lat: match.outcome.lat, lon: match.outcome.lon }]
            : []
        )
      : [];
    if (layer) untrack(() => layer.setPins(pins));
  });

  $effect(() => {
    if (tracking || browseView.loupeId !== null || !map.loaded) return;
    const tab = map.tab;
    untrack(() => (tab === 'view' ? map.feed.attach() : map.unplaced.attach()));
  });

  $effect(() => {
    const target = camera.map;
    const id = trackSession.focused;
    const outcome = id ? trackSession.row(id)?.match.outcome : undefined;
    rowCard = null;
    if (!target || !tracking || !id || outcome?.kind !== 'placed') return;
    return followPoint(target, outcome, (point) => {
      const inView =
        point.x >= 0 && point.y >= 0 && point.x <= point.width && point.y <= point.height;
      rowCard = inView ? { id, at: point } : null;
    });
  });

  $effect(() => {
    const focused = trackSession.focused;
    overlay?.focus(focused);
  });

  $effect(() => {
    const target = camera.map;
    const id = trackSession.focused;
    const outcome = id ? trackSession.row(id)?.match.outcome : undefined;
    if (!target || layout.mode !== 'touch' || !tracking || outcome?.kind !== 'placed') return;
    untrack(() => {
      const at = target.project([outcome.lon, outcome.lat]);
      const box = target.getContainer();
      const inView = at.x >= 0 && at.y >= 0 && at.x <= box.clientWidth && at.y <= box.clientHeight;
      if (!inView) target.easeTo({ center: [outcome.lon, outcome.lat] });
    });
  });

  $effect(() => {
    if (tracking) return;
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
    color={tracking ? 'primary' : 'secondary'}
    class="touch:size-11"
    icon={mdiMapMarkerPath}
    title={tracking ? 'Close track' : 'Match photos to a track'}
    aria-label="Match track"
    aria-pressed={tracking}
    onclick={() => (tracking ? trackSession.close() : trackSession.open())}
  />
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
  {#if !layout.compact && !tracking}
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
  <MapDropZone
    drag={drops.drag}
    onDrop={drops.dropped}
    class="{zone} {tracking
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
    />
    <MapDropConfirm confirm={drops.confirm} map={camera.map} />
    {#if tracking && shownCard}
      <TrackPinPreview
        id={shownCard.id}
        at={shownCard.at}
        compact={layout.mode === 'touch'}
        onOpen={layout.mode === 'touch' ? () => openTrackPhoto(shownCard.id) : undefined}
      />
    {/if}
    {#if picked}
      <MapPlaceCard
        place={picked}
        map={camera.map}
        markers={map.markers}
        onClose={() => (picked = null)}
      />
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

<div class="flex min-h-0 min-w-0 flex-1 flex-col" {@attach fileDrop.attach}>
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
  {#if layout.compact && tracking && layout.landscape}
    <div class="relative flex min-h-0 flex-1">
      {@render mapView('flex min-w-0 flex-1', 'min-w-0 flex-1')}
      <aside
        aria-label="Match track"
        class="flex w-75 shrink-0 flex-col border-s border-hairline bg-light"
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
        <TrackSheet lift={lift.value} hidden={browseView.loupeId !== null} />
      {:else}
        <MapTray {map} {tray} lift={lift.value} hidden={browseView.loupeId !== null} />
      {/if}
    </div>
  {:else}
    <div class="relative flex min-h-0 flex-1">
      {@render mapView('flex min-w-0 flex-1', 'min-w-0 flex-1')}
      <MapSidePanel
        label={tracking ? 'Match track' : 'Photos in view'}
        shown={tracking || panelShown}
      >
        {#if tracking}
          <TrackPanel />
        {:else}
          <MapPhotos {map} />
        {/if}
      </MapSidePanel>
      <BulkBar placement="float" />
    </div>
  {/if}
  <BulkBar placement="dock" />
</div>
