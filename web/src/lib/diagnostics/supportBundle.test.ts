import { describe, it, expect } from 'vitest';
import type { DebugTimings, HealthInfo, LatencyStats } from '#lib/api/diagnostics.ts';
import { buildSupportBundle } from './supportBundle';
import { formatBytes, hitRate } from './format';

const stats = (p50: number, p95: number): LatencyStats => ({
  count: 4,
  p50_us: p50,
  p95_us: p95,
  p99_us: p95,
  max_us: p95
});

const health = {
  version: '0.6.0',
  renderer_mode: 'auto',
  renderer_active: 'gpu',
  gpu_adapter: 'Test GPU',
  gpu_software: false,
  gpu_memory: null,
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
  config: { cache_dir: '/cache' }
} as HealthInfo;

const browser = { state: 'browser', reason: null, adapter: 'Apple M2' };

function timings(overrides: Partial<DebugTimings>): DebugTimings {
  return {
    renderer_active: 'gpu',
    gpu_timestamps: false,
    render_latency: { cpu: stats(0, 0), gpu: stats(40_000, 60_000) },
    stages: [],
    plans: [],
    frames: { fetch: stats(0, 0), decode: stats(0, 0), cache_hits: 3, cache_misses: 1 },
    request_timeouts: 2,
    gpu_pool_bytes: null,
    cache_bytes: {
      preview_frames_used: 0,
      preview_frames_cap: 0,
      quality_frames_used: 0,
      quality_frames_cap: 0,
      rasters_disk_used: 0,
      rasters_disk_cap: 0,
      denoise_disk_used: 0,
      denoise_disk_cap: 0
    },
    ...overrides
  };
}

describe('buildSupportBundle', () => {
  it.each([
    {
      name: 'wall only',
      t: timings({
        stages: [{ renderer: 'gpu', stage: 'demosaic', wall: stats(1500, 2500), gpu: null }]
      }),
      expected: ['| Renderer | Stage | p50 | p95 |', '| gpu | demosaic | 1.5ms | 2.5ms |']
    },
    {
      name: 'gpu timestamps',
      t: timings({
        gpu_timestamps: true,
        stages: [
          { renderer: 'gpu', stage: 'demosaic', wall: stats(200, 300), gpu: stats(900, 1200) },
          { renderer: 'gpu', stage: 'encode', wall: stats(4000, 5000), gpu: null }
        ]
      }),
      expected: [
        '| Renderer | Stage | p50 | p95 | GPU p50 | GPU p95 |',
        '| gpu | demosaic | 200µs | 300µs | 900µs | 1.2ms |',
        '| gpu | encode | 4.0ms | 5.0ms | — | — |',
        '- GPU timestamps: on'
      ]
    },
    {
      name: 'render plans',
      t: timings({
        plans: [
          { renderer: 'gpu', plan: 'single_pass', edge: null, renders: 14, last_tiles: 1 },
          { renderer: 'gpu', plan: 'display_tiled', edge: 2048, renders: 2, last_tiles: 24 }
        ]
      }),
      expected: [
        '### Render plans',
        '| Renderer | Plan | Renders | Last tiles |',
        '| gpu | Single pass | 14 | 1 |',
        '| gpu | Display tiles 2048px | 2 | 24 |'
      ]
    },
    {
      name: 'no renders',
      t: timings({}),
      expected: ['No renders recorded yet.', '- GPU timestamps: off']
    }
  ])('writes the stage breakdown ($name)', ({ t, expected }) => {
    const lines = buildSupportBundle(health, t, browser, 'test-agent').split('\n');
    expect(lines).toEqual(expect.arrayContaining(expected));
    expect(lines).toEqual(
      expect.arrayContaining([
        '- Frame cache: 3 hits, 1 misses (75% hit rate)',
        '- Requests cut off by the timeout: 2'
      ])
    );
  });

  it('reports AI denoise disk use and every GPU pool', () => {
    const pools = {
      texture_pool: 4096,
      uniform_pool: 0,
      output_targets: 0,
      sharpen_targets: 0,
      wb_cache: 0,
      nr_cache: 0,
      capture_cache: 1024,
      denoise_cache: 2048,
      atlas_cache: 0,
      atlas_pool: 512,
      total: 7680
    };
    const t = timings({
      gpu_pool_bytes: pools,
      cache_bytes: { ...timings({}).cache_bytes, denoise_disk_used: 1024, denoise_disk_cap: 4096 }
    });
    const lines = buildSupportBundle(health, t, browser, 'ua').split('\n');
    const pool = lines.find((line) => line.startsWith('- GPU pool total:')) ?? '';
    expect(lines).toContain(
      `- AI denoise planes on disk: ${formatBytes(1024)} / ${formatBytes(4096)}`
    );
    for (const name of ['capture_cache', 'denoise_cache', 'atlas_cache', 'atlas_pool']) {
      expect(pool).toContain(name);
    }
  });

  it.each([
    {
      name: 'a probed unified gpu',
      memory: {
        device: { bytes: 32 * 2 ** 30, unified: true },
        host_budget_bytes: 16 * 2 ** 30,
        budget_bytes: 16 * 2 ** 30,
        texture_cache_bytes: 2 * 2 ** 30
      },
      expected: [
        '- GPU memory: 32.00GiB unified',
        '- GPU budget: 16.00GiB',
        '- Host budget: 16.00GiB',
        '- GPU texture cache: 2.00GiB'
      ]
    },
    {
      name: 'an unprobed gpu',
      memory: {
        device: null,
        host_budget_bytes: null,
        budget_bytes: null,
        texture_cache_bytes: 512 * 2 ** 20
      },
      expected: [
        '- GPU memory: not reported',
        '- GPU budget: none',
        '- Host budget: none',
        '- GPU texture cache: 512.0MiB'
      ]
    }
  ])('reports gpu memory for $name', ({ memory, expected }) => {
    const lines = buildSupportBundle({ ...health, gpu_memory: memory }, null, browser, 'ua').split(
      '\n'
    );
    expect(lines).toEqual(expect.arrayContaining(expected));
  });

  it('notes missing timings', () => {
    expect(buildSupportBundle(health, null, browser, 'ua')).toContain(
      'Render timings: unavailable.'
    );
  });

  it('folds into a details block with the Immich and browser renderer', () => {
    const fallback = { state: 'server', reason: 'no WebGPU', adapter: null };
    const lines = buildSupportBundle(health, null, fallback, 'ua').split('\n');
    expect(lines[0]).toBe('<details><summary>immich-edit 0.6.0 support bundle</summary>');
    expect(lines.at(-1)).toBe('</details>');
    expect(lines).toEqual(
      expect.arrayContaining([
        '- Immich version: 3.2.2',
        '- Preview renderer: server (no WebGPU)',
        '- Browser GPU adapter: none'
      ])
    );
  });
});

describe('hitRate', () => {
  it.each([
    [0, 0, '—'],
    [1, 2, '33%'],
    [5, 0, '100%']
  ])('%i hits and %i misses give %s', (hits, misses, expected) => {
    expect(hitRate(hits, misses)).toBe(expected);
  });
});
