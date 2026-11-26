import {
  asShotText,
  clockExample,
  sameDay,
  shownText,
  zoneName
} from '#lib/metadata/captureLabels.ts';
import { describe, expect, it } from 'vitest';

describe('capture labels', () => {
  it('writes shown times with or without the day', () => {
    const wall = Date.parse('2026-10-31T13:03:12Z');
    expect(shownText(wall, false)).toBe('13:03:12');
    expect(shownText(wall, true)).toBe('31 Oct 2026, 13:03:12');
    expect(sameDay(wall, Date.parse('2026-10-31T23:59:59Z'))).toBe(true);
    expect(sameDay(wall, Date.parse('2026-11-01T00:00:00Z'))).toBe(false);
  });

  it('names zones by place and explains a clock zone', () => {
    expect(zoneName('America/Argentina/Buenos_Aires')).toBe('Buenos Aires');
    expect(zoneName('UTC+1')).toBe('UTC+1');
    expect(
      clockExample(
        Date.parse('2026-10-31T13:05:00Z'),
        Date.parse('2026-10-31T19:05:00Z'),
        'Asia/Bangkok'
      )
    ).toBe('13:05 on the camera → 19:05 in Bangkok');
  });

  it('names the zone the photos were shot in', () => {
    const summer = Date.parse('2026-07-01T12:00:00Z');
    const winter = Date.parse('2026-01-10T12:00:00Z');
    const oslo = { zone: 'Europe/Oslo', at: summer };
    expect(asShotText([oslo, oslo])).toBe('As shot (Oslo · UTC+2)');
    expect(asShotText([oslo, { zone: 'Europe/Oslo', at: winter }])).toBe('As shot (Oslo · UTC+2)');
    expect(asShotText([{ zone: 'UTC+1', at: summer }])).toBe('As shot (UTC+1)');
    expect(asShotText([{ zone: 'UTC+2', at: summer }, oslo])).toBe('As shot (UTC+2)');
    expect(asShotText([{ zone: 'Asia/Bangkok', at: summer }, oslo])).toBe('As shot (2 zones)');
    expect(asShotText([])).toBe('As shot');
  });
});
