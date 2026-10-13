use super::proxy::Proxy;
use super::scene::night;
use crate::PipelineResult;
use crate::edits::{Edits, ToneEdits};

const KEY: f64 = 0.55;
const NIGHT_KEY: f64 = 0.44;
const FACE_KEY: f64 = 0.64;
const WHITE: (f64, f64) = (0.9, 0.985);
const BLACK: (f64, f64) = (0.05, 0.16);
const NIGHT_BLACK: (f64, f64) = (0.03, 0.12);
const HIGHLIGHT: f64 = 0.86;
const SHADOW: f64 = 0.27;
const NIGHT_SHADOW: f64 = 0.17;
const SPREAD: (f64, f64) = (0.18, 0.45);
const CLIP_BUDGET: f64 = 0.015;
const CLIP_PULLBACK: f64 = 0.75;
const CLIP_TOL: f64 = 0.001;
const CHROMA_GOAL: f64 = 0.055;
const CHROMA_LIFT: f64 = 0.5;
const CHROMA_GAIN: f64 = 1.35;
const CHROMA_CEILING: f64 = 0.1;
const LOCAL: f64 = 0.03;
const HAZE_FLOOR: f64 = 0.05;
const HAZE_GAIN: f64 = 300.0;
const DEHAZE_MAX: f64 = 30.0;
const CLARITY_MAX: f64 = 20.0;
const NIGHT_CLARITY_MAX: f64 = 10.0;
const FACE_CLARITY_MAX: f64 = 8.0;
const SWEEPS: usize = 2;
const MAX_BRACKET: usize = 4;
const MAX_REFINE: usize = 4;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Sliders {
    pub(super) exposure: f64,
    pub(super) contrast: f64,
    pub(super) highlights: f64,
    pub(super) shadows: f64,
    pub(super) whites: f64,
    pub(super) blacks: f64,
    pub(super) dehaze: f64,
    pub(super) clarity: f64,
    pub(super) vibrance: f64,
    pub(super) saturation: f64,
}

impl Sliders {
    pub(super) fn edits(&self, base: &Edits, colour: bool) -> Edits {
        let mut edits = base.clone();
        self.write(&mut edits, colour);
        edits
    }

    pub(super) fn write(&self, edits: &mut Edits, colour: bool) {
        edits.basic.exposure_ev = self.exposure;
        edits.basic.brightness = 0.0;
        edits.basic.contrast = self.contrast;
        edits.basic.dehaze = self.dehaze;
        edits.basic.clarity = self.clarity;
        if colour {
            edits.basic.vibrance = self.vibrance;
            edits.basic.saturation = self.saturation;
        }
        edits.tone = ToneEdits {
            highlights: self.highlights,
            shadows: self.shadows,
            whites: self.whites,
            blacks: self.blacks,
        };
    }

    fn rounded(self) -> Self {
        Self {
            exposure: (self.exposure * 100.0).round() / 100.0,
            contrast: self.contrast.round(),
            highlights: self.highlights.round(),
            shadows: self.shadows.round(),
            whites: self.whites.round(),
            blacks: self.blacks.round(),
            dehaze: self.dehaze.round(),
            clarity: self.clarity.round(),
            vibrance: self.vibrance.round(),
            saturation: self.saturation.round(),
        }
    }
}

#[derive(Clone, Copy)]
struct Knob {
    field: fn(&mut Sliders) -> &mut f64,
    lo: f64,
    hi: f64,
    step: f64,
    tol: f64,
}

