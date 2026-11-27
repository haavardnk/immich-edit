import { coordinates } from '#lib/browse/info/photoInfo.ts';
import type { GeoLocation } from '#lib/browse/locationOps.ts';
import { applyLocations } from '#lib/browse/locationOps.ts';
import { fieldKey, sortAssets } from '#lib/browse/sortAssets.ts';
import { applyCaptureTimes, captureTimesAfter } from '#lib/metadata/captureTimeOps.ts';
import { clockPhoto, planWrites, type ClockChange } from '#lib/metadata/captureTimes.ts';
import { TrackCameras } from '#lib/geotag/trackCameras.svelte.ts';
import { countMatches, matchPhotos, type PhotoMatch } from '#lib/geotag/match.ts';
import { cameraTime, deviceZone, wallToInstant, type CameraTime } from '#lib/metadata/photoTime.ts';
import { cellOf, namePlaces } from '#lib/geotag/placeNames.ts';
import { trackCaptureChanges } from '#lib/geotag/trackCapture.ts';
import { loadedTrack, trackLogOf, type LoadedTrack } from '#lib/geotag/loadedTracks.ts';
import { parseTrackFile, trackFormat } from '#lib/geotag/trackFile.ts';
import {
  covers,
  photosAround,
  searchRange,
  stillsOnly,
  type TimeRange
} from '#lib/geotag/trackPhotos.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const TRACK_MARGIN = HOUR;

export const TOLERANCES = [1, 2, 5, 15, 60] as const;

export type PhotoScope = 'selection' | 'track';

export class TrackSession {
  active = $state(false);
  tracks = $state.raw<LoadedTrack[]>([]);
  scope = $state<PhotoScope>('track');
  candidates = $state.raw<AssetSummary[]>([]);
  trackZone = $state(deviceZone());
  toleranceMin = $state<(typeof TOLERANCES)[number]>(5);
  replace = $state(false);
  fixTime = $state(false);
  loading = $state(false);
  placing = $state(false);
  focused = $state<string | null>(null);
  revealed = $state<string | null>(null);
  #seq = 0;
  #loads = 0;
  #fetched: TimeRange | null = null;

