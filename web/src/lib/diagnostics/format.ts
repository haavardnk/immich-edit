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
