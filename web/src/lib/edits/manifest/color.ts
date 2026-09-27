import {
  bandsAllZero,
  BW_CHANNELS,
  bwIsNeutral,
  colorGradeIsZero,
  dcpIsDefault,
  HSL_BANDS,
  lut3dIsActive,
  type BwEdits,
  type BwTint,
  type ColorGradeRegion,
  type DcpEdits,
  type Edits,
  type HslBand
} from '$lib/types/edits';

export function encodeColor(edits: Edits, ops: Record<string, unknown>): void {
  if (!bandsAllZero(edits.color.hsl.bands))
    ops.hsl = {
      bands: edits.color.hsl.bands.map((band) => ({
        hue: band.hue,
        sat: band.sat,
        lum: band.lum
      }))
    };
  if (!bwIsNeutral(edits.color.bw)) {
    const bw = edits.color.bw;
    ops.bw = {
      enabled: bw.enabled,
      mix: { ...bw.mix },
      shadows: { ...bw.shadows },
      highlights: { ...bw.highlights },
      balance: bw.balance
    };
  }
  if (!colorGradeIsZero(edits.color.color_grade)) {
    const colorGrade = edits.color.color_grade;
    const region = (value: ColorGradeRegion) => ({
      hue: value.hue,
      sat: value.sat,
      lum: value.lum
    });
    ops.color_grade = {
      shadows: region(colorGrade.shadows),
      midtones: region(colorGrade.midtones),
      highlights: region(colorGrade.highlights),
      global: region(colorGrade.global),
      balance: colorGrade.balance,
      blend: colorGrade.blend
    };
  }
  if (lut3dIsActive(edits.color.lut_3d))
    ops.lut_3d = {
      lut_id: edits.color.lut_3d.lut_id,
      amount: edits.color.lut_3d.amount
    };
  if (!dcpIsDefault(edits.color.dcp))
    ops.dcp_hue_sat = {
      mode: edits.color.dcp.mode,
      profile_id: edits.color.dcp.profile_id,
      illuminant: edits.color.dcp.illuminant,
      use_tone_curve: edits.color.dcp.use_tone_curve,
      use_base_table: edits.color.dcp.use_base_table,
      use_look_table: edits.color.dcp.use_look_table,
      use_baseline_exposure: edits.color.dcp.use_baseline_exposure
    };
}

export function decodeColor(ops: Record<string, unknown>, edits: Edits): void {
  const hsl = ops.hsl as { bands?: HslBand[] } | undefined;
  if (hsl?.bands) {
    for (let index = 0; index < HSL_BANDS && index < hsl.bands.length; index++) {
      const band = hsl.bands[index];
      const target = edits.color.hsl.bands[index];
      if (!band || !target) continue;
      if (band.hue !== undefined) target.hue = band.hue;
      if (band.sat !== undefined) target.sat = band.sat;
      if (band.lum !== undefined) target.lum = band.lum;
    }
  }
  const bw = ops.bw as
    | (Partial<Omit<BwEdits, 'mix' | 'shadows' | 'highlights'>> & {
        mix?: Partial<BwEdits['mix']>;
        shadows?: Partial<BwTint>;
        highlights?: Partial<BwTint>;
      })
    | undefined;
  if (bw) {
    const target = edits.color.bw;
    if (typeof bw.enabled === 'boolean') target.enabled = bw.enabled;
    for (const channel of BW_CHANNELS) {
      const value = bw.mix?.[channel];
      if (typeof value === 'number') target.mix[channel] = value;
    }
    for (const region of ['shadows', 'highlights'] as const) {
      const source = bw[region];
      if (typeof source?.hue === 'number') target[region].hue = source.hue;
      if (typeof source?.sat === 'number') target[region].sat = source.sat;
    }
    if (typeof bw.balance === 'number') target.balance = bw.balance;
  }
  const colorGrade = ops.color_grade as
    | {
        shadows?: ColorGradeRegion;
        midtones?: ColorGradeRegion;
        highlights?: ColorGradeRegion;
        global?: ColorGradeRegion;
        balance?: number;
        blend?: number;
      }
    | undefined;
  if (colorGrade) {
    const readRegion = (source: ColorGradeRegion | undefined, target: ColorGradeRegion): void => {
      if (!source) return;
      if (source.hue !== undefined) target.hue = source.hue;
      if (source.sat !== undefined) target.sat = source.sat;
      if (source.lum !== undefined) target.lum = source.lum;
    };
    readRegion(colorGrade.shadows, edits.color.color_grade.shadows);
    readRegion(colorGrade.midtones, edits.color.color_grade.midtones);
    readRegion(colorGrade.highlights, edits.color.color_grade.highlights);
    readRegion(colorGrade.global, edits.color.color_grade.global);
    if (colorGrade.balance !== undefined) edits.color.color_grade.balance = colorGrade.balance;
    if (colorGrade.blend !== undefined) edits.color.color_grade.blend = colorGrade.blend;
  }
  const lut3d = ops.lut_3d as { lut_id?: string; amount?: number } | undefined;
  if (lut3d?.lut_id !== undefined) edits.color.lut_3d.lut_id = lut3d.lut_id;
  if (lut3d?.amount !== undefined) edits.color.lut_3d.amount = lut3d.amount;
  const dcp = ops.dcp_hue_sat as Partial<DcpEdits> | undefined;
  if (dcp) {
    if (dcp.mode !== undefined) edits.color.dcp.mode = dcp.mode;
    if (dcp.profile_id !== undefined) edits.color.dcp.profile_id = dcp.profile_id;
    if (dcp.illuminant !== undefined) edits.color.dcp.illuminant = dcp.illuminant;
    if (dcp.use_tone_curve !== undefined) edits.color.dcp.use_tone_curve = dcp.use_tone_curve;
    if (dcp.use_base_table !== undefined) edits.color.dcp.use_base_table = dcp.use_base_table;
    if (dcp.use_look_table !== undefined) edits.color.dcp.use_look_table = dcp.use_look_table;
    if (dcp.use_baseline_exposure !== undefined)
      edits.color.dcp.use_baseline_exposure = dcp.use_baseline_exposure;
  }
}
