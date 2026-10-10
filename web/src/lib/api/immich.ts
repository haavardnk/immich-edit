import { getJson } from './client';

export interface ImmichCapabilities {
  immich_version: string | null;
  structured_search: boolean;
}

export async function getImmichCapabilities(): Promise<ImmichCapabilities> {
  return getJson<ImmichCapabilities>('/api/immich/capabilities', undefined, { silent: true });
}
