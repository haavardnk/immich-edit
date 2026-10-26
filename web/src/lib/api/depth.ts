import { sendJson, url } from './client';

export interface DepthMap {
  raster_id: string;
  width: number;
  height: number;
  model_id: string;
  backend: string;
  elapsed_ms: number;
}

export function generateDepth(assetId: string): Promise<DepthMap> {
  return sendJson<DepthMap>('POST', url`/api/assets/${assetId}/depth`, undefined, undefined, {
    silent: true
  });
}
