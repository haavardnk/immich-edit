import type { AssetType, ExifInfo } from './asset';

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
  assets: AssetSummary[];
  updatedAt: string | null;
}

export interface AssetSummary {
  id: string;
  originalFileName: string;
  type: AssetType;
  fileCreatedAt: string | null;
  updatedAt: string | null;
  checksum: string | null;
  isFavorite: boolean;
  exifInfo: ExifInfo | null;
}
