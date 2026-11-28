use std::collections::HashSet;

use crate::services::edits_store::TileOverlay;

const ICONS: [&str; 11] = [
    "edited",
    "favorite",
    "rejected",
    "label",
    "rating",
    "copy",
    "location",
    "tags",
    "description",
    "albums",
    "format",
];
const DETAILS: [&str; 10] = [
    "filename",
    "taken",
    "place",
    "tags",
    "description",
    "albums",
    "exposure",
    "camera",
    "lens",
    "size",
];
const MAX_DETAILS: usize = 4;

pub fn tile_overlay(overlay: &TileOverlay) -> Result<(), String> {
    keys("tile_overlay.icons", &overlay.icons, &ICONS)?;
    if overlay.details.len() > MAX_DETAILS {
        return Err(format!(
            "tile_overlay.details: at most {MAX_DETAILS} entries"
        ));
    }
    keys("tile_overlay.details", &overlay.details, &DETAILS)
}

fn keys(field: &str, keys: &[String], allowed: &[&str]) -> Result<(), String> {
    let mut seen = HashSet::with_capacity(keys.len());
    for key in keys {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{field}: unknown key {key:?}"));
        }
        if !seen.insert(key.as_str()) {
            return Err(format!("{field}: duplicate key {key:?}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(icons: &[&str], details: &[&str]) -> TileOverlay {
        TileOverlay {
            icons: icons.iter().map(|key| key.to_string()).collect(),
            details: details.iter().map(|key| key.to_string()).collect(),
        }
    }

    #[test]
    fn every_catalog_key_is_accepted() {
        assert!(tile_overlay(&overlay(&ICONS, &DETAILS[..4])).is_ok());
        assert!(tile_overlay(&overlay(&[], &DETAILS[6..])).is_ok());
        assert!(tile_overlay(&overlay(&[], &[])).is_ok());
    }

    #[test]
    fn unknown_keys_name_their_list() {
        assert_eq!(
            tile_overlay(&overlay(&["people"], &[])).unwrap_err(),
            "tile_overlay.icons: unknown key \"people\""
        );
        assert_eq!(
            tile_overlay(&overlay(&[], &["format"])).unwrap_err(),
            "tile_overlay.details: unknown key \"format\""
        );
        assert!(tile_overlay(&overlay(&["Favorite"], &[])).is_err());
    }

    #[test]
    fn keys_are_unique_per_list() {
        assert_eq!(
            tile_overlay(&overlay(&["tags", "tags"], &[])).unwrap_err(),
            "tile_overlay.icons: duplicate key \"tags\""
        );
        assert_eq!(
            tile_overlay(&overlay(&[], &["lens", "lens"])).unwrap_err(),
            "tile_overlay.details: duplicate key \"lens\""
        );
        assert!(tile_overlay(&overlay(&["tags"], &["tags"])).is_ok());
    }

    #[test]
    fn at_most_four_details() {
        assert_eq!(
            tile_overlay(&overlay(&[], &DETAILS[..5])).unwrap_err(),
            "tile_overlay.details: at most 4 entries"
        );
    }
}
