use super::geometry::ProcessGeom;
use crate::edits::Edits;
use crate::gpu::shader_builder::BuiltProcessShader;
use crate::gpu::uniforms::{ProcessHeader, write_active_mask, write_header};
use crate::ops::{OpContext, OpRegistry};

pub(super) fn process_header(
    edits: &Edits,
    geom: &ProcessGeom,
    sensor: (u32, u32),
    out: (u32, u32),
    shadows_mip_f: f32,
    warp: bool,
) -> ProcessHeader {
    ProcessHeader {
        src_size: [sensor.0, sensor.1],
        out_size: [out.0, out.1],
        crop: [geom.crop.x, geom.crop.y, geom.crop.w, geom.crop.h],
        flags: [
            edits.geometry.rotate as u32,
            edits.geometry.flip_h as u32,
            edits.geometry.flip_v as u32,
            geom.orient_packed,
        ],
        geom_extra: [0.0, shadows_mip_f, 0.0, 0.0],
        active_mask: [0; 4],
        geom_extra2: [geom.cos_a, geom.sin_a, geom.bw, geom.bh],
        geom_extra3: [
            geom.oriented.0 as f32,
            geom.oriented.1 as f32,
            if warp && geom.geom_warps { 1.0 } else { 0.0 },
            0.0,
        ],
        output: [0, 0, 0, 0],
        perspective: geom.persp_rows,
    }
}

pub(super) fn build_process_uniform(
    built: &BuiltProcessShader,
    registry: &OpRegistry,
    edits: &Edits,
    ctx: &OpContext,
    header: &ProcessHeader,
) -> Vec<u8> {
    let mut bytes = vec![0u8; built.uniform_size];
    write_header(&mut bytes, header);
    let mut active_mask: [u32; 4] = [0; 4];
    for slot in &built.color_ops {
        let op = &registry.ops()[slot.op_index];
        if op.is_active(edits) {
            let word = (slot.active_bit / 32) as usize;
            let shift = slot.active_bit % 32;
            active_mask[word] |= 1u32 << shift;
        }
        let mut values = vec![0.0f32; slot.vec4_count * 4];
        op.write_gpu_uniform(edits, ctx, &mut values);
        let off = slot.uniform_offset;
        let len = slot.vec4_count * 16;
        bytes[off..off + len].copy_from_slice(bytemuck::cast_slice(&values));
    }
    write_active_mask(&mut bytes, active_mask);
    bytes
}
