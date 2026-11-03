import { MASKED_EDIT_KEYS, neutralEdits, neutraliseSection, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { DESKTOP_SECTIONS, PARAMS, TOUCH_GROUPS } from './catalog';
import { MASK_PARAM_ROWS } from './masks';
import type { ParamDef } from './types';

const OWNED_ELSEWHERE = [
  'basic.white_balance',
  'geometry.rotate',
  'geometry.flip_h',
  'geometry.flip_v',
  'geometry.crop',
  'geometry.aspect',
  'masks',
  'retouch'
];

const FIELDS_AWAITING_CATALOG = [
  'basic.curves',
  'color',
  'detail',
  'effects',
  'lens',
  'lens_blur',
  'geometry.rotate_angle',
  'geometry.perspective'
];

const EXCLUDED = [...OWNED_ELSEWHERE, ...FIELDS_AWAITING_CATALOG];

function leaves(value: unknown, path = ''): Map<string, string> {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return new Map(
      Object.entries(value).flatMap(([key, child]) => [
        ...leaves(child, path ? `${path}.${key}` : key)
      ])
    );
  }
  return new Map([[path, JSON.stringify(value)]]);
}

function changedPaths(before: Edits, after: Edits): string[] {
  const old = leaves(before);
  return [...leaves(after)].filter(([path, value]) => old.get(path) !== value).map(([p]) => p);
}

function otherValue(param: ParamDef): number {
  return param.max === param.defaultValue ? param.min : param.max;
}

function withAll(params: readonly ParamDef[]): Edits {
  const edits = neutralEdits();
  for (const param of params) param.set(edits, otherValue(param));
  return edits;
}

function under(path: string, prefix: string): boolean {
  return path === prefix || path.startsWith(`${prefix}.`);
}

function excluded(path: string): boolean {
  return EXCLUDED.some((prefix) => under(path, prefix));
}

function sorted(params: readonly ParamDef[]): ParamDef[] {
  return [...params].sort((a, b) => a.id.localeCompare(b.id));
}

const covered = new Map(
  PARAMS.map((param) => [param.id, changedPaths(neutralEdits(), withAll([param]))])
);

describe('parameter catalog', () => {
  it('gives every parameter a unique id', () => {
    expect(new Set(PARAMS.map((param) => param.id)).size).toBe(PARAMS.length);
  });

  it.each(PARAMS.map((param) => [param.id, param] as const))(
    '%s round-trips one field from its neutral default',
    (_, param) => {
      const edits = neutralEdits();
      expect(param.get(edits)).toBe(param.defaultValue);
      param.set(edits, otherValue(param));
      expect(param.get(edits)).toBe(otherValue(param));
      expect(covered.get(param.id)).toHaveLength(1);
      expect(param.min).toBeLessThanOrEqual(param.defaultValue);
      expect(param.max).toBeGreaterThanOrEqual(param.defaultValue);
      expect(param.fineStep).toBeGreaterThan(0);
      expect(param.fineStep).toBeLessThanOrEqual(param.step);
    }
  );

  it('covers every edit field or excludes it explicitly', () => {
    const fields = [...leaves(neutralEdits()).keys()];
    const owned = new Set([...covered.values()].flat());
    expect(fields.filter((path) => !owned.has(path) && !excluded(path))).toEqual([]);
    expect([...owned].filter(excluded)).toEqual([]);
    expect(EXCLUDED.filter((prefix) => !fields.some((path) => under(path, prefix)))).toEqual([]);
  });

  it('places every parameter in exactly one desktop section', () => {
    const placed = DESKTOP_SECTIONS.flatMap((section) => section.rows.flat());
    expect(sorted(placed)).toStrictEqual(sorted(PARAMS));
  });

  it('places every parameter in exactly one touch view', () => {
    const viewed = TOUCH_GROUPS.flatMap((group) => group.views.flatMap((view) => view.params));
    expect(sorted(viewed)).toStrictEqual(sorted(PARAMS));
  });

  it.each(DESKTOP_SECTIONS.map((section) => [section.id, section] as const))(
    'bypasses exactly the %s section parameters',
    (id, section) => {
      const own = section.rows.flat();
      const bypassed = neutraliseSection(withAll(PARAMS), id);
      for (const param of PARAMS) {
        const expected = own.includes(param) ? param.defaultValue : otherValue(param);
        expect(param.get(bypassed)).toBe(expected);
      }
    }
  );

  it('gives every masked edit one slider', () => {
    const keys = MASK_PARAM_ROWS.flat().map((param) => param.key);
    expect(keys.sort()).toEqual([...MASKED_EDIT_KEYS].sort());
  });
});
