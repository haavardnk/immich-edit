import { captureTime, type Dated } from '#lib/utils/exif.ts';

export type DayGroup = {
  key: string;
  start: number;
  end: number;
  place: string | null;
  count: number;
};

const zoneFormats = new Map<string, Intl.DateTimeFormat>();

function zoneFormat(zone: string): Intl.DateTimeFormat {
  let format = zoneFormats.get(zone);
  if (!format) {
    format = new Intl.DateTimeFormat('en-CA', {
      timeZone: zone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hourCycle: 'h23'
    });
    zoneFormats.set(zone, format);
  }
  return format;
}

function zoned(at: Date, zone: string): string {
  const parts = Object.fromEntries(
    zoneFormat(zone)
      .formatToParts(at)
      .map((part) => [part.type, part.value])
  );
  return `${parts.year}-${parts.month}-${parts.day}T${parts.hour}:${parts.minute}:${parts.second}`;
}

export function wallClock(asset: Dated): string {
  const time = captureTime(asset);
  if (!time) return '';
  const { at, timeZone } = time;
  if (timeZone === 'UTC') return at.toISOString().slice(0, 19);
  return zoned(at, timeZone ?? Intl.DateTimeFormat().resolvedOptions().timeZone);
}

export function dayKey(asset: Dated): string {
  return wallClock(asset).slice(0, 10);
}

function mostCommon(values: (string | null | undefined)[]): string | null {
  const tally = new Map<string, number>();
  let best: string | null = null;
  let bestCount = 0;
  for (const value of values) {
    if (!value) continue;
    const count = (tally.get(value) ?? 0) + 1;
    tally.set(value, count);
    if (count > bestCount) {
      best = value;
      bestCount = count;
    }
  }
  return best;
}

type Grouped = Dated & { copyOf?: string };

export function groupDays(assets: readonly Grouped[]): DayGroup[] {
  const groups: DayGroup[] = [];
  let start = 0;
  let key: string | null = null;
  const close = (end: number): void => {
    if (key === null || end <= start) return;
    const day = assets.slice(start, end);
    const cities = day.map((asset) => asset.exifInfo?.city);
    const count = day.filter((asset) => !asset.copyOf).length;
    groups.push({ key, start, end, place: mostCommon(cities), count });
  };
  assets.forEach((asset, index) => {
    const next = dayKey(asset);
    if (next === key) return;
    close(index);
    key = next;
    start = index;
  });
  close(assets.length);
  return groups;
}

export function dayTitle(key: string, today: Date = new Date()): string {
  const [year, month, day] = key.split('-').map(Number);
  if (!year || !month || !day) return 'Unknown date';
  const date = new Date(Date.UTC(year, month - 1, day));
  const options: Intl.DateTimeFormatOptions = {
    timeZone: 'UTC',
    weekday: 'long',
    day: 'numeric',
    month: 'long'
  };
  if (year !== today.getFullYear()) options.year = 'numeric';
  return date.toLocaleDateString('en-GB', options).replace(',', '');
}

export function daySubtitle(group: Pick<DayGroup, 'place' | 'count'>): string {
  return group.place ? `${group.place} · ${group.count}` : String(group.count);
}
