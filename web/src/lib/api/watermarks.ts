import { getJson, sendBytes, sendJson, url } from './client';

export interface WatermarkMeta {
  id: string;
  name: string;
  width: number;
  height: number;
  size: number;
  created_at: string;
}

export async function listWatermarks(): Promise<WatermarkMeta[]> {
  return getJson<WatermarkMeta[]>('/api/watermarks');
}

export async function importWatermark(name: string, bytes: Uint8Array): Promise<WatermarkMeta> {
  return sendBytes<WatermarkMeta>(url`/api/watermarks?name=${name}`, bytes);
}

export async function deleteWatermark(id: string): Promise<void> {
  await sendJson<void>('DELETE', url`/api/watermarks/${id}`, undefined);
}

export function watermarkPngUrl(id: string): string {
  return url`/api/watermarks/${id}/png`;
}
