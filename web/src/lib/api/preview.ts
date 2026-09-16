import { getJson, postForBlob, request, url } from './client';
import type { Edits } from '$lib/types/edits';
import type { PreviewMeta, ScopeGrid, ScopeKind } from '$lib/types/preview';
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
  roi?: Roi,
  scopes = false
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
      roi: roi ?? null,
      scopes
    },
    signal
  );
}

export function getPreviewMeta(assetId: string, metaId: string): Promise<PreviewMeta> {
  return getJson(url`/api/assets/${assetId}/preview/meta/${metaId}`);
}

const SCOPE_MAGIC = 0x504f4353;
const SCOPE_HEADER_LEN = 16;

export async function getPreviewScope(
  assetId: string,
  metaId: string,
  kind: ScopeKind,
  signal?: AbortSignal
): Promise<ScopeGrid> {
  const resp = await request(
    url`/api/assets/${assetId}/preview/meta/${metaId}/scope/${kind}`,
    { signal },
    { silent: true }
  );
  const buffer = await resp.arrayBuffer();
  if (buffer.byteLength < SCOPE_HEADER_LEN) throw new Error('scope payload truncated');
  const header = new DataView(buffer, 0, SCOPE_HEADER_LEN);
  if (header.getUint32(0, true) !== SCOPE_MAGIC) throw new Error('scope payload malformed');
  const width = header.getUint16(8, true);
  const height = header.getUint16(10, true);
  const channels = header.getUint8(6);
  const data = new Uint8Array(buffer, SCOPE_HEADER_LEN);
  if (data.length !== width * height * channels) throw new Error('scope payload size mismatch');
  return { kind, width, height, channels, maxCount: header.getUint32(12, true), data };
}
