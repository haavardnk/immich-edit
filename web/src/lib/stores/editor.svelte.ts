import { neutralEdits, isIdentity, manifestToEdits, FULL_CROP, type AspectLock, type CropRect, type Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail, ExifInfo, TagRef } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits } from '$lib/api/edits';
import { livePreview, persistedPreviewUrl, getPreviewMeta, type PreviewMode } from '$lib/api/preview';
import { downloadExport, EXTENSION_BY_FORMAT, uploadToImmich, type ExportOptions, type ImmichExportOptions } from '$lib/api/export';
import { getAsset, updateAsset } from '$lib/api/assets';
import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
import { browsing } from '$lib/stores/browsing.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import { SingleFlight } from '$lib/utils/single-flight';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { downloadBlob } from '$lib/utils/download';
import { constrainCropRect, largestInscribedRect, refitCropAtAspect, aspectRatioFor } from '$lib/utils/geom';

const LIVE_EDGE = 1600;
const MAX_EDGE = 4096;
const HIRES_DEBOUNCE_MS = 300;
const MAX_HISTORY = 50;

function computeHiresEdge(zoom: number): number {
  const dpr = typeof window !== 'undefined' ? window.devicePixelRatio : 1;
  const vp = typeof window !== 'undefined' ? Math.max(window.innerWidth, window.innerHeight) : 1600;
  const needed = Math.ceil(vp * dpr * Math.max(1, zoom / 100));
  return Math.min(needed, MAX_EDGE);
}

class EditorStore {
  assetId = $state<string | null>(null);
  asset = $state<AssetDetail | null>(null);
  edits = $state<Edits>(neutralEdits());
  previewUrl = $state<string | null>(null);
  meta = $state<PreviewMeta | null>(null);
  pending = $state(false);
  saving = $state(false);
  exporting = $state(false);
  exportingToImmich = $state(false);
  lastUpload = $state<{ kind: 'success' | 'duplicate' | 'error'; message: string } | null>(null);
  autoBusy = $state(false);
  error = $state<string | null>(null);
  showingOriginal = $state(false);
  cropSession = $state<CropSession | null>(null);

  private history = $state<Edits[]>([]);
  private historyCursor = $state(-1);
  private skipHistory = false;

  private initialised = false;
  private hiresTimer: ReturnType<typeof setTimeout> | null = null;
  private renderedEdge = 0;

