use std::mem::{offset_of, size_of, size_of_val};

use naga::{AddressSpace, Handle, Module, ScalarKind, Type, TypeInner};

use super::display_depth::DisplayDepth;
use super::helpers::{DemosaicParams, SuperpixelParams, XtransParams};
use super::passes::capture_sharpen::{
    CAPTURE_APPLY_WGSL, CAPTURE_BLUR_WGSL, CAPTURE_LUMA_WGSL, CaptureApplyParams,
    CaptureBlurParams, CaptureLumaParams,
};
use super::passes::dcp_huesat::{DcpHueSatParams, dcp_huesat_wgsl};
use super::passes::dehaze::{
    AB_WGSL, APPLY_WGSL, BOX_WGSL, DOWNSAMPLE_WGSL, DehazeApplyParams, DehazeDownsampleParams,
    DehazeFilterParams, DehazeNormParams, DehazeSizeParams, MIN_WGSL, NORM_WGSL, PACK_WGSL,
};
use super::passes::demosaic::{DEMOSAIC_WGSL, SUPERPIXEL_WGSL};
use super::passes::effects_tone::{EffectsToneParams, effects_tone_wgsl};
use super::passes::lut::{LutParams, lut_wgsl};
use super::passes::mask_blend::{MASK_BLEND_WGSL, MaskBlendParams};
use super::passes::mask_overlay::{MASK_OVERLAY_WGSL, MaskOverlayParams};
use super::passes::mask_weight::{MaskComponent, MaskWeightParams, mask_weight_wgsl};
use super::passes::meta_bins::{BinParams, histogram_wgsl, scopes_wgsl};
use super::passes::nr::{NR_WGSL, NrParams};
use super::passes::nr_smooth::{NR_SMOOTH_WGSL, NrSmoothParams};
use super::passes::presence::{PresenceParams, presence_adjust_wgsl};
use super::passes::resample::{RESAMPLE_WGSL, ResampleParams};
use super::passes::retouch::{RETOUCH_APPLY_WGSL, RETOUCH_PREP_WGSL, RetouchParams};
use super::passes::sensor::{SENSOR_WGSL, SensorParams};
use super::passes::sharpen::{SHARPEN_BLUR_WGSL, SHARPEN_WGSL, SharpenBlurParams, SharpenParams};
use super::passes::xtrans::{XTRANS_GREEN_WGSL, XTRANS_RGB_WGSL};
use super::shader_builder::{self, BuiltProcessShader, StageMask};
use super::uniforms::ProcessHeader;
use crate::edits::CurvesEdits;
use crate::ops::curves::{DISPLAY_CURVES_UNIFORM_SIZE, display_curves_uniform};
use crate::ops::default_registry;

#[derive(Debug, PartialEq)]
struct Member {
    name: String,
    offset: u32,
    size: u32,
    kind: ScalarKind,
}

#[derive(Debug, PartialEq)]
struct Layout {
    size: u32,
    members: Vec<Member>,
}

trait Scalar {
    const KIND: ScalarKind;
}

impl Scalar for f32 {
    const KIND: ScalarKind = ScalarKind::Float;
}

impl Scalar for u32 {
    const KIND: ScalarKind = ScalarKind::Uint;
}

impl Scalar for i32 {
    const KIND: ScalarKind = ScalarKind::Sint;
}

impl<T: Scalar, const N: usize> Scalar for [T; N] {
    const KIND: ScalarKind = T::KIND;
}

fn kind_of<T: Scalar>(_: &T) -> ScalarKind {
    T::KIND
}

macro_rules! layout {
    ($ty:ident { $($field:ident),* $(,)? } pad { $($pad:ident),* $(,)? }) => {{
        let value: $ty = bytemuck::Zeroable::zeroed();
        let $ty { $($field: _,)* $($pad: _,)* } = value;
        Layout {
            size: size_of::<$ty>() as u32,
            members: vec![$(Member {
                name: stringify!($field).to_owned(),
                offset: offset_of!($ty, $field) as u32,
                size: size_of_val(&value.$field) as u32,
                kind: kind_of(&value.$field),
            }),*],
        }
    }};
}

fn parse(src: &str) -> Module {
    naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)))
}

fn scalar_kind(module: &Module, ty: Handle<Type>) -> ScalarKind {
    match &module.types[ty].inner {
        TypeInner::Scalar(scalar) => scalar.kind,
        TypeInner::Vector { scalar, .. } => scalar.kind,
        TypeInner::Array { base, .. } => scalar_kind(module, *base),
        other => panic!("unsupported uniform member type {other:?}"),
    }
}

fn struct_layout(module: &Module, ty: Handle<Type>) -> Layout {
    let TypeInner::Struct { members, span } = &module.types[ty].inner else {
        panic!("{:?} is not a struct", module.types[ty].name);
    };
    Layout {
        size: span.next_multiple_of(16),
        members: members
            .iter()
            .map(|m| Member {
                name: m.name.clone().unwrap_or_default(),
                offset: m.offset,
                size: module.types[m.ty].inner.size(module.to_ctx()),
                kind: scalar_kind(module, m.ty),
            })
            .collect(),
    }
}

