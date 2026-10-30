use raw_pipeline::edits::{Edits, RetouchMode, RetouchStroke, Vec2f};

fn heal_stroke() -> RetouchStroke {
    RetouchStroke {
        id: "heal".into(),
        mode: RetouchMode::Heal,
        points: vec![Vec2f { x: 0.3, y: 0.4 }, Vec2f { x: 0.35, y: 0.45 }],
        radius: 0.05,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f { x: 0.6, y: 0.4 },
        enabled: true,
    }
}

pub fn untileable_edits() -> Edits {
    Edits {
        retouch: vec![heal_stroke()],
        ..Edits::default()
    }
}
