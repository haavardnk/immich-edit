import { immichPreviewUrl, type ThumbSource } from '#lib/api/assets.ts';
import { persistedPreviewUrl } from '#lib/api/preview.ts';
import type { Size } from '#lib/utils/geom.ts';

export type LoadedSize = Size & { id: string };

export type LoadedRender = LoadedSize & { src: string };

export type PaneSources = { render: string | null; underlay: string | null };

const NO_SIZE: Size = { w: 0, h: 0 };

export function paneSources(
  asset: ThumbSource,
  rendered: LoadedRender | null,
  failed: string | null,
  maxEdge: number,
  clipWarn: boolean
): PaneSources {
  const wanted = persistedPreviewUrl(asset.id, maxEdge, clipWarn);
  const kept = rendered?.id === asset.id ? rendered.src : null;
  return {
    render: wanted === failed ? kept : wanted,
    underlay: kept === null ? immichPreviewUrl(asset) : null
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
