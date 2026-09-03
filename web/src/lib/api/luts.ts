import { getJson, sendBytes, sendJson, url } from './client';

export interface LutMeta {
  id: string;
  name: string;
  lut_size: number;
  size: number;
  created_at: string;
}

export async function listLuts(): Promise<LutMeta[]> {
  return getJson<LutMeta[]>('/api/luts');
}

export async function importLut(name: string, bytes: Uint8Array): Promise<LutMeta> {
  return sendBytes<LutMeta>(url`/api/luts?name=${name}`, bytes);
}

export async function deleteLut(id: string): Promise<void> {
  await sendJson<void>('DELETE', url`/api/luts/${id}`, undefined);
}
