import type { PhotoMatch } from '#lib/geotag/match.ts';
import { focusLink, pinSpot, trackMarks } from '#lib/geotag/references.ts';
import { describe, expect, it } from 'vitest';

const ON = { lat: 60, lon: 6 };
const NEAR = { lat: 60.0002, lon: 6 };
const FAR = { lat: 60.001, lon: 6 };
const TRACK = { kind: 'placed', ...ON, gapMs: 0 } as const;

function match(id: string, outcome: PhotoMatch['outcome']): PhotoMatch {
  return { id, time: 0, outcome };
}

const MATCHES = [
  match('placed', { ...TRACK, from: null }),
  match('near', { kind: 'kept', at: NEAR, track: TRACK }),
  match('far', { kind: 'kept', at: FAR, track: TRACK }),
  match('moved', { ...TRACK, from: NEAR }),
  match('lost', { kind: 'kept', at: FAR, track: { kind: 'outside', reason: 'after', gapMs: 1 } }),
  match('outside', { kind: 'outside', reason: 'gap', gapMs: 1 })
];

describe('track marks', () => {
  it('pins placed photos and marks located ones as references', () => {
    const marks = trackMarks(MATCHES);
    expect(marks.pins.map((pin) => pin.id)).toEqual(['placed', 'moved']);
    expect(marks.references).toEqual([
      { id: 'near', ...NEAR, moved: false },
      { id: 'far', ...FAR, moved: false },
      { id: 'moved', ...NEAR, moved: true },
      { id: 'lost', ...FAR, moved: false }
    ]);
  });

  it('connects references more than 50 m away and every moved photo', () => {
    expect(trackMarks(MATCHES).connectors).toEqual([
      { id: 'far', from: FAR, to: ON },
      { id: 'moved', from: NEAR, to: ON }
    ]);
  });

  it('links a focused reference to the track whatever the distance', () => {
    expect(focusLink(MATCHES[1]?.outcome)).toEqual({ id: 'focus', from: NEAR, to: ON });
    expect(focusLink(MATCHES[4]?.outcome)).toBeNull();
    expect(focusLink(MATCHES[0]?.outcome)).toBeNull();
  });

  it('follows placed photos on the track and kept ones where they are', () => {
    expect(pinSpot(MATCHES[0]?.outcome)).toEqual(ON);
    expect(pinSpot(MATCHES[2]?.outcome)).toEqual(FAR);
    expect(pinSpot(MATCHES[5]?.outcome)).toBeNull();
  });
});
