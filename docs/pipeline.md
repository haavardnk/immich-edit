# Raw Pipeline

This is the contributor map for render-pass ownership. Keep it in sync when an operator changes stage, gains GPU support, or moves between passes.

## Operator model

`crates/raw-pipeline/src/ops.rs` defines a single `Op` trait. The registry stores operators as `Box<dyn Op>` and sorts by `(stage, order)`.

Stages run in this order:

`Sensor -> WhiteBalance -> Tone -> Color -> Geometry -> Output`

Every operator implements the metadata half of `Op`:

- `id() -> &'static str`
- `stage() -> Stage`
- `order() -> i32`
- `is_active(&Edits) -> bool`
- `to_doc` / `from_doc` for persisted edit data

The behaviour half has defaults, so each operator only implements what it needs.

Pointwise operators implement `cpu_fused`, returning a `CpuFusedOp` that is batched into generated CPU/GPU process passes, and can contribute WGSL through `gpu()`. Exposure, contrast, curves, HSL, white balance, and color matrix work this way. The default `apply_cpu` runs `cpu_fused` as a one-op segment, so a fused operator never needs its own `apply_cpu`.

Neighbourhood and geometry operators override `apply_cpu` directly and may opt into `GpuOpKind::Presence` or `GpuOpKind::Detail`. `GpuOpKind::Normal` is the default; active normal ops in the GPU process pass must return `gpu()` unless the renderer handles them specially.

The persisted 3D LUT is an `Op` with `Stage::Output` that acts as an orchestration hook. Other CPU output effects are ordinary `Stage::Output` operators; final tone/output conversion and the resolved LUT run after `run_output_ops` in `finish_output` and encode.

`GpuOpKind` currently has three values:

- `Normal` - generated into the process pass
- `Presence` - handled by the presence/luma-pyramid path
- `Detail` - handled by dedicated detail passes

`dehaze` is a special case: it overrides `apply_cpu` with no generated WGSL contribution. The GPU renderer detects `op.id() == "dehaze"` and runs `passes/dehaze.rs` when `basic.dehaze` is active.

## CPU path

Main entry: `cpu::render_with_cancel` in `crates/raw-pipeline/src/cpu/pipeline.rs`.

Flow:

1. Demosaic RAW frames when needed.
2. Resolve the camera profile. Auto mode asks the backend for a camera-model match; the DCP matrix becomes `OpContext.cam_to_srgb`, scaled by the profile's baseline exposure when that toggle is on. The render adds no exposure of its own.
3. Run `Sensor` ops through `run_sensor_ops` when lens edits are active.
4. Apply EXIF orientation.
5. Run edit ops through `run_pipeline_ops`.
6. Resize to `RenderOptions::max_edge`.
7. Run `Stage::Output` ops through `run_output_ops`.
8. Run `finish_output`: DCP LookTable and Adobe-style tone curve in linear ProPhoto, output conversion, then the display-referred 3D LUT.
9. Encode.

`run_pipeline_ops` batches consecutive fused operators into `FusedSegment`s, then flushes before CPU spatial work. Mask layers build their own effective edits and run through masked fused segments so each layer can apply a different local adjustment set.

Mask preview mode is a separate path. It omits the previewed layer's local adjustments, evaluates the complete layer weight, runs geometry and output effects, applies the DCP finish, then blends the translucent red overlay. The creative LUT and non-sRGB output conversion still live beyond the CPU overlay's early `display_ready` return and are not applied on this path.

Current CPU detail behaviour is important: `GpuOpKind::Detail` ops are skipped in `run_pipeline_ops`. Today that means luma NR, color NR, sharpen, and capture sharpen are GPU-owned in the normal render path. If CPU parity for those edits becomes required, change the code and this document together.

Capture sharpening is the one detail op that is on by default. It runs only when the source is RAW and the decoder produced a blur estimate (`RawFrame::capture_sigma`), and it deconvolves the luma channel with eight Richardson-Lucy iterations before any creative tone work. The correction is gated by local contrast, shadow level, and highlight proximity so flat noise, deep shadows, and clipped highlights are left alone, then clamped to the local min/max of the original luma to suppress halos.

## GPU path

Main entry: `GpuRenderer::render_with_cancel` in `crates/raw-pipeline/src/gpu/renderer.rs`.

`RenderPlan::select` chooses:

- `Fast` - no presence inputs. Upload/demosaic plus process pass is enough.
- `Presence` - runs white-balance prep and dedicated detail/presence/dehaze paths before process.

Dispatch order:

