import type { GeoPoint } from '#lib/browse/info/photoInfo.ts';

export type CoordinateSource = 'link' | 'text';

export interface ParsedCoordinates extends GeoPoint {
  source: CoordinateSource;
}

const LINK_PAIRS: readonly RegExp[] = [
  /!3d(-?\d+(?:\.\d+)?)!4d(-?\d+(?:\.\d+)?)/,
  /[?&](?:q|query|ll|sll|daddr|destination|coordinate|center)=(-?\d+(?:\.\d+)?)(?:,|%2C)\s*(-?\d+(?:\.\d+)?)/i,
  /\/@(-?\d+(?:\.\d+)?),(-?\d+(?:\.\d+)?)/,
  /[?&]mlat=(-?\d+(?:\.\d+)?)&mlon=(-?\d+(?:\.\d+)?)/,
  /#map=\d+(?:\.\d+)?\/(-?\d+(?:\.\d+)?)\/(-?\d+(?:\.\d+)?)/,
  /^geo:(-?\d+(?:\.\d+)?),(-?\d+(?:\.\d+)?)/i
];

function inRange(point: GeoPoint): boolean {
  return (
    Number.isFinite(point.lat) &&
    Number.isFinite(point.lon) &&
    Math.abs(point.lat) <= 90 &&
    Math.abs(point.lon) <= 180
  );
}

function fromLink(text: string): GeoPoint | null {
  if (/\s/.test(text) || !(/^geo:/i.test(text) || text.includes('/'))) return null;
  for (const pattern of LINK_PAIRS) {
    const match = pattern.exec(text);
    if (!match) continue;
    const point = { lat: Number(match[1]), lon: Number(match[2]) };
    if (!(point.lat === 0 && point.lon === 0) && inRange(point)) return point;
  }
  return null;
}

interface Angle {
  value: number;
  hemisphere: 'N' | 'S' | 'E' | 'W' | null;
}

const HEMI = '([NSEW])?';
const DECIMAL = new RegExp(`^${HEMI}\\s*([-+]?\\d+(?:\\.\\d+)?)\\s*°?\\s*${HEMI}$`, 'i');
const SYMBOLS = new RegExp(
  `^${HEMI}\\s*([-+]?\\d+)\\s*[°º˚]\\s*(?:(\\d+(?:\\.\\d+)?)\\s*['′’]\\s*)?(?:(\\d+(?:\\.\\d+)?)\\s*(?:"|″|”|'')\\s*)?${HEMI}$`,
  'i'
);
const SPACED = new RegExp(
  `^${HEMI}\\s*(\\d+)\\s+(\\d+(?:\\.\\d+)?)(?:\\s+(\\d+(?:\\.\\d+)?))?\\s*${HEMI}$`,
  'i'
);

function hemisphere(before?: string, after?: string): Angle['hemisphere'] | 'both' {
  if (before && after) return 'both';
  const letter = (before ?? after)?.toUpperCase();
  return letter === 'N' || letter === 'S' || letter === 'E' || letter === 'W' ? letter : null;
}

function sexagesimal(match: RegExpExecArray, spaced: boolean): Angle | null {
  const [, before, degrees, minutes, seconds, after] = match;
  const side = hemisphere(before, after);
  if (side === 'both' || degrees === undefined) return null;
  if (spaced && side === null) return null;
  const whole = Number(degrees);
  const min = minutes === undefined ? 0 : Number(minutes);
  const sec = seconds === undefined ? 0 : Number(seconds);
  if (min >= 60 || sec >= 60 || (seconds !== undefined && !Number.isInteger(min))) return null;
  const magnitude = Math.abs(whole) + min / 60 + sec / 3600;
  return { value: degrees.startsWith('-') ? -magnitude : magnitude, hemisphere: side };
}

function angleOf(part: string): Angle | null {
  const text = part.trim();
  const plain = DECIMAL.exec(text);
  if (plain) {
    const side = hemisphere(plain[1], plain[3]);
    return side === 'both' ? null : { value: Number(plain[2]), hemisphere: side };
  }
  const symbols = SYMBOLS.exec(text);
  if (symbols) return sexagesimal(symbols, false);
  const spaced = SPACED.exec(text);
  return spaced ? sexagesimal(spaced, true) : null;
}

function halves(text: string): [string, string] | null {
  const separated = /^(.+?)\s*[,;/]\s*(.+)$/.exec(text);
  if (separated) return [separated[1] ?? '', separated[2] ?? ''];
  const prefixed = /^([NSEW]\s*[^NSEW]+?)\s+([NSEW]\s*[^NSEW]+)$/i.exec(text);
  if (prefixed) return [prefixed[1] ?? '', prefixed[2] ?? ''];
  const lettered = /^(.*?\d[^NSEW]*?[NSEW])\s+(.+)$/i.exec(text);
  if (lettered && /\d/.test(lettered[2] ?? '')) return [lettered[1] ?? '', lettered[2] ?? ''];
  const words = text.split(/\s+/);
  return words.length === 2 ? [words[0] ?? '', words[1] ?? ''] : null;
}

function signed(angle: Angle): number {
  return angle.hemisphere === 'S' || angle.hemisphere === 'W'
    ? -Math.abs(angle.value)
    : angle.value;
}

function isLatitude(angle: Angle): boolean {
  return angle.hemisphere === 'N' || angle.hemisphere === 'S';
}

function isLongitude(angle: Angle): boolean {
  return angle.hemisphere === 'E' || angle.hemisphere === 'W';
}

function fromText(text: string): GeoPoint | null {
  const parts = halves(text.replace(/^\(|\)$/g, '').trim());
  if (!parts) return null;
  const first = angleOf(parts[0]);
  const second = angleOf(parts[1]);
  if (!first || !second) return null;
  const swapped = isLongitude(first) || isLatitude(second);
  const lat = swapped ? second : first;
  const lon = swapped ? first : second;
  if (isLongitude(lat) || isLatitude(lon)) return null;
  const point = { lat: signed(lat), lon: signed(lon) };
  return inRange(point) ? point : null;
}

export function parseCoordinates(input: string): ParsedCoordinates | null {
  const text = input.trim();
  if (text.length < 3 || text.length > 2048) return null;
  const link = fromLink(text);
  if (link) return { ...link, source: 'link' };
  const plain = fromText(text);
  return plain ? { ...plain, source: 'text' } : null;
}
