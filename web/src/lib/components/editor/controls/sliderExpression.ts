import type { ParamRange } from '#lib/editor/params/types.ts';
import { settleValue } from './sliderStep';

type Operator = '+' | '-' | '*' | '/';
type Token = number | Operator;

const OPERATORS: Record<string, Operator> = { '+': '+', '-': '-', '*': '*', x: '*', '/': '/' };
const RELATIVE = ['+', '*', 'x', '/'];
const NUMBER = /^-?(\d+\.?\d*|\.\d+)/;

function normalise(text: string): string {
  return text
    .toLowerCase()
    .replaceAll('−', '-')
    .replaceAll('×', 'x')
    .replaceAll('÷', '/')
    .replaceAll(',', '.')
    .replace(/\s+/g, '');
}

function tokenize(text: string, tokens: Token[]): Token[] | null {
  let rest = text;
  while (rest.length > 0) {
    if (typeof tokens.at(-1) === 'number') {
      const operator = OPERATORS[rest.charAt(0)];
      if (!operator) return null;
      tokens.push(operator);
      rest = rest.slice(1);
      continue;
    }
    const match = NUMBER.exec(rest);
    if (!match) return null;
    tokens.push(Number(match[0]));
    rest = rest.slice(match[0].length);
  }
  return typeof tokens.at(-1) === 'number' ? tokens : null;
}

function apply(left: number, operator: Operator, right: number): number {
  if (operator === '+') return left + right;
  if (operator === '-') return left - right;
  if (operator === '*') return left * right;
  return left / right;
}

function reduce(tokens: readonly Token[], operators: readonly Operator[]): Token[] {
  const out: Token[] = [];
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index];
    const right = tokens[index + 1];
    const left = out.at(-1);
    if (
      typeof token !== 'number' &&
      token !== undefined &&
      operators.includes(token) &&
      typeof left === 'number' &&
      typeof right === 'number'
    ) {
      out[out.length - 1] = apply(left, token, right);
      index++;
      continue;
    }
    if (token !== undefined) out.push(token);
  }
  return out;
}

export function evaluateEntry(text: string, start: number): number | null {
  const clean = normalise(text);
  if (clean.length === 0) return null;
  const tokens = tokenize(clean, RELATIVE.includes(clean.charAt(0)) ? [start] : []);
  if (!tokens) return null;
  const [result] = reduce(reduce(tokens, ['*', '/']), ['+', '-']);
  return typeof result === 'number' && Number.isFinite(result) ? result : null;
}

export function entryValue(text: string, start: number, range: ParamRange): number | null {
  const value = evaluateEntry(text, start);
  return value === null ? null : settleValue(value, range);
}
