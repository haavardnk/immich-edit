use raw_pipeline::edits::{Edits, RetouchMode, RetouchStroke, Vec2f};

pub fn stroke(
    mode: RetouchMode,
    points: &[(f32, f32)],
    radius: f32,
    source: (f32, f32),
) -> RetouchStroke {
    RetouchStroke {
        id: String::new(),
        mode,
        points: points.iter().map(|&(x, y)| Vec2f { x, y }).collect(),
        radius,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f {
            x: source.0,
            y: source.1,
        },
        enabled: true,
    }
}

pub fn untileable_edits() -> Edits {
    let across = [(0.0, 0.25), (1.0, 0.25)];
    Edits {
        retouch: vec![stroke(RetouchMode::Heal, &across, 0.25, (0.5, 0.75))],
        ..Edits::default()
    }
}
