import type { Model } from '$lib/api/models';

const KIND_LABELS: Record<string, string> = {
  subject: 'Subject',
  people: 'People',
  sky: 'Sky',
  depth: 'Depth',
  semantic: 'Scene classes',
  click: 'Click to select',
  denoise: 'AI Denoise'
};

export function kindLabel(kind: string): string {
  return KIND_LABELS[kind] ?? kind;
}

export function installPercent(m: Model): number {
  if (m.size_bytes <= 0) return 0;
  return Math.min(100, Math.round((m.progress_bytes / m.size_bytes) * 100));
}

export function formatSeconds(ms: number): string {
  return ms >= 1000 ? `${(ms / 1000).toFixed(1)} s` : `${ms} ms`;
}

export function formatMb(bytes: number): string {
  return `${Math.round(bytes / 1_000_000)} MB`;
}
