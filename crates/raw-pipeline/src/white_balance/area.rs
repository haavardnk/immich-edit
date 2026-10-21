use super::{CLIP_LEVEL, display_uv_to_sensor_px, mean_rgb, scene_color, solve_neutral};
use crate::dcp::DcpProfile;
use crate::edits::{Edits, WhiteBalance};
use crate::frame::RawFrame;
use crate::sensor_sample::{cfa_block, display_rgb, mosaic_block_rgb, sample_raw_bilinear};

const GRID: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleArea {
    pub u: f32,
    pub v: f32,
    pub w: f32,
    pub h: f32,
}

pub fn sample_white_balance_area(
    frame: &RawFrame,
    edits: &Edits,
    profile: Option<&DcpProfile>,
    area: SampleArea,
) -> Option<WhiteBalance> {
    let (wb, m, white) = scene_color(frame, edits, profile);
    let block =
        (frame.cpp == 1 && !frame.cfa_pattern.is_empty()).then(|| cfa_block(&frame.cfa_pattern));
    let step = |i: usize| (i as f32 + 0.5) / GRID as f32;
    let raws: Vec<[f32; 3]> = (0..GRID)
        .flat_map(|j| (0..GRID).map(move |i| (i, j)))
        .filter_map(|(i, j)| {
            display_uv_to_sensor_px(
                frame,
                edits,
                area.u + area.w * step(i),
                area.v + area.h * step(j),
            )
        })
        .filter_map(|(px, py)| raw_at(frame, block.as_ref(), px, py))
        .collect();
    let usable: Vec<[f32; 3]> = raws
        .iter()
        .filter(|raw| raw.iter().all(|c| *c < CLIP_LEVEL))
        .map(|raw| display_rgb(*raw, wb, m))
        .collect();
    if usable.is_empty() || usable.len() * 2 < GRID * GRID {
        return None;
    }
    let acc = usable.iter().fold([0.0f64; 3], |acc, s| {
        [
            acc[0] + s[0] as f64,
            acc[1] + s[1] as f64,
            acc[2] + s[2] as f64,
        ]
    });
    solve_neutral(&white, mean_rgb(acc, usable.len() as u32))
}

fn raw_at(
    frame: &RawFrame,
    block: Option<&(usize, Vec<usize>)>,
    px: f32,
    py: f32,
) -> Option<[f32; 3]> {
    let Some((dim, table)) = block else {
        return sample_raw_bilinear(frame, px, py);
    };
    let width = frame.meta.width;
    let height = frame.meta.height;
    if width < *dim || height < *dim {
        return None;
    }
    let bx = (px as usize / dim).min(width / dim - 1) * dim;
    let by = (py as usize / dim).min(height / dim - 1) * dim;
    Some(mosaic_block_rgb(&frame.data, width, *dim, table, bx, by))
}
