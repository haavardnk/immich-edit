use uuid::Uuid;

use crate::immich::dto::TagSummary;
use crate::names::display_name;

pub const MANAGED_ROOT: &str = "immich-edit";

pub fn is_managed(value: &str) -> bool {
    let root = value.split('/').next().unwrap_or(value);
    root.eq_ignore_ascii_case(MANAGED_ROOT)
}

pub fn tag_name(raw: &str) -> Result<String, String> {
    let name = display_name(raw, "tag name")?;
    if name.contains('/') {
        return Err("tag name must not contain /".into());
    }
    Ok(name)
}

pub fn tag_color(raw: &str) -> Result<String, String> {
    let hex = raw.trim().trim_start_matches('#');
    let valid_length = matches!(hex.len(), 3 | 4 | 6 | 8);
    if !valid_length || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("tag color must be a hex color".into());
    }
    Ok(format!("#{}", hex.to_ascii_lowercase()))
}

pub fn sibling_named<'a>(
    tags: &'a [TagSummary],
    parent: Option<Uuid>,
    name: &str,
    except: Option<Uuid>,
) -> Option<&'a TagSummary> {
    tags.iter().find(|tag| {
        tag.parent_id == parent
            && Some(tag.id) != except
            && tag.name.to_lowercase() == name.to_lowercase()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(id: u128, name: &str, parent: Option<u128>) -> TagSummary {
        TagSummary {
            id: Uuid::from_u128(id),
            name: name.into(),
            value: name.into(),
            parent_id: parent.map(Uuid::from_u128),
            color: None,
            created_at: None,
            updated_at: None,
            asset_count: None,
        }
    }

    #[test]
    fn the_managed_root_and_everything_under_it_is_managed() {
        for value in ["immich-edit", "Immich-Edit/reject", "immich-edit/label/red"] {
            assert!(is_managed(value), "{value}");
        }
        for value in ["immich-editor", "Travel/immich-edit", "immich", ""] {
            assert!(!is_managed(value), "{value}");
        }
    }

    #[test]
    fn a_tag_name_follows_the_display_name_rule_without_a_slash() {
        assert_eq!(tag_name("  Norway ").unwrap(), "Norway");
        assert_eq!(tag_name("  ").unwrap_err(), "tag name required");
        assert_eq!(
            tag_name("Travel/Norway").unwrap_err(),
            "tag name must not contain /"
        );
        assert!(tag_name(&"x".repeat(81)).is_err());
    }

    #[test]
    fn a_color_is_a_short_or_long_hex_with_an_optional_hash() {
        for (raw, color) in [
            ("#3E8ED0", "#3e8ed0"),
            ("3e8ed0", "#3e8ed0"),
            ("#fff", "#fff"),
            ("ffff", "#ffff"),
            (" #3e8ed0ff ", "#3e8ed0ff"),
        ] {
            assert_eq!(tag_color(raw).unwrap(), color, "{raw}");
        }
        for raw in ["", "#", "#12", "#12345", "#ggg", "blue", "#3e8ed0f"] {
            assert!(tag_color(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn a_sibling_matches_by_parent_and_case_insensitive_name() {
        let tags = [
            tag(1, "Travel", None),
            tag(2, "Norway", Some(1)),
            tag(3, "Norway", None),
        ];
        let travel = Some(Uuid::from_u128(1));
        assert_eq!(
            sibling_named(&tags, travel, "norway", None).map(|t| t.id),
            Some(Uuid::from_u128(2))
        );
        assert!(sibling_named(&tags, travel, "norway", Some(Uuid::from_u128(2))).is_none());
        assert!(sibling_named(&tags, travel, "Sweden", None).is_none());
        assert_eq!(
            sibling_named(&tags, None, "NORWAY", None).map(|t| t.id),
            Some(Uuid::from_u128(3))
        );
        let nordic = [tag(4, "Ål", None)];
        assert!(sibling_named(&nordic, None, "ål", None).is_some());
    }
}
