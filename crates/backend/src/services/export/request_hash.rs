use sha2::{Digest, Sha256};

use crate::asset_key::AssetKey;

use super::ExportToImmichBody;

const ON_CONFLICT_KEY: &str = "on_conflict";

pub fn hash_request(asset_id: AssetKey, body: &ExportToImmichBody) -> String {
    let mut album_ids = body.album_ids.clone();
    album_ids.sort();
    let mut tag_ids = body.tag_ids.clone();
    tag_ids.sort();
    let mut canonical = serde_json::json!({
        "asset_id": asset_id.to_string(),
        "edits": body.edits.clamped(),
        "format": format!("{:?}", body.params.format),
        "quality": body.params.quality,
        "metadata": format!("{:?}", body.params.metadata()),
        "bit_depth": format!("{:?}", body.params.bit_depth),
        "png_compression": format!("{:?}", body.params.png_compression),
        "tiff_compression": format!("{:?}", body.params.tiff_compression),
        "lossless": body.params.lossless,
        "color_space": format!("{:?}", body.params.color_space),
        "album_ids": album_ids,
        "tag_ids": tag_ids,
        "favorite": body.favorite,
        "stack_with_original": body.stack_with_original,
        "stack_primary": format!("{:?}", body.stack_primary),
        "replace_original": body.replace_original,
        "filename_template": body.params.filename_template,
        "resize_mode": body.params.resize_mode.map(|m| format!("{m:?}")),
        "resize_width": body.params.resize_width,
        "resize_height": body.params.resize_height,
        "resize_megapixels": body.params.resize_megapixels,
        "resize_percent": body.params.resize_percent,
        "resize_enlarge": body.params.resize_enlarge,
        "output_sharpen_media": body.params.output_sharpen_media.map(|m| format!("{m:?}")),
        "output_sharpen_amount": format!("{:?}", body.params.output_sharpen_amount),
        "output_sharpen_ppi": body.params.output_sharpen_ppi,
        "watermark_id": body.params.watermark_id,
        "watermark_size": body.params.watermark_size,
        "watermark_opacity": body.params.watermark_opacity,
        "watermark_anchor": format!("{:?}", body.params.watermark_anchor),
        "watermark_inset": body.params.watermark_inset,
    });
    if let (Some(policy), Some(fields)) = (body.on_conflict.hashed(), canonical.as_object_mut()) {
        fields.insert(ON_CONFLICT_KEY.into(), policy.into());
    }
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;

    const PLAIN_HASH: &str = "c6788514f409f466558f65072efa219af0a1c675f09f9f9166175b6dc62e57c7";
    const FULL_HASH: &str = "6bc864154c8ae057c6b30544c6677c3ad1cbe4e5a8e9d7fd113a9df30cd2ba1b";

    fn full() -> Value {
        json!({
            "album_ids": ["11111111-2222-3333-4444-555555555555"],
            "format": "png",
            "replace_original": true
        })
    }

    fn hash_of(mut value: Value, on_conflict: Option<&str>) -> String {
        if let (Some(policy), Some(fields)) = (on_conflict, value.as_object_mut()) {
            fields.insert(ON_CONFLICT_KEY.into(), policy.into());
        }
        let body: ExportToImmichBody = serde_json::from_value(value).unwrap();
        hash_request(AssetKey::master(Uuid::nil()), &body)
    }

    #[test]
    fn keep_both_keeps_the_hash_of_requests_made_before_the_policy() {
        for policy in [None, Some("keep_both")] {
            assert_eq!(hash_of(json!({}), policy), PLAIN_HASH, "{policy:?}");
            assert_eq!(hash_of(full(), policy), FULL_HASH, "{policy:?}");
        }
    }

    #[test]
    fn skip_and_replace_change_the_hash() {
        let skip = hash_of(full(), Some("skip"));
        let replace = hash_of(full(), Some("replace"));

        assert_ne!(skip, FULL_HASH);
        assert_ne!(replace, FULL_HASH);
        assert_ne!(skip, replace);
    }
}
