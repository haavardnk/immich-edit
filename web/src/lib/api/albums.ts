import { getJson, sendJson, url } from './client';
import type { AlbumPatch, AlbumSummary } from '#lib/types/album.ts';

export function listAlbums(): Promise<AlbumSummary[]> {
  return getJson('/api/albums');
}

export function createAlbum(albumName: string, description = ''): Promise<AlbumSummary> {
  return sendJson('POST', '/api/albums', description ? { albumName, description } : { albumName });
}

export function albumsWithAsset(assetId: string): Promise<AlbumSummary[]> {
  return getJson(url`/api/albums?assetId=${assetId}`);
}

export function albumMemberships(ids: readonly string[]): Promise<Record<string, AlbumSummary[]>> {
  return sendJson('POST', '/api/albums/membership', { ids }, undefined, { silent: true });
}

export function getAlbum(id: string): Promise<AlbumSummary> {
  return getJson(url`/api/albums/${id}`);
}

export function updateAlbum(id: string, patch: AlbumPatch): Promise<AlbumSummary> {
  return sendJson('PATCH', url`/api/albums/${id}`, patch);
}

export function deleteAlbum(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/albums/${id}`, undefined);
}

export function leaveAlbum(id: string): Promise<void> {
  return sendJson('DELETE', url`/api/albums/${id}/membership`, undefined);
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
