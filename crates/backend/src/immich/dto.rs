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
}
