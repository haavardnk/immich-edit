use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    #[serde(
        rename = "assetCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_count: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NewTag {
    pub name: String,
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct TagPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
}

impl TagPatch {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.color.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::TagPatch;

    #[test]
    fn a_patch_sends_only_its_fields_and_null_clears_the_color() {
        let rename = TagPatch {
            name: Some("Norge".into()),
            ..TagPatch::default()
        };
        assert_eq!(
            serde_json::to_value(&rename).unwrap(),
            serde_json::json!({ "name": "Norge" })
        );
        let clear = TagPatch {
            color: Some(None),
            ..TagPatch::default()
        };
        assert_eq!(
            serde_json::to_value(&clear).unwrap(),
            serde_json::json!({ "color": null })
        );
        assert!(TagPatch::default().is_empty());
    }
}
