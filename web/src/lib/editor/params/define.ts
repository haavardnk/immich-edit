import type { Edits } from '#lib/types/edits.ts';
import type { ParamAccess, ParamDef } from './types';

type FieldArea = 'basic' | 'tone' | 'detail' | 'effects' | 'lens' | 'lens_blur';

type NumberKey<T> = { [K in keyof T]-?: T[K] extends number ? K : never }[keyof T];

type ParamSpec = Omit<ParamDef, 'history' | 'synonyms' | 'defaultValue'> &
  Partial<Pick<ParamDef, 'history' | 'synonyms' | 'defaultValue'>>;

export function param(spec: ParamSpec): ParamDef {
  return { history: spec.label, synonyms: [], defaultValue: 0, ...spec };
}

export function field<A extends FieldArea>(area: A, key: NumberKey<Edits[A]>): ParamAccess {
  const slots = (edits: Edits): Record<NumberKey<Edits[A]>, number> =>
    edits[area] as unknown as Record<NumberKey<Edits[A]>, number>;
  return {
    get: (edits) => slots(edits)[key],
    set: (edits, value) => {
      slots(edits)[key] = value;
    }
  };
}

export function paramDefault(param: ParamDef, isRaw: boolean): number {
  return isRaw ? (param.rawDefaultValue ?? param.defaultValue) : param.defaultValue;
}

export function paramsModified(edits: Edits, params: readonly ParamDef[], isRaw: boolean): boolean {
  return params.some((param) => param.get(edits, isRaw) !== paramDefault(param, isRaw));
}

export function resetParams(edits: Edits, params: readonly ParamDef[], isRaw: boolean): void {
  for (const param of params) param.set(edits, paramDefault(param, isRaw), isRaw);
}
