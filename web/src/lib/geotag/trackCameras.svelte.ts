import { camerasText, changeText } from '#lib/geotag/cameraChanges.ts';
import { CameraClocks } from '#lib/metadata/cameraClocks.svelte.ts';
import { cameraKey, camerasOf, type Camera } from '#lib/metadata/cameras.ts';
import { asShotText, type ShotZone } from '#lib/metadata/captureLabels.ts';
import { clockPhoto } from '#lib/metadata/captureTimes.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export interface CameraSource {
  readonly candidates: readonly AssetSummary[];
  readonly trackZone: string;
}

export interface CameraRow {
  camera: Camera;
  shiftMs: number;
  zone: string | null;
  asShot: string;
  change: string;
}

export class TrackCameras {
  readonly clocks = new CameraClocks();
  readonly #source: CameraSource;
  #zones = $state<Record<string, string>>({});

  constructor(source: CameraSource) {
    this.#source = source;
  }

  readonly #ids = $derived.by(
    () => new Map(this.#source.candidates.map((asset) => [asset.id, cameraKey(asset)]))
  );

  readonly #shots = $derived.by(() => {
    const shots = new Map<string, ShotZone[]>();
    for (const asset of this.#source.candidates) {
      const photo = clockPhoto(asset);
      if (!photo) continue;
      const list = shots.get(photo.camera) ?? [];
      list.push({ zone: photo.zone ?? this.#source.trackZone, at: photo.stored ?? photo.wall });
      shots.set(photo.camera, list);
    }
    return shots;
  });

  readonly rows: CameraRow[] = $derived.by(() =>
    camerasOf(this.#source.candidates).map((camera) => {
      const shiftMs = this.clocks.of(camera.key);
      const zone = this.zoneOfCamera(camera.key);
      return {
        camera,
        shiftMs,
        zone,
        asShot: asShotText(this.#shots.get(camera.key) ?? []),
        change: changeText(zone, shiftMs)
      };
    })
  );

  readonly status = $derived(
    camerasText(this.rows.map((row) => ({ label: row.camera.label, change: row.change })))
  );

  keyOf(id: string): string {
    return this.#ids.get(id) ?? '';
  }

  shiftOf(id: string): number {
    return this.clocks.of(this.keyOf(id));
  }

  zoneOfCamera(key: string): string | null {
    return this.#zones[key] ?? null;
  }

  zoneOf(id: string): string | null {
    return this.zoneOfCamera(this.keyOf(id));
  }

  set = (key: string, ms: number): void => {
    this.clocks.set(key, ms);
  };

  setZone = (key: string, zone: string | null): void => {
    const { [key]: _, ...rest } = this.#zones;
    this.#zones = zone === null ? rest : { ...rest, [key]: zone };
  };

  reset = (): void => {
    this.clocks.reset();
    this.#zones = {};
  };
}
