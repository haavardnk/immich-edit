use std::collections::BTreeMap;

use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Deserializer};

use super::{ClickTarget, Preferences, SectionEntry};

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

fn non_null<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    match Option::<T>::deserialize(deserializer)? {
        Some(value) => Ok(Some(value)),
        None => Err(D::Error::invalid_type(Unexpected::Unit, &"a value")),
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferencesPatch {
    #[serde(default, deserialize_with = "non_null")]
    pub click_target: Option<ClickTarget>,
    #[serde(default, deserialize_with = "present")]
    pub pinned_settings: Option<Option<Vec<String>>>,
    #[serde(default, deserialize_with = "present")]
    pub mask_overlay_color: Option<Option<String>>,
    #[serde(default, deserialize_with = "non_null")]
    pub shortcuts: Option<BTreeMap<String, Vec<String>>>,
    #[serde(default, deserialize_with = "present")]
    pub desktop_sections: Option<Option<Vec<SectionEntry>>>,
    #[serde(default, deserialize_with = "present")]
    pub touch_subviews: Option<Option<Vec<SectionEntry>>>,
}

impl PreferencesPatch {
    pub fn apply(self, current: Preferences) -> Preferences {
        Preferences {
            click_target: self.click_target.unwrap_or(current.click_target),
            pinned_settings: self.pinned_settings.unwrap_or(current.pinned_settings),
            mask_overlay_color: self
                .mask_overlay_color
                .unwrap_or(current.mask_overlay_color),
            shortcuts: self.shortcuts.unwrap_or(current.shortcuts),
            desktop_sections: self.desktop_sections.unwrap_or(current.desktop_sections),
            touch_subviews: self.touch_subviews.unwrap_or(current.touch_subviews),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(body: serde_json::Value) -> Result<PreferencesPatch, serde_json::Error> {
        serde_json::from_value(body)
    }

    fn current() -> Preferences {
        Preferences {
            click_target: ClickTarget::Loupe,
            pinned_settings: Some(vec!["exposure".into()]),
            mask_overlay_color: Some("#112233".into()),
            shortcuts: BTreeMap::from([("help".into(), vec!["?".into()])]),
            desktop_sections: Some(vec![SectionEntry {
                id: "basic".into(),
                hidden: true,
            }]),
            touch_subviews: Some(vec![]),
        }
    }

    #[test]
    fn an_empty_patch_changes_nothing() {
        assert_eq!(parse(json!({})).unwrap().apply(current()), current());
    }

    #[test]
    fn absent_fields_keep_and_present_fields_replace() {
        let merged = parse(json!({ "click_target": "editor", "shortcuts": {} }))
            .unwrap()
            .apply(current());
        assert_eq!(
            merged,
            Preferences {
                click_target: ClickTarget::Editor,
                shortcuts: BTreeMap::new(),
                ..current()
            }
        );
    }

    #[test]
    fn null_clears_every_nullable_field() {
        let merged = parse(json!({
            "pinned_settings": null,
            "mask_overlay_color": null,
            "desktop_sections": null,
            "touch_subviews": null
        }))
        .unwrap()
        .apply(current());
        assert_eq!(
            merged,
            Preferences {
                click_target: ClickTarget::Loupe,
                shortcuts: current().shortcuts,
                ..Preferences::default()
            }
        );
    }

    #[test]
    fn null_is_a_data_error_for_fields_that_are_not_nullable() {
        for body in [
            json!({ "click_target": null }),
            json!({ "shortcuts": null }),
        ] {
            let error = parse(body.clone()).unwrap_err();
            assert!(error.is_data(), "{body}: {error}");
            assert!(error.to_string().contains("expected a value"), "{error}");
        }
    }

    #[test]
    fn unknown_fields_and_bad_shapes_are_refused() {
        assert!(parse(json!({ "theme": "dark" })).is_err());
        assert!(parse(json!({ "click_target": "Loupe" })).is_err());
        assert!(parse(json!({ "desktop_sections": [{ "id": "basic" }] })).is_err());
        assert!(
            parse(json!({ "desktop_sections": [{ "id": "basic", "hidden": false, "x": 1 }] }))
                .is_err()
        );
        assert!(parse(json!({ "pinned_settings": "exposure" })).is_err());
    }
}