const EXPOSURE: Knob = Knob {
    field: |s| &mut s.exposure,
    lo: -4.0,
    hi: 4.0,
    step: 0.5,
    tol: 0.004,
};
const WHITES: Knob = Knob {
    field: |s| &mut s.whites,
    lo: -30.0,
    hi: 50.0,
    step: 15.0,
    tol: 0.006,
};
const BLACKS: Knob = Knob {
    field: |s| &mut s.blacks,
    lo: -60.0,
    hi: 20.0,
    step: 15.0,
    tol: 0.006,
};
const HIGHLIGHTS: Knob = Knob {
    field: |s| &mut s.highlights,
    lo: -100.0,
    hi: 0.0,
    step: 25.0,
    tol: 0.008,
};
const SHADOWS: Knob = Knob {
    field: |s| &mut s.shadows,
    lo: 0.0,
    hi: 70.0,
    step: 20.0,
    tol: 0.008,
};
const CONTRAST: Knob = Knob {
    field: |s| &mut s.contrast,
    lo: -20.0,
    hi: 30.0,
    step: 10.0,
    tol: 0.006,
};
const CLARITY: Knob = Knob {
    field: |s| &mut s.clarity,
    lo: 0.0,
    hi: CLARITY_MAX,
    step: 10.0,
    tol: 0.002,
};
const VIBRANCE: Knob = Knob {
    field: |s| &mut s.vibrance,
    lo: -20.0,
    hi: 30.0,
    step: 15.0,
    tol: 0.001,
};
const SATURATION: Knob = Knob {
    field: |s| &mut s.saturation,
    lo: 0.0,
    hi: 10.0,
    step: 5.0,
    tol: 0.003,
};

struct Targets {
    black: (f64, f64),
    shadow: f64,
}

pub(super) fn solve(proxy: &mut Proxy, exposure_value: Option<f64>) -> PipelineResult<Sliders> {
    let mut sliders = Sliders::default();
    let face_mass = proxy.face_mass();
    let key = |scene: f64| scene + (FACE_KEY - scene) * face_mass;
    let budget = proxy.clipped_share(&sliders)? + CLIP_BUDGET;
    settle(proxy, &mut sliders, EXPOSURE, key(KEY), |p, s| {
        Ok(p.metrics(s)?.key)
    })?;
    let night = night(exposure_value.map(|ev| ev - sliders.exposure));
    settle(
        proxy,
        &mut sliders,
        EXPOSURE,
        key(lerp(KEY, NIGHT_KEY, night)),
        |p, s| Ok(p.metrics(s)?.key),
    )?;
    if proxy.clipped_share(&sliders)? > budget {
        let capped = Knob {
            lo: (sliders.exposure - CLIP_PULLBACK).min(0.0),
            hi: sliders.exposure,
            tol: CLIP_TOL,
            ..EXPOSURE
        };
        settle(proxy, &mut sliders, capped, budget, |p, s| {
            p.clipped_share(s)
        })?;
    }
    let targets = Targets {
        black: (
            lerp(BLACK.0, NIGHT_BLACK.0, night),
            lerp(BLACK.1, NIGHT_BLACK.1, night),
        ),
        shadow: lerp(SHADOW, NIGHT_SHADOW, night),
    };
    proxy.exclude_clipped(&sliders)?;
    let haze = proxy.haze(&sliders)?;
    sliders.dehaze = ((haze - HAZE_FLOOR) * HAZE_GAIN).clamp(0.0, DEHAZE_MAX);
    for _ in 0..SWEEPS {
        tone(proxy, &mut sliders, &targets)?;
    }
    let clarity_max = if face_mass > 0.0 {
        FACE_CLARITY_MAX
    } else {
        lerp(CLARITY_MAX, NIGHT_CLARITY_MAX, night)
    };
    let clarity = Knob {
        hi: clarity_max,
        ..CLARITY
    };
    settle(proxy, &mut sliders, clarity, LOCAL, |p, s| {
        p.local_contrast(s)
    })?;
    if proxy.colour() {
        colour(proxy, &mut sliders)?;
    }
    tone(proxy, &mut sliders, &targets)?;
    Ok(sliders.rounded())
}

