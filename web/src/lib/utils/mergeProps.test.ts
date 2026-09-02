import { describe, expect, it, vi } from 'vitest';
import { mergeProps } from './mergeProps';

describe('mergeProps', () => {
  it('chains handlers with the same name in source order', () => {
    const calls: string[] = [];
    const merged = mergeProps(
      { onclick: () => calls.push('first') },
      { onclick: () => calls.push('second') }
    );
    (merged.onclick as (e: unknown) => void)(undefined);
    expect(calls).toEqual(['first', 'second']);
  });

  it('passes the event through to every handler', () => {
    const spy = vi.fn();
    const merged = mergeProps({ onclick: spy }, { onclick: spy });
    const event = { type: 'click' };
    (merged.onclick as (e: unknown) => void)(event);
    expect(spy).toHaveBeenCalledTimes(2);
    expect(spy).toHaveBeenCalledWith(event);
  });

  it.each([
    [{ class: 'a' }, { class: 'b' }, 'a b'],
    [{ class: 'a' }, {}, 'a'],
    [{}, { class: 'b' }, 'b']
  ])('joins class names %#', (a, b, expected) => {
    expect(mergeProps(a, b).class).toBe(expected);
  });

  it('lets later values win and ignores undefined', () => {
    expect(mergeProps({ id: 'a', role: 'button' }, { id: 'b', role: undefined })).toEqual({
      id: 'b',
      role: 'button'
    });
  });

  it('keeps symbol-keyed attachments from every source', () => {
    const a = Symbol('attach-a');
    const b = Symbol('attach-b');
    const merged = mergeProps({ [a]: 'first' }, { [b]: 'second' });
    expect(merged[a as unknown as string]).toBe('first');
    expect(merged[b as unknown as string]).toBe('second');
  });
});
