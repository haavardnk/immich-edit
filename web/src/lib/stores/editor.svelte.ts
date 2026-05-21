import { neutralEdits, isIdentity, manifestToEdits, type Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits } from '$lib/api/edits';
import { livePreview, persistedPreviewUrl, getPreviewMeta } from '$lib/api/preview';
import { downloadExport } from '$lib/api/export';
import { getAsset } from '$lib/api/assets';
import { SingleFlight } from '$lib/utils/single-flight';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { downloadBlob } from '$lib/utils/download';

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
  autoBusy = $state(false);
  error = $state<string | null>(null);
  showingOriginal = $state(false);

  private history = $state<Edits[]>([]);
  private historyCursor = $state(-1);
  private skipHistory = false;

  private initialised = false;
  private hiresTimer: ReturnType<typeof setTimeout> | null = null;
  private renderedEdge = 0;

  private flight = new SingleFlight<{ edits: Edits; maxEdge: number }, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      this.pending = true;
      const { blob, metaId } = await livePreview(this.assetId, args.edits, args.maxEdge, signal);
      return { url: makeObjectUrl(blob), metaId };
    },
    (args, result) => {
      const prev = this.previewUrl;
      this.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.pending = false;
      this.renderedEdge = args.maxEdge;
      if (result.metaId) void this.loadMeta(result.metaId);
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
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE });
        this.scheduleHires();
      } else {
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: hiresEdge });
      }
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  unload(): void {
    this.flight.cancel();
    if (this.hiresTimer) { clearTimeout(this.hiresTimer); this.hiresTimer = null; }
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
    this.flight.submit({ edits: neutralEdits(), maxEdge: this.renderedEdge || LIVE_EDGE });
  }

  onLive = (): void => {
    if (!this.initialised) return;
    this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE });
    this.scheduleHires();
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
    this.edits = neutralEdits();
    this.saving = true;
    try {
      await deleteEdits(this.assetId);
    } finally {
      this.saving = false;
    }
    this.loadPersisted();
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

  onExport = async (): Promise<void> => {
    if (!this.assetId) return;
    this.exporting = true;
    try {
      const blob = await downloadExport(this.assetId, $state.snapshot(this.edits));
      const name =
        (this.asset?.originalFileName ?? this.assetId).replace(/\.[^.]+$/, '') + '_edit.jpg';
      downloadBlob(blob, name);
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.exporting = false;
    }
  };

  private scheduleHires(zoom = 100): void {
    if (this.hiresTimer) clearTimeout(this.hiresTimer);
    this.hiresTimer = setTimeout(() => {
      this.hiresTimer = null;
      if (!this.initialised) return;
      const edge = computeHiresEdge(zoom);
      if (edge <= this.renderedEdge) return;
      this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: edge });
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
}

export const editor = new EditorStore();
