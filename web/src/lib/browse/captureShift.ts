import { zoneOffsetMinutes } from '#lib/geotag/photoTime.ts';

const MINUTE = 60_000;

export interface ShiftSource {
  id: string;
  instant: number;
  recorded: boolean;
}

export interface CaptureShift {
  id: string;
  minutes: number;
  timeZone?: string;
}

export interface ShiftPlan {
  write: CaptureShift[];
  undo: CaptureShift[];
  minutes: number;
  zoned: number;
}

export function wholeMinutes(ms: number): number {
  return Math.round(ms / MINUTE) + 0;
}

export function planShifts(
  sources: readonly ShiftSource[],
  correctionMs: number,
  zone: string
): ShiftPlan {
  const minutes = wholeMinutes(correctionMs);
  const write: CaptureShift[] = [];
  const undo: CaptureShift[] = [];
  let zoned = 0;
  for (const { id, instant, recorded } of sources) {
    if (recorded) {
      if (minutes === 0) continue;
      write.push({ id, minutes });
      undo.push({ id, minutes: -minutes });
      continue;
    }
    const offset = zoneOffsetMinutes(zone, instant + correctionMs);
    write.push({ id, minutes: minutes - offset, timeZone: zone });
    if (minutes !== 0) undo.push({ id, minutes: -minutes });
    zoned += 1;
  }
  return { write, undo, minutes, zoned };
}

export function shiftText(minutes: number): string {
  const total = Math.abs(minutes);
  const hours = Math.floor(total / 60);
  const rest = total % 60;
  const parts = [
    hours > 0 ? `${hours} h` : '',
    rest > 0 ? `${hours > 0 ? String(rest).padStart(2, '0') : rest} min` : ''
  ].filter(Boolean);
  return `${parts.join(' ')} ${minutes < 0 ? 'earlier' : 'later'}`;
}

export function shiftCaption(plan: ShiftPlan, zone: string): string {
  const zoneText =
    plan.zoned > 0 ? `${zone.replaceAll('_', ' ')} for ${plan.zoned} without a zone` : '';
  return [plan.minutes === 0 ? '' : shiftText(plan.minutes), zoneText].filter(Boolean).join(' · ');
}
