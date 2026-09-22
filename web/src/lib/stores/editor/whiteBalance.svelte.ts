import { autoWhiteBalance, sampleWhiteBalance } from '$lib/api/edits';
import { errorMessage } from '$lib/utils/errors';
import type { Edits } from '$lib/types/edits';

export interface WhiteBalanceCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  wbPicking: boolean;
  wbBusy: boolean;
  error: string | null;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
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
    applySolved(ctx, solved.wb_temp, solved.wb_tint);
    ctx.wbPicking = false;
    await ctx.onCommit('White Balance');
  } catch (e) {
    ctx.error = errorMessage(e);
  } finally {
    ctx.wbBusy = false;
  }
}

export async function autoWb(ctx: WhiteBalanceCtx): Promise<void> {
  if (!ctx.assetId || !ctx.initialised || ctx.wbBusy) return;
  ctx.wbBusy = true;
  try {
    const solved = await autoWhiteBalance(ctx.assetId, $state.snapshot(ctx.edits));
    applySolved(ctx, solved.wb_temp, solved.wb_tint);
    await ctx.onCommit('Auto White Balance');
  } catch (e) {
    ctx.error = errorMessage(e);
  } finally {
    ctx.wbBusy = false;
  }
}

function applySolved(ctx: WhiteBalanceCtx, temp: number, tint: number): void {
  ctx.edits.basic.wb_temp = temp;
  ctx.edits.basic.wb_tint = tint;
  ctx.onLive();
}
