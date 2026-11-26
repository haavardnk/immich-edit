import type { GeoLocation } from '#lib/browse/locationOps.ts';
import { applyLocations } from '#lib/browse/locationOps.ts';
import { fieldKey, sortAssets } from '#lib/browse/sortAssets.ts';
import { planShifts, shiftCaption, shiftText, type ShiftPlan } from '#lib/metadata/captureShift.ts';
import { correctTimes } from '#lib/metadata/captureTimeOps.ts';
import { bestFit, type Fit } from '#lib/geotag/bestFit.ts';
import { CameraClocks } from '#lib/metadata/cameraClocks.svelte.ts';
import { cameraKey, camerasOf, type Camera } from '#lib/metadata/cameras.ts';
import { clocksText, driftText } from '#lib/metadata/clockText.ts';
import { countMatches, matchPhotos, type PhotoMatch } from '#lib/geotag/match.ts';
import { cameraTime, deviceZone } from '#lib/metadata/photoTime.ts';
import { cellOf, namePlaces } from '#lib/geotag/placeNames.ts';
import { parseTrackFile, trackFormat, type TrackPoint } from '#lib/geotag/trackFile.ts';
import { TrackLog } from '#lib/geotag/trackLog.ts';
import { photosAround, stillsOnly } from '#lib/geotag/trackPhotos.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const TRACK_MARGIN = HOUR;

export const TOLERANCES = [1, 2, 5, 15, 60] as const;

export interface LoadedTrack {
  id: string;
  fileName: string;
  name: string | null;
  segments: TrackPoint[][];
  points: number;
  start: number;
  end: number;
  metres: number;
}

export type PhotoScope = 'selection' | 'track';

function loaded(fileName: string, name: string | null, segments: TrackPoint[][], id: number) {
  const log = new TrackLog(segments);
  return {
    id: `track-${id}`,
    fileName,
    name,
    segments,
    points: log.size,
    start: log.start,
    end: log.end,
    metres: log.lengthMetres()
  } satisfies LoadedTrack;
}

export class TrackSession {
  active = $state(false);
  tracks = $state.raw<LoadedTrack[]>([]);
  scope = $state<PhotoScope>('track');
  candidates = $state.raw<AssetSummary[]>([]);
  zone = $state(deviceZone());
  zoneFrom = $state<'device' | 'track' | 'picked'>('device');
  toleranceMin = $state<(typeof TOLERANCES)[number]>(5);
  replace = $state(false);
  fixTime = $state(false);
  loading = $state(false);
  placing = $state(false);
  focused = $state<string | null>(null);
  revealed = $state<string | null>(null);
  #seq = 0;

