import { request, sendBytes } from '$lib/api/client';

export interface RasterMeta {
  raster_id: string;
  width: number;
  height: number;
  size: number;
  created_at: string;
}

export function uploadRaster(
  width: number,
  height: number,
  bytes: Uint8Array
): Promise<RasterMeta> {
  return sendBytes<RasterMeta>(`/api/rasters?width=${width}&height=${height}`, bytes);
}

export async function fetchRaster(
  rasterId: string
): Promise<{ width: number; height: number; bytes: Uint8Array }> {
  const r = await request(`/api/rasters/${rasterId}`);
  const width = Number(r.headers.get('x-raster-width') ?? 0);
  const height = Number(r.headers.get('x-raster-height') ?? 0);
  const ab = await r.arrayBuffer();
  return { width, height, bytes: new Uint8Array(ab) };
}
