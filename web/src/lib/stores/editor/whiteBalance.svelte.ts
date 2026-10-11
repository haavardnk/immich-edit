import { autoWhiteBalance, getAsShotWhite, sampleWhiteBalance } from '$lib/api/edits';
import { toasts } from '$lib/stores/toasts.svelte';
import { errorMessage } from '$lib/utils/errors';
import type { AsShotWhite, Edits, WhiteBalance } from '$lib/types/edits';

export interface WhiteBalanceCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  wbPicking: boolean;
  wbBusy: boolean;
  asShot: AsShotWhite | null;
  asShotError: string | null;
  asShotKey: string | null;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
}

export function loadAsShot(ctx: WhiteBalanceCtx): void {
  const id = ctx.assetId;
  if (!id || !ctx.initialised) return;
  const edits = $state.snapshot(ctx.edits) as Edits;
  const key = `${id}:${JSON.stringify(edits.color.dcp)}`;
  if (ctx.asShotKey === key) return;
  ctx.asShotKey = key;
  ctx.asShotError = null;
  getAsShotWhite(id, edits)
    .then((white) => {
      if (ctx.asShotKey === key) ctx.asShot = white;
    })
    .catch((e: unknown) => {
      if (ctx.asShotKey !== key) return;
      ctx.asShot = null;
      ctx.asShotError = errorMessage(e);
    });
}

export function toggleWbPicker(ctx: WhiteBalanceCtx): void {
  if (!ctx.assetId || !ctx.initialised) return;
  ctx.wbPicking = !ctx.wbPicking;
}

export function cancelWbPicker(ctx: WhiteBalanceCtx): void {
  ctx.wbPicking = false;
}

export async function pickWhiteBalance(ctx: WhiteBalanceCtx, u: number, v: number): Promise<void> {
  if (!ctx.assetId || !ctx.initialised || ctx.wbBusy) return;
  ctx.wbBusy = true;
  try {
    const solved = await sampleWhiteBalance(ctx.assetId, u, v, $state.snapshot(ctx.edits));
    applySolved(ctx, solved);
    ctx.wbPicking = false;
    await ctx.onCommit('White Balance');
  } catch (e) {
    toasts.push('warn', errorMessage(e));
  } finally {
    ctx.wbBusy = false;
  }
}

export async function autoWb(ctx: WhiteBalanceCtx): Promise<void> {
  if (!ctx.assetId || !ctx.initialised || ctx.wbBusy) return;
  ctx.wbBusy = true;
  try {
    const solved = await autoWhiteBalance(ctx.assetId, $state.snapshot(ctx.edits));
    applySolved(ctx, solved);
    await ctx.onCommit('Auto White Balance');
  } catch (e) {
    toasts.push('warn', errorMessage(e));
  } finally {
    ctx.wbBusy = false;
  }
}

function applySolved(ctx: WhiteBalanceCtx, solved: WhiteBalance): void {
  ctx.edits.basic.white_balance = solved;
  ctx.onLive();
}
