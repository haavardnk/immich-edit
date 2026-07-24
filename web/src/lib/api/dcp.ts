import { getJson, sendBytes, sendJson } from './client';

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

export async function matchDcp(model: string): Promise<DcpMeta | null> {
  return getJson<DcpMeta | null>(`/api/dcp/match?model=${encodeURIComponent(model)}`);
}

export async function importDcp(name: string, bytes: Uint8Array): Promise<DcpMeta> {
  return sendBytes<DcpMeta>(`/api/dcp?name=${encodeURIComponent(name)}`, bytes);
}

export async function deleteDcp(id: string): Promise<void> {
  await sendJson<void>('DELETE', `/api/dcp/${id}`, undefined);
}
