use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSummary {
    pub id: Uuid,
    #[serde(rename = "albumName")]
    pub album_name: String,
    #[serde(rename = "assetCount", default)]
    pub asset_count: u32,
    #[serde(rename = "albumThumbnailAssetId", default)]
    pub album_thumbnail_asset_id: Option<Uuid>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDetail {
    pub id: Uuid,
    #[serde(rename = "albumName")]
    pub album_name: String,
    #[serde(rename = "assetCount", default)]
    pub asset_count: u32,
    #[serde(default)]
    pub assets: Vec<AssetSummary>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: Uuid,
    #[serde(rename = "originalFileName", default)]
    pub original_file_name: String,
    #[serde(rename = "type", default)]
    pub asset_type: String,
    #[serde(rename = "fileCreatedAt", default)]
    pub file_created_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(rename = "isFavorite", default)]
    pub is_favorite: bool,
    #[serde(rename = "exifInfo", default)]
    pub exif_info: Option<ExifInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetail {
    pub id: Uuid,
    #[serde(rename = "originalFileName", default)]
    pub original_file_name: String,
    #[serde(rename = "type", default)]
    pub asset_type: String,
    #[serde(rename = "originalMimeType", default)]
    pub original_mime_type: Option<String>,
    #[serde(rename = "fileCreatedAt", default)]
    pub file_created_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(rename = "isFavorite", default)]
    pub is_favorite: bool,
    #[serde(rename = "exifInfo", default)]
    pub exif_info: Option<ExifInfo>,
    #[serde(default)]
    pub tags: Vec<TagSummary>,
    #[serde(rename = "stackId", default)]
    pub stack_id: Option<Uuid>,
    #[serde(default)]
    pub stack: Option<StackSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackSummary {
    pub id: Uuid,
    #[serde(rename = "primaryAssetId")]
    pub primary_asset_id: Uuid,
    #[serde(rename = "assetCount", default)]
    pub asset_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackDetail {
    pub id: Uuid,
    #[serde(rename = "primaryAssetId")]
    pub primary_asset_id: Uuid,
    #[serde(default)]
    pub assets: Vec<AssetSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub id: Uuid,
    #[serde(default)]
    pub status: String,
    #[serde(rename = "isTrashed", default)]
    pub is_trashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifInfo {
    #[serde(default)]
    pub make: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(rename = "lensModel", default)]
    pub lens_model: Option<String>,
    #[serde(rename = "fNumber", default)]
    pub f_number: Option<f64>,
    #[serde(rename = "focalLength", default)]
    pub focal_length: Option<f64>,
    #[serde(default)]
    pub iso: Option<i64>,
    #[serde(rename = "exposureTime", default)]
    pub exposure_time: Option<String>,
    #[serde(rename = "exifImageWidth", default)]
    pub exif_image_width: Option<u32>,
    #[serde(rename = "exifImageHeight", default)]
    pub exif_image_height: Option<u32>,
    #[serde(rename = "dateTimeOriginal", default)]
    pub date_time_original: Option<String>,
    #[serde(default)]
    pub rating: Option<i32>,
    #[serde(rename = "fileSizeInByte", default)]
    pub file_size_in_byte: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSummary {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "thumbnailPath", default)]
    pub thumbnail_path: String,
    #[serde(rename = "isHidden", default)]
    pub is_hidden: bool,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeopleResponse {
    pub people: Vec<PersonSummary>,
    #[serde(default)]
    pub total: u32,
    #[serde(rename = "hasNextPage", default)]
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(rename = "parentId", default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkIdResponse {
    pub id: Uuid,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub assets: SearchAssets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAssets {
    #[serde(default)]
    pub items: Vec<AssetSummary>,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(rename = "nextPage", default)]
    pub next_page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatistics {
    #[serde(default)]
    pub total: u64,
}
