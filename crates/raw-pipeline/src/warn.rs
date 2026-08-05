pub const GAMUT_WARN_RGB: [u8; 3] = [255, 0, 255];
pub const HIGHLIGHT_WARN_RGB: [u8; 3] = [255, 0, 0];
pub const SHADOW_WARN_RGB: [u8; 3] = [0, 0, 255];

pub const ALPHA_GAMUT: u8 = 0;
pub const ALPHA_HIGHLIGHT: u8 = 1;
pub const ALPHA_SHADOW: u8 = 2;

pub const HIGHLIGHT_CLIP: f32 = 254.5 / 255.0;
pub const SHADOW_CLIP: f32 = 0.5 / 255.0;

pub fn classify(display: [f32; 3], out_of_gamut: bool, clip_warn: bool) -> Option<[u8; 3]> {
    if out_of_gamut {
        return Some(GAMUT_WARN_RGB);
    }
    if !clip_warn {
        return None;
    }
    if display.iter().any(|&c| c >= HIGHLIGHT_CLIP) {
        return Some(HIGHLIGHT_WARN_RGB);
    }
    if display.iter().any(|&c| c <= SHADOW_CLIP) {
        return Some(SHADOW_WARN_RGB);
    }
    None
}

pub fn paint_rgba8(rgba: &mut [u8], gamut_warn: bool, clip_warn: bool) {
    for px in rgba.chunks_exact_mut(4) {
        let code = px[3];
        px[3] = 255;
        let paint = match code {
            ALPHA_GAMUT if gamut_warn => GAMUT_WARN_RGB,
            ALPHA_HIGHLIGHT if clip_warn => HIGHLIGHT_WARN_RGB,
            ALPHA_SHADOW if clip_warn => SHADOW_WARN_RGB,
            _ => continue,
        };
        px[..3].copy_from_slice(&paint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_table() {
        let cases = [
            ([1.0, 1.0, 1.0], false, true, Some(HIGHLIGHT_WARN_RGB)),
            ([0.4, 1.0, 0.6], false, true, Some(HIGHLIGHT_WARN_RGB)),
            ([0.9985, 0.5, 0.5], false, true, Some(HIGHLIGHT_WARN_RGB)),
            ([0.0, 0.0, 0.0], false, true, Some(SHADOW_WARN_RGB)),
            ([0.4, 0.0, 0.6], false, true, Some(SHADOW_WARN_RGB)),
            ([0.001, 0.5, 0.5], false, true, Some(SHADOW_WARN_RGB)),
            ([0.99, 0.99, 0.99], false, true, None),
            ([0.5, 0.5, 0.5], false, true, None),
            ([1.0, 1.0, 1.0], false, false, None),
            ([0.5, 0.5, 0.5], true, false, Some(GAMUT_WARN_RGB)),
        ];
        for (display, out_of_gamut, clip_warn, want) in cases {
            let got = classify(display, out_of_gamut, clip_warn);
            if got != want {
                panic!(
                    "classify({display:?}, {out_of_gamut}, {clip_warn}) = {got:?}, want {want:?}"
                );
            }
        }
    }

    #[test]
    fn highlight_wins_over_shadow_and_gamut_wins_over_both() {
        if classify([1.0, 0.0, 0.5], false, true) != Some(HIGHLIGHT_WARN_RGB) {
            panic!("a channel at the ceiling should read as a highlight clip");
        }
        if classify([1.0, 0.0, 0.5], true, true) != Some(GAMUT_WARN_RGB) {
            panic!("gamut should take precedence over clipping");
        }
    }

    #[test]
    fn paint_rgba8_maps_alpha_classes_and_resets_alpha() {
        let mut rgba = vec![
            128,
            128,
            128,
            ALPHA_GAMUT, //
            200,
            200,
            200,
            ALPHA_HIGHLIGHT, //
            4,
            4,
            4,
            ALPHA_SHADOW, //
            10,
            10,
            10,
            255, //
        ];
        paint_rgba8(&mut rgba, true, true);
        let want = vec![
            255, 0, 255, 255, //
            255, 0, 0, 255, //
            0, 0, 255, 255, //
            10, 10, 10, 255,
        ];
        if rgba != want {
            panic!("paint_rgba8 = {rgba:?}, want {want:?}");
        }
    }

    #[test]
    fn each_warning_is_ignored_when_its_toggle_is_off() {
        let mut rgba = vec![
            128,
            128,
            128,
            ALPHA_GAMUT, //
            200,
            200,
            200,
            ALPHA_HIGHLIGHT, //
        ];
        paint_rgba8(&mut rgba, false, false);
        if rgba != vec![128, 128, 128, 255, 200, 200, 200, 255] {
            panic!("no warning should paint when both toggles are off, got {rgba:?}");
        }
    }
}
