import { describe, expect, it } from 'vitest';
import { mdiCircle, mdiCircleOutline, mdiCloseCircle, mdiHeart, mdiHeartOutline } from '@mdi/js';
import {
  coverageOf,
  favoriteMark,
  labelLook,
  NOT_OWNER_TRASH,
  ratingLook,
  rejectMark,
  trashReason
} from './bulkMarks';

describe('coverageOf', () => {
  it.each([
    [false, false, 'none'],
    [true, false, 'all'],
    [false, true, 'some'],
    [true, true, 'some']
  ] as const)('reads on=%s mixed=%s as %s', (on, mixed, expected) => {
    expect(coverageOf(on, mixed)).toBe(expected);
  });
});

describe('favoriteMark', () => {
  it('offers to unfavorite only when every photo is a favorite', () => {
    expect(favoriteMark('all')).toEqual({
      icon: mdiHeart,
      label: 'Unfavorite',
      pressed: true,
      tone: 'text-red-400'
    });
    expect(favoriteMark('none')).toEqual({
      icon: mdiHeartOutline,
      label: 'Favorite',
      pressed: false,
      tone: ''
    });
  });

  it('shows a mixed selection as a dimmed filled heart', () => {
    expect(favoriteMark('some')).toEqual({
      icon: mdiHeart,
      label: 'Favorite',
      pressed: 'mixed',
      tone: 'text-red-400/50'
    });
  });
});

describe('rejectMark', () => {
  it('maps every coverage to its icon, label and tone', () => {
    expect(rejectMark('all')).toMatchObject({ label: 'Unreject', tone: 'text-amber-400' });
    expect(rejectMark('some')).toMatchObject({
      icon: mdiCloseCircle,
      label: 'Reject',
      pressed: 'mixed',
      tone: 'text-amber-400/50'
    });
    expect(rejectMark('none')).toMatchObject({ label: 'Reject', pressed: false, tone: '' });
  });
});

describe('looks', () => {
  it('fills the star only for a shared rating', () => {
    expect(ratingLook(3).icon).not.toBe(ratingLook(null).icon);
    expect(ratingLook(0).icon).toBe(ratingLook(null).icon);
  });

  it('tints the label dot with the shared colour', () => {
    expect(labelLook('red')).toEqual({ icon: mdiCircle, tone: expect.stringContaining('red') });
    expect(labelLook(null)).toEqual({ icon: mdiCircleOutline, tone: '' });
  });
});

describe('trashReason', () => {
  it('explains why photos shared by others cannot be trashed', () => {
    expect(trashReason(true)).toBeNull();
    expect(trashReason(false)).toBe(NOT_OWNER_TRASH);
  });
});