  readonly log = $derived(trackLogOf(this.tracks));
  readonly toleranceMs = $derived(this.toleranceMin * MINUTE);
  readonly shifts = new TrackCameras(this);
  readonly bases = $derived(
    new Map(this.candidates.map((asset) => [asset.id, this.#baseOf(asset)]))
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
        at: asset.exifInfo ? coordinates(asset.exifInfo) : null
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
  readonly rows = $derived(
    this.photos
      .flatMap((asset, index) => {
        const match = this.matches[index];
        return match ? [{ asset, match }] : [];
      })
      .sort((a, b) => (a.match.time ?? Infinity) - (b.match.time ?? Infinity))
  );
  readonly #rowById = $derived(new Map(this.rows.map((row) => [row.asset.id, row])));
  readonly captureChanges: ClockChange[] = $derived(
    trackCaptureChanges(
      this.rows.map((row) => ({
        ...row,
        shiftMs: this.shiftOf(row.asset.id),
        zone: this.shifts.zoneOf(row.asset.id)
      })),
      this.trackZone
    )
  );
  readonly captureWrites = $derived(planWrites(this.captureChanges).write.length);
  readonly timeWrites = $derived(this.fixTime ? this.captureWrites : 0);

  row(id: string): { asset: AssetSummary; match: PhotoMatch } | null {
    return this.#rowById.get(id) ?? null;
  }
  readonly located = $derived(
    this.photos.filter((asset) => asset.exifInfo && coordinates(asset.exifInfo)).length
  );
  get selection(): boolean {
    return this.scope === 'selection';
  }

  shiftOf(id: string): number {
    return this.shifts.shiftOf(id);
  }

  #baseOf(asset: AssetSummary): CameraTime | null {
    const zone = this.shifts.zoneOf(asset.id);
    if (!zone) return cameraTime(asset, this.trackZone);
    const photo = clockPhoto(asset);
    return photo ? { instant: wallToInstant(photo.wall, zone), recorded: true } : null;
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
    this.#fetched = null;
    this.#loads += 1;
    this.shifts.reset();
    this.focused = null;
    this.revealed = null;
    this.replace = false;
    this.fixTime = false;
    this.trackZone = deviceZone();
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
          added.push(loadedTrack(file.name, line, this.#seq));
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

  setTrackOffset = (id: string, offsetMs: number): void => {
    this.#changeTrack(id, { offsetMs });
  };

  setTrackZone = (id: string, zone: string | null): void => {
    this.#changeTrack(id, { zone });
  };

  #changeTrack(id: string, change: Partial<Pick<LoadedTrack, 'offsetMs' | 'zone'>>): void {
    this.tracks = this.tracks.map((track) => (track.id === id ? { ...track, ...change } : track));
    const log = this.log;
    if (this.scope !== 'track' || log.size === 0) return;
    if (!covers(this.#fetched, log.start - TRACK_MARGIN, log.end + TRACK_MARGIN)) {
      void this.#loadPhotos();
    }
  }

  async #followTrackZone(): Promise<void> {
    const first = this.tracks[0];
    const { trackZone } = await import('#lib/geotag/trackZone.ts');
    if (this.tracks[0] !== first) return;
    const zone = first ? trackZone(first.segments) : null;
    this.trackZone = zone ?? deviceZone();
  }

  #loadPhotos = async (): Promise<void> => {
    const range = searchRange(this.log.start, this.log.end);
    this.#loads += 1;
    const load = this.#loads;
    this.#fetched = range;
    try {
      const found = await photosAround(range);
      if (load === this.#loads) this.candidates = found;
    } catch (error) {
      if (load !== this.#loads) return;
      this.#fetched = null;
      toasts.fail('Could not find photos taken during the track', error);
    }
  };

  place = async (): Promise<boolean> => {
    if (this.placing) return false;
    const placed = this.matches.flatMap((match) =>
      match.outcome.kind === 'placed'
        ? [{ id: match.id, lat: match.outcome.lat, lon: match.outcome.lon }]
        : []
    );
    const timed = this.timeWrites > 0 ? this.captureChanges : [];
    if (placed.length === 0 && timed.length === 0) return false;
    this.placing = true;
    try {
      const done =
        placed.length > 0 ? await this.#placeAll(placed, timed) : await this.#saveTimes(timed);
      if (done) this.close();
      return done;
    } finally {
      this.placing = false;
    }
  };

  #saveTimes(timed: readonly ClockChange[]): Promise<boolean> {
    return applyCaptureTimes(timed, new Map(this.photos.map((asset) => [asset.id, asset])));
  }

  async #placeAll(
    placed: readonly { id: string; lat: number; lon: number }[],
    timed: readonly ClockChange[]
  ): Promise<boolean> {
    const names = await namePlaces(placed);
    const targets = new Map<string, GeoLocation>(
      placed.map((point) => [point.id, { ...point, place: names.get(cellOf(point)) ?? null }])
    );
    const assets = this.photos.filter((asset) => targets.has(asset.id));
    const byId = new Map(this.photos.map((asset) => [asset.id, asset]));
    const then =
      timed.length > 0
        ? (written: readonly string[]) => {
            const done = new Set(written);
            const kept = timed.filter(
              (change) => !targets.has(change.photo.id) || done.has(change.photo.id)
            );
            return captureTimesAfter(kept, byId);
          }
        : undefined;
    return applyLocations(assets, targets, then);
  }

  get summary(): string {
    const { placed } = this.counts;
    return `${placed} of ${countOf(this.photos.length, 'photo')} on the track`;
  }
}

export const trackSession = new TrackSession();
