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

export function driftText(ms: number): string {
  const total = Math.round(Math.abs(ms) / SECOND);
  if (total === 0) return 'no correction';
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const parts = [
    days > 0 ? `${days} d` : '',
    hours > 0 ? `${hours} h` : '',
    minutes > 0 ? `${pad(minutes)} min` : '',
    seconds > 0 ? `${pad(seconds)} s` : ''
  ].filter(Boolean);
  return `${parts.join(' ')} ${ms < 0 ? 'fast' : 'slow'}`;
}

export function clockDrift(ms: number): string {
  return Math.round(Math.abs(ms) / SECOND) === 0 ? 'No correction' : `Camera ${driftText(ms)}`;
}

export interface CameraDrift {
  label: string;
  ms: number;
}

export function clocksText(cameras: readonly CameraDrift[]): string {
  if (cameras.length <= 1) return clockDrift(cameras[0]?.ms ?? 0);
  const corrected = cameras.filter(({ ms }) => Math.round(Math.abs(ms) / SECOND) > 0);
  if (corrected.length === 0) return 'No correction';
  const unchanged = cameras.length - corrected.length;
  const parts = corrected.map(({ label, ms }) => `${label} ${driftText(ms)}`);
  if (unchanged > 0) parts.push(`${unchanged} ${unchanged === 1 ? 'camera' : 'cameras'} unchanged`);
  return parts.join(' · ');
}

export function clockOfDay(ms: number, zone: string): string {
  return DateTime.fromMillis(ms, { zone }).toFormat('HH:mm:ss');
}
