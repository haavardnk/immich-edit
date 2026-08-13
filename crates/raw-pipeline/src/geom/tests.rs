use super::*;

#[test]
fn scale_to_max_cases() {
    let cases = [
        ((100, 50, 200), (100, 50)),
        ((4000, 3000, 1000), (1000, 750)),
        ((3000, 4000, 1000), (750, 1000)),
        ((6000, 1000, 512), (512, 85)),
    ];
    for ((w, h, max), want) in cases {
        assert_eq!(scale_to_max(w, h, max), want, "{w}x{h} @ {max}");
    }
}

#[test]
fn display_out_dims_follows_orientation_and_crop() {
    let mut edits = crate::edits::Edits::default();
    assert_eq!(
        display_out_dims((false, false, false), &edits, (4000, 3000), 512),
        (512, 384)
    );
    assert_eq!(
        display_out_dims((true, false, false), &edits, (4000, 3000), 512),
        (384, 512)
    );

    edits.geometry.rotate = 90;
    assert_eq!(
        display_out_dims((false, false, false), &edits, (4000, 3000), 512),
        (384, 512)
    );

    edits.geometry.rotate = 0;
    edits.geometry.crop = Some(CropRect {
        x: 0.0,
        y: 0.0,
        w: 0.5,
        h: 0.5,
    });
    assert_eq!(
        display_out_dims((false, false, false), &edits, (4000, 3000), 512),
        (512, 384)
    );
    assert_eq!(
        display_crop_px((false, false, false), &edits, (4000, 3000)),
        (2000, 1500)
    );
}

#[test]
fn bbox_zero_angle() {
    let b = rotated_bbox(100.0, 50.0, 0.0);
    assert!((b.w - 100.0).abs() < 1e-3);
    assert!((b.h - 50.0).abs() < 1e-3);
}

#[test]
fn bbox_90_swaps() {
    let b = rotated_bbox(100.0, 50.0, 90.0);
    assert!((b.w - 50.0).abs() < 1e-3);
    assert!((b.h - 100.0).abs() < 1e-3);
}

#[test]
fn inscribed_identity_at_zero() {
    let r = largest_inscribed_rect(100.0, 50.0, 0.0, 2.0, &IDENTITY);
    let bbox = rotated_bbox(100.0, 50.0, 0.0);
    let w_px = r.w * bbox.w;
    let h_px = r.h * bbox.h;
    assert!((w_px - 100.0).abs() < 1e-2, "w={w_px}");
    assert!((h_px - 50.0).abs() < 1e-2, "h={h_px}");
}

#[test]
fn inscribed_inside_rotated_source() {
    for &angle in &[5.0_f32, 10.0, 20.0, 30.0, -15.0, 45.0] {
        for &aspect in &[1.0_f32, 4.0 / 3.0, 3.0 / 4.0, 16.0 / 9.0] {
            let sw = 1200.0;
            let sh = 800.0;
            let r = largest_inscribed_rect(sw, sh, angle, aspect, &IDENTITY);
            assert!(
                crop_rect_inside_warped_source(r, sw, sh, angle, &IDENTITY),
                "angle={angle} aspect={aspect} rect={r:?}"
            );
        }
    }
}

#[test]
fn inscribed_inside_warped_source() {
    let p = crate::perspective::PerspectiveEdits {
        vertical: 40.0,
        horizontal: -20.0,
        ..Default::default()
    };
    let inv = p.inverse();
    for &angle in &[0.0_f32, 8.0, -12.0] {
        let sw = 1200.0;
        let sh = 800.0;
        let r = largest_inscribed_rect(sw, sh, angle, 1.5, &inv);
        assert!(r.w > 0.1 && r.h > 0.1, "angle={angle} rect={r:?}");
        assert!(
            crop_rect_inside_warped_source(r, sw, sh, angle, &inv),
            "angle={angle} rect={r:?}"
        );
    }
}

