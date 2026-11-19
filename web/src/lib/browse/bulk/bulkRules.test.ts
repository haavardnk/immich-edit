import { describe, expect, it } from 'vitest';
import { fileStem, multiDetail, scopeLine, TOUCH_PANES } from './bulkRules';

describe('scopeLine', () => {
  it.each([
    ['part of the view is loaded', 200, 1204, true, '200 of 1204 loaded'],
    ['everything is loaded', 1204, 1204, false, null],
    ['the feed has no more pages', 30, 1204, false, null],
    ['the total is unknown', 200, undefined, true, null],
    ['the loaded count already reaches the total', 40, 40, true, null]
  ])('reads %s', (_name, loaded, total, partial, expected) => {
    expect(scopeLine(loaded, total, partial)).toBe(expected);
  });
});

describe('multiDetail', () => {
  it.each([
    ['compare', 2, TOUCH_PANES, null],
    ['compare', 14, TOUCH_PANES, 'first 2'],
    ['compare', 14, 9, 'first 2'],
    ['survey', 3, TOUCH_PANES, null],
    ['survey', 14, TOUCH_PANES, 'first 4'],
    ['survey', 9, 9, null],
    ['survey', 14, 9, 'first 9']
  ] as const)('labels %s with %i picked over %i panes', (mode, count, panes, expected) => {
    expect(multiDetail(mode, count, panes)).toBe(expected);
  });
});

describe('fileStem', () => {
  it.each([
    ['IMG_4120.ARW', 'IMG_4120'],
    ['archive.tar.gz', 'archive.tar'],
    ['.hidden', '.hidden'],
    ['README', 'README']
  ])('strips the extension of %s', (name, expected) => {
    expect(fileStem(name)).toBe(expected);
  });
});
