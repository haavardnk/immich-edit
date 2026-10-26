import { describe, expect, it } from 'vitest';
import {
  REPLACE_NONE_OWNED,
  REPLACE_NOT_OWNER,
  REPLACE_TRASH_OFF,
  REPLACE_TWO_VERSIONS,
  replaceBlocked,
  replaceWarning
} from './replace';

describe('replaceBlocked', () => {
  it.each([
    [{ owned: false, trashOn: true }, REPLACE_NOT_OWNER],
    [{ owned: false, trashOn: true, single: false }, REPLACE_NONE_OWNED],
    [{ owned: true, trashOn: true, versionsOfOnePhoto: true }, REPLACE_TWO_VERSIONS],
    [{ owned: true, trashOn: false }, REPLACE_TRASH_OFF],
    [{ owned: true, trashOn: true }, null]
  ])('explains %o', (input, reason) => {
    expect(replaceBlocked(input)).toBe(reason);
  });
});

describe('replaceWarning', () => {
  it('names the trash period', () => {
    expect(replaceWarning(30)).toContain('for good after 30 days');
    expect(replaceWarning(null)).toContain('when its trash period ends');
  });
});
