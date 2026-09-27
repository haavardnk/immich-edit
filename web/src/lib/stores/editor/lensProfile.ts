import { getLensProfile, type LensProfileMatch } from '$lib/api/lensProfile';
import { errorMessage } from '$lib/utils/errors';

export interface LensProfileCtx {
  assetId: string | null;
  lensProfile: LensProfileMatch | null;
  lensProfileError: string | null;
}

export function fetchLensProfile(ctx: LensProfileCtx, id: string): void {
  ctx.lensProfile = null;
  ctx.lensProfileError = null;
  getLensProfile(id)
    .then((p) => {
      if (ctx.assetId === id) ctx.lensProfile = p;
    })
    .catch((e: unknown) => {
      if (ctx.assetId === id) ctx.lensProfileError = errorMessage(e);
    });
}
