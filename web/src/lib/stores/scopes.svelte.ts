import { getPreviewScope } from '$lib/api/preview';
import type { ScopeGrid, ScopeKind } from '$lib/types/preview';
import { readStored, writeStored } from '$lib/utils/storage';

export type ScopeMode = 'histogram' | ScopeKind;
export type WaveformChannels = 'luma' | 'rgb';

const STORAGE_KEY = 'immich-edit:scopes';
const MODES: ScopeMode[] = ['histogram', 'waveform', 'parade', 'vectorscope'];
const CHANNELS: WaveformChannels[] = ['luma', 'rgb'];

export const MIN_GAIN = 1;
export const MAX_GAIN = 8;
export const MIN_SCOPE_HEIGHT = 96;
export const MAX_SCOPE_HEIGHT = 360;
const DEFAULT_SCOPE_HEIGHT = 128;

type Persisted = {
  mode: ScopeMode;
  channels: WaveformChannels;
  gain: number;
  zoom: number;
  pinned: boolean;
  height: number;
};

class ScopesStore {
  mode = $state<ScopeMode>('histogram');
  channels = $state<WaveformChannels>('luma');
  gain = $state(1);
  zoom = $state(1);
  pinned = $state(false);
  height = $state(DEFAULT_SCOPE_HEIGHT);
  panelOpen = $state(true);
  grid = $state<ScopeGrid | null>(null);

  private assetId: string | null = null;
  private metaId: string | null = null;
  private hasScopes = $state(false);
  private failed = $state(false);
  private inflight: AbortController | null = null;

  wants = $derived(this.panelOpen && this.mode !== 'histogram');
  needsRender = $derived(this.wants && !this.hasScopes);
  loading = $derived(this.wants && this.grid === null && !this.failed);

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY);
    if (stored?.mode && MODES.includes(stored.mode)) this.mode = stored.mode;
    if (stored?.channels && CHANNELS.includes(stored.channels)) this.channels = stored.channels;
    if (typeof stored?.gain === 'number') this.gain = clampGain(stored.gain);
    if (stored?.zoom === 1 || stored?.zoom === 2) this.zoom = stored.zoom;
    if (typeof stored?.pinned === 'boolean') this.pinned = stored.pinned;
    if (typeof stored?.height === 'number') this.height = clampHeight(stored.height);
  }

  private persist(): void {
    writeStored(STORAGE_KEY, {
      mode: this.mode,
      channels: this.channels,
      gain: this.gain,
      zoom: this.zoom,
      pinned: this.pinned,
      height: this.height
    } satisfies Persisted);
  }

  setMode(mode: ScopeMode): void {
    if (this.mode === mode) return;
    this.mode = mode;
    this.grid = null;
    this.persist();
    this.fetch();
  }

  setChannels(channels: WaveformChannels): void {
    this.channels = channels;
    this.grid = null;
    this.persist();
    this.fetch();
  }

  setGain(gain: number): void {
    this.gain = clampGain(gain);
    this.persist();
  }

  setZoom(zoom: number): void {
    this.zoom = zoom === 2 ? 2 : 1;
    this.persist();
  }

  togglePinned = (): void => {
    this.pinned = !this.pinned;
    this.persist();
  };

  setHeight = (height: number): void => {
    this.height = clampHeight(height);
  };

  commitHeight = (): void => {
    this.persist();
  };

  setPanelOpen(open: boolean): void {
    this.panelOpen = open;
    if (open) this.fetch();
    else this.cancel();
  }

  onMeta(assetId: string, metaId: string, hasScopes: boolean): void {
    this.assetId = assetId;
    this.metaId = metaId;
    this.hasScopes = hasScopes;
    this.fetch();
  }

  reset(): void {
    this.cancel();
    this.assetId = null;
    this.metaId = null;
    this.hasScopes = false;
    this.failed = false;
    this.grid = null;
  }

  private cancel(): void {
    this.inflight?.abort();
    this.inflight = null;
  }

  private fetch(): void {
    this.cancel();
    this.failed = false;
    const assetId = this.assetId;
    const metaId = this.metaId;
    const kind = this.kind();
    if (!assetId || !metaId || !kind || !this.hasScopes) return;
    const controller = new AbortController();
    this.inflight = controller;
    getPreviewScope(assetId, metaId, kind, controller.signal)
      .then((grid) => {
        if (this.inflight !== controller) return;
        this.grid = grid;
        this.inflight = null;
      })
      .catch(() => {
        if (this.inflight !== controller) return;
        this.grid = null;
        this.failed = true;
        this.inflight = null;
      });
  }

  private kind(): ScopeKind | null {
    if (this.mode === 'histogram') return null;
    if (this.mode === 'waveform' && this.channels === 'rgb') return 'parade';
    return this.mode;
  }
}

function clampGain(gain: number): number {
  if (!Number.isFinite(gain)) return MIN_GAIN;
  return Math.min(MAX_GAIN, Math.max(MIN_GAIN, gain));
}

function clampHeight(height: number): number {
  if (!Number.isFinite(height)) return DEFAULT_SCOPE_HEIGHT;
  return Math.round(Math.min(MAX_SCOPE_HEIGHT, Math.max(MIN_SCOPE_HEIGHT, height)));
}

export const scopes = new ScopesStore();
