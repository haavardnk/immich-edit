import { describe, expect, it } from 'vitest';
import { nextSnapshotName } from './snapshotName';

describe('nextSnapshotName', () => {
  it('starts at one', () => {
    expect(nextSnapshotName([])).toBe('Snapshot 1');
  });

  it('skips used numbers and fills the first gap', () => {
    expect(nextSnapshotName(['Snapshot 1', 'Snapshot 2', 'Snapshot 4'])).toBe('Snapshot 3');
  });

  it('ignores names that only look similar', () => {
    expect(nextSnapshotName(['Snapshot 1 copy', 'snapshot 1', 'Warm v1', 'Snapshot x'])).toBe(
      'Snapshot 1'
    );
  });
});
