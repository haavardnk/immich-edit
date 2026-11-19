const BYTES_PER_MB = 1_000_000;

export function megabytes(value: string | null): number | null {
  if (!value?.trim()) return null;
  const mb = Number(value);
  return Number.isFinite(mb) && mb > 0 ? mb : null;
}

export function toBytes(mb: number): number {
  return Math.round(mb * BYTES_PER_MB);
}

export interface SizePreset {
  id: string;
  label: string;
  min: number | null;
  max: number | null;
}

export const SIZE_PRESETS: readonly SizePreset[] = [
  { id: 'under-1', label: '< 1 MB', min: null, max: 1 },
  { id: '1-10', label: '1–10 MB', min: 1, max: 10 },
  { id: '10-50', label: '10–50 MB', min: 10, max: 50 },
  { id: 'over-50', label: '> 50 MB', min: 50, max: null }
];

export function sizePreset(min: number | null, max: number | null): SizePreset | undefined {
  return SIZE_PRESETS.find((preset) => preset.min === min && preset.max === max);
}

export function sizeLabel(min: number | null, max: number | null): string {
  const preset = sizePreset(min, max);
  if (preset) return preset.label;
  if (min !== null && max !== null) return `${min}–${max} MB`;
  if (min !== null) return `≥ ${min} MB`;
  return `≤ ${max ?? 0} MB`;
}
