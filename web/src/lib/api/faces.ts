import { getJson, url } from './client';

export interface FaceBox {
  source_w: number;
  source_h: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export function getFaces(assetId: string): Promise<FaceBox[]> {
  return getJson(url`/api/assets/${assetId}/faces`);
}
