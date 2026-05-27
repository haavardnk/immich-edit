import { getJson } from './client';

export interface HealthInfo {
  status: string;
  version: string;
  renderer_mode: string;
  renderer_active: string;
  gpu_adapter: string | null;
  immich_reachable: boolean;
  db_ready: boolean;
  db_migration_version: number | null;
  config: Record<string, unknown>;
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

export interface DebugTimings {
  renderer_active: string;
  render_latency: { cpu: LatencyStats; gpu: LatencyStats };
  gpu_pool_bytes: GpuPoolBytes | null;
}

export function getHealth(): Promise<HealthInfo> {
  return getJson('/api/health');
}

export function getDebugTimings(): Promise<DebugTimings> {
  return getJson('/api/debug/timings');
}
