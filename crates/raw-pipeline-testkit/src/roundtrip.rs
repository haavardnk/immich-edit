use raw_pipeline::frame::RawFrame;

pub const SPLIT_TONE_SIZE: u32 = 64;

pub fn split_tone_rgb() -> Vec<u8> {
    (0..SPLIT_TONE_SIZE)
        .flat_map(|_| 0..SPLIT_TONE_SIZE)
        .flat_map(|x| {
            let v = if x < SPLIT_TONE_SIZE / 2 { 0u8 } else { 255u8 };
            [v, v, v]
        })
        .collect()
}

fn region_mean(frame: &RawFrame, x0: usize, x1: usize) -> f32 {
    let samples: Vec<f32> = (0..frame.meta.height)
        .flat_map(|y| (x0..x1).map(move |x| (y * frame.meta.width + x) * 3))
        .flat_map(|i| frame.data[i..i + 3].iter().copied())
        .collect();
    samples.iter().sum::<f32>() / samples.len() as f32
}

pub fn assert_split_tone(frame: &RawFrame) {
    assert_eq!(frame.meta.width, SPLIT_TONE_SIZE as usize);
    assert_eq!(frame.meta.height, SPLIT_TONE_SIZE as usize);
    assert_eq!(frame.cpp, 3);
    assert!(!frame.meta.is_raw);

    let dark = region_mean(frame, 4, 28);
    let bright = region_mean(frame, 36, 60);
    assert!(dark < 0.05, "dark half not preserved: {dark}");
    assert!(bright > 0.85, "bright half not preserved: {bright}");
}
