use raw_pipeline::edits::Edits;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MergeSections {
    #[serde(default)]
    pub basic: bool,
    #[serde(default)]
    pub tone: bool,
    #[serde(default)]
    pub color: bool,
    #[serde(default)]
    pub detail: bool,
    #[serde(default)]
    pub effects: bool,
    #[serde(default)]
    pub lens: bool,
    #[serde(default)]
    pub output: bool,
    #[serde(default)]
    pub geometry: bool,
    #[serde(default)]
    pub masks: bool,
}

impl MergeSections {
    pub fn look_only() -> Self {
        Self {
            basic: true,
            tone: true,
            color: true,
            detail: true,
            effects: true,
            lens: true,
            output: false,
            geometry: false,
            masks: false,
        }
    }

    pub fn paste_default() -> Self {
        Self {
            output: true,
            ..Self::look_only()
        }
    }
}

fn pick<T>(use_incoming: bool, incoming: T, current: T) -> T {
    if use_incoming { incoming } else { current }
}

pub fn merge_edits(current: Edits, incoming: Edits, sections: MergeSections) -> Edits {
    Edits {
        basic: pick(sections.basic, incoming.basic, current.basic),
        tone: pick(sections.tone, incoming.tone, current.tone),
        color: pick(sections.color, incoming.color, current.color),
        detail: pick(sections.detail, incoming.detail, current.detail),
        effects: pick(sections.effects, incoming.effects, current.effects),
        lens: pick(sections.lens, incoming.lens, current.lens),
        geometry: pick(sections.geometry, incoming.geometry, current.geometry),
        masks: pick(sections.masks, incoming.masks, current.masks),
        output: pick(sections.output, incoming.output, current.output),
        unknown_ops: current.unknown_ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_pipeline::edits::TonemapKind;

    fn incoming() -> Edits {
        let mut e = Edits::default();
        e.effects.vignette_amount = 40.0;
        e.geometry.rotate = 2;
        e.output.tonemap = TonemapKind::Agx;
        e
    }

    #[test]
    fn paste_default_applies_look_and_output() {
        let mut current = Edits::default();
        current.geometry.rotate = 3;
        current
            .unknown_ops
            .insert("future_op".to_string(), serde_json::json!({ "x": 1 }));
        let merged = merge_edits(current, incoming(), MergeSections::paste_default());
        assert_eq!(merged.effects.vignette_amount, 40.0);
        assert_eq!(merged.output.tonemap, TonemapKind::Agx);
        assert_eq!(merged.geometry.rotate, 3);
        assert!(merged.unknown_ops.contains_key("future_op"));
    }

    #[test]
    fn only_selected_sections_apply() {
        let current = Edits::default();
        let sections = MergeSections {
            geometry: true,
            ..MergeSections {
                basic: false,
                tone: false,
                color: false,
                detail: false,
                effects: false,
                lens: false,
                output: false,
                geometry: false,
                masks: false,
            }
        };
        let merged = merge_edits(current, incoming(), sections);
        assert_eq!(merged.geometry.rotate, 2);
        assert_eq!(merged.effects.vignette_amount, 0.0);
        assert_eq!(merged.output.tonemap, TonemapKind::Default);
    }

    #[test]
    fn look_only_excludes_output_geometry_masks() {
        let merged = merge_edits(Edits::default(), incoming(), MergeSections::look_only());
        assert_eq!(merged.effects.vignette_amount, 40.0);
        assert_eq!(merged.geometry.rotate, 0);
        assert_eq!(merged.output.tonemap, TonemapKind::Default);
    }
}
