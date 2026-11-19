import { describe, expect, it } from 'vitest';
import { handoffId } from './photoHandoff';

const at = (path: string): URL => new URL(path, 'https://edit.test');

describe('photo handoff', () => {
  it.each<[string, string | undefined, string | undefined, string | null]>([
    ['entering the editor', '/photos', '/assets/a1?from=%2Fphotos', 'a1'],
    ['leaving the editor', '/assets/a1', '/photos', 'a1'],
    ['stepping between editor photos', '/assets/a1', '/assets/a2', null],
    ['moving between browse pages', '/photos', '/albums', null],
    ['a first load', undefined, '/assets/a1', 'a1']
  ])('hands the photo over when %s', (_name, from, to, want) => {
    expect(handoffId(from ? at(from) : undefined, to ? at(to) : undefined)).toBe(want);
  });
});
