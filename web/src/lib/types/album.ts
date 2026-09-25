import type { AssetType, ExifInfo, TagRef } from './asset';

export interface AlbumSummary {
  id: string;
  albumName: string;
  assetCount: number;
  albumThumbnailAssetId: string | null;
  updatedAt: string | null;
}

export interface AlbumDetail {
  id: string;
  albumName: string;
  assetCount: number;
  updatedAt: string | null;
}

export interface AssetSummary {
  id: string;
  originalFileName: string;
  type: AssetType;
  localDateTime?: string | null;
  fileCreatedAt: string | null;
  updatedAt: string | null;
  checksum: string | null;
  isFavorite: boolean;
  exifInfo: ExifInfo | null;
  tags: TagRef[];
  copyOf?: string;
  copyLabel?: string;
}
