import { immichPreviewUrl } from '#lib/api/assets.ts';
import { persistedPreviewUrl } from '#lib/api/preview.ts';
import type { Size } from '#lib/utils/geom.ts';

export type LoadedSize = Size & { id: string };

export type PaneSources = { render: string; underlay: string | null };

const NO_SIZE: Size = { w: 0, h: 0 };

export function paneSources(
  assetId: string,
  renderedId: string | null,
  maxEdge: number,
  clipWarn: boolean
): PaneSources {
  return {
    render: persistedPreviewUrl(assetId, maxEdge, clipWarn),
    underlay: renderedId === assetId ? null : immichPreviewUrl(assetId)
  };
}

export function paneNatural(
  assetId: string,
  render: LoadedSize | null,
  preview: LoadedSize | null
): Size {
  const shown = [render, preview].find((size) => size?.id === assetId);
  return shown ? { w: shown.w, h: shown.h } : NO_SIZE;
}

export function previewStatusLabel(edited: boolean): string {
  return edited ? 'Immich preview · rendering edits' : 'Immich preview · rendering';
}
