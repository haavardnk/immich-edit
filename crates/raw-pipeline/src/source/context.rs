mod grid;
#[cfg(feature = "native")]
mod planes;

use std::iter::once;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cpu::masked::effective_edits_for_layer;
use crate::edits::{Edits, LensBlurEdits};
use crate::ops::presence::{has_shadows, presence_amounts};
pub use grid::{ContextGrid, ContextPart, SourceContext};
#[cfg(feature = "native")]
pub(crate) use planes::{ContextPlane, ContextPlanes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContextNeeds {
    pub dehaze: bool,
    pub texture: bool,
    pub clarity: bool,
    pub shadows: bool,
    pub presence: bool,
}

impl ContextNeeds {
    pub fn new(edits: &Edits) -> Self {
        let global = presence_amounts(edits);
        let layers = edits
            .masks
            .iter()
            .filter(|layer| layer.is_effective())
            .map(|layer| presence_amounts(&effective_edits_for_layer(edits, layer)));
        let (texture, clarity) = once(global)
            .chain(layers)
            .filter(|amounts| amounts.adjusts())
            .fold((false, false), |(t, c), amounts| {
                (t || amounts.texture != 0.0, c || amounts.clarity != 0.0)
            });
        Self {
            dehaze: edits.basic.dehaze != 0.0,
            texture,
            clarity,
            shadows: has_shadows(edits),
            presence: global.adjusts(),
        }
    }

    pub fn levels(&self) -> bool {
        self.texture || self.clarity
    }

    pub fn coarse(&self) -> bool {
        self.levels() || self.shadows
    }
}

#[derive(Serialize)]
struct ContextKey<'a> {
    texture: bool,
    clarity: bool,
    shadows: bool,
    dehaze: Option<f64>,
    lens_blur: Option<&'a LensBlurEdits>,
    adjust: Option<[u32; 3]>,
}

pub fn context_key(edits: &Edits) -> String {
    let edits = edits.clamped();
    let needs = ContextNeeds::new(&edits);
    let coarse = needs.coarse();
    let key = ContextKey {
        texture: needs.texture,
        clarity: needs.clarity,
        shadows: needs.shadows,
        dehaze: coarse.then_some(edits.basic.dehaze),
        lens_blur: (coarse && edits.lens_blur.active()).then_some(&edits.lens_blur),
        adjust: (needs.shadows && needs.presence).then(|| presence_amounts(&edits).bits()),
    };
    hashed(&key)
}

pub fn window_key(edits: &Edits) -> String {
    hashed(&edits.clamped().geometry)
}

fn hashed(value: &impl Serialize) -> String {
    let json = serde_json::to_vec(value).expect("source key serializes");
    hex::encode(&Sha256::digest(&json)[..16])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
        Vec2f,
    };

    fn layer(texture: f64) -> MaskLayer {
        MaskLayer {
            id: "l1".into(),
            name: String::new(),
            enabled: true,
            amount: 1.0,
            invert: false,
            components: vec![MaskComponent {
                id: "c1".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                invert: false,
                kind: MaskComponentKind::Linear {
                    p0: Vec2f { x: 0.0, y: 0.5 },
                    p1: Vec2f { x: 1.0, y: 0.5 },
                    feather: 0.4,
                },
                source: MaskSource::Manual,
                generated: None,
            }],
            edits: MaskedEdits {
                texture: Some(texture),
                ..Default::default()
            },
        }
    }

    #[test]
    fn the_key_follows_only_what_the_context_reads() {
        let mut base = Edits::default();
        base.basic.dehaze = 30.0;
        base.basic.clarity = 20.0;
        base.tone.shadows = 40.0;
        let key = context_key(&base);
        let same = |change: fn(&mut Edits)| {
            let mut edits = base.clone();
            change(&mut edits);
            context_key(&edits) == key
        };
        for (label, change, expected) in [
            (
                "contrast",
                (|e| e.basic.contrast = 30.0) as fn(&mut Edits),
                true,
            ),
            ("shadows amount", |e| e.tone.shadows = 80.0, true),
            ("blur without depth", |e| e.lens_blur.amount = 40.0, true),
            ("dehaze", |e| e.basic.dehaze = 31.0, false),
            ("clarity", |e| e.basic.clarity = 25.0, false),
            (
                "exposure under clarity",
                |e| e.basic.exposure_ev = 0.5,
                false,
            ),
            ("shadows off", |e| e.tone.shadows = 0.0, false),
            (
                "lens blur",
                |e| {
                    e.lens_blur.amount = 40.0;
                    e.lens_blur.depth_raster_id = "depth".into();
                },
                false,
            ),
        ] {
            if same(change) != expected {
                panic!("{label}: the key changed is {expected}");
            }
        }
    }

    #[test]
    fn dehaze_without_levels_leaves_the_key_alone() {
        let mut moved = Edits::default();
        moved.basic.dehaze = 60.0;
        moved.basic.exposure_ev = 1.0;
        if context_key(&moved) != context_key(&Edits::default()) {
            panic!("the haze grid does not depend on the dehaze amount");
        }
    }

    #[test]
    fn the_window_key_follows_every_geometry_edit() {
        let base = window_key(&Edits::default());
        let mut flipped = Edits::default();
        flipped.geometry.flip_h = true;
        let mut graded = Edits::default();
        graded.basic.dehaze = 40.0;
        if window_key(&flipped) == base || window_key(&graded) != base {
            panic!("the window key must follow geometry and nothing else");
        }
    }

    #[test]
    fn a_layer_with_texture_needs_the_texture_level() {
        let edits = Edits {
            masks: vec![layer(30.0)],
            ..Edits::default()
        };
        let needs = ContextNeeds::new(&edits);
        if !needs.texture || needs.presence || needs.clarity {
            panic!("layer texture gave {needs:?}");
        }
    }
}
