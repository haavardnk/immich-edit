import { getJson, postForBlob } from "./client";
import type { Edits } from "$lib/types/edits";
import type { PreviewMeta } from "$lib/types/preview";

export type PreviewMode =
  | "none"
  | "sharpen_mask"
  | "sharpen_radius"
  | "sharpen_detail"
  | { mask_weight: { layer_id: string } };

export function maskWeightPreview(layerId: string): PreviewMode {
  return { mask_weight: { layer_id: layerId } };
}

export function previewModeIsNone(m: PreviewMode): boolean {
  return m === "none";
}

export function persistedPreviewUrl(assetId: string, max: number): string {
  return `/api/assets/${assetId}/preview?max=${max}`;
}

export async function livePreview(
  assetId: string,
  edits: Edits,
  maxEdge: number,
  previewMode: PreviewMode,
  signal?: AbortSignal,
): Promise<{ blob: Blob; metaId: string | null }> {
  return postForBlob(
    `/api/assets/${assetId}/preview`,
    { max_edge: maxEdge, edits, preview_mode: previewMode },
    signal,
  );
}

export function getPreviewMeta(
  assetId: string,
  metaId: string,
): Promise<PreviewMeta> {
  return getJson(`/api/assets/${assetId}/preview/meta/${metaId}`);
}
