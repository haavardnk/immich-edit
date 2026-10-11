const BYTES_PER_MB = 1_000_000;

export function megabytes(value: string | null): number | null {
  if (!value?.trim()) return null;
  const mb = Number(value);
  return Number.isFinite(mb) && mb > 0 ? mb : null;
}

export function toBytes(mb: number): number {
  return Math.round(mb * BYTES_PER_MB);
}
