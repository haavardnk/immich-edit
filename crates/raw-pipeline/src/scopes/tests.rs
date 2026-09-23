use super::*;

fn solid(w: usize, h: usize, rgb: [u8; 3]) -> Vec<u8> {
    rgb.iter().copied().cycle().take(w * h * 3).collect()
}

fn peak_cell(grid: &ScopeGrid, channel: usize) -> (usize, usize) {
    let channels = grid.channels as usize;
    let width = grid.width as usize;
    let index = grid
        .data
        .iter()
        .enumerate()
        .skip(channel)
        .step_by(channels)
        .max_by_key(|(_, v)| **v)
        .map(|(i, _)| i / channels)
        .unwrap_or(0);
    (index % width, index / width)
}

fn peak_row_in_column(grid: &ScopeGrid, column: usize) -> usize {
    let width = grid.width as usize;
    (0..grid.height as usize)
        .max_by_key(|row| grid.data[row * width + column])
        .unwrap_or(0)
}

fn peak_angle(grid: &ScopeGrid) -> f32 {
    let (x, y) = peak_cell(grid, 0);
    let size = VECTORSCOPE_SIZE as f32;
    let cb = (x as f32 + 0.5) / size - 0.5;
    let cr = 0.5 - (y as f32 + 0.5) / size;
    cr.atan2(cb).to_degrees().rem_euclid(360.0)
}

#[test]
fn flat_frame_counts_every_pixel() {
    let grids = ScopeGrids::from_rgb_u8(&solid(64, 32, [128, 128, 128]), 64, 32);
    assert_eq!(grids.vectorscope.max_count, 64 * 32);
    assert_eq!(grids.waveform.max_count, 32);
    assert_eq!(grids.parade.max_count, 32);
}

#[test]
fn grey_frame_collapses_vectorscope_to_centre() {
    let grids = ScopeGrids::from_rgb_u8(&solid(32, 32, [96, 96, 96]), 32, 32);
    let centre = (VECTORSCOPE_SIZE / 2, VECTORSCOPE_SIZE / 2);
    assert_eq!(peak_cell(&grids.vectorscope, 0), centre);
}

#[test]
fn vectorscope_places_primaries_on_their_bar_targets() {
    let cases = [
        ([191u8, 0, 0], 102.93f32),
        ([191, 0, 191], 49.68),
        ([191, 191, 0], 174.76),
        ([0, 191, 0], 229.68),
        ([0, 191, 191], 282.93),
        ([0, 0, 191], 354.76),
    ];
    for (rgb, expected) in cases {
        let grids = ScopeGrids::from_rgb_u8(&solid(16, 16, rgb), 16, 16);
        let measured = peak_angle(&grids.vectorscope);
        let gap = (measured - expected).abs();
        let delta = gap.min(360.0 - gap);
        assert!(
            delta < 1.5,
            "{rgb:?}: expected {expected}, measured {measured}"
        );
    }
}

#[test]
fn parade_separates_channels_by_level() {
    let grids = ScopeGrids::from_rgb_u8(&solid(32, 32, [200, 100, 20]), 32, 32);
    let rows = [0, 1, 2].map(|channel| peak_cell(&grids.parade, channel).1);
    assert_eq!(rows, [LEVELS - 1 - 200, LEVELS - 1 - 100, LEVELS - 1 - 20]);
}

#[test]
fn waveform_follows_a_horizontal_ramp() {
    let width = 256;
    let height = 8;
    let row: Vec<u8> = (0..width)
        .flat_map(|x| [x as u8, x as u8, x as u8])
        .collect();
    let pixels = row.repeat(height);
    let grids = ScopeGrids::from_rgb_u8(&pixels, width, height);
    assert_eq!(peak_row_in_column(&grids.waveform, 0), LEVELS - 1);
    assert_eq!(peak_row_in_column(&grids.waveform, WAVEFORM_COLUMNS - 2), 0);
}

#[test]
fn from_counts_matches_the_cpu_counter() {
    let pixels: Vec<u8> = (0..40 * 30 * 3).map(|i| (i * 37 % 256) as u8).collect();
    let counts = accumulate(&pixels, 40, 30);
    let flat = [counts.waveform, counts.parade, counts.vectorscope].concat();
    assert_eq!(flat.len(), SCOPE_CELLS);
    assert_eq!(
        ScopeGrids::from_counts(&flat),
        ScopeGrids::from_rgb_u8(&pixels, 40, 30)
    );
}

#[test]
fn empty_input_yields_zeroed_grids() {
    let grids = ScopeGrids::from_rgb_u8(&[], 0, 0);
    assert_eq!(grids.waveform.max_count, 0);
    assert_eq!(grids.waveform.data.len(), WAVEFORM_COLUMNS * LEVELS);
    assert_eq!(grids.parade.data.len(), PARADE_COLUMNS * LEVELS * 3);
    assert_eq!(
        grids.vectorscope.data.len(),
        VECTORSCOPE_SIZE * VECTORSCOPE_SIZE
    );
    assert!(grids.vectorscope.data.iter().all(|&v| v == 0));
}
