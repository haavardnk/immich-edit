use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RenderOptions;
use raw_pipeline::gpu::{GpuRenderer, GpuRendererOptions, LinearSource};
use raw_pipeline::lut::{CubeLut, LutMap};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
use raw_pipeline::source::{SourceImage, decode};
use raw_pipeline::{PipelineError, PipelineResult};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::js;
use crate::present::Presenter;
use crate::view::{EditInputs, RenderView};

const TEXTURE_CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;

type Slot = fn(&mut Inputs) -> &mut Option<LinearSource>;

#[derive(Default)]
struct Inputs {
    source: Option<LinearSource>,
    tile: Option<LinearSource>,
    rasters: RasterMap,
    luts: LutMap,
    dcp: Option<Arc<DcpProfile>>,
}

struct Inner {
    gpu: GpuRenderer,
    presenter: RefCell<Presenter>,
    tile_presenter: RefCell<Presenter>,
    inputs: RefCell<Inputs>,
    lost: Arc<Mutex<Option<String>>>,
}

#[wasm_bindgen]
pub fn render_inputs(edits: &str, max_edge: u32) -> Result<JsValue, JsError> {
    Ok(js::inputs(&EditInputs::parse(edits, max_edge)?))
}

#[wasm_bindgen]
pub struct WebRenderer {
    inner: Rc<Inner>,
}

#[wasm_bindgen]
impl WebRenderer {
    pub async fn create() -> Result<WebRenderer, JsError> {
        let gpu = GpuRenderer::new_async(GpuRendererOptions {
            texture_cache_max_bytes: Some(TEXTURE_CACHE_MAX_BYTES),
            ..GpuRendererOptions::default()
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
        let (presenter, tile_presenter) = gpu
            .guarded(async { Ok((new_presenter(&gpu)?, new_presenter(&gpu)?)) })
            .await?;
        if let Some(reason) = device_lost(&lost).await {
            return Err(JsError::new(&format!("the GPU device was lost: {reason}")));
        }
        Ok(Self {
            inner: Rc::new(Inner {
                gpu,
                presenter: RefCell::new(presenter),
                tile_presenter: RefCell::new(tile_presenter),
                inputs: RefCell::new(Inputs::default()),
                lost,
            }),
        })
    }

    pub fn adapter(&self) -> String {
        self.inner.gpu.adapter_label()
    }

    pub fn limits(&self) -> JsValue {
        js::limits(&self.inner.gpu.context().device.limits())
    }

    pub fn set_synthetic_source(&self, edge: u32) -> js_sys::Promise {
        self.install(Ok(crate::bench::synthetic_source(edge)), |inputs| {
            &mut inputs.source
        })
    }

    pub fn drop_source(&self) {
        self.inner.inputs.borrow_mut().source = None;
    }

    pub fn set_source(&self, bytes: &[u8]) -> js_sys::Promise {
        self.install(decode(bytes), |inputs| &mut inputs.source)
    }

    pub fn set_tile(&self, bytes: &[u8]) -> js_sys::Promise {
        self.install(decode(bytes), |inputs| &mut inputs.tile)
    }

    pub fn drop_tile(&self) {
        self.inner.inputs.borrow_mut().tile = None;
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
        let lut = CubeLut::parse(bytes).map_err(|e| JsError::new(&format!("lut {id}: {e}")))?;
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
        self.spawn(async move { render(&inner, &edits, &view).await })
    }
}

impl WebRenderer {
    fn install(&self, image: PipelineResult<SourceImage>, slot: Slot) -> js_sys::Promise {
        let inner = self.inner.clone();
        self.spawn(async move {
            let image = image?;
            let gpu = &inner.gpu;
            let source = gpu.guarded(async { gpu.upload_source(&image) }).await?;
            *slot(&mut inner.inputs.borrow_mut()) = Some(source);
            Ok(js::source(&image.header))
        })
    }

    fn spawn(
        &self,
        work: impl Future<Output = Result<JsValue, JsError>> + 'static,
    ) -> js_sys::Promise {
        let lost = self.inner.lost.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let result = work.await;
            if result.is_err()
                && let Some(reason) = device_lost(&lost).await
            {
                return Err(JsError::new(&format!("the GPU device was lost: {reason}")).into());
            }
            result.map_err(JsValue::from)
        })
    }
}

async fn render(inner: &Inner, edits: &str, view: &str) -> Result<JsValue, JsError> {
    let edits: Edits = serde_json::from_str(edits)?;
    let view: RenderView = serde_json::from_str(view)?;
    let (source, opts) = {
        let inputs = inner.inputs.borrow();
        let slot = if view.tile {
            &inputs.tile
        } else {
            &inputs.source
        };
        let source = slot.clone().ok_or_else(|| {
            JsError::new(if view.tile {
                "no tile has been set"
            } else {
                "no source has been set"
            })
        })?;
        let opts = view.options(
            inputs.rasters.clone(),
            inputs.luts.clone(),
            inputs.dcp.clone(),
        );
        (source, opts)
    };
    inner
        .gpu
        .guarded(draw(inner, &source, &edits, &opts, &view))
        .await
        .map_err(JsError::from)
}

async fn draw(
    inner: &Inner,
    source: &LinearSource,
    edits: &Edits,
    opts: &RenderOptions,
    view: &RenderView,
) -> PipelineResult<JsValue> {
    let ctx = inner.gpu.context();
    let presenter = if view.roi.is_some() {
        &inner.tile_presenter
    } else {
        &inner.presenter
    };
    let mut frame = inner.gpu.render_display(source, edits, opts)?;
    let target = presenter
        .borrow_mut()
        .acquire(ctx, frame.dims(), view.canvas_color_space())
        .map_err(PipelineError::Render)?;
    frame.record(|encoder, texture| presenter.borrow().blit(ctx, encoder, texture, &target.view));
    let meta = inner.gpu.read_display_meta(frame).await?;
    target.present(ctx);
    let bitmap = presenter.borrow().bitmap().map_err(PipelineError::Render)?;
    Ok(js::frame(&meta, bitmap))
}

fn new_presenter(gpu: &GpuRenderer) -> PipelineResult<Presenter> {
    let canvas = web_sys::OffscreenCanvas::new(1, 1)
        .map_err(|e| PipelineError::Render(format!("offscreen canvas: {e:?}")))?;
    Presenter::new(gpu.context(), canvas).map_err(PipelineError::Render)
}

async fn device_lost(lost: &Mutex<Option<String>>) -> Option<String> {
    let _ = JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED)).await;
    lost.lock().unwrap_or_else(PoisonError::into_inner).clone()
}
