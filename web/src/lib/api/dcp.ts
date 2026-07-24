import { getJson, sendJson, ApiError } from './client';

export interface DcpMeta {
  id: string;
  name: string;
  camera_model: string | null;
  copyright: string | null;
  bundled: boolean;
  size: number;
  created_at: string;
}

export async function listDcps(): Promise<DcpMeta[]> {
  return getJson<DcpMeta[]>('/api/dcp');
}

export async function importDcp(name: string, bytes: Uint8Array): Promise<DcpMeta> {
  const buf = bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength
  ) as ArrayBuffer;
  const resp = await fetch(`/api/dcp?name=${encodeURIComponent(name)}`, {
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
  return (await resp.json()) as DcpMeta;
}

export async function deleteDcp(id: string): Promise<void> {
  await sendJson<void>('DELETE', `/api/dcp/${id}`, undefined);
}
