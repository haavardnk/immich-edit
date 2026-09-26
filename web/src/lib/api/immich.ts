import { getJson } from './client';

export interface ImmichCapabilities {
  immich_version: string | null;
  min_rating_filter: boolean;
}

export function getImmichCapabilities(): Promise<ImmichCapabilities> {
  return getJson('/api/immich/capabilities');
}
