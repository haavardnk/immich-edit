const EXP2_MIN: f32 = -126.0;
const EXP2_MAX: f32 = 127.0;
const ROUND_MAGIC: f32 = 12_582_912.0;
const LN2: f32 = std::f32::consts::LN_2;
const LOG2_E_X2: f32 = 2.0 * std::f32::consts::LOG2_E;
const SQRT_2: f32 = std::f32::consts::SQRT_2;
const MANTISSA_MASK: u32 = 0x007f_ffff;
const ONE_BITS: u32 = 0x3f80_0000;

const EXP2_POLY: [f32; 8] = [
    1.0,
    LN2,
    0.240_226_5,
    0.055_504_11,
    0.009_618_129,
    0.001_333_355_8,
    0.000_154_035_3,
    0.000_015_252_734,
];

#[inline(always)]
pub fn exp2(x: f32) -> f32 {
    let x = x.clamp(EXP2_MIN, EXP2_MAX);
    let n = (x + ROUND_MAGIC) - ROUND_MAGIC;
    let f = x - n;
    let p = EXP2_POLY.iter().rev().fold(0.0, |acc, &c| acc * f + c);
    p * f32::from_bits(((n as i32 + 127) as u32) << 23)
}

#[inline(always)]
pub fn log2(x: f32) -> f32 {
    let bits = x.max(f32::MIN_POSITIVE).to_bits();
    let exponent = (bits >> 23) as i32 - 127;
    let m = f32::from_bits((bits & MANTISSA_MASK) | ONE_BITS);
    let high = m > SQRT_2;
    let m = if high { m * 0.5 } else { m };
    let e = exponent as f32 + if high { 1.0 } else { 0.0 };
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let series = t * (1.0 + t2 * (1.0 / 3.0 + t2 * (1.0 / 5.0 + t2 * (1.0 / 7.0 + t2 / 9.0))));
    e + series * LOG2_E_X2
}

// Negative or NaN x yields NaN, as powf does for a non-integer exponent.
#[inline(always)]
pub fn pow(x: f32, y: f32) -> f32 {
    let r = exp2(y * log2(x));
    if x > 0.0 {
        r
    } else if x == 0.0 {
        if y == 0.0 { 1.0 } else { 0.0 }
    } else {
        f32::NAN
    }
}

#[inline(always)]
pub fn exp(x: f32) -> f32 {
    exp2(x * std::f32::consts::LOG2_E)
}

#[inline(always)]
pub fn tanh(x: f32) -> f32 {
    let e = exp2(x.abs() * LOG2_E_X2);
    (1.0 - 2.0 / (e + 1.0)).copysign(x)
}

#[cfg(test)]
mod tests;
