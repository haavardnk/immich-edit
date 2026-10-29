use raw_pipeline::edits::{
    MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
};

pub fn layer(id: &str, kind: MaskComponentKind, edits: MaskedEdits) -> MaskLayer {
    MaskLayer {
        id: id.into(),
        name: String::new(),
        enabled: true,
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: format!("{id}-component"),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind,
            source: MaskSource::Manual,
            generated: None,
        }],
        edits,
    }
}
