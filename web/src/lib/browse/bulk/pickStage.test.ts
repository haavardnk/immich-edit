import { describe, expect, it } from 'vitest';
import { nextStage, shownCheck, stageNote, stageSummary, type Stage } from './pickStage';

describe('pickStage', () => {
  it.each([
    [0, undefined, 'add'],
    [0, 'add', null],
    [4, undefined, 'remove'],
    [4, 'remove', null],
    [2, undefined, 'add'],
    [2, 'add', 'remove'],
    [2, 'remove', null]
  ] as const)('with %i of 4 holding and %s staged, a tap stages %s', (holding, stage, next) => {
    expect(nextStage(holding, 4, stage)).toBe(next);
  });

  it('shows the staged state, else the membership', () => {
    expect([0, 2, 4].map((holding) => shownCheck(holding, 4, undefined))).toEqual([
      false,
      'mixed',
      true
    ]);
    expect(shownCheck(4, 4, 'remove')).toBe(false);
    expect(shownCheck(2, 4, 'add')).toBe(true);
  });

  it('says what a row holds or will change', () => {
    expect(stageNote(0, 4, undefined)).toBeNull();
    expect(stageNote(4, 4, undefined)).toBe('All 4 photos');
    expect(stageNote(1, 4, undefined)).toBe('In 1 of 4');
    expect(stageNote(1, 4, 'add')).toBe('Add 3 photos');
    expect(stageNote(1, 4, 'remove')).toBe('Remove from 1 photo');
  });

  it('sums staged changes for the footer', () => {
    const stages = new Map<string, Stage>([
      ['a', 'add'],
      ['b', 'add'],
      ['c', 'remove']
    ]);
    expect(stageSummary(stages)).toEqual({ count: 3, detail: 'Add 2 · remove 1' });
    expect(stageSummary(new Map())).toEqual({ count: 0, detail: '' });
  });
});
