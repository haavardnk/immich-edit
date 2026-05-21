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
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStatistics {
    pub images: u32,
    pub videos: u32,
    pub total: u32,
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
