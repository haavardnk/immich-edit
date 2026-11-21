use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ImmichAlbum")]
pub struct AlbumSummary {
    pub id: Uuid,
    #[serde(rename = "albumName")]
    pub album_name: String,
    pub description: String,
    #[serde(rename = "assetCount")]
    pub asset_count: u32,
    #[serde(rename = "albumThumbnailAssetId")]
    pub album_thumbnail_asset_id: Option<Uuid>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<Uuid>,
    pub shared: bool,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub members: Vec<AlbumMember>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlbumMember {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    pub name: String,
    pub role: AlbumRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlbumRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Debug, Default, Serialize)]
pub struct AlbumPatch {
    #[serde(rename = "albumName", skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        rename = "albumThumbnailAssetId",
        skip_serializing_if = "Option::is_none"
    )]
    pub album_thumbnail_asset_id: Option<Uuid>,
}

impl AlbumPatch {
    pub fn is_empty(&self) -> bool {
        self.album_name.is_none()
            && self.description.is_none()
            && self.album_thumbnail_asset_id.is_none()
    }
}

#[derive(Deserialize)]
struct ImmichAlbum {
    id: Uuid,
    #[serde(rename = "albumName")]
    album_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "assetCount", default)]
    asset_count: u32,
    #[serde(rename = "albumThumbnailAssetId", default)]
    album_thumbnail_asset_id: Option<Uuid>,
    #[serde(rename = "updatedAt", default)]
    updated_at: Option<String>,
    #[serde(rename = "albumUsers", default)]
    album_users: Vec<ImmichAlbumUser>,
    #[serde(default)]
    shared: bool,
    #[serde(rename = "startDate", default)]
    start_date: Option<String>,
    #[serde(rename = "endDate", default)]
    end_date: Option<String>,
}

#[derive(Deserialize)]
struct ImmichAlbumUser {
    user: ImmichAlbumUserProfile,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Deserialize)]
struct ImmichAlbumUserProfile {
    id: Uuid,
    #[serde(default)]
    name: String,
}

fn role_of(raw: Option<&str>) -> AlbumRole {
    match raw {
        Some("owner") => AlbumRole::Owner,
        Some("editor") => AlbumRole::Editor,
        _ => AlbumRole::Viewer,
    }
}

impl From<ImmichAlbumUser> for AlbumMember {
    fn from(member: ImmichAlbumUser) -> Self {
        Self {
            user_id: member.user.id,
            name: member.user.name,
            role: role_of(member.role.as_deref()),
        }
    }
}

impl From<ImmichAlbum> for AlbumSummary {
    fn from(album: ImmichAlbum) -> Self {
        let owner_id = album.album_users.first().map(|owner| owner.user.id);
        Self {
            id: album.id,
            album_name: album.album_name,
            description: album.description.unwrap_or_default(),
            asset_count: album.asset_count,
            album_thumbnail_asset_id: album.album_thumbnail_asset_id,
            updated_at: album.updated_at,
            owner_id,
            shared: album.shared,
            start_date: album.start_date,
            end_date: album.end_date,
            members: album
                .album_users
                .into_iter()
                .map(AlbumMember::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlbumPatch, AlbumRole, AlbumSummary};

    const OWNER: &str = "00000000-0000-0000-0000-00000000000a";
    const EDITOR: &str = "00000000-0000-0000-0000-00000000000b";

    fn album(users: &str) -> AlbumSummary {
        let json = format!(
            r#"{{"id":"00000000-0000-0000-0000-000000000001","albumName":"Trip","albumUsers":[{users}]}}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn members_carry_their_immich_role() {
        let album = album(&format!(
            r#"{{"user":{{"id":"{OWNER}","name":"Ann"}},"role":"owner"}},{{"user":{{"id":"{EDITOR}","name":"Bo"}},"role":"editor"}}"#
        ));
        let roles: Vec<(String, AlbumRole)> = album
            .members
            .iter()
            .map(|member| (member.name.clone(), member.role))
            .collect();
        assert_eq!(
            roles,
            [
                ("Ann".to_string(), AlbumRole::Owner),
                ("Bo".to_string(), AlbumRole::Editor)
            ]
        );
        assert_eq!(album.owner_id.unwrap().to_string(), OWNER);
    }

    #[test]
    fn an_unknown_or_missing_role_is_a_viewer() {
        let album = album(&format!(
            r#"{{"user":{{"id":"{OWNER}"}},"role":"curator"}},{{"user":{{"id":"{EDITOR}"}}}}"#
        ));
        assert!(
            album
                .members
                .iter()
                .all(|member| member.role == AlbumRole::Viewer && member.name.is_empty())
        );
    }

    #[test]
    fn an_album_without_users_or_description_defaults_both() {
        let album = album("");
        assert!(album.members.is_empty());
        assert!(album.owner_id.is_none());
        assert_eq!(album.description, "");
    }

    #[test]
    fn a_cleared_description_is_sent_as_an_empty_string() {
        let patch = AlbumPatch {
            description: Some(String::new()),
            ..AlbumPatch::default()
        };
        assert_eq!(
            serde_json::to_value(&patch).unwrap(),
            serde_json::json!({ "description": "" })
        );
        assert!(AlbumPatch::default().is_empty());
    }
}
