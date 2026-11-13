import type { Edits, MaskComponentMode } from '#lib/types/edits.ts';

export type MaskHoverTarget = {
  layerId: string;
  componentId: string | null;
  label: string;
  mode: MaskComponentMode | null;
};

export type WeightTarget = { layerId: string; componentId: string | null };

type TargetCtx = {
  edits: Edits;
  activeLayerId: string | null;
  maskOverlayLayerId: string | null;
  maskOverlayHeld: boolean;
  colorPicker: unknown;
  showingOriginal: boolean;
};

class MaskHoverStore {
  target = $state<MaskHoverTarget | null>(null);

  enter = (target: MaskHoverTarget): void => {
    this.target = target;
  };

  leave = (layerId: string, componentId: string | null): void => {
    const current = this.target;
    if (current?.layerId === layerId && current.componentId === componentId) this.target = null;
  };

  clear = (): void => {
    this.target = null;
  };
}

export const maskHover = new MaskHoverStore();

function previewed(ctx: TargetCtx, hover: MaskHoverTarget | null): WeightTarget | null {
  if (!hover) return null;
  if (hover.componentId === null && hover.layerId === ctx.activeLayerId) return null;
  const layer = ctx.edits.masks.find((item) => item.id === hover.layerId);
  if (!layer) return null;
  if (hover.componentId !== null && !layer.components.some((c) => c.id === hover.componentId)) {
    return null;
  }
  return { layerId: hover.layerId, componentId: hover.componentId };
}

function suppressed(ctx: TargetCtx): boolean {
  return ctx.maskOverlayHeld || !!ctx.colorPicker || ctx.showingOriginal;
}

export function weightTarget(ctx: TargetCtx, hover: MaskHoverTarget | null): WeightTarget | null {
  if (suppressed(ctx)) return null;
  const preview = previewed(ctx, hover);
  if (preview) return preview;
  return ctx.maskOverlayLayerId ? { layerId: ctx.maskOverlayLayerId, componentId: null } : null;
}

export function hoverPreview(ctx: TargetCtx, hover: MaskHoverTarget | null): boolean {
  return !suppressed(ctx) && previewed(ctx, hover) !== null;
}
