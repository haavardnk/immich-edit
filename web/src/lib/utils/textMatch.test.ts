import { describe, expect, it } from 'vitest';
import { foldText, highlightParts, matchRanges, textMatches } from './textMatch';

function marked(text: string, query: string): string {
  return highlightParts(text, matchRanges(text, query))
    .map((part) => (part.match ? `[${part.text}]` : part.text))
    .join('');
}

describe('text matching', () => {
  it('folds case and diacritics', () => {
    expect(foldText('Ålesund Café')).toBe('alesund cafe');
  });

  it.each([
    ['Iceland', 'lan', true],
    ['Finland', 'LAN', true],
    ['landscape', 'lan', true],
    ['Ålesund', 'ale', true],
    ['Zürich', 'zur', true],
    ['Bergen', 'lan', false],
    ['anything', '   ', true]
  ])('%s matches %j: %s', (text, query, expected) => {
    expect(textMatches(text, query)).toBe(expected);
  });

  it.each([
    ['landscape', 'lan', '[lan]dscape'],
    ['Iceland', 'lan', 'Ice[lan]d'],
    ['Finland', 'LAN', 'Fin[lan]d'],
    ['Ålesund', 'ale', '[Åle]sund'],
    ['banana', 'an', 'b[an][an]a'],
    ['aaaa', 'aa', '[aa][aa]'],
    ['Bergen', 'x', 'Bergen'],
    ['Bergen', '', 'Bergen'],
    ['  trimmed', ' trim ', '  [trim]med']
  ])('highlights %s for %j as %s', (text, query, expected) => {
    expect(marked(text, query)).toBe(expected);
  });

  it('keeps a decomposed accent inside the highlighted letter', () => {
    const decomposed = 'A\u030alesund';
    expect(matchRanges(decomposed, 'a')).toEqual([{ start: 0, end: 2 }]);
    expect(marked(decomposed, 'les')).toBe('A\u030a[les]und');
  });

  it('maps ranges through characters outside the basic plane', () => {
    expect(marked('📷 Oslo', 'oslo')).toBe('📷 [Oslo]');
  });

  it('splits an empty text into one empty part', () => {
    expect(highlightParts('', [])).toEqual([{ text: '', match: false }]);
  });
});
