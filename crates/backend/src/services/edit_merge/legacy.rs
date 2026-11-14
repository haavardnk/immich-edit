use raw_pipeline::edits::Edits;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct LegacySections {
    pub basic: bool,
    pub tone: bool,
    pub color: bool,
    pub detail: bool,
    pub effects: bool,
    pub lens: bool,
    pub geometry: bool,
    pub masks: bool,
    pub retouch: bool,
}

fn pick<T>(use_incoming: bool, incoming: T, current: T) -> T {
    if use_incoming { incoming } else { current }
}

pub fn merge_legacy(current: Edits, incoming: Edits, sections: LegacySections) -> Edits {
    Edits {
        basic: pick(sections.basic, incoming.basic, current.basic),
        tone: pick(sections.tone, incoming.tone, current.tone),
        color: pick(sections.color, incoming.color, current.color),
        detail: pick(sections.detail, incoming.detail, current.detail),
        effects: pick(sections.effects, incoming.effects, current.effects),
        lens: pick(sections.lens, incoming.lens, current.lens),
        geometry: pick(sections.geometry, incoming.geometry, current.geometry),
        masks: pick(sections.masks, incoming.masks, current.masks),
        retouch: pick(sections.retouch, incoming.retouch, current.retouch),
        lens_blur: current.lens_blur,
        unknown_ops: current.unknown_ops,
    }
}
