use std::path::PathBuf;
use std::time::{Duration, Instant};

use raw_pipeline::edits::{CropRect, Edits, LensEdits};
use raw_pipeline::frame::{JpegSubsampling, OutputFormat, RawFrame, RenderOptions};
use raw_pipeline::timing::StageTiming;
use raw_pipeline::{CpuRenderer, GpuRenderer, GpuRendererOptions, decode};
use raw_pipeline_testkit::fixtures::fixture_path;

fn view_edge() -> u32 {
    std::env::var("JOURNEY_EDGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2560)
}

fn base_edits() -> Edits {
    if std::env::var("JOURNEY_LENS").as_deref() != Ok("1") {
        return Edits::default();
    }
    Edits {
        lens: LensEdits {
            profile_enabled: Some(true),
            ca_enabled: true,
            k1: -0.04,
            k2: 0.01,
            vk1: -0.3,
            ca_red_scale_x10000: 2.0,
            ca_blue_scale_x10000: -2.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

enum Renderer {
    Gpu(Box<GpuRenderer>),
    Cpu(CpuRenderer),
}

impl Renderer {
    fn preview(&self, frame: &RawFrame, edits: &Edits, opts: &RenderOptions) -> Vec<StageTiming> {
        match self {
            Self::Gpu(r) => r.render(frame, edits, opts).unwrap().timings,
            Self::Cpu(r) => r.render(frame, edits, opts).unwrap().timings,
        }
    }

    fn source(&self, frame: &RawFrame, edits: &Edits, opts: &RenderOptions) -> Vec<StageTiming> {
        match self {
            Self::Gpu(r) => r.render_source(frame, edits, opts, None).unwrap().timings,
            Self::Cpu(r) => r.render_source(frame, edits, opts, None).unwrap().timings,
        }
    }
}

fn raws() -> Vec<PathBuf> {
    match std::env::var("JOURNEY_RAWS") {
        Ok(list) => list.split(':').map(PathBuf::from).collect(),
        Err(_) => vec![fixture_path("Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf")],
    }
}

fn view_opts(max_edge: u32) -> RenderOptions {
    RenderOptions {
        max_edge,
        output: OutputFormat::Jpeg {
            quality: 90,
            subsampling: JpegSubsampling::Chroma444,
        },
        histogram: true,
        scopes: true,
        ..Default::default()
    }
}

fn ms(d: Duration) -> String {
    format!("{:.1}", d.as_secs_f64() * 1000.0)
}

fn row(step: &str, total: Duration, timings: &[StageTiming]) {
    let stages: Vec<String> = timings
        .iter()
        .filter(|t| {
            t.wall >= Duration::from_millis(1)
                || t.gpu.is_some_and(|g| g >= Duration::from_millis(1))
        })
        .map(|t| match t.gpu {
            Some(gpu) => format!("{} {} [{}]", t.stage, ms(t.wall), ms(gpu)),
            None => format!("{} {}", t.stage, ms(t.wall)),
        })
        .collect();
    println!("| {step} | {} | {} |", ms(total), stages.join(", "));
}

fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let out = work();
    (out, started.elapsed())
}

fn journey(renderer: &Renderer, bytes: &[u8]) {
    let (frame, decode_ms) = timed(|| decode::decode(bytes).unwrap());
    println!("capture sigma {:?}\n", frame.meta.capture_sigma);
    println!("| Step | Total ms | Stages wall [gpu] ms |\n| --- | --- | --- |");
    row("decode", decode_ms, &[]);
    let (_, quality_ms) = timed(|| decode::decode_quality(bytes).unwrap());
    row("decode quality", quality_ms, &[]);

    let edits = base_edits();
    let long = frame.meta.width.max(frame.meta.height) as u32;
    let fit = view_opts(view_edge());

    let (timings, total) = timed(|| renderer.preview(&frame, &edits, &fit));
    row("open preview", total, &timings);
    let (timings, total) = timed(|| renderer.source(&frame, &edits.sensor_stage(), &fit));
    row("open source", total, &timings);

    let mut exposure = edits.clone();
    exposure.basic.exposure_ev = 0.4;
    let (timings, total) = timed(|| renderer.preview(&frame, &exposure, &fit));
    row("tick exposure", total, &timings);

    let mut dehaze = exposure.clone();
    dehaze.basic.dehaze = 15.0;
    let (timings, total) = timed(|| renderer.preview(&frame, &dehaze, &fit));
    row("tick dehaze", total, &timings);

    let mut nr = dehaze.clone();
    nr.detail.luma_nr_amount = 30.0;
    let (timings, total) = timed(|| renderer.preview(&frame, &nr, &fit));
    row("tick noise reduction", total, &timings);

    let tile = RenderOptions {
        roi: Some(CropRect {
            x: 0.4,
            y: 0.4,
            w: 0.2,
            h: 0.2,
        }),
        ..view_opts(long)
    };
    let (timings, total) = timed(|| renderer.preview(&frame, &nr, &tile));
    row("zoom 1:1 tile", total, &timings);
    let (timings, total) = timed(|| renderer.source(&frame, &nr.sensor_stage(), &tile));
    row("zoom 1:1 source", total, &timings);

    let (_, total) = timed(|| raw_pipeline::auto::auto_adjust(&frame, &edits));
    row("auto", total, &[]);
    let (_, total) = timed(|| raw_pipeline::white_balance::auto_white_balance(&frame, &edits));
    row("auto wb", total, &[]);
    let (_, total) =
        timed(|| raw_pipeline::white_balance::sample_white_balance(&frame, &edits, 0.5, 0.5));
    row("wb picker", total, &[]);
}

#[test]
fn journey_report() {
    if std::env::var_os("JOURNEY_REPORT").is_none() {
        eprintln!("skip: set JOURNEY_REPORT=1 to time an editor session per raw");
        return;
    }
    let use_gpu = std::env::var("JOURNEY_GPU").as_deref() != Ok("0");
    for path in raws() {
        let bytes = std::fs::read(&path).unwrap();
        let renderer = if use_gpu {
            Renderer::Gpu(Box::new(
                GpuRenderer::with_options(GpuRendererOptions {
                    timestamps: true,
                    ..Default::default()
                })
                .unwrap(),
            ))
        } else {
            Renderer::Cpu(CpuRenderer::new())
        };
        let kind = match renderer {
            Renderer::Gpu(_) => "gpu",
            Renderer::Cpu(_) => "cpu",
        };
        println!(
            "\n### {} ({kind}, {} rayon threads)\n",
            path.display(),
            rayon::current_num_threads()
        );
        journey(&renderer, &bytes);
    }
}
