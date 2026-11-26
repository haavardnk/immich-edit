import { sourceId } from '#lib/browse/assetKey.ts';
import { resolveAssets } from '#lib/browse/assetLookup.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import { CameraClocks } from '#lib/metadata/cameraClocks.svelte.ts';
import { CameraZones, type CameraZone } from '#lib/metadata/cameraZones.svelte.ts';
import { cameraLabel, countCameras, type Camera } from '#lib/metadata/cameras.ts';
import { applyCaptureTimes } from '#lib/metadata/captureTimeOps.ts';
import {
  baseMoment,
  clockPhoto,
  correctionTo,
  previewRows,
  setTo,
  shifted,
  shownZone,
  wallOf,
  type ClockPhoto,
  type ClockTarget,
  type PreviewRow,
  type ZoneChoice
} from '#lib/metadata/captureTimes.ts';
import { cameraShiftText, clocksText } from '#lib/metadata/clockText.ts';
import { nameTime, withTimeOfDay } from '#lib/metadata/fileNameTime.ts';
import { deviceZone, wallToInstant } from '#lib/metadata/photoTime.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

export type CaptureMode = 'shift' | 'photo' | 'once' | 'name';
export type PreviewFilter = 'all' | 'changed' | 'reordered';

export const ALL_CAMERAS = '\u0001all';
export const NO_DATE = 'No date in name';

export interface NameSource {
  source: string;
  count: number;
}
export const SHIFT_LIMIT = 100 * 366 * 86_400_000;

const SECOND = 1000;
const REFERENCES = 8;

interface CaptureRequest {
  photos: ClockPhoto[];
  assets: Map<string, AssetSummary>;
}

function byName(a: ClockPhoto, b: ClockPhoto): number {
  return a.name.localeCompare(b.name, undefined, { numeric: true }) || a.id.localeCompare(b.id);
}

function byWall(a: ClockPhoto, b: ClockPhoto): number {
  return a.wall - b.wall || a.id.localeCompare(b.id);
}

function noonOf(wall: number): number {
  const day = new Date(wall);
  day.setUTCHours(12, 0, 0, 0);
  return day.getTime();
}

class CaptureEditor {
  request = $state.raw<CaptureRequest | null>(null);
  mode = $state<CaptureMode>('shift');
  camera = $state(ALL_CAMERAS);
  pick = $state(0);
  reference = $state<string | null>(null);
  onceWall = $state(0);
  spaced = $state(true);
  filter = $state<PreviewFilter>('all');
  saving = $state(false);
  readonly clocks = new CameraClocks();
  readonly fallback = deviceZone();
  readonly #zones = new CameraZones(this.fallback);

