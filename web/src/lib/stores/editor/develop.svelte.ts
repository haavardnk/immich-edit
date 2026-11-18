import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
import { autoEdits } from '#lib/api/edits.ts';
import { applyCopySections } from '#lib/editor/copyPaste.ts';
import { presetEdits } from '#lib/edits/applyPreset.ts';
import { LOOK_AMOUNT_FULL } from '#lib/edits/lookAmount.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { clipboard, NO_EDITS_TO_COPY, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import {
  isDevelopIdentity,
  isIdentity,
  resetDevelopEdits,
  type EditManifest,
  type Edits
} from '#lib/types/edits.ts';
import { errorMessage } from '#lib/utils/errors.ts';

export interface DevelopCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  saving: boolean;
  autoBusy: boolean;
  error: string | null;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
}

function editable(ctx: DevelopCtx): boolean {
  return ctx.assetId !== null && ctx.initialised && !ctx.saving;
}

export function canResetDevelop(ctx: DevelopCtx): boolean {
  return editable(ctx) && !isDevelopIdentity(ctx.edits);
}

export function canPaste(ctx: DevelopCtx): boolean {
  return editable(ctx) && clipboard.has;
}

export async function resetDevelop(ctx: DevelopCtx): Promise<void> {
  if (!canResetDevelop(ctx)) return;
  ctx.edits = resetDevelopEdits(ctx.edits);
  await ctx.onCommit('Reset Develop');
  toasts.push('success', `Reset develop settings. Undo with ${keysFor('undo')}.`, { ttlMs: 3000 });
}

export function copyEdits(ctx: DevelopCtx): void {
  if (isIdentity(ctx.edits)) {
    toasts.push('info', NO_EDITS_TO_COPY, { ttlMs: 3000 });
    return;
  }
  copyDialog.show($state.snapshot(ctx.edits) as Edits, () =>
    toasts.push('success', 'Copied settings', { ttlMs: 3000 })
  );
}

export async function pasteEdits(ctx: DevelopCtx): Promise<void> {
  if (!editable(ctx)) return;
  const snap = clipboard.snapshot();
  if (!snap) {
    toasts.push('info', NOTHING_COPIED, { ttlMs: 3000 });
    return;
  }
  ctx.edits = applyCopySections(ctx.edits, snap.edits, snap.sections);
  ctx.onLive();
  await ctx.onCommit('Paste');
  toasts.push('success', `Pasted settings. Undo with ${keysFor('undo')}.`, { ttlMs: 3000 });
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
