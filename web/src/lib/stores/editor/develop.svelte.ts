import type { ApplyPresetOptions } from '$lib/api/jobs';
import { autoEdits } from '$lib/api/edits';
import { applyCopySections } from '$lib/editor/copyPaste';
import { LOOK_AMOUNT_FULL, withLookAmount } from '$lib/edits/lookAmount';
import { manifestToEdits } from '$lib/edits/manifest';
import { clipboard } from '$lib/stores/clipboard.svelte';
import { copyDialog } from '$lib/stores/copyDialog.svelte';
import { isIdentity, resetDevelopEdits, type EditManifest, type Edits } from '$lib/types/edits';
import { errorMessage } from '$lib/utils/errors';

export interface DevelopCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  autoBusy: boolean;
  error: string | null;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
}

export async function resetDevelop(ctx: DevelopCtx): Promise<void> {
  if (!ctx.assetId) return;
  ctx.edits = resetDevelopEdits(ctx.edits);
  await ctx.onCommit('Reset Develop');
}

export function copyEdits(ctx: DevelopCtx): void {
  if (isIdentity(ctx.edits)) return;
  copyDialog.show($state.snapshot(ctx.edits) as Edits);
}

export async function pasteEdits(ctx: DevelopCtx): Promise<void> {
  const snap = clipboard.snapshot();
  if (!snap || !ctx.initialised) return;
  ctx.edits = applyCopySections(ctx.edits, snap.edits, snap.sections);
  ctx.onLive();
  await ctx.onCommit('Paste');
}

export async function applyPreset(
  ctx: DevelopCtx,
  manifest: EditManifest,
  opts: ApplyPresetOptions,
  name?: string
): Promise<void> {
  if (!ctx.initialised) return;
  const incoming = withLookAmount(manifestToEdits(manifest), opts.amount);
  ctx.edits = {
    basic: incoming.basic,
    tone: incoming.tone,
    color: incoming.color,
    detail: incoming.detail,
    effects: incoming.effects,
    lens: incoming.lens,
    geometry: opts.includeGeometry ? incoming.geometry : ctx.edits.geometry,
    masks: opts.includeMasks ? incoming.masks : ctx.edits.masks,
    retouch: ctx.edits.retouch
  };
  ctx.onLive();
  const action = name ? `Preset: ${name}` : 'Preset';
  await ctx.onCommit(opts.amount === LOOK_AMOUNT_FULL ? action : `${action} (${opts.amount}%)`);
}

export async function autoAdjust(ctx: DevelopCtx): Promise<void> {
  if (!ctx.assetId || !ctx.initialised) return;
  ctx.autoBusy = true;
  try {
    const suggested = await autoEdits(ctx.assetId, $state.snapshot(ctx.edits));
    ctx.edits = {
      ...ctx.edits,
      basic: {
        ...ctx.edits.basic,
        exposure_ev: suggested.basic.exposure_ev,
        brightness: suggested.basic.brightness,
        contrast: suggested.basic.contrast,
        vibrance: suggested.basic.vibrance
      },
      tone: { ...suggested.tone }
    };
    ctx.onLive();
    await ctx.onCommit('Auto');
  } catch (e) {
    ctx.error = errorMessage(e);
  } finally {
    ctx.autoBusy = false;
  }
}
