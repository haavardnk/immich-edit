import { request, url } from './client';
import type { Roi } from './preview';
import type { Edits } from '$lib/types/edits';

export interface SourcePayload {
  bytes: ArrayBuffer;
  dcpId: string | null;
}

export async function fetchSource(
  assetId: string,
  edits: Edits,
  maxEdge: number,
  signal?: AbortSignal,
  roi?: Roi
): Promise<SourcePayload> {
  const resp = await request(url`/api/assets/${assetId}/source`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ max_edge: maxEdge, edits, roi: roi ?? null }),
    signal
  });
  return { bytes: await resp.arrayBuffer(), dcpId: resp.headers.get('x-source-dcp') };
}

export async function fetchDcpBytes(id: string): Promise<ArrayBuffer> {
  return (await request(url`/api/dcp/${id}/raw`)).arrayBuffer();
}

export async function fetchLutCube(id: string): Promise<ArrayBuffer> {
  return (await request(url`/api/luts/${id}/cube`)).arrayBuffer();
}
