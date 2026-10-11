pub const KELVIN_MIN: f64 = 2000.0;
pub const KELVIN_MAX: f64 = 50000.0;
pub const TINT_LIMIT: f64 = 150.0;
const RELATIVE_TEMP_STOPS: f64 = 1.5;
const RELATIVE_TINT_SCALE: f64 = 0.9;
const TINT_SCALE: f64 = -3000.0;

const ISOTHERMS: [[f64; 4]; 31] = [
    [0.0, 0.18006, 0.26352, -0.24341],
    [10.0, 0.18066, 0.26589, -0.25479],
    [20.0, 0.18133, 0.26846, -0.26876],
    [30.0, 0.18208, 0.27119, -0.28539],
    [40.0, 0.18293, 0.27407, -0.30470],
    [50.0, 0.18388, 0.27709, -0.32675],
    [60.0, 0.18494, 0.28021, -0.35156],
    [70.0, 0.18611, 0.28342, -0.37915],
    [80.0, 0.18740, 0.28668, -0.40955],
    [90.0, 0.18880, 0.28997, -0.44278],
    [100.0, 0.19032, 0.29326, -0.47888],
    [125.0, 0.19462, 0.30141, -0.58204],
    [150.0, 0.19962, 0.30921, -0.70471],
    [175.0, 0.20525, 0.31647, -0.84901],
    [200.0, 0.21142, 0.32312, -1.0182],
    [225.0, 0.21807, 0.32909, -1.2168],
    [250.0, 0.22511, 0.33439, -1.4512],
    [275.0, 0.23247, 0.33904, -1.7298],
    [300.0, 0.24010, 0.34308, -2.0637],
    [325.0, 0.24702, 0.34655, -2.4681],
    [350.0, 0.25591, 0.34951, -2.9641],
    [375.0, 0.26400, 0.35200, -3.5814],
    [400.0, 0.27218, 0.35407, -4.3633],
    [425.0, 0.28039, 0.35577, -5.3762],
    [450.0, 0.28863, 0.35714, -6.7262],
    [475.0, 0.29685, 0.35823, -8.5955],
    [500.0, 0.30505, 0.35907, -11.324],
    [525.0, 0.31320, 0.35968, -15.628],
    [550.0, 0.32129, 0.36011, -23.325],
    [575.0, 0.32931, 0.36038, -40.770],
    [600.0, 0.33724, 0.36051, -116.45],
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    pub kelvin: f64,
    pub tint: f64,
}

impl Temperature {
    pub fn from_xy([x, y]: [f64; 2]) -> Self {
        let d = 1.5 - x + 6.0 * y;
        let u = 2.0 * x / d;
        let v = 3.0 * y / d;
        let distance = |i: usize| {
            let [_, iu, iv, slope] = ISOTHERMS[i];
            let [du, dv] = unit_slope(slope);
            [-(u - iu) * dv + (v - iv) * du, du, dv]
        };
        let last_index = ISOTHERMS.len() - 1;
        let index = (1..last_index)
            .find(|&i| distance(i)[0] <= 0.0)
            .unwrap_or(last_index);
        let [dt, du, dv] = distance(index);
        let [last_dt, last_du, last_dv] = distance(index - 1);
        let dt = -dt.min(0.0);
        let f = if index == 1 { 0.0 } else { dt / (last_dt + dt) };
        let [m0, u0, v0, _] = ISOTHERMS[index - 1];
        let [m1, u1, v1, _] = ISOTHERMS[index];
        let uu = u - (u0 * f + u1 * (1.0 - f));
        let vv = v - (v0 * f + v1 * (1.0 - f));
        let du = du * (1.0 - f) + last_du * f;
        let dv = dv * (1.0 - f) + last_dv * f;
        Self {
            kelvin: 1.0e6 / (m0 * f + m1 * (1.0 - f)),
            tint: (uu * du + vv * dv) / du.hypot(dv) * TINT_SCALE,
        }
    }

    pub fn xy(self) -> [f64; 2] {
        let mired = 1.0e6 / self.kelvin;
        let offset = self.tint / TINT_SCALE;
        let index = (0..ISOTHERMS.len() - 2)
            .find(|&i| mired < ISOTHERMS[i + 1][0])
            .unwrap_or(ISOTHERMS.len() - 2);
        let [m0, u0, v0, s0] = ISOTHERMS[index];
        let [m1, u1, v1, s1] = ISOTHERMS[index + 1];
        let f = (m1 - mired) / (m1 - m0);
        let a = unit_slope(s0);
        let b = unit_slope(s1);
        let du = a[0] * f + b[0] * (1.0 - f);
        let dv = a[1] * f + b[1] * (1.0 - f);
        let len = du.hypot(dv);
        let u = u0 * f + u1 * (1.0 - f) + du / len * offset;
        let v = v0 * f + v1 * (1.0 - f) + dv / len * offset;
        let d = u - 4.0 * v + 2.0;
        [1.5 * u / d, v / d]
    }

    pub fn clamped(self) -> Self {
        Self {
            kelvin: self.kelvin.clamp(KELVIN_MIN, KELVIN_MAX),
            tint: self.tint.clamp(-TINT_LIMIT, TINT_LIMIT),
        }
    }

    pub fn shifted(self, temp: f64, tint: f64) -> Self {
        let mired = 1.0e6 / self.kelvin * (-temp * RELATIVE_TEMP_STOPS / 100.0).exp2();
        Self {
            kelvin: 1.0e6 / mired,
            tint: self.tint + tint * RELATIVE_TINT_SCALE,
        }
        .clamped()
    }
}

fn unit_slope(slope: f64) -> [f64; 2] {
    let len = (1.0 + slope * slope).sqrt();
    [1.0 / len, slope / len]
}

#[cfg(test)]
mod tests;
