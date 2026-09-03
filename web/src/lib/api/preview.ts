import { getJson, postForBlob, url } from './client';
import type { Edits } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import type { ColorSpaceOpt } from './export';

export type PreviewMode =
  | 'none'
  | 'sharpen_mask'
  | 'sharpen_radius'
  | 'sharpen_detail'
  | { mask_weight: { layer_id: string } };

export interface ProofOptions {
  colorSpace: ColorSpaceOpt;
  gamutWarn: boolean;
  clipWarn: boolean;
}

export type RenderLane = 'base' | 'original' | 'roi';

export type Roi = [number, number, number, number];

export function maskWeightPreview(layerId: string): PreviewMode {
  return { mask_weight: { layer_id: layerId } };
}

export function previewModeIsNone(m: PreviewMode): boolean {
  return m === 'none';
}

export function persistedPreviewUrl(assetId: string, max: number, clipWarn = false): string {
  return url`/api/assets/${assetId}/preview?max=${max}&clip=${clipWarn}`;
}

export async function livePreview(
  assetId: string,
  edits: Edits,
  maxEdge: number,
  previewMode: PreviewMode,
  proof?: ProofOptions,
  signal?: AbortSignal,
  lane: RenderLane = 'base',
  roi?: Roi
): Promise<{ blob: Blob; metaId: string | null }> {
  return postForBlob(
    url`/api/assets/${assetId}/preview`,
    {
      max_edge: maxEdge,
      edits,
      preview_mode: previewMode,
      output_color_space: proof?.colorSpace ?? 'srgb',
      gamut_warn: proof?.gamutWarn ?? false,
      clip_warn: proof?.clipWarn ?? false,
      lane,
      roi: roi ?? null
    },
    signal
  );
}

export function getPreviewMeta(assetId: string, metaId: string): Promise<PreviewMeta> {
  return getJson(url`/api/assets/${assetId}/preview/meta/${metaId}`);
}
