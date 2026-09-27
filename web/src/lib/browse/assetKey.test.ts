import { describe, expect, it } from 'vitest';
import { copyIndex, isCopy, sourceId } from './assetKey';

const uuid = 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee';

describe('assetKey', () => {
  it.each<[string, boolean, string, number | null]>([
    [uuid, false, uuid, null],
    [`${uuid}_1`, true, uuid, 1],
    [`${uuid}_12`, true, uuid, 12],
    [`${uuid}_0`, false, `${uuid}_0`, null],
    [`${uuid}_01`, false, `${uuid}_01`, null],
    [`${uuid}_x`, false, `${uuid}_x`, null],
    [`${uuid}_1_2`, false, `${uuid}_1_2`, null],
    ['not-a-uuid_1', false, 'not-a-uuid_1', null]
  ])('%s', (id, copy, source, index) => {
    expect(isCopy(id)).toBe(copy);
    expect(sourceId(id)).toBe(source);
    expect(copyIndex(id)).toBe(index);
  });
});
