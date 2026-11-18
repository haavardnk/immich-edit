use std::collections::BTreeMap;

use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Deserializer};

use super::{CropRatio, Preferences, SectionEntry};

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
    #[serde(default, deserialize_with = "present")]
    pub crop_ratios: Option<Option<Vec<CropRatio>>>,
}

impl PreferencesPatch {
    pub fn apply(self, current: Preferences) -> Preferences {
        Preferences {
            pinned_settings: self.pinned_settings.unwrap_or(current.pinned_settings),
            mask_overlay_color: self
                .mask_overlay_color
                .unwrap_or(current.mask_overlay_color),
            shortcuts: self.shortcuts.unwrap_or(current.shortcuts),
            desktop_sections: self.desktop_sections.unwrap_or(current.desktop_sections),
            touch_subviews: self.touch_subviews.unwrap_or(current.touch_subviews),
            crop_ratios: self
                .crop_ratios
                .map(|ratios| {
                    ratios.map(|list| list.into_iter().map(CropRatio::normalized).collect())
                })
                .unwrap_or(current.crop_ratios),
        }
    }
}