  readonly photos: ClockPhoto[] = $derived(this.request?.photos ?? []);
  readonly cameras: Camera[] = $derived(countCameras(this.photos.map((photo) => photo.camera)));
  readonly perCamera = $derived(this.mode !== 'once' && this.cameras.length > 1);
  readonly #scope = $derived(this.perCamera ? this.camera : ALL_CAMERAS);
  readonly scoped: ClockPhoto[] = $derived(
    this.#scope === ALL_CAMERAS
      ? this.photos
      : this.photos.filter((photo) => photo.camera === this.#scope)
  );
  readonly #keys: string[] = $derived(
    this.#scope === ALL_CAMERAS ? this.cameras.map((camera) => camera.key) : [this.#scope]
  );
  readonly zone: CameraZone | null = $derived(this.#zones.common(this.#keys));
  readonly takenIn: string | null = $derived(this.zone?.takenIn ?? null);
  readonly clockZone: string | null = $derived(this.zone?.clock ?? null);

  readonly targets: Map<string, ClockTarget> = $derived.by(() => {
    if (this.mode === 'name') {
      return new Map(
        this.scoped.flatMap((photo) => {
          const named = nameTime(photo.name);
          if (!named) return [];
          const zone = shownZone(photo, this.#zonesOf(photo));
          if (named.utc) return [[photo.id, { instant: named.wall, zone }] as const];
          const wall = named.dateOnly ? withTimeOfDay(named.wall, photo.wall) : named.wall;
          return [[photo.id, setTo(wall, zone)] as const];
        })
      );
    }
    if (this.mode === 'once') {
      return new Map(
        [...this.scoped]
          .sort(byName)
          .map((photo, index) => [
            photo.id,
            setTo(
              this.onceWall + (this.spaced ? index * SECOND : 0),
              this.#zones.of(photo.camera).takenIn ?? this.fallback
            )
          ])
      );
    }
    return new Map(
      this.photos.flatMap((photo) => {
        const target = shifted(photo, this.clocks.of(photo.camera), this.#zonesOf(photo));
        return target ? [[photo.id, target] as const] : [];
      })
    );
  });

  readonly rows: PreviewRow[] = $derived(previewRows(this.photos, this.targets));
  readonly nameSources: NameSource[] = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const photo of this.scoped) {
      const source = nameTime(photo.name)?.source ?? NO_DATE;
      counts.set(source, (counts.get(source) ?? 0) + 1);
    }
    return [...counts]
      .map(([source, count]) => ({ source, count }))
      .sort(
        (a, b) => Number(a.source === NO_DATE) - Number(b.source === NO_DATE) || b.count - a.count
      );
  });
  readonly changed = $derived(this.rows.filter((row) => row.changed).length);
  readonly reordered = $derived(this.rows.filter((row) => row.reordered).length);
  readonly shownFilter: PreviewFilter = $derived(
    (this.filter === 'changed' && this.changed === 0) ||
      (this.filter === 'reordered' && this.reordered === 0)
      ? 'all'
      : this.filter
  );
  readonly visible: PreviewRow[] = $derived(
    this.shownFilter === 'all'
      ? this.rows
      : this.rows.filter((row) => (this.shownFilter === 'changed' ? row.changed : row.reordered))
  );

  readonly pickable: ClockPhoto[] = $derived([...this.scoped].sort(byWall));
  readonly picked: ClockPhoto | null = $derived(
    this.pickable[Math.min(this.pick, this.pickable.length - 1)] ?? null
  );

  readonly references: ClockPhoto[] = $derived.by(() => {
    const picked = this.picked;
    if (!picked || this.camera === ALL_CAMERAS) return [];
    const moment = this.momentOf(picked);
    const distance = (photo: ClockPhoto): number => Math.abs(this.momentOf(photo) - moment);
    return this.photos
      .filter((photo) => photo.camera !== picked.camera)
      .sort((a, b) => distance(a) - distance(b))
      .slice(0, REFERENCES)
      .sort((a, b) => this.momentOf(a) - this.momentOf(b));
  });

  readonly summary: string = $derived.by(() => {
    if (this.mode === 'once' || this.mode === 'name') return '';
    const cameras = this.cameras.map((camera) => ({
      label: camera.label,
      ms: this.clocks.of(camera.key)
    }));
    return cameras.some(({ ms }) => ms !== 0) ? clocksText(cameras) : '';
  });

  get correction(): number {
    const key = this.camera === ALL_CAMERAS ? (this.cameras[0]?.key ?? '') : this.camera;
    return this.clocks.of(key);
  }

  set correction(ms: number) {
    if (this.camera !== ALL_CAMERAS) {
      this.clocks.set(this.camera, ms);
      return;
    }
    const delta = ms - this.correction;
    for (const camera of this.cameras) {
      this.clocks.set(camera.key, this.clocks.of(camera.key) + delta);
    }
  }

