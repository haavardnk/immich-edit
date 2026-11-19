use raw_pipeline::edits::{
    BasicEdits, ColorEdits, DetailEdits, Edits, EffectsEdits, LensBlurEdits,
};
use serde::Deserialize;

pub mod legacy;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct MergeSections {
    pub dcp: bool,
    pub white_balance: bool,
    pub tone: bool,
    pub presence: bool,
    pub curves: bool,
    pub hsl: bool,
    pub color_grading: bool,
    pub bw: bool,
    pub lut: bool,
    pub sharpening: bool,
    pub noise_reduction: bool,
    pub lens: bool,
    pub lens_blur: bool,
    pub vignette: bool,
    pub grain: bool,
    pub geometry: bool,
    pub masks: bool,
    pub retouch: bool,
}

impl MergeSections {
    pub fn look_only() -> Self {
        Self {
            dcp: true,
            white_balance: true,
            tone: true,
            presence: true,
            curves: true,
            hsl: true,
            color_grading: true,
            bw: true,
            lut: true,
            sharpening: true,
            noise_reduction: true,
            lens: true,
            vignette: true,
            grain: true,
            ..Self::default()
        }
    }

    pub fn paste_default() -> Self {
        Self {
            lens_blur: true,
            ..Self::look_only()
        }
    }
}

fn take<T>(on: bool, target: &mut T, source: T) {
    if on {
        *target = source;
    }
}

fn merge_basic(target: &mut BasicEdits, source: BasicEdits, tools: MergeSections) {
    let BasicEdits {
        exposure_ev,
        brightness,
        contrast,
        saturation,
        vibrance,
        white_balance,
        texture,
        clarity,
        dehaze,
        curves,
    } = source;
    take(
        tools.white_balance,
        &mut target.white_balance,
        white_balance,
    );
    take(tools.tone, &mut target.exposure_ev, exposure_ev);
    take(tools.tone, &mut target.brightness, brightness);
    take(tools.tone, &mut target.contrast, contrast);
    take(tools.presence, &mut target.texture, texture);
    take(tools.presence, &mut target.clarity, clarity);
    take(tools.presence, &mut target.dehaze, dehaze);
    take(tools.presence, &mut target.vibrance, vibrance);
    take(tools.presence, &mut target.saturation, saturation);
    take(tools.curves, &mut target.curves, curves);
}

fn merge_color(target: &mut ColorEdits, source: ColorEdits, tools: MergeSections) {
    let ColorEdits {
        hsl,
        color_grade,
        lut_3d,
        dcp,
        bw,
    } = source;
    take(tools.dcp, &mut target.dcp, dcp);
    take(tools.hsl, &mut target.hsl, hsl);
    take(tools.color_grading, &mut target.color_grade, color_grade);
    take(tools.bw, &mut target.bw, bw);
    take(tools.lut, &mut target.lut_3d, lut_3d);
}

fn merge_detail(target: &mut DetailEdits, source: DetailEdits, tools: MergeSections) {
    let DetailEdits {
        sharpen_amount,
        sharpen_radius,
        sharpen_detail,
        sharpen_masking,
        luma_nr_amount,
        luma_nr_detail,
        luma_nr_contrast,
        color_nr_amount,
        color_nr_detail,
        color_nr_smoothness,
        capture_sharpen,
        capture_corner_boost,
        ai_denoise,
        ai_denoise_strength,
        ai_denoise_detail,
    } = source;
    let sharpen = tools.sharpening;
    take(sharpen, &mut target.capture_sharpen, capture_sharpen);
    take(
        sharpen,
        &mut target.capture_corner_boost,
        capture_corner_boost,
    );
    take(sharpen, &mut target.sharpen_amount, sharpen_amount);
    take(sharpen, &mut target.sharpen_radius, sharpen_radius);
    take(sharpen, &mut target.sharpen_detail, sharpen_detail);
    take(sharpen, &mut target.sharpen_masking, sharpen_masking);
    let noise = tools.noise_reduction;
    take(noise, &mut target.luma_nr_amount, luma_nr_amount);
    take(noise, &mut target.luma_nr_detail, luma_nr_detail);
    take(noise, &mut target.luma_nr_contrast, luma_nr_contrast);
    take(noise, &mut target.color_nr_amount, color_nr_amount);
    take(noise, &mut target.color_nr_detail, color_nr_detail);
    take(noise, &mut target.color_nr_smoothness, color_nr_smoothness);
    take(noise, &mut target.ai_denoise, ai_denoise);
    take(noise, &mut target.ai_denoise_strength, ai_denoise_strength);
    take(noise, &mut target.ai_denoise_detail, ai_denoise_detail);
}

fn merge_effects(target: &mut EffectsEdits, source: EffectsEdits, tools: MergeSections) {
    let EffectsEdits {
        vignette_amount,
        vignette_midpoint,
        vignette_feather,
        vignette_roundness,
        grain_amount,
        grain_size,
        grain_roughness,
    } = source;
    let vignette = tools.vignette;
    take(vignette, &mut target.vignette_amount, vignette_amount);
    take(vignette, &mut target.vignette_midpoint, vignette_midpoint);
    take(vignette, &mut target.vignette_feather, vignette_feather);
    take(vignette, &mut target.vignette_roundness, vignette_roundness);
    take(tools.grain, &mut target.grain_amount, grain_amount);
    take(tools.grain, &mut target.grain_size, grain_size);
    take(tools.grain, &mut target.grain_roughness, grain_roughness);
}

fn merge_lens_blur(target: &mut LensBlurEdits, source: LensBlurEdits, tools: MergeSections) {
    let LensBlurEdits {
        amount,
        focus_min,
        focus_max,
        shape,
        boost,
        depth_raster_id: _,
    } = source;
    let on = tools.lens_blur;
    take(on, &mut target.amount, amount);
    take(on, &mut target.focus_min, focus_min);
    take(on, &mut target.focus_max, focus_max);
    take(on, &mut target.shape, shape);
    take(on, &mut target.boost, boost);
}

pub fn merge_edits(current: Edits, incoming: Edits, tools: MergeSections) -> Edits {
    let Edits {
        basic,
        tone,
        color,
        detail,
        effects,
        lens,
        geometry,
        masks,
        retouch,
        lens_blur,
        unknown_ops: _,
    } = incoming;
    let mut merged = current;
    merge_basic(&mut merged.basic, basic, tools);
    take(tools.tone, &mut merged.tone, tone);
    merge_color(&mut merged.color, color, tools);
    merge_detail(&mut merged.detail, detail, tools);
    merge_effects(&mut merged.effects, effects, tools);
    take(tools.lens, &mut merged.lens, lens);
    merge_lens_blur(&mut merged.lens_blur, lens_blur, tools);
    take(tools.geometry, &mut merged.geometry, geometry);
    take(tools.masks, &mut merged.masks, masks);
    take(tools.retouch, &mut merged.retouch, retouch);
    merged
}
