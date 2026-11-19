export type KeypadKey =
  '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.' | 'back' | 'sign';

export type KeypadText = { text: string; fresh: boolean };

function digit(state: KeypadText, key: string): string {
  if (state.fresh) return key;
  if (state.text === '0') return key;
  if (state.text === '-0') return `-${key}`;
  return state.text + key;
}

function point(state: KeypadText): string {
  if (state.fresh || state.text === '') return '0.';
  if (state.text === '-') return '-0.';
  return state.text.includes('.') ? state.text : `${state.text}.`;
}

function sign(text: string): string {
  return text.startsWith('-') ? text.slice(1) : `-${text}`;
}

export function pressKey(state: KeypadText, key: KeypadKey): KeypadText {
  if (key === 'back') return { text: state.fresh ? '' : state.text.slice(0, -1), fresh: false };
  if (key === 'sign') return { text: sign(state.text), fresh: false };
  if (key === '.') return { text: point(state), fresh: false };
  return { text: digit(state, key), fresh: false };
}
