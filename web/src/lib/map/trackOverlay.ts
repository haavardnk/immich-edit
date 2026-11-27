import type { FeatureCollection, LineString, Point } from 'geojson';
import type {
  ExpressionSpecification,
  GeoJSONSource,
  MapLayerMouseEvent,
  MapMouseEvent,
  Map as MapLibreMap
} from 'maplibre-gl';
import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';
import type { Connector, TrackMarks } from '#lib/geotag/references.ts';

const LINE = 'track-line';
const ENDS = 'track-ends';
const PINS = 'track-pins';
const REFERENCES = 'track-references';
const CONNECTORS = 'track-connectors';
const LINK = 'track-link';
const LAYERS = [
  `${LINE}-casing`,
  LINE,
  CONNECTORS,
  LINK,
  ENDS,
  `${LINK}-target`,
  `${PINS}-halo`,
  REFERENCES,
  PINS
] as const;
const SOURCES = [LINE, CONNECTORS, LINK, ENDS, PINS] as const;
const IS_POINT: ExpressionSpecification = ['==', ['geometry-type'], 'Point'];
const IS_PIN: ExpressionSpecification = ['==', ['get', 'role'], 'pin'];
const NO_MARKS: TrackMarks = { pins: [], references: [], connectors: [] };

export interface PinPoint {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PinEvents {
  hover: (id: string | null, at: PinPoint | null) => void;
  open: (id: string) => void;
  blank: () => void;
}

const HIT = `${PINS}-halo`;
const TOUCH_MOUSE_MS = 1000;

interface Colors {
  primary: string;
  reference: string;
  base: string;
  text: string;
}

function probe(host: HTMLElement, className: string): string {
  const span = document.createElement('span');
  span.className = className;
  host.append(span);
  const color = getComputedStyle(span).color;
  span.remove();
  return color;
}

function colors(host: HTMLElement): Colors {
  return {
    primary: probe(host, 'text-primary'),
    reference: probe(host, 'text-success'),
    base: probe(host, 'text-light'),
    text: probe(host, 'text-light-950')
  };
}

function lines(paths: readonly [number, number][][]): FeatureCollection<LineString> {
  return {
    type: 'FeatureCollection',
    features: paths.map((coordinates) => ({
      type: 'Feature',
      properties: {},
      geometry: { type: 'LineString', coordinates }
    }))
  };
}

function points(
  spots: readonly (GeoPoint & { id: string; role?: string })[]
): FeatureCollection<Point> {
  return {
    type: 'FeatureCollection',
    features: spots.map((spot) => ({
      type: 'Feature',
      properties: { id: spot.id, role: spot.role ?? 'pin' },
      geometry: { type: 'Point', coordinates: [spot.lon, spot.lat] }
    }))
  };
}

function marked(marks: TrackMarks): FeatureCollection<Point> {
  return points([
    ...marks.references.map((spot) => ({ ...spot, role: spot.moved ? 'moved' : 'kept' })),
    ...marks.pins
  ]);
}

function connectors(links: readonly Connector[]): FeatureCollection<LineString> {
  return lines(
    links.map((link) => [
      [link.from.lon, link.from.lat],
      [link.to.lon, link.to.lat]
    ])
  );
}

function linked(link: Connector | null): FeatureCollection<LineString | Point> {
  if (!link) return { type: 'FeatureCollection', features: [] };
  return {
    type: 'FeatureCollection',
    features: [...connectors([link]).features, ...points([{ ...link.to, id: link.id }]).features]
  };
}

export class TrackOverlay {
  #map: MapLibreMap;
  #paths: [number, number][][] = [];
  #marks: TrackMarks = NO_MARKS;
  #link: Connector | null = null;
  #focus: string | null = null;
  #events: PinEvents;
  #hovered: string | null = null;
  #touchedAt = Number.NEGATIVE_INFINITY;
  #paint: Colors | null = null;

