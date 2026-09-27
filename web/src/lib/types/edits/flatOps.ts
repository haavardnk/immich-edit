import { NEUTRAL_DETAIL, type DetailEdits } from './detail';
import type { EffectsEdits } from './effects';
import { lensIsZero, type LensEdits } from './lens';
import type { Edits } from './model';
import type { BasicEdits, ToneEdits } from './tone';

type NumericKeys<T> = { [K in keyof T]: T[K] extends number ? K : never }[keyof T];

interface NumField {
  key: string;
  get: (e: Edits) => number;
  set: (e: Edits, v: number) => void;
}

interface BoolField {
  key: string;
  get: (e: Edits) => boolean;
  set: (e: Edits, v: boolean) => void;
}

interface NullField {
  key: string;
  get: (e: Edits) => number | null;
  set: (e: Edits, v: number | null) => void;
}

interface TriField {
  key: string;
  get: (e: Edits) => boolean | null;
  set: (e: Edits, v: boolean | null) => void;
}

interface FlatOp {
  id: string;
  legacyId?: string;
  nums: NumField[];
  bools?: BoolField[];
  nulls?: NullField[];
  tris?: TriField[];
  active: (e: Edits) => boolean;
  identity?: (e: Edits) => boolean;
}

function nf(key: string, get: (e: Edits) => number, set: (e: Edits, v: number) => void): NumField {
  return { key, get, set };
}

function bf(
  key: string,
  get: (e: Edits) => boolean,
  set: (e: Edits, v: boolean) => void
): BoolField {
  return { key, get, set };
}

function basicOp(id: string, key: string, field: NumericKeys<BasicEdits>): FlatOp {
  return {
    id,
    nums: [
      nf(
        key,
        (e) => e.basic[field],
        (e, v) => {
          e.basic[field] = v;
        }
      )
    ],
    active: (e) => e.basic[field] !== 0
  };
}

function toneField(field: NumericKeys<ToneEdits>): NumField {
  return nf(
    field,
    (e) => e.tone[field],
    (e, v) => {
      e.tone[field] = v;
    }
  );
}

function detailField(key: string, field: NumericKeys<DetailEdits>): NumField {
  return nf(
    key,
    (e) => e.detail[field],
    (e, v) => {
      e.detail[field] = v;
    }
  );
}

function effectsField(key: string, field: NumericKeys<EffectsEdits>): NumField {
  return nf(
    key,
    (e) => e.effects[field],
    (e, v) => {
      e.effects[field] = v;
    }
  );
}

function lensField(key: string, field: NumericKeys<LensEdits>): NumField {
  return nf(
    key,
    (e) => e.lens[field],
    (e, v) => {
      e.lens[field] = v;
    }
  );
}

