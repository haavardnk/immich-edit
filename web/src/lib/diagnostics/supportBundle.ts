import type { DebugTimings, HealthInfo, LatencyStats, StageStats } from '$lib/api/diagnostics';
import { codecLabel, formatBytes, formatUs, hitRate } from './format';

function latencyRow(name: string, s: LatencyStats): string {
  return `| ${name} | ${s.count} | ${formatUs(s.p50_us)} | ${formatUs(s.p95_us)} | ${formatUs(s.p99_us)} | ${formatUs(s.max_us)} |`;
}

function stageRows(stages: StageStats[], withGpu: boolean): string[] {
  const header = withGpu
    ? ['| Renderer | Stage | p50 | p95 | GPU p50 | GPU p95 |', '|---|---|---|---|---|---|']
    : ['| Renderer | Stage | p50 | p95 |', '|---|---|---|---|'];
  const rows = stages.map((s) => {
    const base = `| ${s.renderer} | ${s.stage} | ${formatUs(s.wall.p50_us)} | ${formatUs(s.wall.p95_us)} |`;
    if (!withGpu) return base;
    return `${base} ${s.gpu ? formatUs(s.gpu.p50_us) : '—'} | ${s.gpu ? formatUs(s.gpu.p95_us) : '—'} |`;
  });
  return [...header, ...rows];
}

function timingLines(t: DebugTimings): string[] {
  const lines: string[] = [];
  lines.push('### Render latency');
  lines.push('');
  lines.push('| Renderer | Count | p50 | p95 | p99 | max |');
  lines.push('|---|---|---|---|---|---|');
  lines.push(latencyRow('cpu', t.render_latency.cpu));
  lines.push(latencyRow('gpu', t.render_latency.gpu));
  lines.push('');
  lines.push('### Render stages');
  lines.push('');
  if (t.stages.length === 0) {
    lines.push('No renders recorded yet.');
  } else {
    lines.push(
      ...stageRows(
        t.stages,
        t.stages.some((s) => s.gpu !== null)
      )
    );
  }
  lines.push('');
  lines.push(`- GPU timestamps: ${t.gpu_timestamps ? 'on' : 'off'}`);
  lines.push(
    `- Original fetch: p50 ${formatUs(t.frames.fetch.p50_us)}, p95 ${formatUs(t.frames.fetch.p95_us)}`
  );
  lines.push(
    `- Decode: p50 ${formatUs(t.frames.decode.p50_us)}, p95 ${formatUs(t.frames.decode.p95_us)}`
  );
  lines.push(
    `- Frame cache: ${t.frames.cache_hits} hits, ${t.frames.cache_misses} misses (${hitRate(t.frames.cache_hits, t.frames.cache_misses)} hit rate)`
  );
  lines.push(`- Requests cut off by the timeout: ${t.request_timeouts}`);
  lines.push(
    `- Preview frames: ${formatBytes(t.cache_bytes.preview_frames_used)} / ${formatBytes(t.cache_bytes.preview_frames_cap)}`
  );
  lines.push(
    `- Quality frames: ${formatBytes(t.cache_bytes.quality_frames_used)} / ${formatBytes(t.cache_bytes.quality_frames_cap)}`
  );
  lines.push(
    `- Mask rasters on disk: ${formatBytes(t.cache_bytes.rasters_disk_used)} / ${formatBytes(t.cache_bytes.rasters_disk_cap)}`
  );
  if (t.gpu_pool_bytes) {
    lines.push(`- GPU pool total: ${formatBytes(t.gpu_pool_bytes.total)}`);
  }
  return lines;
}

export function buildSupportBundle(
  h: HealthInfo,
  t: DebugTimings | null,
  userAgent: string
): string {
  const statusCode = h.immich_status.status_code ? ` HTTP ${h.immich_status.status_code}` : '';
  const lines: string[] = [];
  lines.push('## immich-edit support bundle');
  lines.push('');
  lines.push(`- Version: ${h.version}`);
  lines.push(
    `- Host: ${h.host.os} ${h.host.arch}${h.host.os_version ? ` (${h.host.os_version})` : ''}`
  );
  lines.push(`- CPU: ${h.host.cpu ?? 'unknown'}, ${h.host.cores} cores available`);
  lines.push(
    `- Memory: ${h.host.memory_total_bytes ? formatBytes(h.host.memory_total_bytes) : 'unknown'} total${h.host.memory_limit_bytes ? `, ${formatBytes(h.host.memory_limit_bytes)} container limit` : ''}`
  );
  lines.push(`- Renderer mode: ${h.renderer_mode}`);
  lines.push(`- Renderer active: ${h.renderer_active}`);
  lines.push(`- GPU adapter: ${h.gpu_adapter ?? 'none'}`);
  if (h.gpu_software) {
    lines.push('- GPU adapter is a software rasterizer (no hardware GPU found)');
  }
  lines.push(
    `- HEIF codecs: hevc decode ${codecLabel(h.heif_codecs.hevc_decode)}, hevc encode ${codecLabel(h.heif_codecs.hevc_encode)}, av1 decode ${codecLabel(h.heif_codecs.av1_decode)}, av1 encode ${codecLabel(h.heif_codecs.av1_encode)}`
  );
  lines.push(`- Immich status: ${h.immich_status.kind}${statusCode} (${h.immich_status.message})`);
  lines.push(`- DB ready: ${h.db_ready} (migration ${h.db_migration_version ?? '—'})`);
  lines.push(`- Cache dir: ${h.config.cache_dir}`);
  lines.push(`- User agent: ${userAgent}`);
  lines.push('');
  if (t) {
    lines.push(...timingLines(t));
  } else {
    lines.push('Render timings: unavailable.');
  }
  lines.push('');
  lines.push('### Redacted config');
  lines.push('');
  lines.push('```json');
  lines.push(JSON.stringify(h.config, null, 2));
  lines.push('```');
  return lines.join('\n');
}
