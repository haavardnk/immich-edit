use super::*;

#[test]
fn geometry_rotate_swaps_dims() {
    let mut img = solid_image(4, 2, [0.5, 0.5, 0.5]);
    let edits = Edits {
        geometry: GeometryEdits {
            rotate: 90,
            ..Default::default()
        },
        ..Default::default()
    };
    transform::TransformOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert_eq!(img.width, 2);
    assert_eq!(img.height, 4);
}

#[test]
fn registry_orders_by_stage() {
    let reg = default_registry();
    let stages: Vec<Stage> = reg.ops().iter().map(|o| o.stage()).collect();
    let mut sorted = stages.clone();
    sorted.sort();
    assert_eq!(stages, sorted);
}

#[test]
fn dehaze_runs_after_nr_and_before_presence() {
    let reg = default_registry();
    let ids: Vec<&str> = reg.ops().iter().map(|o| o.id()).collect();
    let pos = |id: &str| ids.iter().position(|s| *s == id).unwrap();
    assert!(pos("luma_nr") < pos("dehaze"));
    assert!(pos("color_nr") < pos("dehaze"));
    assert!(pos("color_nr") < pos("capture_sharpen"));
    assert!(pos("capture_sharpen") < pos("dehaze"));
    assert!(pos("dehaze") < pos("texture"));
    assert!(pos("dehaze") < pos("clarity"));
}

#[test]
fn registry_skips_inactive_ops() {
    let reg = default_registry();
    let edits = Edits::default();
    let active: Vec<&str> = reg.active(&edits).map(|o| o.id()).collect();
    assert_eq!(
        active,
        vec![
            "camera_wb",
            "color_matrix",
            "capture_sharpen",
            "dcp_hue_sat"
        ]
    );
}

#[test]
fn hsl_runs_before_saturation_and_vibrance() {
    let reg = default_registry();
    let ids: Vec<&str> = reg.ops().iter().map(|o| o.id()).collect();
    let hsl = ids.iter().position(|s| *s == "hsl").unwrap();
    let sat = ids.iter().position(|s| *s == "saturation").unwrap();
    let vib = ids.iter().position(|s| *s == "vibrance").unwrap();
    assert!(hsl < sat);
    assert!(hsl < vib);
}
#[test]
fn ops_inactive_on_default_edits() {
    let ops: Vec<Box<dyn Op>> = vec![
        Box::new(exposure::ExposureOp),
        Box::new(brightness::BrightnessOp),
        Box::new(user_wb::UserWbOp),
        Box::new(texture::TextureOp),
        Box::new(clarity::ClarityOp),
        Box::new(dehaze::DehazeOp),
        Box::new(sharpen::SharpenOp),
        Box::new(retouch::RetouchOp),
    ];
    let edits = Edits::default();
    for op in ops {
        if op.is_active(&edits) {
            panic!("{} active on default edits", op.id());
        }
    }
}
#[test]
fn every_op_declares_a_reachable_gpu_route() {
    for op in default_registry().ops() {
        match op.gpu_route() {
            GpuRoute::Fused => {
                if op.gpu().is_none() {
                    panic!("{}: declares Fused but has no gpu()", op.id());
                }
            }
            GpuRoute::Pass(name) => {
                if !GPU_PASS_NAMES.contains(&name) {
                    panic!("{}: declares unknown gpu pass {name}", op.id());
                }
            }
            GpuRoute::Manifest => {
                if op.is_active(&Edits::default()) {
                    panic!("{}: Manifest ops must never be active", op.id());
                }
            }
            GpuRoute::Presence | GpuRoute::Detail => {}
        }
    }
}

#[test]
fn only_fused_ops_build_shader_snippets() {
    for op in default_registry().ops() {
        if op.gpu().is_some() && op.gpu_route() != GpuRoute::Fused {
            panic!(
                "{}: has gpu() but is not routed through the fused pass",
                op.id()
            );
        }
    }
}
