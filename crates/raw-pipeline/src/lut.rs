mod parse;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use crate::color::{display_p3_to_srgb_lin, srgb_lin_to_display_p3};
use crate::frame::OutputColorSpace;
use crate::math::srgb_to_linear;
use crate::tone::gamut::map_to_gamut;
use crate::tone::srgb_oetf_scalar;

pub use parse::LutParseError;

pub const LUT_MIN_SIZE: usize = 2;
pub const LUT_MAX_SIZE: usize = 65;
pub const LUT_1D_MAX_SIZE: usize = 65_536;
pub const LUT_MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Domain {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Domain {
    pub const UNIT: Self = Self {
        min: [0.0; 3],
        max: [1.0; 3],
    };

    fn uniform([min, max]: [f32; 2]) -> Self {
        Self {
            min: [min; 3],
            max: [max; 3],
        }
    }

    fn is_valid(&self) -> bool {
        (0..3).all(|c| self.max[c] - self.min[c] > f32::EPSILON)
    }

    fn position(&self, rgb: [f32; 3], last: f32) -> [f32; 3] {
        std::array::from_fn(|c| {
            let span = self.max[c] - self.min[c];
            ((rgb[c] - self.min[c]) / span).clamp(0.0, 1.0) * last
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lut1d {
    domain: Domain,
    data: Vec<[f32; 3]>,
}

impl Lut1d {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn domain(&self) -> Domain {
        self.domain
    }

    pub fn data(&self) -> &[[f32; 3]] {
        &self.data
    }

    pub fn sample(&self, rgb: [f32; 3]) -> [f32; 3] {
        let n = self.data.len();
        let pos = self.domain.position(rgb, (n - 1) as f32);
        std::array::from_fn(|c| {
            let i = (pos[c] as usize).min(n - 2);
            let lo = self.data[i][c];
            lo + (self.data[i + 1][c] - lo) * (pos[c] - i as f32)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lut3d {
    size: usize,
    domain: Domain,
    data: Vec<[f32; 3]>,
}

impl Lut3d {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn domain(&self) -> Domain {
        self.domain
    }

    pub fn data(&self) -> &[[f32; 3]] {
        &self.data
    }

    #[inline]
    fn at(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        self.data[(b * self.size + g) * self.size + r]
    }

    pub fn sample(&self, rgb: [f32; 3]) -> [f32; 3] {
        let n = self.size;
        let coord = self.domain.position(rgb, (n - 1) as f32);
        let base = coord.map(|v| (v as usize).min(n - 2));
        let hi = base.map(|v| v + 1);
        let fr = coord[0] - base[0] as f32;
        let fg = coord[1] - base[1] as f32;
        let fb = coord[2] - base[2] as f32;

        let c000 = self.at(base[0], base[1], base[2]);
        let c111 = self.at(hi[0], hi[1], hi[2]);
        let (w0, w1, w2) = tetra_weights(fr, fg, fb);
        let (v1, v2) = tetra_corners(self, base, hi, fr, fg, fb);

        std::array::from_fn(|c| {
            c000[c] + w0 * (v1[c] - c000[c]) + w1 * (v2[c] - v1[c]) + w2 * (c111[c] - v2[c])
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CubeLut {
    shaper: Option<Lut1d>,
    cube: Option<Lut3d>,
}

impl CubeLut {
    pub fn parse(source: &[u8]) -> Result<Self, LutParseError> {
        parse::parse(source)
    }

    pub fn shaper(&self) -> Option<&Lut1d> {
        self.shaper.as_ref()
    }

    pub fn cube(&self) -> Option<&Lut3d> {
        self.cube.as_ref()
    }

    pub fn sample(&self, rgb: [f32; 3]) -> [f32; 3] {
        let shaped = self.shaper.as_ref().map_or(rgb, |s| s.sample(rgb));
        self.cube.as_ref().map_or(shaped, |c| c.sample(shaped))
    }

    pub fn apply(&self, display: [f32; 3], amount: f32, cs: OutputColorSpace) -> [f32; 3] {
        let input = to_lut_space(display, cs);
        let graded = self.sample(input);
        let blended =
            std::array::from_fn(|c| (input[c] + amount * (graded[c] - input[c])).clamp(0.0, 1.0));
        from_lut_space(blended, cs)
    }
}

fn to_lut_space(display: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    match cs {
        OutputColorSpace::SRgb => display,
        OutputColorSpace::DisplayP3 => {
            let linear = display_p3_to_srgb_lin(display.map(srgb_to_linear));
            map_to_gamut(linear, OutputColorSpace::SRgb)
                .map(|v| srgb_oetf_scalar(v.clamp(0.0, 1.0)))
        }
    }
}

fn from_lut_space(srgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    match cs {
        OutputColorSpace::SRgb => srgb,
        OutputColorSpace::DisplayP3 => srgb_lin_to_display_p3(srgb.map(srgb_to_linear))
            .map(|v| srgb_oetf_scalar(v.clamp(0.0, 1.0))),
    }
}

fn tetra_weights(fr: f32, fg: f32, fb: f32) -> (f32, f32, f32) {
    if fr >= fg && fg >= fb {
        (fr, fg, fb)
    } else if fr >= fb && fb >= fg {
        (fr, fb, fg)
    } else if fb >= fr && fr >= fg {
        (fb, fr, fg)
    } else if fg >= fr && fr >= fb {
        (fg, fr, fb)
    } else if fg >= fb && fb >= fr {
        (fg, fb, fr)
    } else {
        (fb, fg, fr)
    }
}

fn tetra_corners(
    lut: &Lut3d,
    base: [usize; 3],
    hi: [usize; 3],
    fr: f32,
    fg: f32,
    fb: f32,
) -> ([f32; 3], [f32; 3]) {
    let (b0, b1, b2) = (base[0], base[1], base[2]);
    let (h0, h1, h2) = (hi[0], hi[1], hi[2]);
    if fr >= fg && fg >= fb {
        (lut.at(h0, b1, b2), lut.at(h0, h1, b2))
    } else if fr >= fb && fb >= fg {
        (lut.at(h0, b1, b2), lut.at(h0, b1, h2))
    } else if fb >= fr && fr >= fg {
        (lut.at(b0, b1, h2), lut.at(h0, b1, h2))
    } else if fg >= fr && fr >= fb {
        (lut.at(b0, h1, b2), lut.at(h0, h1, b2))
    } else if fg >= fb && fb >= fr {
        (lut.at(b0, h1, b2), lut.at(b0, h1, h2))
    } else {
        (lut.at(b0, b1, h2), lut.at(b0, h1, h2))
    }
}

pub type LutMap = HashMap<String, Arc<CubeLut>>;

pub fn empty_luts() -> LutMap {
    LutMap::new()
}
