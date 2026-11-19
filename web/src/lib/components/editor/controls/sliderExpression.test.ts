import { describe, expect, it } from 'vitest';
import { entryValue, evaluateEntry } from './sliderExpression';

const EXPOSURE = { min: -5, max: 5, step: 0.05, fineStep: 0.01, format: String };
const PERCENT = { min: -100, max: 100, step: 1, fineStep: 1, format: String };

describe('typed slider entry', () => {
  it.each([
    ['12', 12],
    ['-38', -38],
    ['−38', -38],
    ['.5', 0.5],
    ['0,25', 0.25],
    ['+5', 15],
    ['x1.5', 15],
    ['×2', 20],
    ['*2', 20],
    ['/4', 2.5],
    ['÷4', 2.5],
    ['12-0.5', 11.5],
    ['2+3*4', 14],
    ['10-2-3', 5],
    ['1 + 1', 2],
    ['5*-2', -10]
  ])('reads %s from 10', (text, expected) => {
    expect(evaluateEntry(text, 10)).toBe(expected);
  });

  it.each([[''], ['  '], ['+'], ['abc'], ['5+'], ['--5'], ['1/0'], ['2..5']])(
    'rejects %j',
    (text) => {
      expect(evaluateEntry(text, 10)).toBeNull();
    }
  );

  it('keeps a leading minus absolute', () => {
    expect(evaluateEntry('-3', 40)).toBe(-3);
  });

  it('clamps and snaps the result to the range', () => {
    expect(entryValue('+100', 0.35, EXPOSURE)).toBe(5);
    expect(entryValue('x2', 0.35, EXPOSURE)).toBe(0.7);
    expect(entryValue('-0.3', 1, EXPOSURE)).toBe(-0.3);
    expect(entryValue('-380', 0, PERCENT)).toBe(-100);
    expect(entryValue('12.6', 0, PERCENT)).toBe(13);
    expect(entryValue('nope', 0, PERCENT)).toBeNull();
  });
});
