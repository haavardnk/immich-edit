import { describe, it, expect } from 'vitest';
import { blankBuffer, stampBuffer, parseHexColor } from './brush';

describe('blankBuffer', () => {
  it('allocates a zeroed buffer of the right size', () => {
    const buf = blankBuffer(4, 3);
    expect(buf.width).toBe(4);
    expect(buf.height).toBe(3);
    expect(buf.bytes.length).toBe(12);
    expect(buf.bytes.every((b) => b === 0)).toBe(true);
  });
});

describe('stampBuffer', () => {
  it('paints the strongest value at the stamp center', () => {
    const buf = blankBuffer(11, 11);
    stampBuffer(buf, 5, 5, 4, 1, 200, false);
    const center = buf.bytes[5 * 11 + 5];
    const edge = buf.bytes[5 * 11 + 1];
    expect(center).toBe(200);
    expect(center).toBeGreaterThanOrEqual(edge);
  });

  it('leaves pixels outside the radius untouched', () => {
    const buf = blankBuffer(11, 11);
    stampBuffer(buf, 5, 5, 2, 1, 255, false);
    expect(buf.bytes[0]).toBe(0);
  });

  it('accumulates additively and clamps at 255', () => {
    const buf = blankBuffer(5, 5);
    stampBuffer(buf, 2, 2, 1, 1, 200, false);
    stampBuffer(buf, 2, 2, 1, 1, 200, false);
    expect(buf.bytes[2 * 5 + 2]).toBe(255);
  });

  it('subtracts when erasing and clamps at zero', () => {
    const buf = blankBuffer(5, 5);
    stampBuffer(buf, 2, 2, 1, 1, 100, false);
    stampBuffer(buf, 2, 2, 1, 1, 150, true);
    expect(buf.bytes[2 * 5 + 2]).toBe(0);
  });
});

describe('parseHexColor', () => {
  it('parses a six-digit hex with a leading hash', () => {
    expect(parseHexColor('#ff8040')).toEqual([255, 128, 64]);
  });

  it('expands a three-digit shorthand', () => {
    expect(parseHexColor('#f80')).toEqual([255, 136, 0]);
  });

  it('falls back to a default color on invalid input', () => {
    expect(parseHexColor('#zzzzzz')).toEqual([255, 60, 60]);
  });
});
