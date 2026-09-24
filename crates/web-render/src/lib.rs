#![cfg(target_arch = "wasm32")]

mod js;
mod present;
mod view;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RenderOptions;
use raw_pipeline::gpu::{GpuRenderer, GpuRendererOptions, LinearSource};
use raw_pipeline::lut::{Lut3d, LutMap};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wgpu::ErrorFilter;

use present::Presenter;
use view::RenderView;

const TEXTURE_CACHE_MAX_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Default)]
struct Inputs {
    source: Option<LinearSource>,
    rasters: RasterMap,
    luts: LutMap,
    dcp: Option<Arc<DcpProfile>>,
}

struct Inner {
    gpu: GpuRenderer,
    presenter: RefCell<Presenter>,
    inputs: RefCell<Inputs>,
    lost: Arc<Mutex<Option<String>>>,
}

#[wasm_bindgen]
pub fn render_inputs(edits: &str, max_edge: u32) -> Result<JsValue, JsError> {
    let edits = serde_json::from_str::<Edits>(edits)?.clamped();
    let sensor_key = format!("{}-{max_edge}", edits.sensor_stage().stable_hash());
    Ok(js::inputs(
        &sensor_key,
        &edits.referenced_raster_ids(),
        edits.referenced_lut_id().as_deref(),
    ))
}

#[wasm_bindgen]
pub struct WebRenderer {
    inner: Rc<Inner>,
}

#[wasm_bindgen]
impl WebRenderer {
    pub async fn create() -> Result<WebRenderer, JsError> {
        let canvas = web_sys::OffscreenCanvas::new(1, 1)
            .map_err(|e| JsError::new(&format!("offscreen canvas: {e:?}")))?;
        let gpu = GpuRenderer::new_async(GpuRendererOptions {
            texture_cache_max_bytes: TEXTURE_CACHE_MAX_BYTES,
            timestamps: false,
        })
        .await?;
        let lost = Arc::new(Mutex::new(None));
        let sink = lost.clone();
        gpu.context()
            .device
            .set_device_lost_callback(move |reason, message| {
                *sink.lock().unwrap_or_else(PoisonError::into_inner) =
                    Some(format!("{reason:?}, {message}"));
            });
        let scope = gpu
            .context()
            .device
            .push_error_scope(ErrorFilter::Validation);
        let presenter = Presenter::new(gpu.context(), canvas).map_err(|e| JsError::new(&e))?;
        if let Some(err) = scope.pop().await {
            return Err(JsError::new(&format!("presenter: {err}")));
        }
        if let Some(reason) = device_lost(&lost).await {
            return Err(JsError::new(&format!("the GPU device was lost: {reason}")));
        }
        Ok(Self {
            inner: Rc::new(Inner {
                gpu,
                presenter: RefCell::new(presenter),
                inputs: RefCell::new(Inputs::default()),
                lost,
            }),
        })
    }

    pub fn adapter(&self) -> String {
        self.inner.gpu.adapter_label()
    }

    pub fn set_source(&self, bytes: &[u8]) -> Result<JsValue, JsError> {
        let image = raw_pipeline::source::decode(bytes)?;
        let source = self.inner.gpu.upload_source(&image)?;
        let info = js::source(&image.header);
        self.inner.inputs.borrow_mut().source = Some(source);
        Ok(info)
    }

    pub fn set_raster(
        &self,
        id: String,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    ) -> Result<(), JsError> {
        let raster = MaskRaster::new(width, height, bytes)
            .ok_or_else(|| JsError::new(&format!("raster {id} does not match {width}x{height}")))?;
        self.inner
            .inputs
            .borrow_mut()
            .rasters
            .insert(id, Arc::new(raster));
        Ok(())
    }

    pub fn drop_raster(&self, id: &str) {
        self.inner.inputs.borrow_mut().rasters.remove(id);
    }

    pub fn set_lut(&self, id: String, bytes: &[u8]) -> Result<(), JsError> {
        let lut = Lut3d::parse_cube(bytes).map_err(|e| JsError::new(&format!("lut {id}: {e}")))?;
        self.inner
            .inputs
            .borrow_mut()
            .luts
            .insert(id, Arc::new(lut));
        Ok(())
    }

    pub fn drop_lut(&self, id: &str) {
        self.inner.inputs.borrow_mut().luts.remove(id);
    }

    pub fn set_dcp(&self, bytes: Option<Vec<u8>>) -> Result<(), JsError> {
        let dcp = match bytes {
            Some(bytes) => Some(Arc::new(
                raw_pipeline::parse_dcp(&bytes).map_err(|e| JsError::new(&format!("dcp: {e}")))?,
            )),
            None => None,
        };
        self.inner.inputs.borrow_mut().dcp = dcp;
        Ok(())
    }

    pub fn render(&self, edits: String, view: String) -> js_sys::Promise {
        let inner = self.inner.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let rendered = render(&inner, &edits, &view).await;
            if rendered.is_err()
                && let Some(reason) = device_lost(&inner.lost).await
            {
                return Err(JsError::new(&format!("the GPU device was lost: {reason}")).into());
            }
            rendered.map_err(JsValue::from)
        })
    }
}

async fn render(inner: &Inner, edits: &str, view: &str) -> Result<JsValue, JsError> {
    let edits: Edits = serde_json::from_str(edits)?;
    let view: RenderView = serde_json::from_str(view)?;
    let (source, opts) = {
        let inputs = inner.inputs.borrow();
        let source = inputs
            .source
            .clone()
            .ok_or_else(|| JsError::new("no source has been set"))?;
        let opts = view.options(
            inputs.rasters.clone(),
            inputs.luts.clone(),
            inputs.dcp.clone(),
        );
        (source, opts)
    };
    let ctx = inner.gpu.context();
    let scope = ctx.device.push_error_scope(ErrorFilter::Validation);
    let drawn = draw(inner, &source, &edits, &opts, &view).await;
    if let Some(err) = scope.pop().await {
        return Err(JsError::new(&format!("render: {err}")));
    }
    drawn
}

async fn draw(
    inner: &Inner,
    source: &LinearSource,
    edits: &Edits,
    opts: &RenderOptions,
    view: &RenderView,
) -> Result<JsValue, JsError> {
    let ctx = inner.gpu.context();
    let mut frame = inner.gpu.render_display(source, edits, opts)?;
    let target = inner
        .presenter
        .borrow_mut()
        .acquire(ctx, frame.dims(), view.canvas_color_space())
        .map_err(|e| JsError::new(&e))?;
    frame.record(|encoder, texture| {
        inner
            .presenter
            .borrow()
            .blit(ctx, encoder, texture, &target.view)
    });
    let meta = inner.gpu.finish_display(frame).await?;
    target.present(ctx);
    let bitmap = inner
        .presenter
        .borrow()
        .bitmap()
        .map_err(|e| JsError::new(&e))?;
    Ok(js::frame(&meta, bitmap))
}

async fn device_lost(lost: &Mutex<Option<String>>) -> Option<String> {
    let _ = JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED)).await;
    lost.lock().unwrap_or_else(PoisonError::into_inner).clone()
}