#[test]
fn aspect_resolves() {
    assert_eq!(aspect_ratio_for(AspectLock::Free, 100.0, 50.0), None);
    assert_eq!(
        aspect_ratio_for(AspectLock::Original, 100.0, 50.0),
        Some(2.0)
    );
    assert_eq!(
        aspect_ratio_for(AspectLock::Ratio { num: 16, den: 9 }, 100.0, 50.0),
        Some(16.0 / 9.0)
    );
}

fn xform(rot: u16, flip_h: bool, flip_v: bool, angle: f32, crop: CropRect) -> GeometryTransform {
    xform_with(rot, flip_h, flip_v, angle, crop, IDENTITY, IDENTITY)
}

fn xform_with(
    rot: u16,
    flip_h: bool,
    flip_v: bool,
    angle: f32,
    crop: CropRect,
    persp_fwd: Mat3,
    persp_inv: Mat3,
) -> GeometryTransform {
    let (iw, ih) = (1200u32, 800u32);
    let (ow, oh) = match rot {
        90 | 270 => (ih, iw),
        _ => (iw, ih),
    };
    let bbox = rotated_bbox(ow as f32, oh as f32, angle);
    let out_w = (crop.w * bbox.w).round().max(1.0) as u32;
    let out_h = (crop.h * bbox.h).round().max(1.0) as u32;
    GeometryTransform {
        input_w: iw,
        input_h: ih,
        rotate_quarter: rot,
        flip_h,
        flip_v,
        angle_deg: angle,
        crop,
        perspective_forward: persp_fwd,
        perspective_inverse: persp_inv,
        output_w: out_w,
        output_h: out_h,
    }
}

#[test]
fn geom_identity_passthrough() {
    let t = xform(0, false, false, 0.0, CropRect::full());
    let uv = [0.37, 0.81];
    let m = display_uv_to_mask_uv(&t, uv);
    assert!((m[0] - uv[0]).abs() < 1e-7 && (m[1] - uv[1]).abs() < 1e-7);
}

#[test]
fn geom_round_trip() {
    let crops = [
        CropRect::full(),
        CropRect {
            x: 0.1,
            y: 0.15,
            w: 0.7,
            h: 0.6,
        },
    ];
    for &rot in &[0u16, 90, 180, 270] {
        for &flip_h in &[false, true] {
            for &flip_v in &[false, true] {
                for &angle in &[0.0f32, 5.0, -7.5] {
                    for &crop in &crops {
                        let t = xform(rot, flip_h, flip_v, angle, crop);
                        for uv in [[0.1, 0.2], [0.5, 0.5], [0.85, 0.9]] {
                            let m = display_uv_to_mask_uv(&t, uv);
                            let d = mask_uv_to_display_uv(&t, m);
                            if (d[0] - uv[0]).abs() > 1e-4 || (d[1] - uv[1]).abs() > 1e-4 {
                                panic!(
                                    "round trip rot={rot} fh={flip_h} fv={flip_v} a={angle} crop={crop:?} uv={uv:?} back={d:?} mask={m:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn geom_round_trip_with_perspective() {
    let p = crate::perspective::PerspectiveEdits {
        vertical: 55.0,
        horizontal: -30.0,
        aspect: 20.0,
        ..Default::default()
    };
    for &rot in &[0u16, 90, 270] {
        for &angle in &[0.0f32, 6.0] {
            let t = xform_with(
                rot,
                false,
                true,
                angle,
                CropRect {
                    x: 0.1,
                    y: 0.05,
                    w: 0.75,
                    h: 0.8,
                },
                p.forward(),
                p.inverse(),
            );
            assert!(!t.is_identity());
            for uv in [[0.2, 0.3], [0.5, 0.5], [0.9, 0.7]] {
                let m = display_uv_to_mask_uv(&t, uv);
                let d = mask_uv_to_display_uv(&t, m);
                assert!(
                    (d[0] - uv[0]).abs() < 1e-3 && (d[1] - uv[1]).abs() < 1e-3,
                    "rot={rot} angle={angle} uv={uv:?} back={d:?}"
                );
            }
        }
    }
}
