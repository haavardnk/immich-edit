mod flagged;
mod gain;
mod opcodes;
mod read;

use super::mosaic::Mosaic;
use crate::geom::dng_warp::{DngWarp, EmbeddedLens};
use gain::{Gain, Gains, StagedGain};
use opcodes::{BadPixelList, Opcode, OpcodeList};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Default)]
pub(super) struct DngCorrections {
    bad_constant: Option<u32>,
    bad_lists: Vec<(OpcodeList, BadPixelList)>,
    gains: Gains,
    warp: Option<DngWarp>,
}

impl DngCorrections {
    pub(super) fn read(data: &[u8]) -> Self {
        catch_unwind(AssertUnwindSafe(|| read::opcode_blobs(data)))
            .ok()
            .flatten()
            .map(Self::from_blobs)
            .unwrap_or_default()
    }

    fn from_blobs(blobs: [Vec<u8>; 3]) -> Self {
        let lists = [OpcodeList::One, OpcodeList::Two, OpcodeList::Three];
        let mut corrections = Self::default();
        lists
            .into_iter()
            .zip(blobs)
            .flat_map(|(list, blob)| {
                opcodes::parse(&blob, list)
                    .into_iter()
                    .map(move |op| (list, op))
            })
            .for_each(|(list, op)| corrections.add(list, op));
        corrections
    }

    fn add(&mut self, list: OpcodeList, op: Opcode) {
        match (list, op) {
            (OpcodeList::One, Opcode::FixBadPixelsConstant(value)) => {
                self.bad_constant = Some(value);
            }
            (OpcodeList::One | OpcodeList::Two, Opcode::FixBadPixelsList(bad)) => {
                self.bad_lists.push((list, bad));
            }
            (OpcodeList::Two | OpcodeList::Three, Opcode::GainMap(map)) => {
                self.push_gain(list, Gain::Map(map));
            }
            (OpcodeList::Two | OpcodeList::Three, Opcode::FixVignetteRadial(radial)) => {
                self.push_gain(list, Gain::Radial(radial));
            }
            (OpcodeList::Three, Opcode::WarpRectilinear(warp)) if self.warp.is_none() => {
                self.warp = Some(warp);
            }
            (list, op) => {
                tracing::info!(
                    ?list,
                    op = op.name(),
                    "skipping DNG opcode outside its supported stage"
                );
            }
        }
    }

    fn push_gain(&mut self, list: OpcodeList, gain: Gain) {
        let after_warp = list == OpcodeList::Three && self.warp.is_some();
        self.gains.push(StagedGain {
            list,
            after_warp,
            gain,
        });
    }

    pub(super) fn flagged_pixels(&self, raw: &rawler::RawImage) -> Vec<[usize; 2]> {
        flagged::flagged_pixels(self.bad_constant, &self.bad_lists, raw)
    }

    pub(super) fn has_gains(&self) -> bool {
        !self.gains.is_empty()
    }

    pub(super) fn apply_gains(&self, mosaic: &mut Mosaic) {
        self.gains.apply_mosaic(mosaic, self.warp.as_ref());
    }

    pub(super) fn apply_gains_rgb(&self, rgb: &mut [f32], width: usize, height: usize) {
        self.gains.apply_rgb(rgb, width, height, self.warp.as_ref());
    }

    pub(super) fn embedded_lens(&self, gains_applied: bool) -> EmbeddedLens {
        match (&self.warp, gains_applied && self.has_gains()) {
            (Some(warp), _) => EmbeddedLens::Warp(warp.clone()),
            (None, true) => EmbeddedLens::Shading,
            (None, false) => EmbeddedLens::Absent,
        }
    }
}

#[cfg(test)]
mod tests;
