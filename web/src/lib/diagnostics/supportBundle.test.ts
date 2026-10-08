import { describe, it, expect } from 'vitest';
import type { DebugTimings, HealthInfo, LatencyStats } from '$lib/api/diagnostics';
import { buildSupportBundle } from './supportBundle';
import { hitRate } from './format';

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
    frames: { fetch: stats(0, 0), decode: stats(0, 0), cache_hits: 3, cache_misses: 1 },
    request_timeouts: 2,
    gpu_pool_bytes: null,
    cache_bytes: {
      preview_frames_used: 0,
      preview_frames_cap: 0,
      quality_frames_used: 0,
      quality_frames_cap: 0,
      rasters_disk_used: 0,
      rasters_disk_cap: 0
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

  it('notes missing timings', () => {
    expect(buildSupportBundle(health, null, browser, 'ua')).toContain(
      'Render timings: unavailable.'
    );
  });

  it('folds into a details block with the Immich and browser renderer', () => {
    const fallback = { state: 'server', reason: 'WebGPU is unavailable', adapter: null };
    const lines = buildSupportBundle(health, null, fallback, 'ua').split('\n');
    expect(lines[0]).toBe('<details><summary>immich-edit 0.6.0 support bundle</summary>');
    expect(lines.at(-1)).toBe('</details>');
    expect(lines).toEqual(
      expect.arrayContaining([
        '- Immich version: 3.2.2',
        '- Browser previews: server (WebGPU is unavailable)',
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
