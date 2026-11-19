import type { Page } from '@playwright/test';
import { json } from './helpers';

function stats(p50: number, p95: number) {
  return { count: 3, p50_us: p50, p95_us: p95, p99_us: p95, max_us: p95 };
}

const HEALTH = {
  status: 'ok',
  version: '0.6.0',
  renderer_mode: 'auto',
  renderer_active: 'gpu',
  gpu_adapter: 'Test GPU',
  gpu_software: false,
  gpu_memory: {
    device: { bytes: 8 * 2 ** 30, unified: false },
    host_budget_bytes: null,
    budget_bytes: 7.2 * 2 ** 30,
    texture_cache_bytes: 2 ** 30
  },
  host: {
    os: 'linux',
    arch: 'x86_64',
    os_version: null,
    cpu: null,
    cores: 4,
    memory_total_bytes: null,
    memory_limit_bytes: null
  },
  heif_codecs: { hevc_decode: true, hevc_encode: true, av1_decode: true, av1_encode: true },
  immich_reachable: true,
  immich_status: { ok: true, kind: 'ok', message: 'Connected', status_code: null },
  immich_version: '3.2.2',
  db_ready: true,
  db_migration_version: 1,
  config: {}
};

const TIMINGS = {
  renderer_active: 'gpu',
  gpu_timestamps: true,
  render_latency: { cpu: stats(0, 0), gpu: stats(42_000, 61_000) },
  stages: [
    { renderer: 'gpu', stage: 'demosaic', wall: stats(300, 500), gpu: stats(2_100, 2_600) },
    { renderer: 'gpu', stage: 'encode', wall: stats(9_400, 12_000), gpu: null }
  ],
  plans: [
    { renderer: 'gpu', plan: 'single_pass', edge: null, renders: 14, last_tiles: 1 },
    { renderer: 'gpu', plan: 'display_tiled', edge: 2048, renders: 2, last_tiles: 24 }
  ],
  frames: {
    fetch: stats(180_000, 240_000),
    decode: stats(95_000, 130_000),
    cache_hits: 3,
    cache_misses: 1
  },
  request_timeouts: 2,
  gpu_pool_bytes: null,
  cache_bytes: {
    preview_frames_used: 0,
    preview_frames_cap: 0,
    quality_frames_used: 0,
    quality_frames_cap: 0,
    rasters_disk_used: 0,
    rasters_disk_cap: 0
  }
};

export async function mockDiagnostics(page: Page): Promise<void> {
  await page.route('**/api/health', (route) => route.fulfill(json(HEALTH)));
  await page.route('**/api/debug/timings', (route) => route.fulfill(json(TIMINGS)));
}