fn uniform_struct(src: &str) -> Layout {
    let module = parse(src);
    let Some(ty) = module
        .global_variables
        .iter()
        .find_map(|(_, var)| (var.space == AddressSpace::Uniform).then_some(var.ty))
    else {
        panic!("shader declares no uniform");
    };
    struct_layout(&module, ty)
}

fn named_struct(src: &str, name: &str) -> Layout {
    let module = parse(src);
    let Some((ty, _)) = module
        .types
        .iter()
        .find(|(_, t)| t.name.as_deref() == Some(name))
    else {
        panic!("shader declares no struct {name}");
    };
    struct_layout(&module, ty)
}

#[test]
fn uniform_structs_mirror_wgsl() {
    let depth = DisplayDepth::Eight;
    let filter = || layout!(DehazeFilterParams { size, radius, axis } pad {});
    let size_only = || layout!(DehazeSizeParams { size } pad { _pad });
    let xtrans = || layout!(XtransParams { size, pattern } pad { _pad });
    let bins = || layout!(BinParams { size, step } pad { _pad });
    let retouch = || {
        layout!(RetouchParams {
            dims,
            bbox_origin,
            bbox_size,
            point_count,
            clone_mode,
            offset,
            radius_px,
            hardness,
            opacity,
        } pad { _pad })
    };
    let cases = [
        (
            "capture_luma",
            layout!(CaptureLumaParams { size } pad { _pad }),
            uniform_struct(CAPTURE_LUMA_WGSL),
        ),
        (
            "capture_blur",
            layout!(CaptureBlurParams { size, radius, axis, mode, kernel } pad { _pad }),
            uniform_struct(CAPTURE_BLUR_WGSL),
        ),
        (
            "capture_apply",
            layout!(CaptureApplyParams { size, radius } pad { _pad }),
            uniform_struct(CAPTURE_APPLY_WGSL),
        ),
        (
            "dcp_huesat",
            layout!(DcpHueSatParams { dims, to_pp, from_pp, flags, tone_lut } pad {}),
            uniform_struct(&dcp_huesat_wgsl(wgpu::TextureFormat::Rgba16Float)),
        ),
        (
            "dehaze_downsample",
            layout!(DehazeDownsampleParams { size, scale } pad { _pad }),
            uniform_struct(DOWNSAMPLE_WGSL),
        ),
        (
            "dehaze_norm",
            layout!(DehazeNormParams { size, atmosphere } pad { _pad }),
            uniform_struct(NORM_WGSL),
        ),
        ("dehaze_min", filter(), uniform_struct(MIN_WGSL)),
        ("dehaze_pack", size_only(), uniform_struct(PACK_WGSL)),
        ("dehaze_box", filter(), uniform_struct(BOX_WGSL)),
        ("dehaze_ab", size_only(), uniform_struct(AB_WGSL)),
        (
            "dehaze_apply",
            layout!(DehazeApplyParams { size, lo_size, atmosphere, amount, scale } pad { _pad }),
            uniform_struct(APPLY_WGSL),
        ),
        (
            "demosaic",
            layout!(DemosaicParams { size, cfa } pad { _pad }),
            uniform_struct(DEMOSAIC_WGSL),
        ),
        (
            "superpixel",
            layout!(SuperpixelParams { size, block, period, pattern } pad {}),
            uniform_struct(SUPERPIXEL_WGSL),
        ),
        ("xtrans_green", xtrans(), uniform_struct(XTRANS_GREEN_WGSL)),
        ("xtrans_rgb", xtrans(), uniform_struct(XTRANS_RGB_WGSL)),
        (
            "effects_tone",
            layout!(EffectsToneParams {
                size,
                vignette,
                grain,
                display_p3,
                warn_flags,
                output_scale,
                roi,
            } pad { _pad0, _pad1 }),
            uniform_struct(&effects_tone_wgsl(depth)),
        ),
        (
            "lut",
            layout!(LutParams { size, lut_size, domain_min, domain_max, amount } pad { _pad0, _pad1 }),
            uniform_struct(&lut_wgsl(depth)),
        ),
        (
            "mask_blend",
            layout!(MaskBlendParams { out_size, sharpen_delta, sharpen_flags } pad {}),
            uniform_struct(MASK_BLEND_WGSL),
        ),
        (
            "mask_overlay",
            layout!(MaskOverlayParams { out_size, strength } pad { _pad }),
            uniform_struct(MASK_OVERLAY_WGSL),
        ),
        (
            "mask_weight",
            layout!(MaskWeightParams {
                out_size,
                n_components,
                layer_amount,
                crop,
                flags,
                geom_extra2,
                geom_extra3,
                lens,
                perspective,
            } pad {}),
            uniform_struct(&mask_weight_wgsl()),
        ),
        (
            "mask_component",
            layout!(MaskComponent { kind, mode, invert, slot, geom_a, geom_b } pad {}),
            named_struct(&mask_weight_wgsl(), "Component"),
        ),
        ("histogram", bins(), uniform_struct(&histogram_wgsl(depth))),
        ("scopes", bins(), uniform_struct(&scopes_wgsl(depth))),
        (
            "nr",
            layout!(NrParams {
                size,
                radius,
                stage,
                inv_2ss,
                inv_2sr_luma,
                inv_2sr_chroma,
                alpha_luma,
                alpha_chroma,
                contrast,
            } pad { _pad }),
            uniform_struct(NR_WGSL),
        ),
        (
            "nr_smooth",
            layout!(NrSmoothParams { size, smoothness, alpha_chroma } pad {}),
            uniform_struct(NR_SMOOTH_WGSL),
        ),
        (
            "presence_adjust",
            layout!(PresenceParams { size, amounts, mips } pad { _pad0 }),
            uniform_struct(&presence_adjust_wgsl()),
        ),
        (
            "resample",
            layout!(ResampleParams { dst_size, src_size, scale, inv_filter_scale, axis } pad { _pad }),
            uniform_struct(RESAMPLE_WGSL),
        ),
        ("retouch_prep", retouch(), uniform_struct(RETOUCH_PREP_WGSL)),
        (
            "retouch_apply",
            retouch(),
            uniform_struct(RETOUCH_APPLY_WGSL),
        ),
        (
            "sensor",
            layout!(SensorParams { size, zoom, vig_amount, coeffs, ca_vig } pad {}),
            uniform_struct(SENSOR_WGSL),
        ),
        (
            "sharpen_blur",
            layout!(SharpenBlurParams { size, radius, axis, weights } pad {}),
            uniform_struct(SHARPEN_BLUR_WGSL),
        ),
        (
            "sharpen",
            layout!(SharpenParams {
                amount,
                detail_weight,
                masking_thresh,
                masking_softness,
                size,
                use_mask,
                preview_mode,
                masked_sharpen,
            } pad { _pad }),
            uniform_struct(SHARPEN_WGSL),
        ),
    ];
    let failures: Vec<String> = cases
        .iter()
        .filter(|(_, rust, wgsl)| rust != wgsl)
        .map(|(name, rust, wgsl)| format!("{name}\nrust: {rust:#?}\nwgsl: {wgsl:#?}"))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn display_curves_uniform_mirrors_wgsl() {
    let bytes = display_curves_uniform(&CurvesEdits::default()).len() * 4;
    let shaders = [
        effects_tone_wgsl(DisplayDepth::Eight),
        dcp_huesat_wgsl(wgpu::TextureFormat::Rgba8Unorm),
    ];
    for src in &shaders {
        let size = named_struct(src, "DisplayCurves").size as u64;
        if size != DISPLAY_CURVES_UNIFORM_SIZE || size != bytes as u64 {
            panic!("DisplayCurves is {size} bytes, rust writes {bytes}");
        }
    }
}

#[test]
fn process_uniform_mirrors_header_and_op_slots() {
    let registry = default_registry();
    let header = layout!(ProcessHeader {
        src_size,
        out_size,
        crop,
        flags,
        geom_extra,
        active_mask,
        geom_extra2,
        geom_extra3,
        output,
        perspective,
        src_window,
    } pad {});
    let masks = [
        StageMask::fast(),
        StageMask::white_balance(),
        StageMask::tone_color(),
    ];
    let shaders: Vec<BuiltProcessShader> = masks
        .into_iter()
        .map(|mask| shader_builder::build_for(&registry, mask, DisplayDepth::Eight))
        .chain([shader_builder::build_prepare_wb(&registry)])
        .collect();
    for built in &shaders {
        let wgsl = uniform_struct(&built.wgsl);
        assert_eq!(wgsl.size as usize, built.uniform_size);
        assert_eq!(wgsl.members[..header.members.len()], header.members);
        let slots: Vec<(usize, u32)> = built
            .color_ops
            .iter()
            .map(|slot| (slot.uniform_offset, (slot.vec4_count * 16) as u32))
            .collect();
        let fields: Vec<(usize, u32)> = built
            .color_ops
            .iter()
            .map(|slot| {
                let Some(gpu) = registry.ops()[slot.op_index].gpu() else {
                    panic!("color op slot {} has no gpu op", slot.op_index);
                };
                let Some(member) = wgsl.members.iter().find(|m| m.name == gpu.field_name) else {
                    panic!("ProcessParams has no member {}", gpu.field_name);
                };
                (member.offset as usize, member.size)
            })
            .collect();
        assert_eq!(fields, slots);
    }
}
