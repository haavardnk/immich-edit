export interface RasterMeta {
  raster_id: string;
  width: number;
  height: number;
  size: number;
  created_at: string;
}

export async function uploadRaster(
  width: number,
  height: number,
  bytes: Uint8Array
): Promise<RasterMeta> {
  const buf = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
  const r = await fetch(`/api/rasters?width=${width}&height=${height}`, {
    method: 'POST',
    headers: { 'content-type': 'application/octet-stream' },
    body: buf
  });
  if (!r.ok) throw new Error(`raster upload failed: ${r.status}`);
  return (await r.json()) as RasterMeta;
}

export async function fetchRaster(
  rasterId: string
): Promise<{ width: number; height: number; bytes: Uint8Array }> {
  const r = await fetch(`/api/rasters/${rasterId}`);
  if (!r.ok) throw new Error(`raster fetch failed: ${r.status}`);
  const width = Number(r.headers.get('x-raster-width') ?? 0);
  const height = Number(r.headers.get('x-raster-height') ?? 0);
  const ab = await r.arrayBuffer();
  return { width, height, bytes: new Uint8Array(ab) };
}
