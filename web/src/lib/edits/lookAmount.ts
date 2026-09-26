import {
  BW_CHANNELS,
  neutralEdits,
  type BasicEdits,
  type BwEdits,
  type BwTint,
  type ColorEdits,
  type ColorGradeRegion,
  type CurvePoint,
  type DetailEdits,
  type Edits,
  type EffectsEdits,
  type ToneEdits
} from '$lib/types/edits';

export const LOOK_AMOUNT_FULL = 100;
export const LOOK_AMOUNT_MAX = 200;

function lerp(neutral: number, preset: number, t: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, neutral + (preset - neutral) * t));
}

function signed(neutral: number, preset: number, t: number): number {
  return lerp(neutral, preset, t, -100, 100);
}

function unsigned(neutral: number, preset: number, t: number): number {
  return lerp(neutral, preset, t, 0, 100);
}

function scaleCurve(points: CurvePoint[], t: number): CurvePoint[] {
  return points.map((p) => ({ x: p.x, y: Math.min(1, Math.max(0, p.x + (p.y - p.x) * t)) }));
}

function scaleBasic(p: BasicEdits, n: BasicEdits, t: number): BasicEdits {
  return {
    exposure_ev: lerp(n.exposure_ev, p.exposure_ev, t, -5, 5),
    brightness: signed(n.brightness, p.brightness, t),
    contrast: signed(n.contrast, p.contrast, t),
    saturation: signed(n.saturation, p.saturation, t),
    vibrance: signed(n.vibrance, p.vibrance, t),
    wb_temp: signed(n.wb_temp, p.wb_temp, t),
    wb_tint: signed(n.wb_tint, p.wb_tint, t),
    texture: signed(n.texture, p.texture, t),
    clarity: signed(n.clarity, p.clarity, t),
    dehaze: signed(n.dehaze, p.dehaze, t),
    curves: {
      composite: scaleCurve(p.curves.composite, t),
      r: scaleCurve(p.curves.r, t),
      g: scaleCurve(p.curves.g, t),
      b: scaleCurve(p.curves.b, t),
      luma: scaleCurve(p.curves.luma, t)
    }
  };
}

function scaleTone(p: ToneEdits, n: ToneEdits, t: number): ToneEdits {
  return {
    highlights: signed(n.highlights, p.highlights, t),
    shadows: signed(n.shadows, p.shadows, t),
    blacks: signed(n.blacks, p.blacks, t),
    whites: signed(n.whites, p.whites, t)
  };
}

function scaleRegion(p: ColorGradeRegion, n: ColorGradeRegion, t: number): ColorGradeRegion {
  return { hue: p.hue, sat: unsigned(n.sat, p.sat, t), lum: lerp(n.lum, p.lum, t, -50, 50) };
}

function scaleTint(p: BwTint, t: number): BwTint {
  return { hue: p.hue, sat: unsigned(0, p.sat, t) };
}

function scaleBw(p: BwEdits, t: number): BwEdits {
  return {
    enabled: p.enabled,
    mix: Object.fromEntries(
      BW_CHANNELS.map((channel) => [channel, signed(0, p.mix[channel], t)])
    ) as BwEdits['mix'],
    shadows: scaleTint(p.shadows, t),
    highlights: scaleTint(p.highlights, t),
    balance: signed(0, p.balance, t)
  };
}

function scaleColor(p: ColorEdits, n: ColorEdits, t: number): ColorEdits {
  const grade = p.color_grade;
  const neutralGrade = n.color_grade;
  return {
    hsl: {
      bands: p.hsl.bands.map((band) => ({
        hue: signed(0, band.hue, t),
        sat: signed(0, band.sat, t),
        lum: signed(0, band.lum, t)
      }))
    },
    color_grade: {
      shadows: scaleRegion(grade.shadows, neutralGrade.shadows, t),
      midtones: scaleRegion(grade.midtones, neutralGrade.midtones, t),
      highlights: scaleRegion(grade.highlights, neutralGrade.highlights, t),
      global: scaleRegion(grade.global, neutralGrade.global, t),
      balance: signed(neutralGrade.balance, grade.balance, t),
      blend: unsigned(neutralGrade.blend, grade.blend, t)
    },
    lut_3d: {
      lut_id: p.lut_3d.lut_id,
      amount: p.lut_3d.lut_id ? unsigned(0, p.lut_3d.amount, t) : p.lut_3d.amount
    },
    dcp: p.dcp,
    bw: scaleBw(p.bw, t)
  };
}

function scaleDetail(p: DetailEdits, n: DetailEdits, t: number): DetailEdits {
  return {
    ...p,
    luma_nr_amount: unsigned(n.luma_nr_amount, p.luma_nr_amount, t),
    luma_nr_detail: unsigned(n.luma_nr_detail, p.luma_nr_detail, t),
    luma_nr_contrast: unsigned(n.luma_nr_contrast, p.luma_nr_contrast, t),
    color_nr_amount: unsigned(n.color_nr_amount, p.color_nr_amount, t),
    color_nr_detail: unsigned(n.color_nr_detail, p.color_nr_detail, t),
    color_nr_smoothness: unsigned(n.color_nr_smoothness, p.color_nr_smoothness, t)
  };
}

function scaleEffects(p: EffectsEdits, n: EffectsEdits, t: number): EffectsEdits {
  return {
    vignette_amount: signed(n.vignette_amount, p.vignette_amount, t),
    vignette_midpoint: unsigned(n.vignette_midpoint, p.vignette_midpoint, t),
    vignette_feather: unsigned(n.vignette_feather, p.vignette_feather, t),
    vignette_roundness: signed(n.vignette_roundness, p.vignette_roundness, t),
    grain_amount: unsigned(n.grain_amount, p.grain_amount, t),
    grain_size: unsigned(n.grain_size, p.grain_size, t),
    grain_roughness: unsigned(n.grain_roughness, p.grain_roughness, t)
  };
}

export function withLookAmount(preset: Edits, amount: number): Edits {
  const t = Math.min(LOOK_AMOUNT_MAX, Math.max(0, amount)) / LOOK_AMOUNT_FULL;
  if (t === 1) return preset;
  const n = neutralEdits();
  if (t === 0) {
    return {
      ...preset,
      basic: n.basic,
      tone: n.tone,
      color: n.color,
      detail: n.detail,
      effects: n.effects,
      lens: n.lens
    };
  }
  return {
    ...preset,
    basic: scaleBasic(preset.basic, n.basic, t),
    tone: scaleTone(preset.tone, n.tone, t),
    color: scaleColor(preset.color, n.color, t),
    detail: scaleDetail(preset.detail, n.detail, t),
    effects: scaleEffects(preset.effects, n.effects, t)
  };
}