  constructor(map: MapLibreMap, events: PinEvents) {
    this.#map = map;
    this.#events = events;
    map.on('styledata', this.#ensure);
    map.on('idle', this.#ensure);
    map.on('mousemove', HIT, this.#move);
    map.on('mouseleave', HIT, this.#leave);
    map.on('click', HIT, this.#click);
    map.on('click', this.#elsewhere);
    map.on('touchstart', this.#touched);
    this.#ensure();
  }

  setTrack(paths: [number, number][][]): void {
    this.#paths = paths;
    void this.#source(LINE)?.setData(lines(paths));
    const ends = paths.flatMap((path, index) => {
      const first = path[0];
      const last = path[path.length - 1];
      return first && last
        ? [
            { id: `start-${index}`, lon: first[0], lat: first[1] },
            { id: `end-${index}`, lon: last[0], lat: last[1] }
          ]
        : [];
    });
    void this.#source(ENDS)?.setData(points(ends));
  }

  setMarks(marks: TrackMarks): void {
    this.#marks = marks;
    void this.#source(PINS)?.setData(marked(marks));
    void this.#source(CONNECTORS)?.setData(connectors(marks.connectors));
  }

  setLink(link: Connector | null): void {
    this.#link = link;
    void this.#source(LINK)?.setData(linked(link));
  }

  focus(id: string | null): void {
    this.#focus = id;
    const map = this.#map;
    const paint = this.#paint;
    if (!paint || !map.getLayer(PINS) || !map.getLayer(HIT) || !map.getLayer(REFERENCES)) return;
    const focused: ExpressionSpecification = ['==', ['get', 'id'], id ?? ''];
    const accent: ExpressionSpecification = ['case', IS_PIN, paint.primary, paint.reference];
    map.setPaintProperty(PINS, 'circle-radius', ['case', focused, 10, 6]);
    map.setPaintProperty(PINS, 'circle-color', ['case', focused, paint.primary, paint.base]);
    map.setPaintProperty(PINS, 'circle-stroke-color', ['case', focused, paint.base, paint.primary]);
    map.setPaintProperty(REFERENCES, 'circle-radius', ['case', focused, 8, 5]);
    map.setPaintProperty(HIT, 'circle-radius', ['case', focused, 18, IS_PIN, 9, 7]);
    map.setPaintProperty(HIT, 'circle-color', ['case', focused, accent, paint.base]);
    map.setPaintProperty(HIT, 'circle-opacity', ['case', focused, 0.3, 0.6]);
    map.setLayoutProperty(PINS, 'circle-sort-key', ['case', focused, 1, 0]);
    map.setLayoutProperty(REFERENCES, 'circle-sort-key', ['case', focused, 1, 0]);
    map.setLayoutProperty(HIT, 'circle-sort-key', ['case', focused, 1, 0]);
  }

  destroy(): void {
    this.#map.off('styledata', this.#ensure);
    this.#map.off('idle', this.#ensure);
    this.#map.off('mousemove', HIT, this.#move);
    this.#map.off('mouseleave', HIT, this.#leave);
    this.#map.off('click', HIT, this.#click);
    this.#map.off('click', this.#elsewhere);
    this.#map.off('touchstart', this.#touched);
    if (!this.#map.style) return;
    for (const layer of [...LAYERS].reverse()) {
      if (this.#map.getLayer(layer)) this.#map.removeLayer(layer);
    }
    for (const source of SOURCES) {
      if (this.#map.getSource(source)) this.#map.removeSource(source);
    }
  }

  #pinAt(event: MapLayerMouseEvent): string | null {
    const id: unknown = event.features?.[0]?.properties.id;
    return typeof id === 'string' ? id : null;
  }

  #touched = (): void => {
    this.#touchedAt = performance.now();
  };

  #move = (event: MapLayerMouseEvent): void => {
    if (performance.now() - this.#touchedAt < TOUCH_MOUSE_MS) return;
    const id = this.#pinAt(event);
    const canvas = this.#map.getCanvas();
    if (id !== this.#hovered) canvas.style.cursor = id ? 'pointer' : '';
    this.#hovered = id;
    const box = this.#map.getContainer();
    this.#events.hover(
      id,
      id
        ? { x: event.point.x, y: event.point.y, width: box.clientWidth, height: box.clientHeight }
        : null
    );
  };

  #leave = (): void => {
    if (this.#hovered === null) return;
    this.#hovered = null;
    this.#map.getCanvas().style.cursor = '';
    this.#events.hover(null, null);
  };

  #click = (event: MapLayerMouseEvent): void => {
    const id = this.#pinAt(event);
    if (id) this.#events.open(id);
  };

  #elsewhere = (event: MapMouseEvent): void => {
    const map = this.#map;
    const hit = map.getLayer(HIT) ? map.queryRenderedFeatures(event.point, { layers: [HIT] }) : [];
    if (hit.length === 0) this.#events.blank();
  };

  #source(id: string): GeoJSONSource | undefined {
    return this.#map.getSource<GeoJSONSource>(id);
  }

  #ensure = (): void => {
    const map = this.#map;
    if (!map.isStyleLoaded() || map.getSource(LINE)) return;
    const paint = colors(map.getContainer());
    this.#paint = paint;
    map.addSource(LINE, { type: 'geojson', data: lines(this.#paths) });
    map.addSource(CONNECTORS, { type: 'geojson', data: connectors(this.#marks.connectors) });
    map.addSource(LINK, { type: 'geojson', data: linked(this.#link) });
    map.addSource(ENDS, { type: 'geojson', data: points([]) });
    map.addSource(PINS, { type: 'geojson', data: marked(this.#marks) });
    map.addLayer({
      id: `${LINE}-casing`,
      type: 'line',
      source: LINE,
      layout: { 'line-cap': 'round', 'line-join': 'round' },
      paint: { 'line-color': paint.base, 'line-width': 8, 'line-opacity': 0.7 }
    });
    map.addLayer({
      id: LINE,
      type: 'line',
      source: LINE,
      layout: { 'line-cap': 'round', 'line-join': 'round' },
      paint: { 'line-color': paint.primary, 'line-width': 4 }
    });
    map.addLayer({
      id: CONNECTORS,
      type: 'line',
      source: CONNECTORS,
      layout: { 'line-cap': 'round' },
      paint: { 'line-color': paint.reference, 'line-width': 2, 'line-dasharray': [2, 2] }
    });
    map.addLayer({
      id: LINK,
      type: 'line',
      source: LINK,
      filter: ['!', IS_POINT],
      layout: { 'line-cap': 'round' },
      paint: { 'line-color': paint.reference, 'line-width': 3 }
    });
    map.addLayer({
      id: ENDS,
      type: 'circle',
      source: ENDS,
      paint: {
        'circle-radius': 6,
        'circle-color': paint.text,
        'circle-stroke-color': paint.base,
        'circle-stroke-width': 2
      }
    });
    map.addLayer({
      id: `${LINK}-target`,
      type: 'circle',
      source: LINK,
      filter: IS_POINT,
      paint: {
        'circle-radius': 7,
        'circle-color': paint.base,
        'circle-stroke-color': paint.reference,
        'circle-stroke-width': 3
      }
    });
    map.addLayer({
      id: `${PINS}-halo`,
      type: 'circle',
      source: PINS,
      paint: { 'circle-radius': 9, 'circle-color': paint.base, 'circle-opacity': 0.6 }
    });
    map.addLayer({
      id: REFERENCES,
      type: 'circle',
      source: PINS,
      filter: ['!', IS_PIN],
      paint: {
        'circle-radius': 5,
        'circle-color': paint.reference,
        'circle-opacity': ['case', ['==', ['get', 'role'], 'moved'], 0, 1],
        'circle-stroke-color': [
          'case',
          ['==', ['get', 'role'], 'moved'],
          paint.reference,
          paint.base
        ],
        'circle-stroke-width': 2
      }
    });
    map.addLayer({
      id: PINS,
      type: 'circle',
      source: PINS,
      filter: IS_PIN,
      paint: {
        'circle-radius': 6,
        'circle-color': paint.base,
        'circle-stroke-color': paint.primary,
        'circle-stroke-width': 3
      }
    });
    this.setTrack(this.#paths);
    this.focus(this.#focus);
  };
}
