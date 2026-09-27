export interface HslBand {
  hue: number;
  sat: number;
  lum: number;
}

export const HSL_BANDS = 8;

export const HSL_BAND_NAMES: readonly string[] = [
  'Red',
  'Orange',
  'Yellow',
  'Green',
  'Aqua',
  'Blue',
  'Purple',
  'Magenta'
];

export const HSL_BAND_HUES: readonly number[] = [0, 30, 60, 120, 180, 240, 300, 340];

export const HSL_BAND_COLORS: readonly string[] = HSL_BAND_HUES.map((h) => `hsl(${h}, 70%, 65%)`);

export interface HslEdits {
  bands: HslBand[];
}

export function bandsAllZero(bands: HslBand[]): boolean {
  return bands.every((b) => b.hue === 0 && b.sat === 0 && b.lum === 0);
}

export interface ColorGradeRegion {
  hue: number;
  sat: number;
  lum: number;
}

export interface ColorGradeEdits {
  shadows: ColorGradeRegion;
  midtones: ColorGradeRegion;
  highlights: ColorGradeRegion;
  global: ColorGradeRegion;
  balance: number;
  blend: number;
}

function neutralRegion(): ColorGradeRegion {
  return { hue: 0, sat: 0, lum: 0 };
}

function regionIsZero(r: ColorGradeRegion): boolean {
  return r.sat === 0 && r.lum === 0;
}

export function colorGradeIsZero(cg: ColorGradeEdits): boolean {
  return (
    regionIsZero(cg.shadows) &&
    regionIsZero(cg.midtones) &&
    regionIsZero(cg.highlights) &&
    regionIsZero(cg.global)
  );
}

export interface Lut3dEdits {
  lut_id: string | null;
  amount: number;
}

export function neutralLut3d(): Lut3dEdits {
  return { lut_id: null, amount: 100 };
}

export function lut3dIsActive(l: Lut3dEdits): boolean {
  return !!l.lut_id && l.amount > 0;
}

export type DcpMode = 'off' | 'auto' | 'profile' | 'flat';
export type DcpIlluminant = 'interpolated' | 'first' | 'second';

export interface DcpEdits {
  mode: DcpMode;
  profile_id: string | null;
  illuminant: DcpIlluminant;
  use_tone_curve: boolean;
  use_base_table: boolean;
  use_look_table: boolean;
  use_baseline_exposure: boolean;
}

export function neutralDcp(): DcpEdits {
  return {
    mode: 'auto',
    profile_id: null,
    illuminant: 'interpolated',
    use_tone_curve: true,
    use_base_table: true,
    use_look_table: true,
    use_baseline_exposure: true
  };
}

export function dcpIsDefault(d: DcpEdits): boolean {
  const neutral = neutralDcp();
  return (
    d.mode === neutral.mode &&
    d.profile_id === neutral.profile_id &&
    d.illuminant === neutral.illuminant &&
    d.use_tone_curve === neutral.use_tone_curve &&
    d.use_base_table === neutral.use_base_table &&
    d.use_look_table === neutral.use_look_table &&
    d.use_baseline_exposure === neutral.use_baseline_exposure
  );
}

export const BW_CHANNELS = ['red', 'yellow', 'green', 'aqua', 'blue', 'magenta'] as const;
export type BwChannel = (typeof BW_CHANNELS)[number];

export interface BwTint {
  hue: number;
  sat: number;
}

export interface BwEdits {
  enabled: boolean;
  mix: Record<BwChannel, number>;
  shadows: BwTint;
  highlights: BwTint;
  balance: number;
}

export function neutralBw(): BwEdits {
  return {
    enabled: false,
    mix: { red: 0, yellow: 0, green: 0, aqua: 0, blue: 0, magenta: 0 },
    shadows: { hue: 0, sat: 0 },
    highlights: { hue: 0, sat: 0 },
    balance: 0
  };
}

export function bwIsNeutral(bw: BwEdits): boolean {
  return (
    !bw.enabled &&
    BW_CHANNELS.every((channel) => bw.mix[channel] === 0) &&
    bw.shadows.hue === 0 &&
    bw.shadows.sat === 0 &&
    bw.highlights.hue === 0 &&
    bw.highlights.sat === 0 &&
    bw.balance === 0
  );
}

export interface ColorEdits {
  hsl: HslEdits;
  color_grade: ColorGradeEdits;
  lut_3d: Lut3dEdits;
  dcp: DcpEdits;
  bw: BwEdits;
}

export function neutralColor(): ColorEdits {
  return {
    hsl: { bands: Array.from({ length: HSL_BANDS }, () => ({ hue: 0, sat: 0, lum: 0 })) },
    color_grade: {
      shadows: neutralRegion(),
      midtones: neutralRegion(),
      highlights: neutralRegion(),
      global: neutralRegion(),
      balance: 0,
      blend: 0
    },
    lut_3d: neutralLut3d(),
    dcp: neutralDcp(),
    bw: neutralBw()
  };
}
