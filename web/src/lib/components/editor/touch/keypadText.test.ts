import { describe, expect, it } from 'vitest';
import { pressKey, type KeypadKey, type KeypadText } from './keypadText';

function press(state: KeypadText, keys: readonly KeypadKey[]): KeypadText {
  return keys.reduce(pressKey, state);
}

const FRESH = { text: '-38', fresh: true };

describe('keypad text', () => {
  it('replaces the selected value with the first digit, then appends', () => {
    expect(press(FRESH, ['1', '2'])).toEqual({ text: '12', fresh: false });
  });

  it('starts a decimal from zero', () => {
    expect(press(FRESH, ['.', '5'])).toEqual({ text: '0.5', fresh: false });
    expect(press({ text: '12', fresh: false }, ['.', '.', '3'])).toEqual({
      text: '12.3',
      fresh: false
    });
  });

  it('toggles the sign without losing the digits', () => {
    expect(press(FRESH, ['sign'])).toEqual({ text: '38', fresh: false });
    expect(press({ text: '5', fresh: true }, ['sign', '0'])).toEqual({ text: '-50', fresh: false });
    expect(press({ text: '', fresh: false }, ['sign', '4'])).toEqual({ text: '-4', fresh: false });
  });

  it('drops a leading zero', () => {
    expect(press({ text: '0', fresh: false }, ['7'])).toEqual({ text: '7', fresh: false });
    expect(press({ text: '-0', fresh: false }, ['7'])).toEqual({ text: '-7', fresh: false });
  });

  it('clears the selection or deletes one character', () => {
    expect(press(FRESH, ['back'])).toEqual({ text: '', fresh: false });
    expect(press({ text: '-38', fresh: false }, ['back', 'back'])).toEqual({
      text: '-',
      fresh: false
    });
  });
});
