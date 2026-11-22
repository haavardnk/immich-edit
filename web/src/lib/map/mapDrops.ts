import { placeAt } from '#lib/browse/locationOps.ts';
import type { Point } from '#lib/gestures/point.ts';
import { DropConfirm, type DropVerb } from '#lib/map/dropConfirm.svelte.ts';
import type { MapTrayState } from '#lib/map/mapTray.svelte.ts';
import { PhotoDrag } from '#lib/map/photoDrag.svelte.ts';
import type { Map as MapLibreMap } from 'maplibre-gl';

const TRAY_GRAB = 56;
const SHEET = '[data-map-sheet]';

export interface DropSurface {
  map: () => MapLibreMap | null;
  area: () => HTMLElement | null;
  tray: MapTrayState;
}

export class MapDrops {
  readonly confirm = new DropConfirm();
  readonly drag: PhotoDrag;
  #surface: DropSurface;

  constructor(surface: DropSurface) {
    this.#surface = surface;
    this.drag = new PhotoDrag({
      leaves: (point) => this.#leavesTray(point),
      accepts: (point) => this.#overMap(point),
      lift: () => surface.tray.carry(),
      settle: () => surface.tray.close(),
      drop: (ids, point) => this.dropped(ids, point)
    });
  }

  dropped = (ids: readonly string[], client: Point): void => {
    const map = this.#surface.map();
    if (!map) return;
    const box = map.getContainer().getBoundingClientRect();
    const spot = map.unproject([client.x - box.left, client.y - box.top]);
    void this.ask(ids, spot.lat, spot.lng, 'place', true);
  };

  ask = async (
    ids: readonly string[],
    lat: number,
    lon: number,
    verb: DropVerb,
    pin: boolean
  ): Promise<boolean> => {
    const place = await this.confirm.ask({ ids, lat, lon, verb, pin });
    if (place === false) return false;
    return placeAt(ids, lat, lon, place);
  };

  #leavesTray(point: Point): boolean {
    const sheet = document.querySelector(SHEET)?.getBoundingClientRect();
    return sheet !== undefined && point.y < sheet.top + TRAY_GRAB;
  }

  #overMap(point: Point): boolean {
    const area = (
      this.#surface.area() ?? this.#surface.map()?.getContainer()
    )?.getBoundingClientRect();
    if (!area) return false;
    const sheet =
      this.#surface.tray.state === 'open'
        ? document.querySelector(SHEET)?.getBoundingClientRect()
        : undefined;
    const bottom = sheet ? Math.min(area.bottom, sheet.top) : area.bottom;
    return point.x >= area.left && point.x <= area.right && point.y >= area.top && point.y < bottom;
  }
}
