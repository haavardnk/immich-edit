use js_sys::{Array, Object, Reflect, Uint8Array, Uint32Array};
use raw_pipeline::gpu::DisplayMeta;
use raw_pipeline::histogram::Histogram;
use raw_pipeline::scopes::ScopeGrid;
use raw_pipeline::source::SourceHeader;
use wasm_bindgen::JsValue;

pub fn source(header: &SourceHeader) -> JsValue {
    let meta = &header.meta;
    object(&[
        ("width", header.dims.0.into()),
        ("height", header.dims.1.into()),
        ("frame_width", (meta.width as u32).into()),
        ("frame_height", (meta.height as u32).into()),
        ("is_raw", meta.is_raw.into()),
        ("model", meta.model.as_str().into()),
    ])
}

pub fn frame(meta: &DisplayMeta) -> JsValue {
    let mut fields = vec![
        ("width", meta.dims.0.into()),
        ("height", meta.dims.1.into()),
        ("source_w", meta.source_dims.0.into()),
        ("source_h", meta.source_dims.1.into()),
        ("timings", timings(meta)),
    ];
    if let Some(h) = &meta.histogram {
        fields.push(("histogram", histogram(h)));
    }
    if let Some(h) = &meta.linear_histogram {
        fields.push(("linear_histogram", histogram(h)));
    }
    if let Some(grids) = &meta.scopes {
        fields.push((
            "scopes",
            object(&[
                ("waveform", scope("waveform", &grids.waveform)),
                ("parade", scope("parade", &grids.parade)),
                ("vectorscope", scope("vectorscope", &grids.vectorscope)),
            ]),
        ));
    }
    object(&fields)
}

fn histogram(h: &Histogram) -> JsValue {
    let channel = |bins: &[u32]| Uint32Array::from(bins).into();
    object(&[
        ("r", channel(&h.r)),
        ("g", channel(&h.g)),
        ("b", channel(&h.b)),
        ("l", channel(&h.l)),
    ])
}

fn scope(kind: &str, grid: &ScopeGrid) -> JsValue {
    object(&[
        ("kind", kind.into()),
        ("width", grid.width.into()),
        ("height", grid.height.into()),
        ("channels", grid.channels.into()),
        ("maxCount", grid.max_count.into()),
        ("data", Uint8Array::from(grid.data.as_slice()).into()),
    ])
}

fn timings(meta: &DisplayMeta) -> JsValue {
    meta.timings
        .iter()
        .map(|t| {
            let mut fields = vec![
                ("stage", t.stage.into()),
                ("wall_ms", (t.wall.as_secs_f64() * 1000.0).into()),
            ];
            if let Some(gpu) = t.gpu {
                fields.push(("gpu_ms", (gpu.as_secs_f64() * 1000.0).into()));
            }
            object(&fields)
        })
        .collect::<Array>()
        .into()
}

fn object(fields: &[(&str, JsValue)]) -> JsValue {
    let out = Object::new();
    for (key, value) in fields {
        Reflect::set(&out, &JsValue::from_str(key), value)
            .expect("setting a field on a fresh object");
    }
    out.into()
}
