export interface ExifInfo {
  make: string | null;
  model: string | null;
  lensModel: string | null;
  fNumber: number | null;
  focalLength: number | null;
  iso: number | null;
  exposureTime: string | null;
  exifImageWidth: number | null;
  exifImageHeight: number | null;
  dateTimeOriginal: string | null;
  rating: number | null;
  fileSizeInByte: number | null;
}

export interface TagRef {
  id: string;
  name: string;
  value: string;
  parentId?: string | null;
  color?: string | null;
}

export interface AssetDetail {
  id: string;
  originalFileName: string;
  type: string;
  originalMimeType: string | null;
  fileCreatedAt: string | null;
  updatedAt: string | null;
  checksum: string | null;
  isFavorite: boolean;
  exifInfo: ExifInfo | null;
  tags: TagRef[];
}
