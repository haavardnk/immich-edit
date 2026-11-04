use std::collections::HashSet;

use crate::services::edits_store::{PreferencesPatch, SectionEntry};

mod shortcuts;

const MAX_ID_LEN: usize = 64;
const MAX_LIST_LEN: usize = 64;

pub fn patch(patch: &PreferencesPatch) -> Result<(), String> {
    if let Some(Some(pinned)) = &patch.pinned_settings {
        pinned_settings(pinned)?;
    }
    if let Some(Some(color)) = &patch.mask_overlay_color {
        mask_overlay_color(color)?;
    }
    if let Some(bindings) = &patch.shortcuts {
        shortcuts::shortcuts(bindings)?;
    }
    if let Some(Some(sections)) = &patch.desktop_sections {
        section_list("desktop_sections", sections)?;
    }
    if let Some(Some(subviews)) = &patch.touch_subviews {
        section_list("touch_subviews", subviews)?;
    }
    Ok(())
}

fn catalog_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    id.len() <= MAX_ID_LEN
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && bytes.all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
}

fn catalog_ids<'a>(field: &str, ids: impl ExactSizeIterator<Item = &'a str>) -> Result<(), String> {
    if ids.len() > MAX_LIST_LEN {
        return Err(format!("{field}: at most {MAX_LIST_LEN} entries"));
    }
    let mut seen = HashSet::new();
    for id in ids {
        if !catalog_id(id) {
            return Err(format!("{field}: invalid id"));
        }
        if !seen.insert(id) {
            return Err(format!("{field}: duplicate id {id:?}"));
        }
    }
    Ok(())
}

fn pinned_settings(ids: &[String]) -> Result<(), String> {
    catalog_ids("pinned_settings", ids.iter().map(String::as_str))
}

fn section_list(field: &str, sections: &[SectionEntry]) -> Result<(), String> {
    catalog_ids(field, sections.iter().map(|section| section.id.as_str()))
}

fn mask_overlay_color(color: &str) -> Result<(), String> {
    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if valid {
        Ok(())
    } else {
        Err("mask_overlay_color: expected lowercase #rrggbb".into())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|id| id.to_string()).collect()
    }

    fn sections(list: &[&str]) -> Vec<SectionEntry> {
        list.iter()
            .map(|id| SectionEntry {
                id: id.to_string(),
                hidden: false,
            })
            .collect()
    }

    fn check(body: serde_json::Value) -> Result<(), String> {
        patch(&serde_json::from_value(body).unwrap())
    }

    #[test]
    fn catalog_ids_follow_the_pattern() {
        for id in [
            "exposure",
            "white_balance",
            "color-grading",
            "lens_blur_amount",
            "0",
            "a.b",
            &"a".repeat(64),
        ] {
            assert!(catalog_id(id), "{id}");
        }
        for id in [
            "",
            "Exposure",
            "_exposure",
            "-a",
            ".a",
            "a b",
            "a/b",
            "é",
            &"a".repeat(65),
        ] {
            assert!(!catalog_id(id), "{id}");
        }
    }

    #[test]
    fn pinned_settings_are_unique_and_bounded() {
        assert!(pinned_settings(&[]).is_ok());
        assert!(pinned_settings(&ids(&["exposure", "contrast"])).is_ok());
        let full: Vec<String> = (0..64).map(|n| format!("s{n}")).collect();
        assert!(pinned_settings(&full).is_ok());
        let over: Vec<String> = (0..65).map(|n| format!("s{n}")).collect();
        assert!(
            pinned_settings(&over)
                .unwrap_err()
                .contains("pinned_settings")
        );
        assert!(
            pinned_settings(&ids(&["exposure", "exposure"]))
                .unwrap_err()
                .contains("duplicate")
        );
        assert!(pinned_settings(&ids(&["Exposure"])).is_err());
    }

    #[test]
    fn section_lists_are_unique_bounded_and_name_their_field() {
        assert!(section_list("touch_subviews", &sections(&["light", "color"])).is_ok());
        assert!(section_list("touch_subviews", &[]).is_ok());
        let over: Vec<String> = (0..65).map(|n| format!("v{n}")).collect();
        let over: Vec<&str> = over.iter().map(String::as_str).collect();
        assert!(
            section_list("desktop_sections", &sections(&over))
                .unwrap_err()
                .starts_with("desktop_sections")
        );
        assert!(
            section_list("touch_subviews", &sections(&["light", "light"]))
                .unwrap_err()
                .starts_with("touch_subviews")
        );
        assert!(section_list("desktop_sections", &sections(&["Basic"])).is_err());
    }

    #[test]
    fn mask_overlay_color_is_lowercase_hex() {
        for color in ["#000000", "#ff8800", "#a1b2c3"] {
            assert!(mask_overlay_color(color).is_ok(), "{color}");
        }
        for color in [
            "",
            "#FF8800",
            "#ff880",
            "#ff88000",
            "ff8800",
            "#gg8800",
            "#ff 800",
            "#ff8800ff",
        ] {
            assert!(mask_overlay_color(color).is_err(), "{color}");
        }
    }

    #[test]
    fn a_patch_checks_only_present_values() {
        assert!(check(json!({})).is_ok());
        assert!(
            check(json!({
                "click_target": "loupe",
                "pinned_settings": null,
                "mask_overlay_color": null,
                "desktop_sections": null,
                "touch_subviews": null
            }))
            .is_ok()
        );
        assert!(
            check(json!({
                "pinned_settings": ["exposure"],
                "mask_overlay_color": "#ff0000",
                "shortcuts": { "help": ["?"] },
                "desktop_sections": [{ "id": "basic", "hidden": false }],
                "touch_subviews": [{ "id": "light", "hidden": true }]
            }))
            .is_ok()
        );
    }

    #[test]
    fn a_patch_names_the_failing_field() {
        for (field, body) in [
            ("pinned_settings", json!({ "pinned_settings": ["X"] })),
            ("mask_overlay_color", json!({ "mask_overlay_color": "red" })),
            ("shortcuts", json!({ "shortcuts": { "1help": [] } })),
            (
                "desktop_sections",
                json!({ "desktop_sections": [
                    { "id": "a", "hidden": false },
                    { "id": "a", "hidden": true }
                ] }),
            ),
            (
                "touch_subviews",
                json!({ "touch_subviews": [{ "id": "", "hidden": false }] }),
            ),
        ] {
            assert!(check(body).unwrap_err().starts_with(field), "{field}");
        }
    }
}
