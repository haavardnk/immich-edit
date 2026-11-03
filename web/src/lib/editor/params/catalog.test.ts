import { MASKED_EDIT_KEYS, neutralEdits, neutraliseSection, type Edits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { DESKTOP_SECTIONS, PARAMS, TOUCH_GROUPS } from './catalog';
import { paramDefault } from './define';
import { MASK_PARAM_ROWS } from './masks';
import type { ParamDef } from './types';

const OWNED_ELSEWHERE = [
  'basic.white_balance',
  'detail.capture_sharpen',
  'detail.ai_denoise',
  'lens.profile_enabled',
  'lens.ca_enabled',
  'lens.constrain_crop',
  'lens.k1',
  'lens.k2',
  'lens.k3',
  'lens.vk1',
  'lens.vk2',
  'lens.vk3',
  'lens.ca_red_scale_x10000',
  'lens.ca_blue_scale_x10000',
  'lens_blur.focus_min',
  'lens_blur.focus_max',
  'lens_blur.shape',
  'lens_blur.depth_raster_id',
  'color.bw.enabled',
  'color.lut_3d.lut_id',
  'color.dcp',
  'geometry',
  'masks',
  'retouch'
];

const FIELDS_AWAITING_CATALOG = [
  'basic.curves',
  'color.hsl',
  'color.color_grade',
  'color.bw.shadows',
  'color.bw.highlights'
];

const RAW_STATES = [false, true];

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

function otherValue(param: ParamDef, isRaw: boolean): number {
  return param.max === paramDefault(param, isRaw) ? param.min : param.max;
}

function withAll(params: readonly ParamDef[], isRaw: boolean): Edits {
  const edits = neutralEdits();
  for (const param of params) param.set(edits, otherValue(param, isRaw), isRaw);
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
  PARAMS.map((param) => [
    param.id,
    RAW_STATES.flatMap((isRaw) => changedPaths(neutralEdits(), withAll([param], isRaw)))
  ])
);

const cases = PARAMS.flatMap((param) =>
  RAW_STATES.map((isRaw) => [param.id, isRaw, param] as const)
);

describe('parameter catalog', () => {
  it('gives every parameter a unique id', () => {
    expect(new Set(PARAMS.map((param) => param.id)).size).toBe(PARAMS.length);
  });

  it.each(cases)(
    '%s round-trips one field from its neutral default (raw %s)',
    (_, isRaw, param) => {
      const edits = neutralEdits();
      const neutral = paramDefault(param, isRaw);
      expect(param.get(edits, isRaw)).toBe(neutral);
      param.set(edits, otherValue(param, isRaw), isRaw);
      expect(param.get(edits, isRaw)).toBe(otherValue(param, isRaw));
      expect(changedPaths(neutralEdits(), edits)).toStrictEqual([covered.get(param.id)?.[0]]);
      param.set(edits, neutral, isRaw);
      expect(edits).toStrictEqual(neutralEdits());
      expect(new Set(covered.get(param.id)).size).toBe(1);
      expect(param.min).toBeLessThanOrEqual(neutral);
      expect(param.max).toBeGreaterThanOrEqual(neutral);
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

  it.each(
    DESKTOP_SECTIONS.flatMap((section) =>
      RAW_STATES.map((isRaw) => [section.id, isRaw, section] as const)
    )
  )('bypasses exactly the %s section parameters (raw %s)', (id, isRaw, section) => {
    const own = section.rows.flat();
    const bypassed = neutraliseSection(withAll(PARAMS, isRaw), id);
    for (const param of PARAMS) {
      const expected = own.includes(param) ? paramDefault(param, isRaw) : otherValue(param, isRaw);
      expect(param.get(bypassed, isRaw)).toBe(expected);
    }
  });

  it('gives every masked edit one slider', () => {
    const keys = MASK_PARAM_ROWS.flat().map((param) => param.key);
    expect(keys.sort()).toEqual([...MASKED_EDIT_KEYS].sort());
  });
});