fn tone(proxy: &mut Proxy, sliders: &mut Sliders, targets: &Targets) -> PipelineResult<()> {
    fit(proxy, sliders, CONTRAST, SPREAD, |p, s| {
        Ok(p.metrics(s)?.spread)
    })?;
    fit(proxy, sliders, HIGHLIGHTS, (0.0, HIGHLIGHT), |p, s| {
        Ok(p.metrics(s)?.highlight)
    })?;
    fit(proxy, sliders, WHITES, WHITE, |p, s| {
        Ok(p.metrics(s)?.white)
    })?;
    let deeper = Knob {
        hi: sliders.highlights,
        ..HIGHLIGHTS
    };
    let softer = Knob {
        lo: 0.0_f64.min(sliders.contrast),
        hi: sliders.contrast,
        ..CONTRAST
    };
    for knob in [deeper, softer] {
        if proxy.metrics(sliders)?.white <= WHITE.1 {
            break;
        }
        settle(proxy, sliders, knob, WHITE.1, |p, s| {
            Ok(p.metrics(s)?.white)
        })?;
    }
    fit(proxy, sliders, BLACKS, targets.black, |p, s| {
        Ok(p.metrics(s)?.black)
    })?;
    fit(proxy, sliders, SHADOWS, (targets.shadow, 1.0), |p, s| {
        Ok(p.metrics(s)?.shadow)
    })
}

fn colour(proxy: &mut Proxy, sliders: &mut Sliders) -> PipelineResult<()> {
    let chroma = proxy.metrics(sliders)?.chroma;
    let target = (chroma + (CHROMA_GOAL - chroma).max(0.0) * CHROMA_LIFT)
        .min(chroma * CHROMA_GAIN)
        .min(CHROMA_CEILING);
    settle(proxy, sliders, VIBRANCE, target, |p, s| {
        Ok(p.metrics(s)?.chroma)
    })?;
    if sliders.vibrance < VIBRANCE.hi {
        return Ok(());
    }
    settle(proxy, sliders, SATURATION, target, |p, s| {
        Ok(p.metrics(s)?.chroma)
    })
}

fn fit(
    proxy: &mut Proxy,
    sliders: &mut Sliders,
    knob: Knob,
    band: (f64, f64),
    metric: impl Fn(&mut Proxy, &Sliders) -> PipelineResult<f64>,
) -> PipelineResult<()> {
    *(knob.field)(sliders) = 0.0;
    let value = metric(proxy, sliders)?;
    let target = value.clamp(band.0, band.1);
    if target == value {
        return Ok(());
    }
    settle(proxy, sliders, knob, target, metric)
}

fn settle(
    proxy: &mut Proxy,
    sliders: &mut Sliders,
    knob: Knob,
    target: f64,
    metric: impl Fn(&mut Proxy, &Sliders) -> PipelineResult<f64>,
) -> PipelineResult<()> {
    let start = *sliders;
    let x0 = *(knob.field)(sliders);
    let solved = root(knob, x0, |x| {
        let mut probe = start;
        *(knob.field)(&mut probe) = x;
        Ok(metric(proxy, &probe)? - target)
    })?;
    *(knob.field)(sliders) = solved;
    Ok(())
}

fn root(knob: Knob, x0: f64, mut g: impl FnMut(f64) -> PipelineResult<f64>) -> PipelineResult<f64> {
    let mut a = x0.clamp(knob.lo, knob.hi);
    let mut ga = g(a)?;
    if ga.abs() <= knob.tol {
        return Ok(a);
    }
    let dir = -ga.signum();
    let mut step = knob.step;
    let mut b = a;
    let mut gb = ga;
    for _ in 0..MAX_BRACKET {
        b = (a + dir * step).clamp(knob.lo, knob.hi);
        if b == a {
            return Ok(a);
        }
        gb = g(b)?;
        if gb.abs() <= knob.tol || gb.signum() != ga.signum() {
            break;
        }
        a = b;
        ga = gb;
        step *= 2.0;
    }
    if gb.abs() <= knob.tol || gb.signum() == ga.signum() {
        return Ok(b);
    }
    for _ in 0..MAX_REFINE {
        let x = b - gb * (b - a) / (gb - ga);
        let gx = g(x)?;
        if gx.abs() <= knob.tol {
            return Ok(x);
        }
        if gx.signum() == gb.signum() {
            ga *= 0.5;
        } else {
            a = b;
            ga = gb;
        }
        b = x;
        gb = gx;
    }
    Ok(b)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
