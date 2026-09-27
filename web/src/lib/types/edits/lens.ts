export interface LensEdits {
  profile_enabled: boolean | null;
  ca_enabled: boolean;
  constrain_crop: boolean;
  distortion_amount: number;
  vignette_amount: number;
  k1: number;
  k2: number;
  k3: number;
  vk1: number;
  vk2: number;
  vk3: number;
  ca_red_scale_x10000: number;
  ca_blue_scale_x10000: number;
}

export const NEUTRAL_LENS: LensEdits = {
  profile_enabled: null,
  ca_enabled: false,
  constrain_crop: false,
  distortion_amount: 100,
  vignette_amount: 100,
  k1: 0,
  k2: 0,
  k3: 0,
  vk1: 0,
  vk2: 0,
  vk3: 0,
  ca_red_scale_x10000: 0,
  ca_blue_scale_x10000: 0
};

export function lensDistortionActive(l: LensEdits): boolean {
  return (
    l.profile_enabled === true &&
    l.distortion_amount !== 0 &&
    (l.k1 !== 0 || l.k2 !== 0 || l.k3 !== 0)
  );
}
export function lensVignetteActive(l: LensEdits): boolean {
  return (
    l.profile_enabled === true &&
    l.vignette_amount !== 0 &&
    (l.vk1 !== 0 || l.vk2 !== 0 || l.vk3 !== 0)
  );
}
export function lensCaActive(l: LensEdits): boolean {
  return l.ca_enabled && (l.ca_red_scale_x10000 !== 0 || l.ca_blue_scale_x10000 !== 0);
}
export function lensIsZero(l: LensEdits): boolean {
  return (
    l.profile_enabled === null &&
    !lensDistortionActive(l) &&
    !lensVignetteActive(l) &&
    !lensCaActive(l)
  );
}

export interface LensProfileCoefficients {
  k1: number;
  k2: number;
  k3: number;
  vk1: number;
  vk2: number;
  vk3: number;
}

export function effectiveLens(l: LensEdits, profile: LensProfileCoefficients | null): LensEdits {
  if (l.profile_enabled !== null || !profile) return l;
  return {
    ...l,
    profile_enabled: true,
    constrain_crop: true,
    k1: profile.k1,
    k2: profile.k2,
    k3: profile.k3,
    vk1: profile.vk1,
    vk2: profile.vk2,
    vk3: profile.vk3
  };
}
