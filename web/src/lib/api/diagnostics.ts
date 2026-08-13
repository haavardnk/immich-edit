import { getJson } from './client';

export interface HeifCodecs {
  hevc_decode: boolean;
  hevc_encode: boolean;
  av1_decode: boolean;
  av1_encode: boolean;
}

export interface RedactedConfig {
  bind_addr: string;
  data_dir: string;
  cache_dir: string;
  preview_max_edge: number;
  render_max_concurrency: number;
  thumb_max_concurrency: number;
  mask_cache_mb: number;
  embedding_cache_mb: number;
  raw_frame_cache_mb: number;
  quality_frame_cache_mb: number;
  gpu_texture_cache_mb: number;
  renderer: string;
  allowed_origins: string[];
  max_body_mb: number;
  request_timeout_secs: number;
  original_timeout_secs: number;
  export_timeout_secs: number;
  ml_runtime: string;
  ml_max_edge: number;
  ml_max_concurrency: number;
  ml_idle_secs: number;
}

export interface HealthInfo {
  status: string;
  version: string;
  renderer_mode: string;
  renderer_active: string;
  gpu_adapter: string | null;
  heif_codecs: HeifCodecs;
  immich_reachable: boolean;
  immich_status: ImmichConnectionStatus;
  db_ready: boolean;
  db_migration_version: number | null;
  config: RedactedConfig;
}

export interface ImmichConnectionStatus {
  ok: boolean;
  kind: string;
  message: string;
  status_code: number | null;
}

export interface LatencyStats {
  count: number;
  p50_us: number;
  p95_us: number;
  p99_us: number;
  max_us: number;
}

export interface GpuPoolBytes {
  texture_pool: number;
  uniform_pool: number;
  output_targets: number;
  sharpen_targets: number;
  wb_cache: number;
  nr_cache: number;
  atlas_cache: number;
  total: number;
}

export interface CacheBytes {
  preview_frames_used: number;
  preview_frames_cap: number;
  quality_frames_used: number;
  quality_frames_cap: number;
  rasters_disk_used: number;
  rasters_disk_cap: number;
}

export interface DebugTimings {
  renderer_active: string;
  render_latency: { cpu: LatencyStats; gpu: LatencyStats };
  gpu_pool_bytes: GpuPoolBytes | null;
  cache_bytes: CacheBytes;
}

export function getHealth(): Promise<HealthInfo> {
  return getJson('/api/health');
}

export function getDebugTimings(): Promise<DebugTimings> {
  return getJson('/api/debug/timings');
}