  momentOf(photo: ClockPhoto): number {
    return this.targets.get(photo.id)?.instant ?? baseMoment(photo, this.#zonesOf(photo));
  }

  shownOf(photo: ClockPhoto): number {
    const target = this.targets.get(photo.id);
    return target ? wallOf(target) : photo.wall;
  }

  shownAsPicked(photo: ClockPhoto): number {
    const picked = this.picked;
    if (!picked) return this.shownOf(photo);
    return wallOf({
      instant: this.momentOf(photo),
      zone: shownZone(picked, this.#zonesOf(picked))
    });
  }

  shiftTextOf = (key: string): string => cameraShiftText(this.clocks.of(key));

  labelOf(photo: ClockPhoto): string {
    return cameraLabel(photo.camera);
  }

  chooseCamera = (key: string): void => {
    this.camera = key;
    this.pick = 0;
    this.reference = null;
  };

  step(by: number): void {
    const count = this.pickable.length;
    if (count === 0) return;
    this.pick = (Math.min(this.pick, count - 1) + by + count) % count;
    this.reference = null;
  }

  pickPhoto = (id: string): void => {
    const photo = this.photos.find((candidate) => candidate.id === id);
    if (!photo) return;
    if (this.camera !== ALL_CAMERAS) this.camera = photo.camera;
    this.pick = Math.max(
      0,
      this.pickable.findIndex((candidate) => candidate.id === id)
    );
    this.reference = null;
  };

  setPickedTime(wall: number): void {
    const picked = this.picked;
    if (!picked) return;
    const zones = this.#zonesOf(picked);
    const moment = wallToInstant(wall, shownZone(picked, zones));
    this.#correct(picked, correctionTo(picked, moment, zones));
    this.reference = null;
  }

  matchTo(id: string): void {
    const picked = this.picked;
    const reference = this.photos.find((photo) => photo.id === id);
    if (!picked || !reference) return;
    this.#correct(picked, correctionTo(picked, this.momentOf(reference), this.#zonesOf(picked)));
    this.reference = id;
  }

  #correct(picked: ClockPhoto, ms: number): void {
    if (this.camera === ALL_CAMERAS) {
      this.correction = this.correction + ms - this.clocks.of(picked.camera);
    } else this.clocks.set(picked.camera, ms);
  }

  setTakenIn(zone: string | null): void {
    this.#zones.setTakenIn(this.#keys, zone);
  }

  setClockZone(zone: string | null): void {
    this.#zones.setClock(this.#keys, zone);
  }

  #zonesOf(photo: ClockPhoto): ZoneChoice {
    return this.#zones.choice(photo.camera);
  }

  open = (assets: readonly AssetSummary[]): void => {
    const owned = assets.filter((asset) => ownsAsset(asset));
    if (owned.length === 0) {
      toasts.push('warn', 'Another Immich user owns these photos');
      return;
    }
    if (owned.length < assets.length) {
      toasts.push(
        'info',
        `${countOf(assets.length - owned.length, 'photo')} owned by someone else left out`
      );
    }
    const sources = new Set<string>();
    const photos = owned.flatMap((asset) => {
      const photo = clockPhoto(asset);
      const source = sourceId(asset.id);
      if (!photo || sources.has(source)) return [];
      sources.add(source);
      return [photo];
    });
    this.#reset();
    this.onceWall = noonOf(photos[0]?.wall ?? Date.now());
    this.request = { photos, assets: new Map(owned.map((asset) => [asset.id, asset])) };
  };

  saveOne = async (asset: AssetSummary, wall: number, zone: string): Promise<boolean> => {
    const photo = clockPhoto(asset);
    if (!photo) return false;
    if (wall === photo.wall && zone === (photo.zone ?? this.fallback)) return true;
    return applyCaptureTimes([{ photo, target: setTo(wall, zone) }], new Map([[asset.id, asset]]));
  };

  openIds = async (ids: readonly string[]): Promise<void> => {
    try {
      this.open(await resolveAssets(ids));
    } catch (error) {
      toasts.fail('Capture time', error);
    }
  };

  close = (): void => {
    if (!this.saving) this.request = null;
  };

  apply = async (): Promise<void> => {
    const request = this.request;
    if (!request || this.saving) return;
    const changes = this.rows.flatMap((row) => {
      const target = this.targets.get(row.photo.id);
      return row.changed && target ? [{ photo: row.photo, target }] : [];
    });
    this.saving = true;
    try {
      if (await applyCaptureTimes(changes, request.assets)) this.request = null;
    } finally {
      this.saving = false;
    }
  };

  #reset(): void {
    this.mode = 'shift';
    this.camera = ALL_CAMERAS;
    this.pick = 0;
    this.reference = null;
    this.spaced = true;
    this.filter = 'all';
    this.clocks.reset();
    this.#zones.reset();
  }
}

export const captureEditor = new CaptureEditor();
