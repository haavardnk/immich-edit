import type { ExifInfo } from '$lib/types/asset';

export const DEFAULT_FILENAME_TEMPLATE = '{name}_edit';
export const FILENAME_TOKENS = ['{name}', '{date}', '{seq}'] as const;

const MAX_TEMPLATE_CHARS = 64;
const FALLBACK_STEM = 'export';
const UNDATED = 'undated';
const RESERVED = '/\\:*?"<>|';
const RFC3339 = /^(\d{4}-\d{2}-\d{2})T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/i;

type Token = 'name' | 'date' | 'seq';
type Part = { text: string } | { token: Token };

export interface NameContext {
  original: string;
  date: string | null;
  position: number;
  total: number;
}

function isToken(value: string): value is Token {
  return value === 'name' || value === 'date' || value === 'seq';
}

function forbidden(char: string): boolean {
  const code = char.codePointAt(0) ?? 0;
  return code <= 0x1f || (code >= 0x7f && code <= 0x9f) || RESERVED.includes(char);
}

function parse(raw: string): Part[] {
  const source = raw.trim() || DEFAULT_FILENAME_TEMPLATE;
  if ([...source].length > MAX_TEMPLATE_CHARS) throw new Error('Filename template is too long');
  if ([...source].some(forbidden)) {
    throw new Error('Filename template contains a character that is not allowed in file names');
  }
  const parts: Part[] = [];
  let rest = source;
  let at = rest.search(/[{}]/);
  while (at >= 0) {
    if (at > 0) parts.push({ text: rest.slice(0, at) });
    if (rest[at] === '}') throw new Error('Filename template has an unmatched }');
    const close = rest.indexOf('}', at);
    if (close < 0) throw new Error('Filename template has an unmatched {');
    const name = rest.slice(at + 1, close);
    if (!isToken(name)) throw new Error(`Unknown filename token {${name}}`);
    parts.push({ token: name });
    rest = rest.slice(close + 1);
    at = rest.search(/[{}]/);
  }
  if (rest) parts.push({ text: rest });
  return parts;
}

export function templateError(raw: string): string | null {
  try {
    parse(raw);
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : String(e);
  }
}

function cleanStem(original: string): string {
  const dot = original.lastIndexOf('.');
  const stem = dot < 0 ? original : original.slice(0, dot);
  return [...stem].map((char) => (forbidden(char) ? '_' : char)).join('');
}

function renderPart(part: Part, ctx: NameContext): string {
  if ('text' in part) return part.text;
  if (part.token === 'name') return cleanStem(ctx.original);
  if (part.token === 'date') return ctx.date ?? UNDATED;
  return String(ctx.position).padStart(String(Math.max(1, ctx.total)).length, '0');
}

export function renderTemplate(raw: string, ctx: NameContext): string {
  const rendered = parse(raw)
    .map((part) => renderPart(part, ctx))
    .join('')
    .replace(/^[\s.]+|[\s.]+$/g, '');
  return rendered || FALLBACK_STEM;
}

export function captureDate(asset: {
  localDateTime?: string | null;
  exifInfo: ExifInfo | null;
  fileCreatedAt: string | null;
}): string | null {
  const candidates = [asset.localDateTime, asset.exifInfo?.dateTimeOriginal, asset.fileCreatedAt];
  for (const value of candidates) {
    const match = value ? RFC3339.exec(value) : null;
    if (match?.[1]) return match[1];
  }
  return null;
}
