import type { ApplyPresetOptions } from '$lib/api/jobs';
import { autoEdits } from '$lib/api/edits';
import { applyCopySections } from '$lib/editor/copyPaste';
import { presetEdits } from '$lib/edits/applyPreset';
import { LOOK_AMOUNT_FULL } from '$lib/edits/lookAmount';
import { keysFor } from '$lib/shortcuts/labels';
import { clipboard } from '$lib/stores/clipboard.svelte';
import { copyDialog } from '$lib/stores/copyDialog.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
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
  toasts.push('success', `Reset develop settings. Undo with ${keysFor('undo')}.`, 3000);
}

export function copyEdits(ctx: DevelopCtx): void {
  if (isIdentity(ctx.edits)) {
    toasts.push('info', 'This photo has no edits to copy', 3000);
    return;
  }
  copyDialog.show($state.snapshot(ctx.edits) as Edits, () =>
    toasts.push('success', 'Copied settings', 3000)
  );
}

export async function pasteEdits(ctx: DevelopCtx): Promise<void> {
  if (!ctx.initialised) return;
  const snap = clipboard.snapshot();
  if (!snap) {
    toasts.push('info', 'Nothing copied', 3000);
    return;
  }
  ctx.edits = applyCopySections(ctx.edits, snap.edits, snap.sections);
  ctx.onLive();
  await ctx.onCommit('Paste');
  toasts.push('success', `Pasted settings. Undo with ${keysFor('undo')}.`, 3000);
}

export async function applyPreset(
  ctx: DevelopCtx,
  manifest: EditManifest,
  opts: ApplyPresetOptions,
  name?: string
): Promise<void> {
  if (!ctx.initialised) return;
  ctx.edits = presetEdits(ctx.edits, { manifest, opts });
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
        white_balance: suggested.basic.white_balance,
        exposure_ev: suggested.basic.exposure_ev,
        brightness: suggested.basic.brightness,
        contrast: suggested.basic.contrast,
        dehaze: suggested.basic.dehaze,
        clarity: suggested.basic.clarity,
        vibrance: suggested.basic.vibrance,
        saturation: suggested.basic.saturation
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
