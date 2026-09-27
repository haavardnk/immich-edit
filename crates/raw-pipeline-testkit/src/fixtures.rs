use raw_pipeline::decode;
use raw_pipeline::frame::RawFrame;
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../raw-pipeline/tests")
}

pub fn fixture_path(name: &str) -> PathBuf {
    tests_dir().join("fixtures").join(name)
}

pub fn baseline_path(name: &str) -> PathBuf {
    tests_dir().join("baselines").join(name)
}

pub fn fixtures() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(tests_dir().join("fixtures")) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| RAW_EXTS.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    paths
}

pub fn any_fixture() -> Option<PathBuf> {
    fixtures().into_iter().next()
}

pub fn first_decodable_fixture() -> Option<PathBuf> {
    fixtures().into_iter().find(|p| {
        std::fs::read(p)
            .ok()
            .map(|b| decode::decode(&b).is_ok())
            .unwrap_or(false)
    })
}

pub fn first_fixture_frame() -> Option<RawFrame> {
    fixtures()
        .into_iter()
        .find_map(|p| std::fs::read(&p).ok().and_then(|b| decode::decode(&b).ok()))
}

pub fn each_fixture_frame(test: impl Fn(&str, &RawFrame)) {
    let paths = fixtures();
    if paths.is_empty() {
        eprintln!("no fixtures found; skipping");
        return;
    }
    let mut decoded = 0;
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(p).unwrap();
        let frame = match decode::decode(&bytes) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("skip {name}: decode unsupported ({e})");
                continue;
            }
        };
        test(&name, &frame);
        decoded += 1;
    }
    if decoded == 0 {
        panic!("no fixtures decoded successfully out of {}", paths.len());
    }
}
