export interface EffectsEdits {
  vignette_amount: number;
  vignette_midpoint: number;
  vignette_feather: number;
  vignette_roundness: number;
  grain_amount: number;
  grain_size: number;
  grain_roughness: number;
}

export const NEUTRAL_EFFECTS: EffectsEdits = {
  vignette_amount: 0,
  vignette_midpoint: 50,
  vignette_feather: 50,
  vignette_roundness: 0,
  grain_amount: 0,
  grain_size: 25,
  grain_roughness: 50
};
