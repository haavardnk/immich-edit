# Raw Pipeline

This is the contributor map for render-pass ownership. Keep it in sync when an operator changes stage, gains GPU support, or moves between passes.

## Operator model

`crates/raw-pipeline/src/ops.rs` defines shared metadata plus three operator roles. The registry stores them as `AnyOp` and sorts by `(stage, order)`.

Stages run in this order:

`Sensor -> WhiteBalance -> Tone -> Color -> Geometry -> Output`

Every operator implements `OpMeta`:

- `id() -> &'static str`
- `stage() -> Stage`
- `order() -> i32`
- `is_active(&Edits) -> bool`
- `to_doc` / `from_doc` for sidecar persistence

`FusedOp` is for pointwise work. It returns a `CpuFusedOp`, can contribute WGSL through `gpu()`, and is batched into generated CPU/GPU process passes. Exposure, contrast, curves, HSL, white balance, and color matrix live here.

`SpatialOp` is for neighbourhood or geometry work. It has an `apply_cpu` implementation and may opt into `GpuOpKind::Presence` or `GpuOpKind::Detail`. `GpuOpKind::Normal` is the default; active normal ops in the GPU process pass must return `gpu()` unless the renderer handles them specially.

`OutputStageOp` currently only wraps the persisted output settings. CPU output effects are separate `SpatialOp`s with `Stage::Output`; final tone/output conversion happens after `run_output_ops` in `finish_output` and encode.

`GpuOpKind` currently has three values:

- `Normal` - generated into the process pass
- `Presence` - handled by the presence/luma-pyramid path
- `Detail` - handled by dedicated detail passes

`dehaze` is a special case: it is a `SpatialOp` with no generated WGSL contribution. The GPU renderer detects `op.id() == "dehaze"` and runs `passes/dehaze.rs` when `basic.dehaze` is active.

## CPU path

Main entry: `cpu::render_with_cancel` in `crates/raw-pipeline/src/cpu/pipeline.rs`.

Flow:

1. Demosaic RAW frames when needed.
2. Resolve white balance and camera-to-sRGB matrix into `OpContext`.
3. Run `Sensor` ops through `run_sensor_ops` when lens edits are active.
4. Apply EXIF orientation.
5. Run edit ops through `run_pipeline_ops`.
6. Resize to `RenderOptions::max_edge`.
7. Run `Stage::Output` ops through `run_output_ops`.
8. Run `finish_output` and encode.

`run_pipeline_ops` batches consecutive `FusedOp`s into `FusedSegment`s, then flushes before CPU spatial work. Mask layers build their own effective edits and run through masked fused segments so each layer can apply a different local adjustment set.

Mask preview mode is a separate path: `render_mask_weight` writes the mask view, then only active `Geometry` ops run.

Current CPU detail behaviour is important: `GpuOpKind::Detail` ops are skipped in `run_pipeline_ops`. Today that means luma NR, color NR, and sharpen are GPU-owned in the normal render path. If CPU parity for those edits becomes required, change the code and this document together.

## GPU path

Main entry: `GpuRenderer::render_with_cancel` in `crates/raw-pipeline/src/gpu/renderer.rs`.

`RenderPlan::select` chooses:

- `Fast` - no presence inputs. Upload/demosaic plus process pass is enough.
- `Presence` - runs white-balance prep and dedicated detail/presence/dehaze paths before process.

Dispatch order:

| Order | Pass | Source | Owns |
|---|---|---|---|
| 1 | upload / demosaic | `get_or_demosaic`, `passes/demosaic.rs` | RAW or RGB input to linear `Rgba16Float`; mipgen for source textures. |
| 2 | sensor | `run_sensor`, `passes/sensor.rs` | Active lens sensor ops before orientation/crop sampling. |
| 3 | wb_prepare | `run_wb_prepare`, `passes/wb_prepare.rs` | White balance plus camera-to-sRGB pre-pass for presence/detail work. |
| 4 | NR | `run_nr`, `passes/nr.rs`, `passes/nr_smooth.rs` | `luma_nr`, `color_nr` detail work. |
| 5 | dehaze | `atmosphere_for`, `run_dehaze`, `passes/dehaze.rs` | Atmosphere estimate and DCP/guided-filter dehaze. |
| 6 | presence | `run_presence`, `passes/presence.rs`, `passes/luma_pyramid.rs` | Texture and clarity. Shadows builds a luma pyramid later when needed. |
| 7 | process | `process`, `passes/process.rs`, generated `process.wgsl` | Pointwise ops, crop/rotate/flip/angle sampling, fast-path tone. |
| 8 | masks | `passes/mask_weight.rs`, `passes/mask_blend.rs` | Per-layer mask weight and local adjustment blend. |
| 9 | sharpen | `encode_sharpen`, `passes/sharpen.rs` | Sharpen and sharpen preview modes. |
| 10 | effects + tone | `encode_effects_tone`, `passes/effects_tone.rs` | Vignette, grain, final tone when final pass is active. |
| 11 | readback / encode | `gpu/readback.rs`, `encode::encode_from_rgba8` | RGBA readback, histogram, JPEG/other output encode. |

## Color-space rules

Intermediate GPU textures from upload through `process` are linear scene-referred sRGB in `Rgba16Float`. Tone mapping happens once:

- In `process.wgsl` for the fast path with no sharpen/effects/masks.
- In `effects_tone.wgsl` whenever sharpen, vignette, grain, or masks require the final pass.

There is no type-level distinction between linear and gamma-encoded textures. The one-line `color-space:` headers in pass files are the current guardrail; review new passes carefully.

## Effect ownership

`vignette` and `grain` are `SpatialOp`s with `Stage::Output`. On CPU they run in `run_output_ops` before `finish_output`. On GPU they are baked into `effects_tone.wgsl` and run before tone mapping.

`transform` is a generated process-pass contribution on GPU. On CPU it is a normal `SpatialOp` in `Stage::Geometry`.

`masks` is not active as a normal registry op. CPU masks are handled inside `run_pipeline_ops`; GPU masks are handled inside `process` with `mask_weight` and `mask_blend` passes.

Lens ownership is split. `lens_vignette` is handled by `passes/sensor.rs` on GPU. Lens distortion and chromatic aberration have CPU implementations; the GPU path still uses lens warp parameters for mask sampling.

## Adding an operator

1. Pick a role:
   - Pointwise: implement `FusedOp`, return `cpu_fused`, add `gpu()` if GPU should support it in the generated process pass.
   - Spatial CPU-only: implement `SpatialOp::apply_cpu`. Make sure the GPU path either rejects it clearly or has an equivalent dedicated path.
   - Spatial GPU detail/presence: implement `SpatialOp`, return `GpuOpKind::Detail` or `GpuOpKind::Presence`, then wire the renderer pass.
   - Output setting: implement `OutputStageOp` only for persisted output options.
2. Register it in `default_registry()`.
3. Add sidecar round-trip coverage in `ops/tests.rs` when it persists edits.
4. Add render-path coverage in CPU/GPU tests when it changes pixels.
5. Update the dispatch table above.

## Tracing and cancellation

Top-level CPU and GPU boundaries have tracing spans such as `cpu.pipeline_ops`, `gpu.upload_rgb`, `gpu.demosaic`, `gpu.mipgen`, `gpu.run_nr`, `gpu.run_presence`, `gpu.run_wb_prepare`, `gpu.run_sensor`, `gpu.encode_effects_tone`, `gpu_dehaze`, and `gpu_dehaze_atm`.

`CancelToken` is checked between CPU-side boundaries in both renderers. Long GPU readbacks use `map_buffer_cancellable`, which polls the device and returns `PipelineError::Cancelled` when the token flips.

Once submitted to the GPU queue, a sub-pass runs to completion. Cancellation is honoured at the next CPU checkpoint after submit.