import type { Edits, RetouchMode, RetouchStroke, Vec2f } from '$lib/types/edits';

export interface RetouchCtx {
  edits: Edits;
  initialised: boolean;
  activeRetouchId: string | null;
  retouchAnchor: Vec2f | null;
  retouchTool: {
    mode: RetouchMode;
    size: number;
    hardness: number;
    opacity: number;
  };
  onLive(): void;
  onCommit(action?: string): Promise<void>;
}

export async function addRetouchStroke(
  ctx: RetouchCtx,
  stroke: RetouchStroke,
  maxStrokes: number
): Promise<void> {
  if (!ctx.initialised || ctx.edits.retouch.length >= maxStrokes) return;
  ctx.edits.retouch.push(stroke);
  ctx.activeRetouchId = stroke.id;
  ctx.onLive();
  await ctx.onCommit('Retouch');
}

export function updateRetouchStroke(
  ctx: RetouchCtx,
  id: string,
  patch: Partial<RetouchStroke>
): void {
  const index = ctx.edits.retouch.findIndex((stroke) => stroke.id === id);
  const current = ctx.edits.retouch[index];
  if (!current) return;
  ctx.edits.retouch[index] = { ...current, ...patch };
}

export async function setRetouchStroke(
  ctx: RetouchCtx,
  id: string,
  patch: Partial<RetouchStroke>,
  commit: boolean
): Promise<void> {
  updateRetouchStroke(ctx, id, patch);
  ctx.onLive();
  if (commit) await ctx.onCommit('Retouch');
}

export async function commitRetouch(ctx: RetouchCtx): Promise<void> {
  await ctx.onCommit('Retouch');
}

export async function removeRetouchStroke(ctx: RetouchCtx, id: string): Promise<void> {
  const index = ctx.edits.retouch.findIndex((stroke) => stroke.id === id);
  if (index < 0) return;
  ctx.edits.retouch.splice(index, 1);
  if (ctx.activeRetouchId === id) ctx.activeRetouchId = null;
  ctx.onLive();
  await ctx.onCommit('Remove Retouch');
}

export async function toggleRetouchStroke(ctx: RetouchCtx, id: string): Promise<void> {
  const stroke = ctx.edits.retouch.find((retouch) => retouch.id === id);
  if (!stroke) return;
  stroke.enabled = !stroke.enabled;
  ctx.onLive();
  await ctx.onCommit(stroke.enabled ? 'Enable Retouch' : 'Disable Retouch');
}

export async function clearRetouch(ctx: RetouchCtx): Promise<void> {
  if (ctx.edits.retouch.length === 0) return;
  ctx.edits.retouch = [];
  ctx.activeRetouchId = null;
  ctx.onLive();
  await ctx.onCommit('Clear Retouch');
}

export function setRetouchTool(
  ctx: RetouchCtx,
  patch: Partial<{ mode: RetouchMode; size: number; hardness: number; opacity: number }>
): void {
  ctx.retouchTool = { ...ctx.retouchTool, ...patch };
}

export function setRetouchMode(ctx: RetouchCtx, mode: RetouchMode): void {
  ctx.retouchTool = { ...ctx.retouchTool, mode };
  if (ctx.activeRetouchId) void setRetouchStroke(ctx, ctx.activeRetouchId, { mode }, true);
}
