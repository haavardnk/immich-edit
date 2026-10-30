use super::GpuRenderer;
use super::display::{OutputShape, StageState};
use super::geometry::ProcessGeom;
use crate::edits::Edits;
use crate::gpu::shader_builder::BuiltProcessShader;
use crate::gpu::uniform_pool::PooledUniform;
use crate::gpu::uniforms::{ProcessHeader, write_active_mask, write_header};
use crate::ops::{OpContext, OpRegistry};

#[cfg(test)]
mod tests;

impl GpuRenderer {
    pub(super) fn uniform<T: bytemuck::Pod>(
        &self,
        value: &T,
        label: &'static str,
    ) -> PooledUniform {
        self.uniform_pool.acquire(
            &self.ctx.device,
            &self.ctx.queue,
            bytemuck::bytes_of(value),
            label,
        )
    }
}

pub(super) fn header_uniform(
    edits: &Edits,
    geom: &ProcessGeom,
    state: &StageState,
    shape: &OutputShape,
    warp: bool,
) -> ProcessHeader {
    let (src_w, src_h) = state.extent.dims;
    let shadows = state.shadows.as_ref();
    let shadows_origin_level = shadows.map_or([0; 4], |s| [s.origin.0, s.origin.1, s.level, 0]);
    ProcessHeader {
        src_size: [src_w, src_h],
        out_size: [shape.full.0, shape.full.1],
        crop: [geom.crop.x, geom.crop.y, geom.crop.w, geom.crop.h],
        flags: [
            edits.geometry.rotate as u32,
            edits.geometry.flip_h as u32,
            edits.geometry.flip_v as u32,
            geom.orient_packed,
        ],
        shadows_origin_level,
        active_mask: [0; 4],
        geom_extra2: [geom.cos_a, geom.sin_a, geom.bw, geom.bh],
        geom_extra3: [
            geom.oriented.0 as f32,
            geom.oriented.1 as f32,
            if warp && geom.geom_warps { 1.0 } else { 0.0 },
            0.0,
        ],
        output: shape.output(),
        perspective: geom.persp_rows,
        src_window: state.window(),
        shadows_window: shadows.map_or([0; 4], |s| s.window),
        shadows_blur_origin: shadows.map_or([0; 4], |s| [s.blur_origin.0, s.blur_origin.1, 0, 0]),
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
