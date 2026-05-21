export interface AssetDetail {
  id: string;
  originalFileName: string;
  type: string;
  originalMimeType: string | null;
  fileCreatedAt: string | null;
  updatedAt: string | null;
  checksum: string | null;
}
