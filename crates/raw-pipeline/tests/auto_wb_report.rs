use std::path::{Path, PathBuf};

use raw_pipeline::cpu::pipeline;
use raw_pipeline::decode;
use raw_pipeline::edits::{Edits, WhiteBalance};
use raw_pipeline::frame::{OutputFormat, RawFrame, RenderOptions};
use raw_pipeline::white_balance::{as_shot_white, auto_white_balance};

const THUMB_EDGE: u32 = 360;
const SHEET_ROWS: usize = 6;

fn raws(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read AUTO_WB_RAWS")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    paths
}

fn describe(wb: &WhiteBalance) -> String {
    match wb {
        WhiteBalance::Absolute { kelvin, tint } => format!("{kelvin:.0} K {tint:+.0}"),
        WhiteBalance::Relative { temp, tint } => format!("{temp:+.0} {tint:+.0}"),
    }
}

fn thumb(frame: &RawFrame, edits: &Edits) -> (Vec<u8>, usize, usize) {
    let options = RenderOptions {
        max_edge: THUMB_EDGE,
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    };
    let out = pipeline::render(frame, edits, &options).expect("render");
    (out.bytes, out.width as usize, out.height as usize)
}

fn write_sheet(path: &Path, rows: &[Vec<(Vec<u8>, usize, usize)>]) {
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
fn auto_wb_report() {
    let Some(dir) = std::env::var_os("AUTO_WB_RAWS") else {
        eprintln!("skip: set AUTO_WB_RAWS=<dir of raws> to compare auto white balance");
        return;
    };
    let out_dir = std::env::var_os("AUTO_WB_OUT").map(PathBuf::from);
    let mut rows = Vec::new();
    let mut sheet = 0;
    println!("| file | as shot | auto |");
    println!("| --- | --- | --- |");
    for path in raws(Path::new(&dir)) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Ok(frame) = decode::decode(&std::fs::read(&path).expect("read raw")) else {
            println!("| {name} | decode failed | |");
            continue;
        };
        let base = raw_pipeline::auto::auto_adjust(&frame, &Edits::default());
        let shot = as_shot_white(&frame.meta, &base, None);
        let auto = auto_white_balance(&frame, &base, None);
        println!(
            "| {name} | {:.0} K {:+.0} | {} |",
            shot.kelvin,
            shot.tint,
            auto.as_ref().map_or("none".to_string(), describe)
        );
        let Some(out_dir) = &out_dir else {
            continue;
        };
        let mut row = vec![thumb(&frame, &base)];
        if let Some(wb) = auto {
            let mut edits = base.clone();
            edits.basic.white_balance = wb;
            row.push(thumb(&frame, &edits));
        }
        rows.push(row);
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
