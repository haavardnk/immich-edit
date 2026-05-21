import { NEUTRAL_EDITS, isIdentity, manifestToEdits, type Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits } from '$lib/api/edits';
import { livePreview, persistedPreviewUrl, getPreviewMeta } from '$lib/api/preview';
import { downloadExport } from '$lib/api/export';
import { getAsset } from '$lib/api/assets';
import { SingleFlight } from '$lib/utils/single-flight';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { downloadBlob } from '$lib/utils/download';

const MAX_EDGE = 1600;

class EditorStore {
  assetId = $state<string | null>(null);
  asset = $state<AssetDetail | null>(null);
  edits = $state<Edits>({ ...NEUTRAL_EDITS });
  previewUrl = $state<string | null>(null);
  meta = $state<PreviewMeta | null>(null);
  pending = $state(false);
  saving = $state(false);
  exporting = $state(false);
  autoBusy = $state(false);
  error = $state<string | null>(null);

  private initialised = false;
  private flight = new SingleFlight<{ edits: Edits }, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      this.pending = true;
      const { blob, metaId } = await livePreview(this.assetId, args.edits, MAX_EDGE, signal);
      return { url: makeObjectUrl(blob), metaId };
    },
    (_args, result) => {
      const prev = this.previewUrl;
      this.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.pending = false;
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
      this.edits = { ...NEUTRAL_EDITS, ...manifestToEdits(s.manifest) };
      this.initialised = true;
      this.flight.submit({ edits: $state.snapshot(this.edits) });
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  unload(): void {
    this.flight.cancel();
    if (this.previewUrl?.startsWith('blob:')) revoke(this.previewUrl);
    this.previewUrl = null;
    this.asset = null;
    this.meta = null;
    this.assetId = null;
    this.initialised = false;
    this.edits = { ...NEUTRAL_EDITS };
  }

  loadPersisted(): void {
    if (!this.assetId) return;
    const prev = this.previewUrl;
    this.previewUrl = persistedPreviewUrl(this.assetId, MAX_EDGE) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
  }

  onLive = (): void => {
    if (!this.initialised) return;
    this.flight.submit({ edits: $state.snapshot(this.edits) });
  };

  onCommit = async (): Promise<void> => {
    if (!this.initialised || !this.assetId) return;
    this.onLive();
    this.saving = true;
    try {
      if (isIdentity(this.edits)) {
        await deleteEdits(this.assetId);
      } else {
        const saved = await putEdits(this.assetId, $state.snapshot(this.edits));
        this.edits = { ...this.edits, ...manifestToEdits(saved.manifest) };
      }
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.saving = false;
    }
  };

  onReset = async (): Promise<void> => {
    if (!this.assetId) return;
    this.edits = { ...NEUTRAL_EDITS };
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
        exposure_ev: suggested.exposure_ev,
        contrast: suggested.contrast,
        highlights: suggested.highlights,
        shadows: suggested.shadows,
        saturation: suggested.saturation,
        wb_temp: suggested.wb_temp,
        wb_tint: suggested.wb_tint
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