  readonly log = $derived(new TrackLog(this.tracks.flatMap((track) => track.segments)));
  readonly toleranceMs = $derived(this.toleranceMin * MINUTE);
  readonly bases = $derived(
    new Map(this.candidates.map((asset) => [asset.id, cameraTime(asset, this.zone)]))
  );
  readonly clocks = new CameraClocks();
  readonly #cameraIds = $derived(
    new Map(this.candidates.map((asset) => [asset.id, cameraKey(asset)]))
  );
  readonly photos = $derived(
    sortAssets(
      this.scope === 'selection'
        ? this.candidates
        : this.candidates.filter((asset) => this.#near(asset)),
      fieldKey('taken'),
      'asc'
    )
  );
  readonly matches: PhotoMatch[] = $derived(
    matchPhotos(
      this.photos.map((asset) => ({
        id: asset.id,
        base: this.bases.get(asset.id)?.instant ?? null,
        shiftMs: this.shiftOf(asset.id),
        located: typeof asset.exifInfo?.latitude === 'number'
      })),
      this.log,
      {
        toleranceMs: this.toleranceMs,
        interpolate: true,
        replace: this.replace
      }
    )
  );
  readonly counts = $derived(countMatches(this.matches));
  readonly timeShift: ShiftPlan = $derived(
    planShifts(
      this.matches.flatMap((match) => {
        const base = this.bases.get(match.id);
        return match.outcome.kind === 'placed' && base
          ? [{ id: match.id, ...base, correctionMs: this.shiftOf(match.id) }]
          : [];
      }),
      this.zone
    )
  );
  readonly rows = $derived(
    this.photos.flatMap((asset, index) => {
      const match = this.matches[index];
      return match ? [{ asset, match }] : [];
    })
  );
  readonly #rowById = $derived(new Map(this.rows.map((row) => [row.asset.id, row])));

  row(id: string): { asset: AssetSummary; match: PhotoMatch } | null {
    return this.#rowById.get(id) ?? null;
  }
  readonly located = $derived(
    this.photos.filter((asset) => typeof asset.exifInfo?.latitude === 'number').length
  );
  readonly unzoned = $derived(
    this.photos.filter((asset) => this.bases.get(asset.id)?.recorded === false).length
  );
  readonly #fits = $derived.by(() => {
    const bases = new Map<string, number[]>();
    for (const asset of this.candidates) {
      const base = this.bases.get(asset.id);
      if (!base) continue;
      const key = this.#cameraIds.get(asset.id) ?? '';
      bases.set(key, [...(bases.get(key) ?? []), base.instant]);
    }
    return new Map(
      [...bases].map(([key, instants]) => [key, bestFit(instants, this.log, this.toleranceMs)])
    );
  });
  readonly cameras: Camera[] = $derived.by(() => {
    const near = new Set(this.photos.map((asset) => this.#cameraIds.get(asset.id) ?? ''));
    return camerasOf(this.candidates).filter(
      (camera) => near.has(camera.key) || (this.#fits.get(camera.key) ?? null) !== null
    );
  });
  readonly fits: { camera: Camera; fit: Fit }[] = $derived(
    this.cameras.flatMap((camera) => {
      const fit = this.#fits.get(camera.key) ?? null;
      return fit && fit.correctionMs !== this.clocks.of(camera.key) ? [{ camera, fit }] : [];
    })
  );
  readonly fitMatched = $derived(
    this.cameras.reduce((sum, camera) => sum + (this.#fits.get(camera.key)?.matched ?? 0), 0)
  );
  readonly fitOf = $derived(this.cameras.reduce((sum, camera) => sum + camera.count, 0));
  readonly clockText = $derived(
    clocksText(
      this.cameras.map((camera) => ({ label: camera.label, ms: this.clocks.of(camera.key) }))
    )
  );
  readonly fixCaption = $derived(
    shiftCaption(
      this.timeShift,
      this.zone,
      this.cameras.flatMap((camera) => {
        const minutes = Math.round(this.clocks.of(camera.key) / MINUTE);
        return minutes === 0 ? [] : [`${camera.label} ${shiftText(minutes)}`];
      })
    )
  );

  get camera(): string {
    return this.clocks.active(this.cameras);
  }

  get correctionMs(): number {
    return this.clocks.of(this.camera);
  }

  set correctionMs(ms: number) {
    this.clocks.set(this.camera, ms);
  }

  shiftOf(id: string): number {
    return this.clocks.of(this.#cameraIds.get(id) ?? '');
  }

  driftOf(key: string): string {
    return driftText(this.clocks.of(key));
  }

  #near(asset: AssetSummary): boolean {
    const base = this.bases.get(asset.id);
    if (!base || this.log.size === 0) return false;
    const time = base.instant + this.shiftOf(asset.id);
    return time >= this.log.start - TRACK_MARGIN && time <= this.log.end + TRACK_MARGIN;
  }

  open = (selected: readonly AssetSummary[] = []): void => {
    this.active = true;
    this.scope = selected.length > 0 ? 'selection' : 'track';
    this.candidates = stillsOnly(selected);
  };

  hoverPin = (id: string | null): void => {
    this.focused = id;
    if (id) this.revealed = id;
  };

  close = (): void => {
    this.active = false;
    this.tracks = [];
    this.candidates = [];
    this.clocks.reset();
    this.focused = null;
    this.revealed = null;
    this.replace = false;
    this.fixTime = false;
    this.zone = deviceZone();
    this.zoneFrom = 'device';
  };

  addFiles = async (files: readonly File[]): Promise<void> => {
    const readable = files.filter((file) => trackFormat(file.name) !== null);
    if (readable.length === 0) {
      toasts.push('warn', 'Choose a GPX, KML or TCX file');
      return;
    }
    this.loading = true;
    try {
      const added: LoadedTrack[] = [];
      for (const file of readable) {
        const format = trackFormat(file.name);
        if (!format) continue;
        const parsed = parseTrackFile(await file.text(), format);
        if (parsed.lines.length === 0) {
          toasts.push('warn', `${file.name} has no track with times`);
          continue;
        }
        for (const line of parsed.lines) {
          this.#seq += 1;
          added.push(loaded(file.name, line.name, line.segments, this.#seq));
        }
      }
      if (added.length === 0) return;
      this.tracks = [...this.tracks, ...added].sort((a, b) => a.start - b.start);
      await this.#followTrackZone();
      this.active = true;
      if (this.scope === 'track') await this.#loadPhotos();
    } finally {
      this.loading = false;
    }
  };

  removeTrack = (id: string): void => {
    this.tracks = this.tracks.filter((track) => track.id !== id);
    void this.#followTrackZone();
  };

  pickZone = (zone: string): void => {
    this.zone = zone;
    this.zoneFrom = 'picked';
  };

  #zonePicked(): boolean {
    return this.zoneFrom === 'picked';
  }

  async #followTrackZone(): Promise<void> {
    if (this.#zonePicked()) return;
    const first = this.tracks[0];
    const { trackZone } = await import('#lib/geotag/trackZone.ts');
    if (this.#zonePicked() || this.tracks[0] !== first) return;
    const zone = first ? trackZone(first.segments) : null;
    this.zone = zone ?? deviceZone();
    this.zoneFrom = zone ? 'track' : 'device';
  }

  #loadPhotos = async (): Promise<void> => {
    try {
      this.candidates = await photosAround(this.log.start, this.log.end);
    } catch (error) {
      toasts.fail('Could not find photos taken during the track', error);
    }
  };

  useFit = (): void => {
    for (const { camera, fit } of this.fits) this.clocks.set(camera.key, fit.correctionMs);
  };

  place = async (): Promise<boolean> => {
    if (this.placing) return false;
    const placed = this.matches.flatMap((match) =>
      match.outcome.kind === 'placed'
        ? [{ id: match.id, lat: match.outcome.lat, lon: match.outcome.lon }]
        : []
    );
    if (placed.length === 0) return false;
    this.placing = true;
    try {
      const names = await namePlaces(placed);
      const targets = new Map<string, GeoLocation>(
        placed.map((point) => [point.id, { ...point, place: names.get(cellOf(point)) ?? null }])
      );
      const assets = this.photos.filter((asset) => targets.has(asset.id));
      const shift = this.fixTime ? this.timeShift : null;
      const done = await applyLocations(
        assets,
        targets,
        shift ? (written) => correctTimes(shift, written) : undefined
      );
      if (done) this.close();
      return done;
    } finally {
      this.placing = false;
    }
  };

  get summary(): string {
    const { placed } = this.counts;
    return `${placed} of ${countOf(this.photos.length, 'photo')} on the track`;
  }
}

export const trackSession = new TrackSession();
