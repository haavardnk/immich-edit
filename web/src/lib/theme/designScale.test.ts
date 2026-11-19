import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>(['/src/**/*.svelte', '/src/**/*.ts', '/src/app.css'], {
  query: '?raw',
  import: 'default',
  eager: true
});

const ICON_PX = new Set(['12', '14', '16', '20', '24', '40']);
const ICON_EM = new Set(['1', '1.15', '1.375']);

const RULES: Array<{
  name: string;
  pattern: RegExp;
  allowed?: (match: RegExpExecArray) => boolean;
}> = [
  { name: 'arbitrary text size', pattern: /\btext-\[\d[^\]]*\]/g },
  { name: 'raw duration', pattern: /\bduration-\d+\b/g },
  { name: 'arbitrary radius', pattern: /\brounded(?:-[a-z]+)?-\[[^\]]+\]/g },
  {
    name: 'hand-tuned separator',
    pattern: /\b(?:border|divide|ring)(?:-[trblxyse])?-(?:dark|light-950)\/10\b/g
  },
  { name: 'bare uppercase', pattern: /(?<![\w:-])uppercase\b/g },
  {
    name: 'off-scale icon size',
    pattern: /\bsize="(\d+(?:\.\d+)?)(px|em)"/g,
    allowed: ([, value = '', unit]) => (unit === 'px' ? ICON_PX : ICON_EM).has(value)
  }
];

function violations(): string[] {
  const found: string[] = [];
  for (const [path, source] of Object.entries(sources)) {
    if (path.endsWith('.test.ts')) continue;
    for (const { name, pattern, allowed } of RULES) {
      for (const match of source.matchAll(pattern)) {
        if (allowed?.(match as RegExpExecArray)) continue;
        if (name === 'bare uppercase' && path === '/src/app.css') continue;
        found.push(`${path}: ${name} ${match[0]}`);
      }
    }
  }
  return found;
}

describe('design scale', () => {
  it('keeps type, motion, radius, separators, labels and icons on their tokens', () => {
    expect(violations()).toEqual([]);
  });
});
