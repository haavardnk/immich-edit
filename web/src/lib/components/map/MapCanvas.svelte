<script lang="ts">
  import { loadMapLibre } from '#lib/map/maplibre.ts';
  import type { MapView } from '#lib/map/mapArea.ts';
  import { centreShift } from '#lib/map/resizeHold.ts';
  import MapZoom from '#lib/components/map/MapZoom.svelte';
  import { errorMessage } from '#lib/utils/errors.ts';
  import type { Map as MapLibreMap, StyleSpecification } from 'maplibre-gl';
  import { untrack } from 'svelte';
  import type { Attachment } from 'svelte/attachments';

  type MapLibre = typeof import('maplibre-gl');

  let {
    style,
    view,
    interactive = true,
    label,
    onReady,
    onMoved,
    class: className = ''
  }: {
    style: string | null;
    view: MapView;
    interactive?: boolean;
    label: string;
    onReady?: (map: MapLibreMap, lib: MapLibre) => (() => void) | void;
    onMoved?: (view: MapView) => void;
    class?: string;
  } = $props();

  let map = $state.raw<MapLibreMap | null>(null);
  let failure = $state<string | null>(null);

  function blankStyle(host: HTMLElement): StyleSpecification {
    const color = getComputedStyle(host).backgroundColor;
    return {
      version: 8,
      sources: {},
      layers: [{ id: 'background', type: 'background', paint: { 'background-color': color } }]
    };
  }

  function styleFor(host: HTMLElement, url: string | null): string | StyleSpecification {
    return url ?? blankStyle(host);
  }

  function foldAttribution(target: MapLibreMap): void {
    target
      .getContainer()
      .querySelector('.maplibregl-ctrl-attrib')
      ?.classList.remove('maplibregl-compact-show');
  }

  function currentView(target: MapLibreMap): MapView {
    const center = target.getCenter();
    return { lat: center.lat, lon: center.lng, zoom: target.getZoom() };
  }

  function holdInPlace(element: HTMLElement, target: MapLibreMap): () => void {
    let last = element.getBoundingClientRect();
    const observer = new ResizeObserver(() => {
      const next = element.getBoundingClientRect();
      const shift = centreShift(last, next);
      last = next;
      target.resize();
      if (shift.x !== 0 || shift.y !== 0) target.panBy([shift.x, shift.y], { duration: 0 });
    });
    observer.observe(element);
    return () => observer.disconnect();
  }

  const host: Attachment<HTMLDivElement> = (element) => {
    let created: MapLibreMap | null = null;
    let release: (() => void) | void;
    let unwatch: (() => void) | undefined;
    let gone = false;
    const initial = untrack(() => ({ style, view, interactive }));
    loadMapLibre()
      .then((lib) => {
        if (gone) return;
        created = new lib.Map({
          container: element,
          style: styleFor(element, initial.style),
          center: [initial.view.lon, initial.view.lat],
          zoom: initial.view.zoom,
          interactive: initial.interactive,
          attributionControl: { compact: true },
          dragRotate: false,
          pitchWithRotate: false,
          trackResize: false
        });
        created.touchZoomRotate.disableRotation();
        shownStyle = initial.style;
        const target = created;
        target.on('styledata', () => target.once('idle', () => foldAttribution(target)));
        created.on('moveend', () => {
          if (created) onMoved?.(currentView(created));
        });
        unwatch = holdInPlace(element, created);
        release = onReady?.(created, lib);
        map = created;
      })
      .catch((error: unknown) => {
        failure = errorMessage(error);
      });
    return () => {
      gone = true;
      unwatch?.();
      release?.();
      created?.remove();
      map = null;
    };
  };

  let shownStyle: string | null = null;

  $effect(() => {
    const next = style;
    const target = map;
    if (!target || next === shownStyle) return;
    shownStyle = next;
    target.setStyle(styleFor(target.getContainer(), next));
  });
</script>

<div class="relative isolate overflow-hidden bg-subtle {className}">
  <div {@attach host} class="h-full w-full" role="region" aria-label={label}></div>
  {#if interactive}
    <MapZoom {map} />
  {/if}
  {#if failure}
    <p
      class="absolute inset-0 grid place-items-center p-6 text-center text-sm text-dark/65"
      role="status"
    >
      The map could not load: {failure}
    </p>
  {/if}
</div>
