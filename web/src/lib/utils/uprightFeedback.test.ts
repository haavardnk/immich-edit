import type { UprightGuide } from '#lib/utils/upright.ts';
import {
  countText,
  drawingHint,
  guideLabels,
  guideStatus,
  replacedLabel,
  tiltFromLevel
} from '#lib/utils/uprightFeedback.ts';
import { describe, expect, it } from 'vitest';

const V: UprightGuide = { kind: 'vertical', from: [0.2, 0.1], to: [0.21, 0.9] };
const H: UprightGuide = { kind: 'horizontal', from: [0.1, 0.5], to: [0.9, 0.52] };

describe('upright feedback', () => {
  it('numbers guides within their kind', () => {
    expect(guideLabels([V, H, V])).toEqual(['V1', 'H1', 'V2']);
  });

  it('counts each kind against its limit', () => {
    expect(countText([V, H, V], 'vertical')).toBe('Vertical 2 of 2');
    expect(countText([V], 'horizontal')).toBe('Horizontal 0 of 2');
  });

  it('names the oldest guide a full kind would replace', () => {
    expect(replacedLabel([V], 'vertical')).toBeNull();
    expect(replacedLabel([V, V], 'vertical')).toBe('V1');
  });

  it('measures tilt from level in either direction', () => {
    expect(tiltFromLevel([0, 0], [10, 10])).toBeCloseTo(45);
    expect(tiltFromLevel([10, 0], [0, 10])).toBeCloseTo(45);
    expect(tiltFromLevel([0, 0], [0, 10])).toBeCloseTo(90);
  });

  it('describes the line being drawn', () => {
    expect(drawingHint([], [0, 0], [1, 1], null, 1000)).toBeNull();
    expect(drawingHint([], [0, 0], [100, 93], null, 1000)).toBe(
      '43° · too diagonal, release drops it'
    );
    expect(drawingHint([], [0, 0], [3, 100], 'vertical', 1000)).toBe('V1 · 2°');
    expect(drawingHint([V, V], [0, 0], [3, 100], 'vertical', 1000)).toBe('V2 · 2° · replaces V1');
  });

  it('reports dropped lines, unsolved guides and what comes next', () => {
    expect(guideStatus([], null)).toEqual([]);
    expect(guideStatus([V], { kind: 'dropped', degrees: 43.2 })).toEqual([
      {
        tone: 'warn',
        text: 'Last line dropped: 43° is neither level nor upright (within 35°)'
      },
      { tone: 'info', text: 'Draw one more line' }
    ]);
    expect(guideStatus([V, V, H], { kind: 'unsolved' })).toEqual([
      { tone: 'warn', text: 'These lines cannot be corrected together' },
      { tone: 'info', text: 'A third vertical replaces V1, the oldest' }
    ]);
  });
});
