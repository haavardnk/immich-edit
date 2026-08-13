import { describe, it, expect } from 'vitest';
import { errorMessage } from './errors';

describe('errorMessage', () => {
  it.each([
    { input: new Error('boom'), expected: 'boom' },
    { input: 'plain', expected: 'plain' },
    { input: { code: 500 }, expected: 'Something went wrong.' },
    { input: undefined, expected: 'Something went wrong.' }
  ])('describes $input', ({ input, expected }) => {
    expect(errorMessage(input)).toBe(expected);
  });
});
