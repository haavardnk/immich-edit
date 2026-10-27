import { solveUpright, withGuide, type UprightGuide } from '$lib/utils/upright';
import { commit, ready, refitDraftCrop, sourceDims, type GeometryCtx } from './geometry.svelte';

const UPRIGHT_ACTION = 'Guided upright';

export function addGuide(ctx: GeometryCtx, guide: UprightGuide): Promise<void> {
  return applyGuides(ctx, (guides) => withGuide(guides, guide));
}

export function moveGuide(ctx: GeometryCtx, index: number, guide: UprightGuide): Promise<void> {
  return applyGuides(ctx, (guides) =>
    guides[index] ? guides.map((g, i) => (i === index ? guide : g)) : null
  );
}

export function removeGuide(ctx: GeometryCtx, index: number): Promise<void> {
  return applyGuides(ctx, (guides) =>
    guides[index] ? guides.filter((_, i) => i !== index) : null
  );
}

async function applyGuides(
  ctx: GeometryCtx,
  change: (guides: UprightGuide[]) => UprightGuide[] | null
): Promise<void> {
  const sess = ready(ctx);
  const guides = sess ? change(sess.guides) : null;
  if (!sess || !guides) return;
  sess.guides = guides;
  const { sw, sh } = sourceDims(sess);
  const solution = solveUpright(guides, sw, sh, sess.draftPerspective.aspect);
  if (!solution) return;
  sess.draftAngle = solution.angle;
  sess.draftPerspective = solution.perspective;
  refitDraftCrop(sess);
  await commit(ctx, UPRIGHT_ACTION);
}