| Order | Pass | Source | Owns |
| --- | --- | --- | --- |
| 1 | upload / demosaic | `get_or_demosaic`, `passes/demosaic.rs`, `passes/xtrans.rs` | RAW or RGB input to linear `Rgba16Float`; Bayer takes one dispatch, X-Trans takes two; mipgen for source textures. |
| 2 | sensor | `run_sensor`, `passes/sensor.rs` | Active lens sensor ops before orientation/crop sampling. |
| 3 | wb_prepare | `run_wb_prepare`, `passes/wb_prepare.rs` | White balance plus camera-to-sRGB pre-pass for presence/detail work. |
| 4 | NR | `run_nr`, `passes/nr.rs`, `passes/nr_smooth.rs` | `luma_nr`, `color_nr` detail work. |
| 5 | capture sharpen | `run_capture_sharpen`, `passes/capture_sharpen.rs` | RAW-only Richardson-Lucy deconvolution of sensor blur, after NR so noise is not amplified. |
| 6 | dehaze | `atmosphere_for`, `run_dehaze`, `passes/dehaze.rs` | Atmosphere estimate and DCP/guided-filter dehaze. |
| 7 | presence | `run_presence`, `passes/presence.rs`, `passes/luma_pyramid.rs` | Texture and clarity. Shadows builds a luma pyramid later when needed. |
| 8 | process | `process`, `passes/process.rs`, generated `process.wgsl` | Pointwise ops, crop/rotate/flip/angle sampling, fast-path tone. |
| 9 | DCP base table | `encode_dcp_huesat`, `passes/dcp_huesat.rs` | Camera HueSatMap in linear ProPhoto; skipped when disabled or unmatched. |
| 10 | masks | `passes/mask_weight.rs`, `passes/mask_blend.rs` | Per-layer mask weight and local adjustment blend. |
| 11 | sharpen | `encode_sharpen`, `passes/sharpen.rs` | Sharpen and sharpen preview modes. |
| 12 | effects + output | `encode_effects_tone`, `passes/effects_tone.rs` | Vignette, grain, destination-gamut projection, and output conversion when a final pass is active. |
| 13 | DCP finish | `encode_dcp_huesat`, `dcp_huesat.wgsl` | LookTable value-axis encoding and Adobe hue-preserving profile tone curve. |
| 14 | 3D LUT | `maybe_encode_lut`, `passes/lut.rs` | Display-referred `.cube` LUT with tetrahedral interpolation. |
| 15 | mask preview overlay | `passes/mask_overlay.rs` | Optional translucent red layer-weight overlay after DCP and LUT. |
| 16 | readback / encode | `gpu/readback.rs`, `encode::encode_from_rgba8` | RGBA readback, histogram, JPEG/other output encode. |

## Resolution reduction

Requests smaller than the sensor are downscaled by a separable Lanczos3 compute pass (`passes/resample.rs`, `assets/shaders/resample.wgsl`, driven by `resample_lanczos` in `gpu/renderer/resample.rs`), matching the CPU path's `fast_image_resize` Lanczos3. Two dispatches run in scene-linear, horizontal then vertical, and negative lobes are clamped at zero so ringing cannot produce negative radiance.

The reduction factor comes from the cropped region, not the full sensor, so a crop is rendered from source pixels at its own scale instead of being softened by the uncropped ratio. Output dimensions are computed once from the original frame dimensions and passed into `process`, so an already-downscaled working texture cannot shift the result size by a rounding step.

`Presence` additionally downscales once up front when a preview is more than 2x smaller than the source, so spatial passes run at preview scale. `process` then samples the working texture with a Catmull-Rom bicubic tap rather than a bilinear one: geometry sampling lands on fractional coordinates whenever a crop origin, rotation, or perspective warp is not pixel-aligned, and bilinear visibly softens those cases.

## Region of interest

`RenderOptions.roi` renders only part of the image, at the full `max_edge` budget. It is a normalised rectangle in display space — after orientation, perspective, angle and the user crop — which is what a zoomed viewer can measure directly off the screen. The point is detail: a fit-to-window preview of a 60 MP frame throws away most of the sensor, so per-pixel work such as capture sharpening and noise reduction cannot be judged from it.

The rectangle is composed into the geometry crop at the top of the render, so everything downstream — the transform op, output sizing, masks, lens corrections — treats the tile as an ordinary crop and needs no special case. Vignette and grain are the exception, since they are defined against the output frame rather than the source: they receive the rectangle and remap back onto the full frame, so a tile shows the same falloff and the same grain as the region it came from. Both renderers do this; on the GPU the remap rides along in the `effects_tone` uniform.

The editor uses this for everything it paints, not only when zoomed. `web/src/lib/utils/view-geometry.ts` works out where the image frame lands in the viewport, which part of it is visible, and how many device pixels that region will occupy. It then asks for exactly that rectangle at exactly that pixel count, so the browser never resamples: one source pixel lands on one device pixel, at an integer device-pixel offset. This holds at fit as well as at 400%, and it is why resizing a sidebar changes the render instead of stretching it.