  private flight = new SingleFlight<{ edits: Edits; maxEdge: number; previewMode: PreviewMode }, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      this.pending = true;
      const { blob, metaId } = await livePreview(this.assetId, args.edits, args.maxEdge, args.previewMode, signal);
      return { url: makeObjectUrl(blob), metaId };
    },
    (args, result) => {
      const prev = this.previewUrl;
      this.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.pending = false;
      if (args.previewMode === 'none') {
        this.renderedEdge = args.maxEdge;
        if (result.metaId) void this.loadMeta(result.metaId);
      }
    },
    (err) => {
      this.pending = false;
      this.error = (err as Error).message;
    }
  );

  async load(id: string): Promise<void> {
    if (this.assetId === id && this.initialised) return;
    this.unload();
    this.assetId = id;
    this.error = null;
    try {
      const [a, s] = await Promise.all([getAsset(id), getEdits(id)]);
      this.asset = a;
      this.edits = manifestToEdits(s.manifest);
      this.initialised = true;
      this.pushHistory();
      const hiresEdge = computeHiresEdge(100);
      if (hiresEdge > LIVE_EDGE) {
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: 'none' });
        this.scheduleHires();
      } else {
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: hiresEdge, previewMode: 'none' });
      }
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  unload(): void {
    this.flight.cancel();
    if (this.hiresTimer) { clearTimeout(this.hiresTimer); this.hiresTimer = null; }
    if (this.cropSession) {
      if (this.cropSession.pinnedUrl) revoke(this.cropSession.pinnedUrl);
      this.cropSession = null;
    }
    if (this.previewUrl?.startsWith('blob:')) revoke(this.previewUrl);
    this.previewUrl = null;
    this.asset = null;
    this.meta = null;
    this.assetId = null;
    this.initialised = false;
    this.edits = neutralEdits();
    this.history = [];
    this.historyCursor = -1;
    this.showingOriginal = false;
    this.renderedEdge = 0;
  }

  private pushHistory(): void {
    const trimmed = this.history.slice(0, this.historyCursor + 1);
    trimmed.push($state.snapshot(this.edits));
    if (trimmed.length > MAX_HISTORY) trimmed.shift();
    this.history = trimmed;
    this.historyCursor = this.history.length - 1;
  }

  get canUndo(): boolean {
    return this.historyCursor > 0;
  }

  get canRedo(): boolean {
    return this.historyCursor < this.history.length - 1;
  }

  undo = (): void => {
    if (!this.canUndo) return;
    this.historyCursor--;
    this.edits = $state.snapshot(this.history[this.historyCursor]) as Edits;
    this.skipHistory = true;
    void this.onCommit();
    this.skipHistory = false;
  };

  redo = (): void => {
    if (!this.canRedo) return;
    this.historyCursor++;
    this.edits = $state.snapshot(this.history[this.historyCursor]) as Edits;
    this.skipHistory = true;
    void this.onCommit();
    this.skipHistory = false;
  };

  loadPersisted(): void {
    if (!this.assetId) return;
    const prev = this.previewUrl;
    this.previewUrl = persistedPreviewUrl(this.assetId, MAX_EDGE) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
  }

  showOriginal(): void {
    if (!this.initialised) return;
    this.flight.submit({ edits: neutralEdits(), maxEdge: this.renderedEdge || LIVE_EDGE, previewMode: 'none' });
  }

  onLive = (): void => {
    if (!this.initialised) return;
    this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: 'none' });
    this.scheduleHires();
  };

  onPreview = (mode: PreviewMode): void => {
    if (!this.initialised) return;
    if (this.hiresTimer) { clearTimeout(this.hiresTimer); this.hiresTimer = null; }
    this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: mode });
  };

  endPreview = (): void => {
    if (!this.initialised) return;
    this.onLive();
  };

  onCommit = async (): Promise<void> => {
    if (!this.initialised || !this.assetId) return;
    if (!this.skipHistory) this.pushHistory();
    this.onLive();
    this.saving = true;
    try {
      if (isIdentity(this.edits)) {
        await deleteEdits(this.assetId);
      } else {
        const saved = await putEdits(this.assetId, $state.snapshot(this.edits));
        this.edits = manifestToEdits(saved.manifest);
      }
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.saving = false;
    }
  };

  onReset = async (): Promise<void> => {
    if (!this.assetId) return;
    this.edits = { ...neutralEdits(), geometry: this.edits.geometry };
    await this.onCommit();
  };

  onAutoAdjust = async (): Promise<void> => {
    if (!this.assetId || !this.initialised) return;
    this.autoBusy = true;
    try {
      const suggested = await autoEdits(this.assetId);
      this.edits = {
        ...this.edits,
        basic: { ...this.edits.basic, ...suggested.basic },
        tone: { ...this.edits.tone, ...suggested.tone }
      };
      this.onLive();
      await this.onCommit();
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.autoBusy = false;
    }
  };

  onExport = async (opts: ExportOptions): Promise<void> => {
    if (!this.assetId) return;
    this.exporting = true;
    try {
      const blob = await downloadExport(this.assetId, $state.snapshot(this.edits), opts);
      const base = (this.asset?.originalFileName ?? this.assetId).replace(/\.[^.]+$/, '');
      const name = `${base}_edit.${EXTENSION_BY_FORMAT[opts.format]}`;
      downloadBlob(blob, name);
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.exporting = false;
    }
  };

  onUploadToImmich = async (opts: ImmichExportOptions): Promise<void> => {
    if (!this.assetId) return;
    this.exportingToImmich = true;
    this.lastUpload = null;
    try {
      const result = await uploadToImmich(this.assetId, $state.snapshot(this.edits), opts);
      const dup = result.status.toLowerCase() === 'duplicate';
      const msg = dup
        ? `Not uploaded: identical asset already exists in Immich (matched by content hash)`
        : `Uploaded ${result.filename} to Immich`;
      toasts.push(dup ? 'warn' : 'success', msg, 10000);
      for (const w of result.warnings) toasts.push('warn', w, 10000);
      this.lastUpload = {
        kind: dup ? 'duplicate' : 'success',
        message: result.warnings.length > 0 ? `${msg} — ${result.warnings.join('; ')}` : msg
      };
      if (opts.stackWithOriginal || opts.favorite) {
        try {
          this.asset = await getAsset(this.assetId);
        } catch {
          /* ignore */
        }
      }
    } catch (e) {
      const m = (e as Error).message;
      this.error = m;
      this.lastUpload = { kind: 'error', message: `Upload failed: ${m}` };
      toasts.push('error', `Upload failed: ${m}`, 10000);
    } finally {
      this.exportingToImmich = false;
    }
  };

  private syncBrowsing(): void {
    if (!this.assetId || !this.asset) return;
    browsing.patch(this.assetId, {
      isFavorite: this.asset.isFavorite,
      exifInfo: this.asset.exifInfo ?? null
    });
  }

  toggleFavorite = async (): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prev = this.asset.isFavorite;
    this.asset = { ...this.asset, isFavorite: !prev };
    try {
      const updated = await updateAsset(this.assetId, { isFavorite: !prev });
      this.asset = updated;
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, isFavorite: prev };
      this.error = (e as Error).message;
    }
  };

  setRating = async (rating: number | null): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prevExif: ExifInfo | null = this.asset.exifInfo;
    const nextExif: ExifInfo = prevExif
      ? { ...prevExif, rating }
      : {
          make: null,
          model: null,
          lensModel: null,
          fNumber: null,
          focalLength: null,
          iso: null,
          exposureTime: null,
          exifImageWidth: null,
          exifImageHeight: null,
          dateTimeOriginal: null,
          rating,
          fileSizeInByte: null
        };
    this.asset = { ...this.asset, exifInfo: nextExif };
    try {
      const updated = await updateAsset(this.assetId, { rating });
      this.asset = updated;
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, exifInfo: prevExif };
      this.error = (e as Error).message;
    }
  };

  addTag = async (tag: TagRef): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (this.asset.tags.some((t) => t.id === tag.id)) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: [...prev, tag] };
    try {
      await addTagToAsset(tag.id, this.assetId);
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  removeTag = async (tagId: string): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: prev.filter((t) => t.id !== tagId) };
    try {
      await removeTagFromAsset(tagId, this.assetId);
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  createAndAddTag = async (value: string): Promise<TagRef | null> => {
    if (!this.assetId || !this.asset) return null;
    try {
      const created = await upsertTags([value]);
      const tag = created[0];
      if (!tag) return null;
      const ref: TagRef = { id: tag.id, name: tag.name, value: tag.value };
      if (!this.asset.tags.some((t) => t.id === ref.id)) {
        const prev = this.asset.tags;
        this.asset = { ...this.asset, tags: [...prev, ref] };
        try {
          await addTagToAsset(ref.id, this.assetId);
        } catch (e) {
          if (this.asset) this.asset = { ...this.asset, tags: prev };
          this.error = (e as Error).message;
          return null;
        }
      }
      return ref;
    } catch (e) {
      this.error = (e as Error).message;
      return null;
    }
  };

  private scheduleHires(zoom = 100): void {
    if (this.hiresTimer) clearTimeout(this.hiresTimer);
    this.hiresTimer = setTimeout(() => {
      this.hiresTimer = null;
      if (!this.initialised) return;
      const edge = computeHiresEdge(zoom);
      if (edge <= this.renderedEdge) return;
      this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: edge, previewMode: 'none' });
    }, HIRES_DEBOUNCE_MS);
  }

  onZoomChange = (zoom: number): void => {
    if (!this.initialised) return;
    const edge = computeHiresEdge(zoom);
    if (edge <= this.renderedEdge) return;
    this.scheduleHires(zoom);
  };

  private async loadMeta(metaId: string): Promise<void> {
    if (!this.assetId) return;
    try {
      this.meta = await getPreviewMeta(this.assetId, metaId);
    } catch {
      this.meta = null;
    }
  }

  enterCropMode = (): void => {
    if (!this.assetId || !this.initialised || this.cropSession) return;
    const baseEdits = $state.snapshot(this.edits) as Edits;
    this.cropSession = {
      pinnedUrl: null,
      pinnedReady: false,
      srcW: 0,
      srcH: 0,
      baseEdits,
      draftRotate: baseEdits.geometry.rotate,
      draftFlipH: baseEdits.geometry.flip_h,
      draftFlipV: baseEdits.geometry.flip_v,
      draftAngle: baseEdits.geometry.rotate_angle,
      draftCrop: baseEdits.geometry.crop ?? FULL_CROP,
      draftAspect: baseEdits.geometry.aspect,
      userEditedCrop: baseEdits.geometry.crop !== null
    };
    void this.loadPinnedPreview(baseEdits);
  };

  private async loadPinnedPreview(baseEdits: Edits): Promise<void> {
    if (!this.assetId) return;
    const canonical: Edits = {
      ...baseEdits,
      geometry: {
        ...baseEdits.geometry,
        rotate: 0,
        flip_h: false,
        flip_v: false,
        rotate_angle: 0,
        crop: null
      }
    };
    try {
      const { blob } = await livePreview(this.assetId, canonical, LIVE_EDGE, 'none');
      const url = makeObjectUrl(blob);
      const dims = await new Promise<{ w: number; h: number }>((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight });
        img.onerror = () => reject(new Error('pinned preview decode failed'));
        img.src = url;
      });
      const sess = this.cropSession;
      if (!sess || dims.w <= 0 || dims.h <= 0) {
        revoke(url);
        return;
      }
      if (sess.pinnedUrl) revoke(sess.pinnedUrl);
      sess.pinnedUrl = url;
      sess.srcW = dims.w;
      sess.srcH = dims.h;
      sess.pinnedReady = true;
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  exitCropMode = async (): Promise<void> => {
    const sess = this.cropSession;
    if (!sess) return;
    if (sess.pinnedUrl) revoke(sess.pinnedUrl);
    this.cropSession = null;
    const dc = sess.draftCrop;
    const full = dc.x === 0 && dc.y === 0 && dc.w === 1 && dc.h === 1;
    this.edits = {
      ...this.edits,
      geometry: {
        ...this.edits.geometry,
        rotate: sess.draftRotate,
        flip_h: sess.draftFlipH,
        flip_v: sess.draftFlipV,
        rotate_angle: sess.draftAngle,
        crop: full ? null : sess.draftCrop,
        aspect: sess.draftAspect
      }
    };
    await this.onCommit();
  };

  rotateStep = (delta: 90 | 270): void => {
    const sess = this.cropSession;
    if (sess) {
      sess.draftRotate = ((sess.draftRotate + delta) % 360) as 0 | 90 | 180 | 270;
      const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
      const sw = swapped ? sess.srcH : sess.srcW;
      const sh = swapped ? sess.srcW : sess.srcH;
      const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
      sess.draftCrop = ratio !== null
        ? largestInscribedRect(sw, sh, sess.draftAngle, ratio)
        : FULL_CROP;
      sess.userEditedCrop = false;
      return;
    }
    this.edits.geometry.rotate = ((this.edits.geometry.rotate + delta) % 360) as 0 | 90 | 180 | 270;
    void this.onCommit();
  };

  flipStep = (axis: 'h' | 'v'): void => {
    const sess = this.cropSession;
    if (sess) {
      if (axis === 'h') sess.draftFlipH = !sess.draftFlipH;
      else sess.draftFlipV = !sess.draftFlipV;
      return;
    }
    if (axis === 'h') this.edits.geometry.flip_h = !this.edits.geometry.flip_h;
    else this.edits.geometry.flip_v = !this.edits.geometry.flip_v;
    void this.onCommit();
  };

  updateCropDraftAngle = (angle: number): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftAngle = angle;
    const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = refitCropAtAspect(sess.draftCrop, sw, sh, angle, ratio);
    } else if (!sess.userEditedCrop) {
      sess.draftCrop = largestInscribedRect(sw, sh, angle, sw / sh);
    } else {
      sess.draftCrop = constrainCropRect(sess.draftCrop, sess.draftCrop, sw, sh, angle);
    }
  };

  updateCropDraftCrop = (crop: CropRect): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftCrop = constrainCropRect(crop, sess.draftCrop, sw, sh, sess.draftAngle);
    sess.userEditedCrop = true;
  };

  updateCropDraftAspect = (aspect: AspectLock): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftAspect = aspect;
    const ratio = aspectRatioFor(aspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = largestInscribedRect(sw, sh, sess.draftAngle, ratio);
      sess.userEditedCrop = false;
    }
  };

  resetCropDraft = (): void => {
    const sess = this.cropSession;
    if (!sess) return;
    sess.draftAngle = 0;
    sess.draftAspect = { kind: 'original' };
    sess.draftCrop = FULL_CROP;
    sess.userEditedCrop = false;
  };
}

function sourceDims(sess: CropSession): { sw: number; sh: number } {
  const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
  return { sw: swapped ? sess.srcH : sess.srcW, sh: swapped ? sess.srcW : sess.srcH };
}

interface CropSession {
  pinnedUrl: string | null;
  pinnedReady: boolean;
  srcW: number;
  srcH: number;
  baseEdits: Edits;
  draftRotate: 0 | 90 | 180 | 270;
  draftFlipH: boolean;
  draftFlipV: boolean;
  draftAngle: number;
  draftCrop: CropRect;
  draftAspect: AspectLock;
  userEditedCrop: boolean;
}

export const editor = new EditorStore();
