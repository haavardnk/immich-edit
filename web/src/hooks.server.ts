import { FAMILIES } from '#lib/theme/registry.ts';
import { THEME_REVISION_PLACEHOLDER, themeRevision } from '#lib/theme/revision.ts';
import type { Handle } from '@sveltejs/kit/hooks';

const revision = themeRevision(FAMILIES);

export const handle: Handle = ({ event, resolve }) =>
  resolve(event, {
    transformPageChunk: ({ html }) => html.replace(THEME_REVISION_PLACEHOLDER, revision)
  });
