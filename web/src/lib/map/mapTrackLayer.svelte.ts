import { layout } from '#lib/device/layout.svelte.ts';
import type { PhotoOutcome } from '#lib/geotag/match.ts';
import { focusLink, pinSpot, trackMarks } from '#lib/geotag/references.ts';
import { openTrackPhoto } from '#lib/geotag/trackLoupe.ts';
import { followPoint } from '#lib/map/followPoint.ts';
import { boundsOf } from '#lib/map/mapArea.ts';
import type { MapCamera } from '#lib/map/mapCamera.svelte.ts';
import { TrackOverlay, type PinPoint } from '#lib/map/trackOverlay.ts';
import { trackSession } from '#lib/stores/trackSession.svelte.ts';
import type { Map as MapLibreMap } from 'maplibre-gl';
import { untrack } from 'svelte';

export interface PinCard {
  id: string;
  at: PinPoint;
}

export class MapTrackLayer {
  readonly #camera: MapCamera;
  #overlay = $state.raw<TrackOverlay | null>(null);
  #pinCard = $state.raw<PinCard | null>(null);
  #rowCard = $state.raw<PinCard | null>(null);

  readonly card = $derived(trackSession.active ? (this.#pinCard ?? this.#rowCard) : null);

  constructor(camera: MapCamera) {
    this.#camera = camera;

    $effect(() => {
      const layer = this.#overlay;
      const paths = trackSession.active ? trackSession.log.path() : [];
      if (!layer) return;
      untrack(() => layer.setTrack(paths));
      const points = paths.flat().map(([lon, lat]) => ({ lat, lon }));
      if (points.length > 0) untrack(() => this.#camera.fitAll(boundsOf(points)));
    });

    $effect(() => {
      const layer = this.#overlay;
      const marks = trackMarks(trackSession.active ? trackSession.matches : []);
      if (layer) untrack(() => layer.setMarks(marks));
    });

    $effect(() => {
      const layer = this.#overlay;
      const id = trackSession.focused;
      const link = trackSession.active && id ? focusLink(this.#outcome(id)) : null;
      if (layer) untrack(() => layer.setLink(link));
    });

    $effect(() => {
      const target = this.#camera.map;
      const id = trackSession.focused;
      const spot = id ? pinSpot(this.#outcome(id)) : null;
      this.#rowCard = null;
      if (!target || !trackSession.active || !id || !spot) return;
      return followPoint(target, spot, (point) => {
        const inView =
          point.x >= 0 && point.y >= 0 && point.x <= point.width && point.y <= point.height;
        this.#rowCard = inView ? { id, at: point } : null;
      });
    });

    $effect(() => {
      const focused = trackSession.focused;
      this.#overlay?.focus(focused);
    });

    $effect(() => {
      const target = this.#camera.map;
      const id = trackSession.focused;
      const spot = id ? pinSpot(this.#outcome(id)) : null;
      if (!target || layout.mode !== 'touch' || !trackSession.active || !spot) return;
      untrack(() => {
        const at = target.project([spot.lon, spot.lat]);
        const box = target.getContainer();
        const inView =
          at.x >= 0 && at.y >= 0 && at.x <= box.clientWidth && at.y <= box.clientHeight;
        if (!inView) target.easeTo({ center: [spot.lon, spot.lat] });
      });
    });
  }

  attach(target: MapLibreMap): () => void {
    const overlay = new TrackOverlay(target, {
      hover: (id, at) => {
        trackSession.hoverPin(id);
        this.#pinCard = id && at ? { id, at } : null;
      },
      open: (id) => {
        if (layout.mode === 'touch' && trackSession.focused !== id) trackSession.hoverPin(id);
        else openTrackPhoto(id);
      },
      blank: () => {
        if (layout.mode === 'touch') trackSession.hoverPin(null);
      }
    });
    this.#overlay = overlay;
    return () => {
      overlay.destroy();
      this.#overlay = null;
    };
  }

  #outcome(id: string): PhotoOutcome | undefined {
    return trackSession.row(id)?.match.outcome;
  }
}
