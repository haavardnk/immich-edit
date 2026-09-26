import { getJson, sendJson, url } from './client';
import type { AlbumDetail, AlbumSummary } from '$lib/types/album';

export function listAlbums(): Promise<AlbumSummary[]> {
  return getJson('/api/albums');
}

export function getAlbum(id: string): Promise<AlbumDetail> {
  return getJson(url`/api/albums/${id}`);
}

export interface AlbumAssetResult {
  id: string;
  success: boolean;
  error?: string;
}

export function addAssetsToAlbum(albumId: string, ids: string[]): Promise<AlbumAssetResult[]> {
  return sendJson('PUT', url`/api/albums/${albumId}/assets`, { ids });
}

export function removeAssetsFromAlbum(albumId: string, ids: string[]): Promise<AlbumAssetResult[]> {
  return sendJson('DELETE', url`/api/albums/${albumId}/assets`, { ids });
}
