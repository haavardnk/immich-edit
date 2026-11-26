import { DateTime } from 'luxon';

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

const TRACK_LIMIT = DAY;

function pad(value: number): string {
  return String(value).padStart(2, '0');
}

export function correctionText(ms: number): string {
  const sign = ms < 0 ? '−' : '+';
  const total = Math.round(Math.abs(ms) / SECOND);
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const clock = `${hours}:${pad(minutes)}:${pad(total % 60)}`;
  return `${sign}${days > 0 ? `${days} d ` : ''}${clock}`;
}

export function parseCorrection(text: string, limit = TRACK_LIMIT): number | null {
  const match =
    /^\s*([+\-−]?)\s*(?:(\d{1,6})\s*d\s*)?(\d{1,2})?(?::(\d{1,2}))?(?::(\d{1,2}))?\s*$/.exec(text);
  if (!match) return null;
  const [, sign, days, hours, minutes, seconds] = match;
  if (days === undefined && hours === undefined) return null;
  const [d = 0, h = 0, m = 0, s = 0] = [days, hours, minutes, seconds].map((part) =>
    part === undefined ? 0 : Number(part)
  );
  if (m >= 60 || s >= 60 || (days !== undefined && h >= 24)) return null;
  const ms = (((d * 24 + h) * 60 + m) * 60 + s) * SECOND;
  if (ms > limit) return null;
  return sign === '-' || sign === '−' ? -ms : ms;
}

function shifts(ms: number): boolean {
  return Math.round(Math.abs(ms) / SECOND) > 0;
}

export function cameraShiftText(ms: number): string {
  return shifts(ms) ? correctionText(ms) : 'unchanged';
}

export interface CameraShift {
  label: string;
  ms: number;
}

export function clocksText(cameras: readonly CameraShift[]): string {
  const moved = cameras.filter(({ ms }) => shifts(ms));
  const [only] = moved;
  if (!only) return 'No shift';
  if (cameras.length === 1) return `Shift ${correctionText(only.ms)}`;
  const parts = moved.map(({ label, ms }) => `${label} ${correctionText(ms)}`);
  const still = cameras.length - moved.length;
  if (still > 0) parts.push(`${still} ${still === 1 ? 'camera' : 'cameras'} unchanged`);
  return parts.join(' · ');
}

export function clockOfDay(ms: number, zone: string): string {
  return DateTime.fromMillis(ms, { zone }).toFormat('HH:mm:ss');
}
