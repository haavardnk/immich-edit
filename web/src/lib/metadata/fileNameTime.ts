export interface NameTime {
  source: string;
  wall: number;
  dateOnly: boolean;
  utc: boolean;
}

interface Pattern {
  source: string;
  regex: RegExp;
  dateOnly?: boolean;
  utc?: boolean;
}

const YEARS = { min: 1990, max: 2100 };
const DAY = 86_400_000;

const PATTERNS: readonly Pattern[] = [
  {
    source: 'WhatsApp',
    regex: /^(?:IMG|VID|PTT|AUD)-(?<y>\d{4})(?<mo>\d{2})(?<d>\d{2})-WA\d+/i,
    dateOnly: true
  },
  {
    source: 'Pixel',
    regex:
      /^PXL_(?<y>\d{4})(?<mo>\d{2})(?<d>\d{2})_(?<h>\d{2})(?<mi>\d{2})(?<s>\d{2})(?<ms>\d{3})?/i,
    utc: true
  },
  {
    source: 'Signal',
    regex:
      /^signal-(?<y>\d{4})-(?<mo>\d{2})-(?<d>\d{2})-(?<h>\d{2})-?(?<mi>\d{2})-?(?<s>\d{2})(?:-(?<ms>\d{3}))?/i
  },
  {
    source: 'Screenshot',
    regex:
      /^Screen ?shot[_-](?<y>\d{4})-?(?<mo>\d{2})-?(?<d>\d{2})[_-](?<h>\d{2})[.\-:]?(?<mi>\d{2})[.\-:]?(?<s>\d{2})/i
  },
  {
    source: 'Screenshot',
    regex:
      /^Screen ?shot (?<y>\d{4})-(?<mo>\d{2})-(?<d>\d{2}) at (?<h>\d{1,2})[.:](?<mi>\d{2})[.:](?<s>\d{2})(?:[\s\u202f]?(?<ampm>AM|PM))?/i
  },
  {
    source: 'Date and time',
    regex:
      /^(?:IMG|VID|MVIMG|PANO|BURST)?_?(?<y>\d{4})(?<mo>\d{2})(?<d>\d{2})[_-](?<h>\d{2})(?<mi>\d{2})(?<s>\d{2})(?<ms>\d{3})?/i
  },
  {
    source: 'Date and time',
    regex: /^(?<y>\d{4})-(?<mo>\d{2})-(?<d>\d{2})[ _](?<h>\d{2})[.\-:](?<mi>\d{2})[.\-:](?<s>\d{2})/
  }
];

function hourOf(hour: number, ampm: string | undefined): number | null {
  if (!ampm) return hour;
  if (hour < 1 || hour > 12) return null;
  return (hour % 12) + (ampm.toUpperCase() === 'PM' ? 12 : 0);
}

function wallOf(groups: Record<string, string | undefined>): number | null {
  const part = (key: string): number => Number(groups[key] ?? 0);
  const h = hourOf(part('h'), groups.ampm);
  const [y, mo, d, mi, s] = [part('y'), part('mo'), part('d'), part('mi'), part('s')];
  if (h === null || y < YEARS.min || y > YEARS.max || h > 23 || mi > 59 || s > 59) return null;
  const wall = Date.UTC(y, mo - 1, d, h, mi, s, part('ms'));
  const back = new Date(wall);
  if (back.getUTCMonth() !== mo - 1 || back.getUTCDate() !== d) return null;
  return wall;
}

export function nameTime(name: string): NameTime | null {
  for (const { source, regex, dateOnly = false, utc = false } of PATTERNS) {
    const groups = regex.exec(name)?.groups;
    if (!groups) continue;
    const wall = wallOf(groups);
    if (wall !== null) return { source, wall, dateOnly, utc };
  }
  return null;
}

export function withTimeOfDay(day: number, timeOf: number): number {
  return Math.floor(day / DAY) * DAY + (((timeOf % DAY) + DAY) % DAY);
}
