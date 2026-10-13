use std::path::{Path, PathBuf};
use std::time::Instant;

use raw_pipeline::auto::auto_adjust;
use raw_pipeline::cpu::pipeline;
use raw_pipeline::decode;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{OutputFormat, RawFrame, RenderOptions};

const THUMB_EDGE: u32 = 360;
const SHEET_ROWS: usize = 6;

type Thumb = (Vec<u8>, usize, usize);

fn raws(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read AUTO_WB_RAWS")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    paths
}

fn thumb(frame: &RawFrame, edits: &Edits) -> Thumb {
    let options = RenderOptions {
        max_edge: THUMB_EDGE,
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    };
    let out = pipeline::render(frame, edits, &options).expect("render");
    (out.bytes, out.width as usize, out.height as usize)
}

fn write_sheet(path: &Path, rows: &[Vec<Thumb>]) {
    let cell = THUMB_EDGE as usize;
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let width = cols * cell;
    let height = rows.len() * cell;
    let mut pixels = vec![32u8; width * height * 3];
    for (r, c) in (0..rows.len()).flat_map(|r| (0..rows[r].len()).map(move |c| (r, c))) {
        let (bytes, w, h) = &rows[r][c];
        for y in 0..*h {
            let dst = ((r * cell + y) * width + c * cell) * 3;
            pixels[dst..dst + w * 3].copy_from_slice(&bytes[y * w * 3..(y + 1) * w * 3]);
        }
    }
    let image = turbojpeg::Image {
        pixels: pixels.as_slice(),
        width,
        pitch: width * 3,
        height,
        format: turbojpeg::PixelFormat::RGB,
    };
    let jpeg = turbojpeg::compress(image, 85, turbojpeg::Subsamp::None).expect("encode sheet");
    std::fs::write(path, jpeg.as_ref()).expect("write sheet");
}

#[test]
fn auto_adjust_report() {
    let Some(dir) = std::env::var_os("AUTO_WB_RAWS") else {
        eprintln!("skip: set AUTO_WB_RAWS=<dir of raws> to compare auto adjust");
        return;
    };
    let out_dir = std::env::var_os("AUTO_ADJUST_OUT").map(PathBuf::from);
    let mut rows = Vec::new();
    let mut sheet = 0;
    println!("| file | ms | exp | con | hi | sh | wh | bl | dehaze | clar | vib | sat |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for path in raws(Path::new(&dir)) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Ok(frame) = decode::decode(&std::fs::read(&path).expect("read raw")) else {
            println!("| {name} | decode failed |");
            continue;
        };
        let start = Instant::now();
        let auto = auto_adjust(&frame, &Edits::default(), None, &[]).expect("auto adjust");
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        println!(
            "| {} | {ms:.0} | {:+.2} | {:+.0} | {:+.0} | {:+.0} | {:+.0} | {:+.0} | {:+.0} | {:+.0} | {:+.0} | {:+.0} |",
            &name[..8],
            auto.basic.exposure_ev,
            auto.basic.contrast,
            auto.tone.highlights,
            auto.tone.shadows,
            auto.tone.whites,
            auto.tone.blacks,
            auto.basic.dehaze,
            auto.basic.clarity,
            auto.basic.vibrance,
            auto.basic.saturation
        );
        let Some(out_dir) = &out_dir else {
            continue;
        };
        rows.push(vec![thumb(&frame, &Edits::default()), thumb(&frame, &auto)]);
        if rows.len() == SHEET_ROWS {
            write_sheet(&out_dir.join(format!("sheet-{sheet}.jpg")), &rows);
            rows.clear();
            sheet += 1;
        }
    }
    if let Some(out_dir) = &out_dir
        && !rows.is_empty()
    {
        write_sheet(&out_dir.join(format!("sheet-{sheet}.jpg")), &rows);
    }
}
