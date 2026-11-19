import { layout } from '#lib/device/layout.svelte.ts';
import {
  availableMode,
  nextCompareMode,
  ORIGINAL_SOURCE,
  preferredMode,
  sameSource,
  type CompareMode,
  type CompareSource
} from '#lib/editor/compare.ts';

export interface CompareCtx {
  readonly geometrySession: unknown;
  readonly snapshots: { readonly items: readonly { id: string; manifest_hash: string }[] };
}

export function sideBySideAvailable(): boolean {
  return !(layout.mode === 'touch' && layout.phone && !layout.landscape);
}

export class CompareControl {
  private chosen = $state<CompareMode>('off');
  private picked = $state.raw<CompareSource>(ORIGINAL_SOURCE);
  private split = $state(0.5);
  private pickedHash: string | null = null;
  private readonly ctx: CompareCtx;
  private readonly changed: (wasOn: boolean) => void;
  private readonly sideAvailable: () => boolean;

  constructor(
    ctx: CompareCtx,
    changed: (wasOn: boolean) => void,
    sideAvailable: () => boolean = sideBySideAvailable
  ) {
    this.ctx = ctx;
    this.changed = changed;
    this.sideAvailable = sideAvailable;
  }

  get mode(): CompareMode {
    return availableMode(this.chosen, this.sideAvailable());
  }

  get source(): CompareSource {
    return this.picked;
  }

  get sideBySide(): boolean {
    return this.sideAvailable();
  }

  get splitPos(): number {
    return this.split;
  }

  setSplitPos(pos: number): void {
    this.split = Math.min(1, Math.max(0, pos));
  }

  cycle(): void {
    if (this.ctx.geometrySession) return;
    this.apply(nextCompareMode(this.mode, this.sideAvailable()), this.picked);
  }

  setMode(mode: CompareMode): void {
    if (this.ctx.geometrySession && mode !== 'off') return;
    this.apply(availableMode(mode, this.sideAvailable()), this.picked);
  }

  setSource(source: CompareSource): void {
    if (this.chosen === 'off') return;
    this.apply(this.chosen, source);
  }

  open(source: CompareSource): void {
    if (this.ctx.geometrySession) return;
    this.apply(preferredMode(this.sideAvailable()), source);
  }

  end(): void {
    this.apply('off', ORIGINAL_SOURCE);
  }

  snapshotsChanged(): void {
    if (this.picked.kind !== 'snapshot') return;
    const hash = this.hashOf(this.picked);
    if (hash !== null && hash === this.pickedHash) return;
    if (hash === null) this.picked = ORIGINAL_SOURCE;
    this.pickedHash = hash;
    this.changed(true);
  }

  versionRemoved(id: string): void {
    if (this.picked.kind !== 'version' || this.picked.id !== id) return;
    this.picked = ORIGINAL_SOURCE;
    this.pickedHash = null;
    this.changed(true);
  }

  reset(): void {
    this.chosen = 'off';
    this.picked = ORIGINAL_SOURCE;
    this.pickedHash = null;
  }

  private hashOf(source: CompareSource): string | null {
    if (source.kind !== 'snapshot') return null;
    return this.ctx.snapshots.items.find((item) => item.id === source.id)?.manifest_hash ?? null;
  }

  private apply(mode: CompareMode, source: CompareSource): void {
    const wasOn = this.chosen !== 'off';
    const next = mode === 'off' ? ORIGINAL_SOURCE : source;
    if (mode === this.chosen && sameSource(next, this.picked)) return;
    this.chosen = mode;
    this.picked = next;
    this.pickedHash = this.hashOf(next);
    if (wasOn || mode !== 'off') this.changed(wasOn);
  }
}
