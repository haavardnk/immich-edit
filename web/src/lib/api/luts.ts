import { getJson, sendJson, ApiError } from './client';

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
  const buf = bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength
  ) as ArrayBuffer;
  const resp = await fetch(`/api/luts?name=${encodeURIComponent(name)}`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'content-type': 'application/octet-stream' },
    body: buf
  });
  if (!resp.ok) {
    let code = 'unknown';
    let message = resp.statusText;
    try {
      const body = (await resp.json()) as { code?: string; message?: string };
      if (body.code) code = body.code;
      if (body.message) message = body.message;
    } catch {
      /* ignore */
    }
    throw new ApiError(resp.status, code, message);
  }
  return (await resp.json()) as LutMeta;
}

export async function deleteLut(id: string): Promise<void> {
  await sendJson<void>('DELETE', `/api/luts/${id}`, undefined);
}
