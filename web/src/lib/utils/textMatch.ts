export interface TextRange {
  start: number;
  end: number;
}

export interface TextPart {
  text: string;
  match: boolean;
}

export function foldText(text: string): string {
  return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
}

interface FoldedText {
  text: string;
  starts: number[];
  ends: number[];
}

function foldWithOrigins(text: string): FoldedText {
  let folded = '';
  const starts: number[] = [];
  const ends: number[] = [];
  let index = 0;
  for (const point of text) {
    const next = index + point.length;
    const piece = foldText(point);
    if (piece === '' && ends.length > 0) ends[ends.length - 1] = next;
    for (let k = 0; k < piece.length; k++) {
      starts.push(index);
      ends.push(next);
    }
    folded += piece;
    index = next;
  }
  return { text: folded, starts, ends };
}

export function textMatches(text: string, query: string): boolean {
  const needle = foldText(query.trim());
  return needle === '' || foldText(text).includes(needle);
}

export function matchRanges(text: string, query: string): TextRange[] {
  const needle = foldText(query.trim());
  if (needle === '') return [];
  const folded = foldWithOrigins(text);
  const ranges: TextRange[] = [];
  let from = folded.text.indexOf(needle);
  while (from >= 0) {
    const last = from + needle.length - 1;
    ranges.push({ start: folded.starts[from] ?? 0, end: folded.ends[last] ?? text.length });
    from = folded.text.indexOf(needle, from + needle.length);
  }
  return ranges;
}

export function highlightParts(text: string, ranges: readonly TextRange[]): TextPart[] {
  const parts: TextPart[] = [];
  let at = 0;
  for (const { start, end } of ranges) {
    if (start > at) parts.push({ text: text.slice(at, start), match: false });
    if (end > start) parts.push({ text: text.slice(start, end), match: true });
    at = Math.max(at, end);
  }
  if (at < text.length || parts.length === 0) parts.push({ text: text.slice(at), match: false });
  return parts;
}
