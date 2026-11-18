import { describe, expect, it } from 'vitest';
import { recentEdits } from './recentEdits';

function entry(id: string, updatedAt: string) {
  return { id, hash: `h-${id}`, updated_at: updatedAt };
}

describe('recentEdits', () => {
  it('keeps the four newest edits, newest first', () => {
    const entries = [
      entry('a', '2026-01-01T00:00:00Z'),
      entry('b', '2026-03-01T00:00:00Z'),
      entry('c', '2026-02-01T00:00:00Z'),
      entry('d', '2026-05-01T00:00:00Z'),
      entry('e', '2026-04-01T00:00:00Z')
    ];
    expect(recentEdits(entries).map((e) => e.id)).toEqual(['d', 'e', 'b', 'c']);
  });

  it('does not reorder the source list', () => {
    const entries = [entry('a', '2026-01-01T00:00:00Z'), entry('b', '2026-03-01T00:00:00Z')];
    recentEdits(entries);
    expect(entries.map((e) => e.id)).toEqual(['a', 'b']);
  });
});
