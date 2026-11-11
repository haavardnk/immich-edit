import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
import { presetEdits } from '#lib/edits/applyPreset.ts';
import { LOOK_AMOUNT_FULL } from '#lib/edits/lookAmount.ts';
import type { EditManifest, Edits } from '#lib/types/edits.ts';
import { sameValue } from '#lib/utils/sameValue.ts';

export interface PresetSource {
  id: string;
  name: string;
  manifest: EditManifest;
}

export interface AppliedPreset {
  id: string;
  name: string;
  amount: number;
}

interface Applied {
  preset: PresetSource;
  opts: ApplyPresetOptions;
  base: Edits;
  edits: Edits;
  step: number;
}

export interface PresetAppliedCtx {
  initialised: boolean;
  edits: Edits;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
}

export interface PresetSteps {
  readonly topStep: number | null;
  replaceStep(id: number, edits: Edits): boolean;
}

export interface PresetAmend {
  steps: PresetSteps;
  save(edits: Edits, action: string): Promise<void>;
}

function presetAction(name: string, amount: number): string {
  return amount === LOOK_AMOUNT_FULL ? `Preset: ${name}` : `Preset: ${name} (${amount}%)`;
}

export class PresetApplied {
  private ctx: PresetAppliedCtx;
  private amend: PresetAmend;
  private applied = $state.raw<Applied | null>(null);

  readonly current = $derived.by<AppliedPreset | null>(() => {
    const applied = this.live();
    if (!applied) return null;
    return { id: applied.preset.id, name: applied.preset.name, amount: applied.opts.amount };
  });

  constructor(ctx: PresetAppliedCtx, amend: PresetAmend) {
    this.ctx = ctx;
    this.amend = amend;
  }

  apply = (preset: PresetSource, opts: ApplyPresetOptions): Promise<void> => {
    if (!this.ctx.initialised) return Promise.resolve();
    const base = $state.snapshot(this.ctx.edits) as Edits;
    this.ctx.edits = presetEdits(this.ctx.edits, { manifest: preset.manifest, opts });
    this.ctx.onLive();
    const top = this.amend.steps.topStep;
    const saved = this.ctx.onCommit(presetAction(preset.name, opts.amount));
    const step = this.amend.steps.topStep;
    this.applied =
      step === null || step === top
        ? null
        : { preset, opts, base, edits: $state.snapshot(this.ctx.edits) as Edits, step };
    return saved;
  };

  blend = (amount: number): void => {
    const applied = this.live();
    if (!applied) return;
    const opts = { ...applied.opts, amount };
    const base = $state.snapshot(applied.base) as Edits;
    this.ctx.edits = presetEdits(base, { manifest: applied.preset.manifest, opts });
    this.applied = { ...applied, opts, edits: $state.snapshot(this.ctx.edits) as Edits };
    this.ctx.onLive();
  };

  commitBlend = (): Promise<void> => {
    const applied = this.live();
    if (!applied) return Promise.resolve();
    const action = presetAction(applied.preset.name, applied.opts.amount);
    if (!this.amend.steps.replaceStep(applied.step, applied.edits))
      return this.ctx.onCommit(action);
    return this.amend.save(applied.edits, action);
  };

  drop = (): void => {
    this.applied = null;
  };

  private live(): Applied | null {
    const applied = this.applied;
    return applied && sameValue(this.ctx.edits, applied.edits) ? applied : null;
  }
}
