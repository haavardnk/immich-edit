import { parseCoordinates } from '#lib/geotag/coordinates.ts';
import { describe, expect, it } from 'vitest';

function point(text: string) {
  const parsed = parseCoordinates(text);
  return parsed
    ? [Number(parsed.lat.toFixed(5)), Number(parsed.lon.toFixed(5)), parsed.source]
    : null;
}

describe('coordinates typed or pasted into place search', () => {
  it.each([
    ['60.3913, 5.3221', [60.3913, 5.3221, 'text']],
    ['60.3913 5.3221', [60.3913, 5.3221, 'text']],
    ['-33.8568;151.2153', [-33.8568, 151.2153, 'text']],
    ['(40.6892, -74.0445)', [40.6892, -74.0445, 'text']],
    ['60.3913° N, 5.3221° E', [60.3913, 5.3221, 'text']],
    ['33.8568 S 151.2153 E', [-33.8568, 151.2153, 'text']],
    ['N 60.3913 E 5.3221', [60.3913, 5.3221, 'text']],
    ['5.3221E 60.3913N', [60.3913, 5.3221, 'text']],
    [`60°23'29"N 5°19'20"E`, [60.39139, 5.32222, 'text']],
    ['60°23′29″N, 5°19′20″W', [60.39139, -5.32222, 'text']],
    [`60°23.5'N 5°19.3'E`, [60.39167, 5.32167, 'text']],
    ['60 23 29 N 5 19 20 E', [60.39139, 5.32222, 'text']]
  ])('reads %s', (text, expected) => {
    expect(point(text)).toEqual(expected);
  });

  it.each([
    ['https://www.google.com/maps/@60.3913,5.3221,15z', [60.3913, 5.3221, 'link']],
    ['google.com/maps/@60.3913,5.3221,15z', [60.3913, 5.3221, 'link']],
    [
      'https://www.google.com/maps/place/Bryggen/@60.39,5.31,17z/data=!3m1!4b1!4m6!3m5!1s0x0:0x0!8m2!3d60.3975!4d5.324',
      [60.3975, 5.324, 'link']
    ],
    ['https://www.google.com/maps/search/?api=1&query=60.3913%2C5.3221', [60.3913, 5.3221, 'link']],
    ['https://maps.apple.com/?ll=60.3913,5.3221&q=Bryggen', [60.3913, 5.3221, 'link']],
    ['https://maps.apple.com/?coordinate=60.3913,5.3221', [60.3913, 5.3221, 'link']],
    [
      'https://www.openstreetmap.org/?mlat=60.3913&mlon=5.3221#map=15/60.39/5.32',
      [60.3913, 5.3221, 'link']
    ],
    ['https://www.openstreetmap.org/#map=15/60.3913/5.3221', [60.3913, 5.3221, 'link']],
    ['geo:60.3913,5.3221;u=35', [60.3913, 5.3221, 'link']],
    ['geo:0,0?q=60.3913,5.3221(Bryggen)', [60.3913, 5.3221, 'link']]
  ])('reads the link %s', (text, expected) => {
    expect(point(text)).toEqual(expected);
  });

  it.each([
    'bryggen',
    'Bergen 5000',
    '60.39',
    '91, 5',
    '60, 181',
    '60 N 5 N',
    'N60.39N, 5.32E',
    `60°61'N 5°E`,
    '1 2 3',
    'https://maps.apple.com/?q=Bryggen',
    'geo:0,0'
  ])('ignores %s', (text) => {
    expect(parseCoordinates(text)).toBeNull();
  });
});
