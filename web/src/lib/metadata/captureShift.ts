import { zoneOffsetMinutes } from '#lib/metadata/photoTime.ts';

const MINUTE = 60_000;

export interface ShiftSource {
  id: string;
  instant: number;
  recorded: boolean;
  correctionMs: number;
}

export interface CaptureShift {
  id: string;
  minutes: number;
  timeZone?: string;
}

export interface ShiftPlan {
  write: CaptureShift[];
  undo: CaptureShift[];
  minutes: number | null;
  zoned: number;
}

export function wholeMinutes(ms: number): number {
  return Math.round(ms / MINUTE) + 0;
}

export function planShifts(sources: readonly ShiftSource[], zone: string): ShiftPlan {
  const write: CaptureShift[] = [];
  const undo: CaptureShift[] = [];
  const moved = new Set<number>();
  let zoned = 0;
  for (const { id, instant, recorded, correctionMs } of sources) {
    const minutes = wholeMinutes(correctionMs);
    if (recorded) {
      if (minutes === 0) continue;
      write.push({ id, minutes });
      undo.push({ id, minutes: -minutes });
      moved.add(minutes);
      continue;
    }
    const offset = zoneOffsetMinutes(zone, instant + correctionMs);
    write.push({ id, minutes: minutes - offset, timeZone: zone });
    if (minutes !== 0) {
      undo.push({ id, minutes: -minutes });
      moved.add(minutes);
    }
    zoned += 1;
  }
  const [only] = moved;
  return { write, undo, minutes: moved.size <= 1 ? (only ?? 0) : null, zoned };
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

export function zoneText(plan: ShiftPlan, zone: string): string {
  return plan.zoned > 0 ? `${zone.replaceAll('_', ' ')} for ${plan.zoned} without a zone` : '';
}

export function shiftCaption(plan: ShiftPlan, zone: string, moves: readonly string[] = []): string {
  const shift =
    plan.minutes === null ? moves.join(' · ') : plan.minutes === 0 ? '' : shiftText(plan.minutes);
  return [shift, zoneText(plan, zone)].filter(Boolean).join(' · ');
}
