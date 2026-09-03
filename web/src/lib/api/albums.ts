import { getJson, url } from './client';
import type { AlbumDetail, AlbumSummary } from '$lib/types/album';

export function listAlbums(): Promise<AlbumSummary[]> {
  return getJson('/api/albums');
}

export function getAlbum(id: string): Promise<AlbumDetail> {
  return getJson(url`/api/albums/${id}`);
}
