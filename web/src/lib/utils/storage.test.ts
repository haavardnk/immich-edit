import { beforeEach, describe, expect, it, vi } from 'vitest';
import { writeStored } from './storage';

const values = new Map<string, string>();
const setItem = vi.fn((key: string, value: string) => values.set(key, value));

vi.stubGlobal('localStorage', {
  getItem: (key: string) => values.get(key) ?? null,
  setItem
});

describe('writeStored', () => {
  beforeEach(() => {
    values.clear();
    setItem.mockClear();
  });

  it('serializes values into local storage', () => {
    writeStored('key', { enabled: true });
    expect(values.get('key')).toBe('{"enabled":true}');
  });

  it('ignores storage write failures', () => {
    setItem.mockImplementationOnce(() => {
      throw new DOMException('Quota exceeded', 'QuotaExceededError');
    });
    expect(() => writeStored('key', true)).not.toThrow();
  });

  it('ignores serialization failures', () => {
    const value: Record<string, unknown> = {};
    value.self = value;
    expect(() => writeStored('key', value)).not.toThrow();
  });
});
