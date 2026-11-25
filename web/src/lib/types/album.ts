import type { AssetType, ExifInfo, TagRef } from './asset';

export type AlbumRole = 'owner' | 'editor' | 'viewer';

export interface AlbumMember {
  userId: string;
  name: string;
  role: AlbumRole;
}

export interface AlbumSummary {
  id: string;
  albumName: string;
  description: string;
  assetCount: number;
  albumThumbnailAssetId: string | null;
  updatedAt: string | null;
  ownerId: string | null;
  shared: boolean;
  startDate: string | null;
  endDate: string | null;
  members: AlbumMember[];
}

export interface AlbumPatch {
  albumName?: string;
  description?: string;
  albumThumbnailAssetId?: string;
}

export interface AssetSummary {
  id: string;
  originalFileName: string;
  originalPath?: string;
  type: AssetType;
  ownerId?: string;
  libraryId?: string;
  localDateTime?: string | null;
  fileCreatedAt: string | null;
  createdAt?: string | null;
  updatedAt: string | null;
  checksum: string | null;
  thumbhash?: string | null;
  isFavorite: boolean;
  exifInfo: ExifInfo | null;
  tags: TagRef[];
  copyOf?: string;
  copyLabel?: string;
}
