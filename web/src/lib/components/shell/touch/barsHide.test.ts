import { describe, expect, it } from 'vitest';
import { SHOWN_BARS, trackBars, type BarsTrack, type ScrollSample } from './barsHide';

const scroller = {};

function sample(top: number, extra: Partial<ScrollSample> = {}): ScrollSample {
  return { source: scroller, top, scrollHeight: 5000, clientHeight: 400, ...extra };
}

function run(tops: number[]): BarsTrack {
  return tops.reduce((track, top) => trackBars(track, sample(top)), SHOWN_BARS);
}

describe('trackBars', () => {
  it('hides the bars once a scroll down passes the slop', () => {
    expect(run([0, 4, 8]).hidden).toBe(false);
    expect(run([0, 4, 20]).hidden).toBe(true);
  });

  it('shows them again after a scroll up past the slop from the lowest point', () => {
    expect(run([0, 100, 400, 395]).hidden).toBe(true);
    expect(run([0, 100, 400, 380]).hidden).toBe(false);
  });

  it('measures a scroll up from the furthest point reached', () => {
    expect(run([0, 100, 300, 500, 495]).hidden).toBe(true);
    expect(run([0, 100, 300, 500, 480]).hidden).toBe(false);
  });

  it('hides again after a small scroll up then a scroll down', () => {
    expect(run([0, 300, 100, 95, 110]).hidden).toBe(true);
  });

  it('always shows the bars at the top', () => {
    expect(run([0, 300, 0]).hidden).toBe(false);
    expect(run([0, 300, -20]).hidden).toBe(false);
  });

  it('starts a new baseline for another scroller', () => {
    const hidden = run([0, 300]);
    const other = trackBars(hidden, sample(50, { source: {} }));
    expect(other).toMatchObject({ anchor: 50, hidden: true });
    expect(trackBars(hidden, sample(0, { source: {} })).hidden).toBe(false);
  });

  it('ignores the bounce at the end of the list', () => {
    const end = 5000 - 400;
    const atEnd = run([0, 300, end - 2]);
    expect(atEnd.hidden).toBe(true);
    expect(trackBars(atEnd, sample(end - 6)).hidden).toBe(true);
    expect(trackBars(atEnd, sample(end - 4)).anchor).toBe(atEnd.anchor);
  });

  it('starts a new baseline when the viewport height changes', () => {
    const hidden = run([0, 300]);
    const resized = trackBars(hidden, sample(250, { clientHeight: 452 }));
    expect(resized).toMatchObject({ anchor: 250, height: 452, hidden: true });
  });

  it('ignores sources that do not scroll vertically', () => {
    const hidden = run([0, 300]);
    expect(trackBars(hidden, sample(0, { source: {}, scrollHeight: 400 }))).toBe(hidden);
  });
});