const FLAT_OPS: FlatOp[] = [
  basicOp('exposure', 'ev', 'exposure_ev'),
  basicOp('brightness', 'amount', 'brightness'),
  basicOp('contrast', 'amount', 'contrast'),
  basicOp('saturation', 'amount', 'saturation'),
  basicOp('vibrance', 'amount', 'vibrance'),
  basicOp('texture', 'amount', 'texture'),
  basicOp('clarity', 'amount', 'clarity'),
  basicOp('dehaze', 'amount', 'dehaze'),
  {
    id: 'white_balance',
    nums: [
      nf(
        'temp',
        (e) => e.basic.wb_temp,
        (e, v) => {
          e.basic.wb_temp = v;
        }
      ),
      nf(
        'tint',
        (e) => e.basic.wb_tint,
        (e, v) => {
          e.basic.wb_tint = v;
        }
      )
    ],
    active: (e) => e.basic.wb_temp !== 0 || e.basic.wb_tint !== 0
  },
  {
    id: 'tone_regions',
    legacyId: 'highlights_shadows',
    nums: [toneField('highlights'), toneField('shadows'), toneField('blacks'), toneField('whites')],
    active: (e) =>
      e.tone.highlights !== 0 || e.tone.shadows !== 0 || e.tone.blacks !== 0 || e.tone.whites !== 0
  },
  {
    id: 'capture_sharpen',
    nums: [],
    bools: [
      bf(
        'enabled',
        (e) => e.detail.capture_sharpen,
        (e, v) => {
          e.detail.capture_sharpen = v;
        }
      )
    ],
    active: (e) => !e.detail.capture_sharpen
  },
  {
    id: 'sharpen',
    nums: [
      detailField('radius', 'sharpen_radius'),
      detailField('detail', 'sharpen_detail'),
      detailField('masking', 'sharpen_masking')
    ],
    nulls: [
      {
        key: 'amount',
        get: (e) => e.detail.sharpen_amount,
        set: (e, v) => {
          e.detail.sharpen_amount = v;
        }
      }
    ],
    active: (e) =>
      e.detail.sharpen_amount !== null ||
      e.detail.sharpen_radius !== NEUTRAL_DETAIL.sharpen_radius ||
      e.detail.sharpen_detail !== NEUTRAL_DETAIL.sharpen_detail ||
      e.detail.sharpen_masking !== NEUTRAL_DETAIL.sharpen_masking
  },
  {
    id: 'luma_nr',
    nums: [
      detailField('amount', 'luma_nr_amount'),
      detailField('detail', 'luma_nr_detail'),
      detailField('contrast', 'luma_nr_contrast')
    ],
    active: (e) => e.detail.luma_nr_amount !== 0
  },
  {
    id: 'color_nr',
    nums: [
      detailField('amount', 'color_nr_amount'),
      detailField('detail', 'color_nr_detail'),
      detailField('smoothness', 'color_nr_smoothness')
    ],
    active: (e) => e.detail.color_nr_amount !== 0
  },
  {
    id: 'vignette',
    nums: [
      effectsField('amount', 'vignette_amount'),
      effectsField('midpoint', 'vignette_midpoint'),
      effectsField('feather', 'vignette_feather'),
      effectsField('roundness', 'vignette_roundness')
    ],
    active: (e) => e.effects.vignette_amount !== 0
  },
  {
    id: 'grain',
    nums: [
      effectsField('amount', 'grain_amount'),
      effectsField('size', 'grain_size'),
      effectsField('roughness', 'grain_roughness')
    ],
    active: (e) => e.effects.grain_amount !== 0
  },
  {
    id: 'lens_profile',
    nums: [
      lensField('distortion_amount', 'distortion_amount'),
      lensField('vignette_amount', 'vignette_amount'),
      lensField('k1', 'k1'),
      lensField('k2', 'k2'),
      lensField('k3', 'k3'),
      lensField('vk1', 'vk1'),
      lensField('vk2', 'vk2'),
      lensField('vk3', 'vk3'),
      lensField('ca_red', 'ca_red_scale_x10000'),
      lensField('ca_blue', 'ca_blue_scale_x10000')
    ],
    tris: [
      {
        key: 'profile_enabled',
        get: (e) => e.lens.profile_enabled,
        set: (e, v) => {
          e.lens.profile_enabled = v;
        }
      }
    ],
    bools: [
      bf(
        'ca_enabled',
        (e) => e.lens.ca_enabled,
        (e, v) => {
          e.lens.ca_enabled = v;
        }
      ),
      bf(
        'constrain_crop',
        (e) => e.lens.constrain_crop,
        (e, v) => {
          e.lens.constrain_crop = v;
        }
      )
    ],
    active: (e) =>
      e.lens.profile_enabled !== null ||
      e.lens.ca_enabled ||
      e.lens.constrain_crop ||
      e.lens.k1 !== 0 ||
      e.lens.k2 !== 0 ||
      e.lens.k3 !== 0 ||
      e.lens.vk1 !== 0 ||
      e.lens.vk2 !== 0 ||
      e.lens.vk3 !== 0 ||
      e.lens.ca_red_scale_x10000 !== 0 ||
      e.lens.ca_blue_scale_x10000 !== 0,
    identity: (e) => lensIsZero(e.lens)
  }
];

export function flatOpsAreIdentity(e: Edits): boolean {
  return FLAT_OPS.every((op) => (op.identity ? op.identity(e) : !op.active(e)));
}

export function encodeFlatOps(e: Edits, ops: Record<string, unknown>): void {
  for (const op of FLAT_OPS) {
    if (!op.active(e)) continue;
    const obj: Record<string, number | boolean | null> = {};
    for (const f of op.nums) obj[f.key] = f.get(e);
    for (const f of op.bools ?? []) obj[f.key] = f.get(e);
    for (const f of op.nulls ?? []) obj[f.key] = f.get(e);
    for (const f of op.tris ?? []) obj[f.key] = f.get(e);
    ops[op.id] = obj;
  }
}

export function decodeFlatOps(ops: Record<string, unknown>, e: Edits): void {
  for (const op of FLAT_OPS) {
    const src = ops[op.id] ?? (op.legacyId ? ops[op.legacyId] : undefined);
    if (!src || typeof src !== 'object') continue;
    const raw = src as Record<string, unknown>;
    for (const f of op.nums) {
      const v = raw[f.key];
      if (typeof v === 'number') f.set(e, v);
    }
    for (const f of op.bools ?? []) {
      const v = raw[f.key];
      if (typeof v === 'boolean') f.set(e, v);
    }
    for (const f of op.nulls ?? []) {
      const v = raw[f.key];
      f.set(e, typeof v === 'number' ? v : null);
    }
    for (const f of op.tris ?? []) {
      const v = raw[f.key];
      f.set(e, typeof v === 'boolean' ? v : null);
    }
  }
}
