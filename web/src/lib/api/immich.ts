import { getJson, request } from './client';

export interface ImmichCapabilities {
  immich_version: string | null;
  external_host: string | null;
  structured_search: boolean;
  tag_rename: boolean;
  smart_search: boolean;
  ocr: boolean;
  trash: boolean;
  trash_days: number | null;
}

export async function getImmichCapabilities(): Promise<ImmichCapabilities> {
  return getJson<ImmichCapabilities>('/api/immich/capabilities', undefined, { silent: true });
}

export async function pingImmich(): Promise<void> {
  await request('/api/immich/ping', undefined, { silent: true });
}
