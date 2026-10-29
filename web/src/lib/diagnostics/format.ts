import type { GpuMemory } from '$lib/api/diagnostics';

export function formatUs(us: number): string {
  if (us === 0) return '—';
  if (us < 1000) return `${us}µs`;
  return `${(us / 1000).toFixed(1)}ms`;
}

export function formatBytes(b: number): string {
  if (b < 1024) return `${b}B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)}KiB`;
  if (b < 1024 * 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)}MiB`;
  return `${(b / 1024 / 1024 / 1024).toFixed(2)}GiB`;
}

export function codecLabel(ok: boolean): string {
  return ok ? 'yes' : 'missing';
}

export function hitRate(hits: number, misses: number): string {
  const total = hits + misses;
  if (total === 0) return '—';
  return `${Math.round((hits / total) * 100)}%`;
}

export function perMegapixel(ms: number): string {
  return `${ms.toFixed(1)} ms/MP`;
}

export function gpuMemoryRows(memory: GpuMemory): [string, string][] {
  const { device, budget_bytes, host_budget_bytes, texture_cache_bytes } = memory;
  return [
    [
      'GPU memory',
      device
        ? `${formatBytes(device.bytes)} ${device.unified ? 'unified' : 'dedicated'}`
        : 'not reported'
    ],
    ['GPU budget', budget_bytes === null ? 'none' : formatBytes(budget_bytes)],
    ['Host budget', host_budget_bytes === null ? 'none' : formatBytes(host_budget_bytes)],
    ['GPU texture cache', formatBytes(texture_cache_bytes)]
  ];
}
