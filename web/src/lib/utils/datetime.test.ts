import { describe, expect, it } from 'vitest';
import { formatAgo, formatDay } from './datetime';

const now = new Date(2026, 10, 10, 14, 30);

function at(minutesAgo: number): string {
  return new Date(now.getTime() - minutesAgo * 60_000).toISOString();
}

describe('formatAgo', () => {
  it('says now under a minute', () => {
    expect(formatAgo(at(0.5), now)).toBe('now');
  });

  it('counts minutes then hours', () => {
    expect(formatAgo(at(4), now)).toBe('4 min ago');
    expect(formatAgo(at(125), now)).toBe('2 h ago');
  });

  it('falls back to a date after a day', () => {
    const d = new Date(2026, 10, 2, 18, 5);
    expect(formatAgo(d.toISOString(), now)).toBe(
      d.toLocaleDateString(undefined, { day: 'numeric', month: 'short' })
    );
  });

  it('keeps an unparseable value', () => {
    expect(formatAgo('soon', now)).toBe('soon');
  });
});

describe('formatDay', () => {
  it('names today', () => {
    const d = new Date(2026, 10, 10, 14, 12);
    const time = d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    expect(formatDay(d.toISOString(), now)).toBe(`Today, ${time}`);
  });

  it('dates other days and adds the year when it differs', () => {
    const d = new Date(2025, 10, 2, 18, 5);
    const time = d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    const day = d.toLocaleDateString(undefined, {
      day: 'numeric',
      month: 'short',
      year: 'numeric'
    });
    expect(formatDay(d.toISOString(), now)).toBe(`${day}, ${time}`);
  });
});