The request is padded by 10% and quantised to 1/512 so small pans reuse the render already on screen, and it is skipped entirely when the rectangle in hand already covers the visible region at sufficient resolution. Resolution is capped by the source: the store tracks the sensor's long edge, and when the server returns fewer pixels than asked for it learns the real limit from the response and stops over-requesting. Beyond that point the render is upscaled to fill its box rather than left the wrong size.

The result is drawn over the full-frame preview rather than replacing it, so overlays, the histogram and the crop tool keep measuring one unchanging base image, and the base stays visible around the edges while a pan is in flight. Any edit drops it and it returns 150 ms after the view settles.

## Color-space rules

Intermediate GPU textures from upload through profile/edit processing are linear scene-referred sRGB in `Rgba16Float`. Output conversion happens once:

- In `process.wgsl` for the fast path with no sharpen/effects/masks.
- In `effects_tone.wgsl` whenever sharpen, vignette, grain, masks, DCP, or Display P3 output require the final pass.

DCP tables operate in linear ProPhoto. `ProfileHueSatMapEncoding` and `ProfileLookTableEncoding` affect only the HSV value lookup/scaling axis. The LookTable runs before the profile tone curve. The tone curve follows Adobe's hue-preserving min/max transform rather than applying the curve independently to all RGB channels.

The user LUT is separate from camera profiling: it runs last on display-referred sRGB. CPU and GPU both use tetrahedral interpolation.

There is no type-level distinction between linear and gamma-encoded textures. The one-line `color-space:` headers in pass files are the current guardrail; review new passes carefully.

## Effect ownership

`vignette` and `grain` are `SpatialOp`s with `Stage::Output`. On CPU they run in `run_output_ops` before `finish_output`. On GPU they are baked into `effects_tone.wgsl` and run before tone mapping.

`transform` is a generated process-pass contribution on GPU. On CPU it is a normal `SpatialOp` in `Stage::Geometry`.

`masks` is registered in `default_registry()` so its parameters persist through `to_doc`/`from_doc`, but `is_active()` always returns false, so it never runs as a normal op. CPU masks are handled inside `run_pipeline_ops`; GPU masks use `mask_weight` and `mask_blend`, both submitted from within `process()` on the same command encoder rather than as separate calls. A GPU mask preview evaluates the selected layer into `mask_weight`, skips normal local-mask blending, and runs `mask_overlay` after DCP and LUT.

`DcpProfileOp` retains manifest ID `dcp_hue_sat` for compatibility. It owns profile persistence and CPU base-table dispatch; the GPU renderer uses dedicated 3D-texture passes. Matrix selection and profile setup live in `dcp_pipeline.rs`.

Lens ownership is split. `lens_vignette` is handled by `passes/sensor.rs` on GPU. Lens distortion and chromatic aberration have CPU implementations; the GPU path still uses lens warp parameters for mask sampling.

## Mask generation renders

AI mask generation first renders an `OutputFormat::Rgb8` frame for the segmentation service. The backend loads the saved edits, then clears geometry, lens, effects, and masks. Exposure, white balance, camera profiling, and other global color edits remain, so the model sees a useful image while its raster stays in the same scene-space coordinates as manual masks.

`Rgb8` is raw interleaved RGB bytes with no image container. It is an internal pipeline output and is not accepted by public preview or export requests.

Inference runs outside raw-pipeline in the backend `SegmentService`. The result and its probability map are stored as `r8` rasters. Rendering sees the result as a normal brush component, so generated masks use the same CPU and GPU paths as painted masks.

## Adding an operator

1. Pick a role:
   - Pointwise: implement `FusedOp`, return `cpu_fused`, add `gpu()` if GPU should support it in the generated process pass.
   - Spatial CPU-only: implement `SpatialOp::apply_cpu`. Make sure the GPU path either rejects it clearly or has an equivalent dedicated path.
   - Spatial GPU detail/presence: implement `SpatialOp`, return `GpuOpKind::Detail` or `GpuOpKind::Presence`, then wire the renderer pass.
   - Output setting: implement `OutputStageOp` only for persisted output options.
2. Register it in `default_registry()`.
3. Add edit round-trip coverage in `ops/tests.rs` when it persists edits.
4. Add render-path coverage in CPU/GPU tests when it changes pixels.
5. Update the dispatch table above.

## Tracing and cancellation

Top-level CPU and GPU boundaries have tracing spans such as `cpu.pipeline_ops`, `gpu.upload_rgb`, `gpu.demosaic`, `gpu.mipgen`, `gpu.run_nr`, `gpu.run_presence`, `gpu.run_wb_prepare`, `gpu.run_sensor`, `gpu.encode_effects_tone`, `gpu_dehaze`, and `gpu_dehaze_atm`.

`CancelToken` is checked between CPU-side boundaries in both renderers. Long GPU readbacks use `map_buffer_cancellable`, which polls the device and returns `PipelineError::Cancelled` when the token flips.

Once submitted to the GPU queue, a sub-pass runs to completion. Cancellation is honoured at the next CPU checkpoint after submit.
