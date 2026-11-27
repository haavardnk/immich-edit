import { zoneName } from '#lib/metadata/captureLabels.ts';
import { correctionText } from '#lib/metadata/clockText.ts';

const HALF_SECOND = 500;

export interface CameraChange {
  label: string;
  change: string;
}

export function shifts(ms: number): boolean {
  return Math.abs(ms) >= HALF_SECOND;
}

export function changeText(zone: string | null, shiftMs: number): string {
  return [zone ? zoneName(zone) : '', shifts(shiftMs) ? correctionText(shiftMs) : '']
    .filter(Boolean)
    .join(' · ');
}

export function camerasText(cameras: readonly CameraChange[]): string {
  const changed = cameras.filter((camera) => camera.change !== '');
  const [only] = changed;
  if (!only) return 'No change';
  if (cameras.length === 1) return only.change;
  const parts = changed.map(({ label, change }) => `${label} ${change}`);
  const still = cameras.length - changed.length;
  if (still > 0) parts.push(`${still} unchanged`);
  return parts.join(' · ');
}
