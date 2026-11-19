import { describe, expect, it } from 'vitest';
import { enteringEditor, handoffId } from './photoHandoff';

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

  it.each<[string, string | undefined, string, boolean]>([
    ['from a browse page', '/photos', '/assets/a1', true],
    ['on a first load', undefined, '/assets/a1', false],
    ['between editor photos', '/assets/a1', '/assets/a2', false],
    ['to a browse page', '/assets/a1', '/photos', false]
  ])('starts a fresh edit only when entering %s', (_name, from, to, want) => {
    expect(enteringEditor(from ? at(from) : undefined, at(to))).toBe(want);
  });
});
